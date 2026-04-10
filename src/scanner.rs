use std::{
    collections::BTreeSet,
    env, fs,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::model::{ProjectSummary, SearchHit, SessionSummary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSource {
    Cache,
    Scan,
}

pub fn scan_sessions() -> Result<Vec<SessionSummary>> {
    let sessions_root = codex_home()?.join("sessions");
    let known_product_roots = discover_known_product_roots();
    let mut sessions = Vec::new();

    if !sessions_root.exists() {
        return Ok(sessions);
    }

    for entry in WalkDir::new(&sessions_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
    {
        if let Some(summary) = parse_session_file(entry.path(), &known_product_roots)? {
            sessions.push(summary);
        }
    }

    sessions.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    Ok(sessions)
}

pub fn load_sessions() -> Result<(Vec<SessionSummary>, SessionSource)> {
    if let Some(cached) = read_cached_sessions(None)? {
        return Ok((cached, SessionSource::Cache));
    }

    let fingerprint = session_fingerprint()?;
    let sessions = scan_sessions()?;
    write_cached_sessions(&fingerprint, &sessions)?;
    Ok((sessions, SessionSource::Scan))
}

pub fn rebuild_session_index() -> Result<Vec<SessionSummary>> {
    let fingerprint = session_fingerprint()?;
    let sessions = scan_sessions()?;
    write_cached_sessions(&fingerprint, &sessions)?;
    Ok(sessions)
}

pub fn summarize_projects(sessions: &[SessionSummary]) -> Vec<ProjectSummary> {
    let mut grouped = std::collections::BTreeMap::<String, ProjectSummary>::new();

    for session in sessions {
        let key = normalize_path(&session.attributed_repo_root);
        let entry = grouped.entry(key).or_insert_with(|| ProjectSummary {
            repo_root: session.attributed_repo_root.clone(),
            session_count: 0,
            latest_started_at: session.started_at.clone(),
            latest_goal: session
                .summary
                .clone()
                .or_else(|| session.first_user_goal.clone()),
        });

        entry.session_count += 1;

        if session.started_at > entry.latest_started_at {
            entry.latest_started_at = session.started_at.clone();
            entry.latest_goal = session
                .summary
                .clone()
                .or_else(|| session.first_user_goal.clone());
            entry.repo_root = session.attributed_repo_root.clone();
        }
    }

    let mut projects: Vec<_> = grouped.into_values().collect();
    projects.sort_by(|left, right| right.latest_started_at.cmp(&left.latest_started_at));
    projects
}

pub fn search_sessions(
    sessions: &[SessionSummary],
    query: &str,
    repo_filter: Option<&Path>,
    limit: usize,
) -> Vec<SearchHit> {
    let trimmed_query = compact_text(query);
    if trimmed_query.is_empty() {
        return Vec::new();
    }

    let query_norm = trimmed_query.to_lowercase();
    let terms = query_norm
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let repo_key = repo_filter.map(normalize_path);

    let mut strict_hits = sessions
        .iter()
        .filter(|session| repo_matches(session, repo_key.as_deref()))
        .filter_map(|session| score_session(session, &query_norm, &terms, true))
        .collect::<Vec<_>>();

    sort_search_hits(&mut strict_hits);
    if !strict_hits.is_empty() {
        strict_hits.truncate(limit);
        return strict_hits;
    }

    let mut fallback_hits = sessions
        .iter()
        .filter(|session| repo_matches(session, repo_key.as_deref()))
        .filter_map(|session| score_session(session, &query_norm, &terms, false))
        .collect::<Vec<_>>();

    sort_search_hits(&mut fallback_hits);
    fallback_hits.truncate(limit);
    fallback_hits
}

pub fn find_session<'a>(
    sessions: &'a [SessionSummary],
    session_id: &str,
) -> Option<&'a SessionSummary> {
    let needle = session_id.trim().to_lowercase();
    sessions
        .iter()
        .find(|session| session.id.to_lowercase() == needle)
}

pub fn current_repo_root(explicit_repo: Option<&str>) -> Result<PathBuf> {
    let start = match explicit_repo {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().context("failed to read current directory")?,
    };

    Ok(find_repo_root(&start).unwrap_or(start))
}

fn parse_session_file(
    path: &Path,
    known_product_roots: &[PathBuf],
) -> Result<Option<SessionSummary>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open session file {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut session_id = String::new();
    let mut started_at = String::new();
    let mut cwd: Option<PathBuf> = None;
    let mut first_user_goal: Option<String> = None;
    let mut last_assistant_outcome: Option<String> = None;
    let mut user_messages = Vec::<String>::new();
    let mut assistant_messages = Vec::<String>::new();
    let mut mentioned_repo_roots = BTreeSet::<String>::new();
    let mut mentioned_files = BTreeSet::<String>::new();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        if line.contains("\"type\":\"session_meta\"") {
            if let Some(meta) = extract_session_meta_fields(&line) {
                if session_id.is_empty() {
                    session_id = meta.id;
                }

                if started_at.is_empty() {
                    started_at = meta.timestamp;
                }

                if cwd.is_none() {
                    cwd = Some(PathBuf::from(meta.cwd));
                }
            }
            continue;
        }

        if line.contains("\"type\":\"response_item\"")
            && line.contains("\"payload\":{\"type\":\"message\"")
        {
            collect_mentioned_repo_roots(&line, known_product_roots, &mut mentioned_repo_roots);
            collect_mentioned_files(&line, &mut mentioned_files);
            let text =
                extract_json_string(&line, "\"text\":\"").map(|text| unescape_json_string(&text));

            if line.contains("\"role\":\"user\"") {
                if let Some(text) = text.as_deref().and_then(sanitize_user_text) {
                    if first_user_goal.is_none() {
                        first_user_goal = Some(text.clone());
                    }
                    user_messages.push(text);
                }
            } else if line.contains("\"role\":\"assistant\"")
                && let Some(text) = text.as_deref().and_then(sanitize_assistant_text)
            {
                last_assistant_outcome = Some(text.clone());
                assistant_messages.push(text);
            }
        }
    }

    let cwd = match cwd {
        Some(cwd) => cwd,
        None => return Ok(None),
    };

    if session_id.is_empty() {
        if let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) {
            session_id = name
                .chars()
                .rev()
                .take(36)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
        }
    }

    let repo_root = find_repo_root(&cwd).unwrap_or_else(|| cwd.clone());
    let mentioned_repo_roots = mentioned_repo_roots
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mentioned_files = mentioned_files.into_iter().collect::<Vec<_>>();
    let attributed_repo_root = choose_attributed_repo_root(&repo_root, &cwd, &mentioned_repo_roots);
    let digest = derive_session_digest(
        &user_messages,
        &assistant_messages,
        first_user_goal.as_deref(),
        last_assistant_outcome.as_deref(),
    );

    Ok(Some(SessionSummary {
        id: session_id,
        started_at,
        cwd,
        repo_root,
        attributed_repo_root,
        mentioned_repo_roots,
        mentioned_files,
        first_user_goal,
        last_assistant_outcome,
        summary: digest.summary,
        verification_notes: digest.verification_notes,
        next_step: digest.next_step,
    }))
}

fn sanitize_user_text(text: &str) -> Option<String> {
    let compact = compact_text(text);

    if compact.is_empty()
        || compact.starts_with("# AGENTS.md instructions")
        || compact.starts_with("<environment_context>")
        || compact.starts_with("Warning: The maximum number of unified exec processes")
    {
        return None;
    }

    Some(compact)
}

fn sanitize_assistant_text(text: &str) -> Option<String> {
    let compact = compact_text(text);

    if compact.is_empty() {
        return None;
    }

    Some(compact)
}

#[derive(Debug, Default)]
struct SessionDigest {
    summary: Option<String>,
    verification_notes: Option<String>,
    next_step: Option<String>,
}

fn derive_session_digest(
    user_messages: &[String],
    assistant_messages: &[String],
    first_user_goal: Option<&str>,
    last_assistant_outcome: Option<&str>,
) -> SessionDigest {
    let summary = best_summary_candidate(assistant_messages)
        .or(last_assistant_outcome.map(str::to_owned))
        .or(first_user_goal.map(str::to_owned))
        .map(|text| limit_text(&text, 900));

    let verification_notes = collect_signal_summary(
        assistant_messages,
        &[
            "verified",
            "verification",
            "tested",
            "test",
            "passed",
            "checked",
            "confirmed",
            "ran",
            "smoke",
            "succeeded",
            "working",
        ],
        2,
        320,
    )
    .or_else(|| {
        collect_signal_summary(
            user_messages,
            &["tested", "verified", "confirmed", "checked"],
            1,
            220,
        )
    });

    let next_step = find_next_step(assistant_messages)
        .or_else(|| find_next_step(user_messages))
        .map(|text| limit_text(&text, 260));

    SessionDigest {
        summary,
        verification_notes,
        next_step,
    }
}

fn best_summary_candidate(assistant_messages: &[String]) -> Option<String> {
    assistant_messages
        .iter()
        .max_by_key(|message| summary_candidate_score(message))
        .and_then(|message| {
            if summary_candidate_score(message) == 0 {
                None
            } else {
                Some(message.clone())
            }
        })
}

fn summary_candidate_score(message: &str) -> usize {
    let lower = message.to_lowercase();
    let mut score = lower.len().min(450) / 6;

    for needle in [
        "here’s what we did",
        "here's what we did",
        "in this chat",
        "turn-by-turn",
        "what changed",
        "summary",
        "recap",
        "implemented",
        "added",
        "migrated",
        "rewrote",
        "fixed",
        "updated",
        "confirmed",
        "verified",
        "recommended",
        "next",
    ] {
        if lower.contains(needle) {
            score += 45;
        }
    }

    if lower.starts_with("don’t call me that") || lower.starts_with("don't call me that") {
        score = score.saturating_sub(120);
    }

    score
}

fn collect_signal_summary(
    messages: &[String],
    keywords: &[&str],
    limit: usize,
    max_len: usize,
) -> Option<String> {
    let mut picks = Vec::new();

    for message in messages.iter().rev() {
        for clause in message_clauses(message) {
            let lower = clause.to_lowercase();
            if keywords.iter().any(|keyword| lower.contains(keyword))
                && !picks
                    .iter()
                    .any(|existing: &String| existing.eq_ignore_ascii_case(&clause))
            {
                picks.push(limit_text(&clause, max_len));
                if picks.len() >= limit {
                    return Some(picks.join(" | "));
                }
            }
        }
    }

    if picks.is_empty() {
        None
    } else {
        Some(picks.join(" | "))
    }
}

fn find_next_step(messages: &[String]) -> Option<String> {
    for message in messages.iter().rev() {
        for clause in message_clauses(message) {
            let lower = clause.to_lowercase();
            if [
                "next",
                "recommended",
                "recommend",
                "follow-up",
                "remaining",
                "still needs",
                "should",
                "need to",
                "obvious move",
                "best move",
                "best next",
            ]
            .iter()
            .any(|needle| lower.contains(needle))
            {
                return Some(clause);
            }
        }
    }

    None
}

fn message_clauses(message: &str) -> Vec<String> {
    let with_breaks = message
        .replace(" - ", "\n- ")
        .replace(". ", ".\n")
        .replace("; ", ";\n");

    with_breaks
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_start_matches("- ").trim().to_owned())
        .filter(|line| line.len() > 12)
        .collect()
}

fn compact_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn limit_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_owned();
    }

    let clipped: String = text.chars().take(max_len.saturating_sub(1)).collect();
    format!("{clipped}…")
}

fn find_repo_root(path: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn codex_home() -> Result<PathBuf> {
    if let Ok(path) = env::var("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }

    let user_profile = env::var("USERPROFILE").context("USERPROFILE is not set")?;
    Ok(PathBuf::from(user_profile).join(".codex"))
}

fn continuity_home() -> Result<PathBuf> {
    if let Ok(path) = env::var("CCX_HOME") {
        return Ok(PathBuf::from(path));
    }

    let user_profile = env::var("USERPROFILE").context("USERPROFILE is not set")?;
    Ok(PathBuf::from(user_profile).join(".codex-continuity"))
}

fn cache_file_path() -> Result<PathBuf> {
    Ok(continuity_home()?.join("cache").join("session_index.tsv"))
}

fn session_fingerprint() -> Result<SessionFingerprint> {
    let sessions_root = codex_home()?.join("sessions");
    let mut file_count = 0usize;
    let mut latest_modified_epoch = 0u64;

    if !sessions_root.exists() {
        return Ok(SessionFingerprint {
            file_count,
            latest_modified_epoch,
        });
    }

    for entry in WalkDir::new(&sessions_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
    {
        file_count += 1;
        let modified_epoch = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        if modified_epoch > latest_modified_epoch {
            latest_modified_epoch = modified_epoch;
        }
    }

    Ok(SessionFingerprint {
        file_count,
        latest_modified_epoch,
    })
}

fn read_cached_sessions(
    fingerprint: Option<&SessionFingerprint>,
) -> Result<Option<Vec<SessionSummary>>> {
    let cache_file = cache_file_path()?;
    if !cache_file.exists() {
        return Ok(None);
    }

    let file = File::open(&cache_file)
        .with_context(|| format!("failed to open cache file {}", cache_file.display()))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let Some(header_line) = lines.next() else {
        return Ok(None);
    };
    let header_line = header_line?;

    if !cache_header_matches(&header_line, fingerprint) {
        return Ok(None);
    }

    let mut sessions = Vec::new();
    for line in lines {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if let Some(session) = parse_cached_session_line(&line) {
            sessions.push(session);
        }
    }

    sessions.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    Ok(Some(sessions))
}

fn write_cached_sessions(
    fingerprint: &SessionFingerprint,
    sessions: &[SessionSummary],
) -> Result<()> {
    let cache_file = cache_file_path()?;
    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache directory {}", parent.display()))?;
    }

    let mut output = String::new();
    output.push_str(&format!(
        "CCX2\t{}\t{}\n",
        fingerprint.file_count, fingerprint.latest_modified_epoch
    ));

    for session in sessions {
        output.push_str(&serialize_cached_session_line(session));
        output.push('\n');
    }

    fs::write(&cache_file, output)
        .with_context(|| format!("failed to write cache file {}", cache_file.display()))?;
    Ok(())
}

fn cache_header_matches(header: &str, fingerprint: Option<&SessionFingerprint>) -> bool {
    let mut parts = header.split('\t');
    let Some(version) = parts.next() else {
        return false;
    };
    let Some(file_count) = parts.next() else {
        return false;
    };
    let Some(latest_modified_epoch) = parts.next() else {
        return false;
    };

    if version != "CCX2" {
        return false;
    }

    match fingerprint {
        Some(fingerprint) => {
            file_count.parse::<usize>().ok() == Some(fingerprint.file_count)
                && latest_modified_epoch.parse::<u64>().ok()
                    == Some(fingerprint.latest_modified_epoch)
        }
        None => true,
    }
}

fn serialize_cached_session_line(session: &SessionSummary) -> String {
    [
        sanitize_cache_field(&session.id),
        sanitize_cache_field(&session.started_at),
        sanitize_cache_field(&session.cwd.display().to_string()),
        sanitize_cache_field(&session.repo_root.display().to_string()),
        sanitize_cache_field(&session.attributed_repo_root.display().to_string()),
        sanitize_cache_field(
            &session
                .mentioned_repo_roots
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("||"),
        ),
        sanitize_cache_field(&session.mentioned_files.join("||")),
        sanitize_cache_field(session.first_user_goal.as_deref().unwrap_or_default()),
        sanitize_cache_field(
            session
                .last_assistant_outcome
                .as_deref()
                .unwrap_or_default(),
        ),
        sanitize_cache_field(session.summary.as_deref().unwrap_or_default()),
        sanitize_cache_field(session.verification_notes.as_deref().unwrap_or_default()),
        sanitize_cache_field(session.next_step.as_deref().unwrap_or_default()),
    ]
    .join("\t")
}

fn parse_cached_session_line(line: &str) -> Option<SessionSummary> {
    let parts = line.splitn(12, '\t').collect::<Vec<_>>();
    if parts.len() != 12 {
        return None;
    }

    let mentioned_repo_roots = split_cached_list(parts[5])
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mentioned_files = split_cached_list(parts[6]);

    Some(SessionSummary {
        id: parts[0].to_owned(),
        started_at: parts[1].to_owned(),
        cwd: PathBuf::from(parts[2]),
        repo_root: PathBuf::from(parts[3]),
        attributed_repo_root: PathBuf::from(parts[4]),
        mentioned_repo_roots,
        mentioned_files,
        first_user_goal: empty_to_none(parts[7]),
        last_assistant_outcome: empty_to_none(parts[8]),
        summary: empty_to_none(parts[9]),
        verification_notes: empty_to_none(parts[10]),
        next_step: empty_to_none(parts[11]),
    })
}

fn sanitize_cache_field(value: &str) -> String {
    value
        .replace('\t', "    ")
        .replace('\n', " ")
        .replace('\r', " ")
}

fn split_cached_list(value: &str) -> Vec<String> {
    if value.trim().is_empty() {
        return Vec::new();
    }

    value
        .split("||")
        .filter(|item| !item.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn empty_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

#[derive(Debug, Clone, Copy)]
struct SessionFingerprint {
    file_count: usize,
    latest_modified_epoch: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session(id: &str, repo: &str, goal: &str, outcome: &str) -> SessionSummary {
        SessionSummary {
            id: id.to_owned(),
            started_at: "2026-04-04T00:00:00.000Z".to_owned(),
            cwd: PathBuf::from(repo),
            repo_root: PathBuf::from(repo),
            attributed_repo_root: PathBuf::from(repo),
            mentioned_repo_roots: vec![PathBuf::from(repo)],
            mentioned_files: vec![
                format!("{repo}/docs/PROMPT_PROFILES.md"),
                format!("{repo}/backend/app/core/config.py"),
            ],
            first_user_goal: Some(goal.to_owned()),
            last_assistant_outcome: Some(outcome.to_owned()),
            summary: Some(outcome.to_owned()),
            verification_notes: Some("Verified via smoke test".to_owned()),
            next_step: Some("Continue with the next milestone.".to_owned()),
        }
    }

    #[test]
    fn extract_session_meta_fields_parses_expected_values() {
        let line = r#"{"timestamp":"2026-04-04T00:00:00.000Z","type":"session_meta","payload":{"id":"019-test-session","timestamp":"2026-04-04T00:00:00.000Z","cwd":"D:\\saas-workspace\\products\\roompilot-ai","originator":"codex_cli"}}"#;

        let fields = extract_session_meta_fields(line).expect("session meta should parse");

        assert_eq!(fields.id, "019-test-session");
        assert_eq!(fields.timestamp, "2026-04-04T00:00:00.000Z");
        assert_eq!(fields.cwd, r#"D:\saas-workspace\products\roompilot-ai"#);
    }

    #[test]
    fn indirect_workspace_prefers_mentioned_repo_root() {
        let repo_root = PathBuf::from(r"D:\saas-workspace\templates\saas-mvp-template");
        let cwd = repo_root.clone();
        let mentioned = vec![PathBuf::from(r"D:\saas-workspace\products\roompilot-ai")];

        let attributed = choose_attributed_repo_root(&repo_root, &cwd, &mentioned);

        assert_eq!(
            normalize_path(&attributed),
            "d:/saas-workspace/products/roompilot-ai"
        );
    }

    #[test]
    fn direct_workspace_keeps_repo_root() {
        let repo_root = PathBuf::from(r"D:\saas-workspace\products\roompilot-ai");
        let cwd = repo_root.clone();
        let mentioned = vec![PathBuf::from(r"D:\saas-workspace\products\pix-finder")];

        let attributed = choose_attributed_repo_root(&repo_root, &cwd, &mentioned);

        assert_eq!(normalize_path(&attributed), normalize_path(&repo_root));
    }

    #[test]
    fn file_path_detection_filters_reasonable_candidates() {
        assert!(looks_like_file_path("backend/app/core/config.py"));
        assert!(looks_like_file_path("PROMPT_PROFILES.md"));
        assert!(!looks_like_file_path("roompilot-ai"));
        assert!(!looks_like_file_path("2026-04-04"));
    }

    #[test]
    fn search_sessions_respects_repo_filter_and_query_terms() {
        let roompilot = sample_session(
            "roompilot-session",
            r"D:\saas-workspace\products\roompilot-ai",
            "Recover roompilot-ai continuity",
            "PROMPT_PROFILES.md was updated during the FastAPI migration.",
        );
        let pixfinder = sample_session(
            "pixfinder-session",
            r"D:\saas-workspace\products\pix-finder",
            "Work on pix-finder",
            "No prompt profile changes here.",
        );
        let sessions = vec![roompilot, pixfinder];

        let hits = search_sessions(
            &sessions,
            "prompt profiles",
            Some(Path::new(r"D:\saas-workspace\products\roompilot-ai")),
            5,
        );

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].session.id, "roompilot-session");
    }

    #[test]
    fn cached_session_line_roundtrip_preserves_fields() {
        let original = sample_session(
            "roundtrip-session",
            r"D:\saas-workspace\products\roompilot-ai",
            "Goal text",
            "Outcome text",
        );

        let serialized = serialize_cached_session_line(&original);
        let parsed = parse_cached_session_line(&serialized).expect("cached line should parse");

        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.started_at, original.started_at);
        assert_eq!(parsed.mentioned_files, original.mentioned_files);
        assert_eq!(parsed.first_user_goal, original.first_user_goal);
        assert_eq!(
            parsed.last_assistant_outcome,
            original.last_assistant_outcome
        );
        assert_eq!(parsed.summary, original.summary);
        assert_eq!(parsed.verification_notes, original.verification_notes);
        assert_eq!(parsed.next_step, original.next_step);
    }

    #[test]
    fn derive_session_digest_prefers_recap_style_summary() {
        let digest = derive_session_digest(
            &["Need the exact repo continuity view.".to_owned()],
            &[
                "Done.".to_owned(),
                "Here’s what we did: rewrote the response contract, added SESSION_MAP.md, and wired the workflow docs so the repo resumes cleanly.".to_owned(),
            ],
            Some("Need the exact repo continuity view."),
            Some("Done."),
        );

        assert_eq!(
            digest.summary,
            Some(
                "Here’s what we did: rewrote the response contract, added SESSION_MAP.md, and wired the workflow docs so the repo resumes cleanly."
                    .to_owned()
            )
        );
    }

    #[test]
    fn derive_session_digest_extracts_verification_and_next_step() {
        let digest = derive_session_digest(
            &["Can you make the dashboard feel launch-ready?".to_owned()],
            &[
                "I verified the roompilot-ai flow with a live smoke test and confirmed the resume pack worked.".to_owned(),
                "The next obvious move is to tighten the dashboard detail pane and ship a cleaner first-run flow.".to_owned(),
            ],
            Some("Can you make the dashboard feel launch-ready?"),
            Some("The next obvious move is to tighten the dashboard detail pane and ship a cleaner first-run flow."),
        );

        assert_eq!(
            digest.verification_notes,
            Some(
                "I verified the roompilot-ai flow with a live smoke test and confirmed the resume pack worked."
                    .to_owned()
            )
        );
        assert_eq!(
            digest.next_step,
            Some(
                "The next obvious move is to tighten the dashboard detail pane and ship a cleaner first-run flow."
                    .to_owned()
            )
        );
    }
}

fn repo_matches(session: &SessionSummary, repo_key: Option<&str>) -> bool {
    match repo_key {
        Some(repo_key) => {
            normalize_path(&session.attributed_repo_root) == repo_key
                || normalize_path(&session.repo_root) == repo_key
        }
        None => true,
    }
}

fn sort_search_hits(hits: &mut [SearchHit]) {
    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.matched_terms.cmp(&left.matched_terms))
            .then_with(|| right.session.started_at.cmp(&left.session.started_at))
    });
}

fn score_session(
    session: &SessionSummary,
    query_norm: &str,
    terms: &[String],
    require_all_terms: bool,
) -> Option<SearchHit> {
    let fields = session_search_fields(session);
    let combined = fields
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let matched_terms = terms
        .iter()
        .filter(|term| combined.contains(term.as_str()))
        .count();
    let mut score = 0usize;
    let mut why = Vec::new();

    for (label, text) in &fields {
        if text.contains(query_norm) {
            let boost = match *label {
                "goal" => 90,
                "outcome" => 70,
                "summary" => 95,
                "verification" => 50,
                "next_step" => 55,
                "attributed_repo" => 65,
                "workspace_repo" => 40,
                "mentioned_repos" => 45,
                "session_id" => 80,
                _ => 20,
            };
            score += boost;
            why.push(format!("{label} matched the full query"));
        }
    }

    for (label, text) in &fields {
        let term_hits = terms
            .iter()
            .filter(|term| text.contains(term.as_str()))
            .count();
        if term_hits == 0 {
            continue;
        }

        let per_term = match *label {
            "goal" => 12,
            "outcome" => 9,
            "summary" => 13,
            "verification" => 7,
            "next_step" => 8,
            "attributed_repo" => 8,
            "workspace_repo" => 5,
            "mentioned_repos" => 6,
            "session_id" => 15,
            _ => 3,
        };
        score += term_hits * per_term;
        why.push(format!("{label} matched {term_hits} term(s)"));
    }

    let phrase_match = why.iter().any(|reason| reason.contains("full query"));
    if require_all_terms {
        if matched_terms < terms.len() && !phrase_match {
            return None;
        }
    } else if matched_terms == 0 && !phrase_match {
        return None;
    }

    score += matched_terms * 10;
    why.sort();
    why.dedup();

    Some(SearchHit {
        session: session.clone(),
        score,
        matched_terms,
        why,
    })
}

fn session_search_fields(session: &SessionSummary) -> Vec<(&'static str, String)> {
    vec![
        ("session_id", session.id.to_lowercase()),
        (
            "goal",
            session
                .first_user_goal
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        ),
        (
            "outcome",
            session
                .last_assistant_outcome
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        ),
        (
            "summary",
            session
                .summary
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        ),
        (
            "verification",
            session
                .verification_notes
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        ),
        (
            "next_step",
            session
                .next_step
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
        ),
        ("workspace_repo", normalize_path(&session.repo_root)),
        (
            "attributed_repo",
            normalize_path(&session.attributed_repo_root),
        ),
        (
            "mentioned_repos",
            session
                .mentioned_repo_roots
                .iter()
                .map(|path| normalize_path(path))
                .collect::<Vec<_>>()
                .join(" "),
        ),
    ]
}

fn discover_known_product_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let products_root = PathBuf::from(r"D:\saas-workspace\products");

    if let Ok(entries) = std::fs::read_dir(&products_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if !path.is_dir() || name.starts_with('.') || name.starts_with('_') {
                continue;
            }

            roots.push(path);
        }
    }

    roots
}

fn collect_mentioned_repo_roots(
    line: &str,
    known_product_roots: &[PathBuf],
    mentions: &mut BTreeSet<String>,
) {
    let normalized_line = line.to_lowercase();

    for root in known_product_roots {
        let root_string = root.to_string_lossy().to_string();
        let root_lower = root_string.to_lowercase();
        let repo_name = root
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if normalized_line.contains(&root_lower)
            || (!repo_name.is_empty() && normalized_line.contains(&repo_name))
        {
            mentions.insert(root_string);
        }
    }
}

fn collect_mentioned_files(line: &str, mentions: &mut BTreeSet<String>) {
    let mut current = String::new();

    for ch in line.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '/' | '\\' | '.' | '_' | '-') {
            current.push(ch);
            continue;
        }

        maybe_record_file_candidate(&current, mentions);
        current.clear();
    }

    maybe_record_file_candidate(&current, mentions);
}

fn maybe_record_file_candidate(candidate: &str, mentions: &mut BTreeSet<String>) {
    let cleaned = candidate
        .trim_matches(|ch: char| {
            !ch.is_ascii_alphanumeric() && !matches!(ch, ':' | '/' | '\\' | '.' | '_' | '-')
        })
        .trim_end_matches('.')
        .replace("\\\\", "\\");

    if cleaned.is_empty() || !looks_like_file_path(&cleaned) {
        return;
    }

    let normalized = cleaned.replace('\\', "/");
    mentions.insert(normalized);
}

fn looks_like_file_path(candidate: &str) -> bool {
    let lower = candidate.to_lowercase();
    let trimmed = lower
        .split_once(':')
        .map(|(path, suffix)| {
            if suffix.chars().all(|ch| ch.is_ascii_digit()) {
                path
            } else {
                &lower
            }
        })
        .unwrap_or(&lower);

    const FILE_EXTENSIONS: &[&str] = &[
        ".md", ".rs", ".toml", ".json", ".jsonl", ".py", ".ts", ".tsx", ".js", ".mjs", ".html",
        ".css", ".yaml", ".yml", ".txt", ".sql", ".sh", ".ps1",
    ];

    if !FILE_EXTENSIONS.iter().any(|ext| trimmed.ends_with(ext)) {
        return false;
    }

    trimmed.contains('/')
        || trimmed.contains('\\')
        || (trimmed.matches('.').count() == 1 && trimmed.len() > 4)
        || trimmed == "readme.md"
        || trimmed == "agents.md"
        || trimmed == "cargo.toml"
        || trimmed == "cargo.lock"
}

fn choose_attributed_repo_root(
    repo_root: &Path,
    cwd: &Path,
    mentioned_repo_roots: &[PathBuf],
) -> PathBuf {
    if is_indirect_workspace(repo_root, cwd) && !mentioned_repo_roots.is_empty() {
        return mentioned_repo_roots[0].clone();
    }

    repo_root.to_path_buf()
}

fn is_indirect_workspace(repo_root: &Path, cwd: &Path) -> bool {
    let repo = normalize_path(repo_root);
    let current = normalize_path(cwd);

    repo.contains("/.codex")
        || repo.contains("/.agents")
        || repo.contains("/.claude")
        || repo.contains("/templates/")
        || repo.contains("/worktrees/")
        || current.contains("/templates/")
        || current.contains("/worktrees/")
}

struct SessionMetaFields {
    id: String,
    timestamp: String,
    cwd: String,
}

fn extract_session_meta_fields(line: &str) -> Option<SessionMetaFields> {
    let payload_marker = "\"payload\":{\"id\":\"";
    let payload_start = line.find(payload_marker)? + payload_marker.len();
    let payload_tail = &line[payload_start..];

    let id = extract_until_quote(payload_tail)?;
    let timestamp_marker = "\",\"timestamp\":\"";
    let timestamp_start = payload_tail.find(timestamp_marker)? + timestamp_marker.len();
    let timestamp_tail = &payload_tail[timestamp_start..];
    let timestamp = extract_until_quote(timestamp_tail)?;

    let cwd_marker = "\",\"cwd\":\"";
    let cwd_start = timestamp_tail.find(cwd_marker)? + cwd_marker.len();
    let cwd_tail = &timestamp_tail[cwd_start..];
    let cwd = extract_until_quote(cwd_tail)?;

    Some(SessionMetaFields { id, timestamp, cwd })
}

fn extract_json_string(line: &str, needle: &str) -> Option<String> {
    let start = line.find(needle)? + needle.len();
    let rest = &line[start..];
    extract_until_quote(rest)
}

fn extract_until_quote(rest: &str) -> Option<String> {
    let mut result = String::new();
    let mut escaped = false;

    for ch in rest.chars() {
        if escaped {
            result.push(match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000C}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            return Some(result);
        }

        result.push(ch);
    }

    None
}

fn unescape_json_string(text: &str) -> String {
    text.to_owned()
}
