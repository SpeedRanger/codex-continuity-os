use std::{
    collections::BTreeSet,
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::model::{ProjectSummary, SearchHit, SessionSummary};

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

pub fn summarize_projects(sessions: &[SessionSummary]) -> Vec<ProjectSummary> {
    let mut grouped = std::collections::BTreeMap::<String, ProjectSummary>::new();

    for session in sessions {
        let key = normalize_path(&session.attributed_repo_root);
        let entry = grouped.entry(key).or_insert_with(|| ProjectSummary {
            repo_root: session.attributed_repo_root.clone(),
            session_count: 0,
            latest_started_at: session.started_at.clone(),
            latest_goal: session.first_user_goal.clone(),
        });

        entry.session_count += 1;

        if session.started_at > entry.latest_started_at {
            entry.latest_started_at = session.started_at.clone();
            entry.latest_goal = session.first_user_goal.clone();
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

pub fn find_session<'a>(sessions: &'a [SessionSummary], session_id: &str) -> Option<&'a SessionSummary> {
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

fn parse_session_file(path: &Path, known_product_roots: &[PathBuf]) -> Result<Option<SessionSummary>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open session file {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut session_id = String::new();
    let mut started_at = String::new();
    let mut cwd: Option<PathBuf> = None;
    let mut first_user_goal: Option<String> = None;
    let mut last_assistant_outcome: Option<String> = None;
    let mut mentioned_repo_roots = BTreeSet::<String>::new();

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
            let text = extract_json_string(&line, "\"text\":\"").map(|text| unescape_json_string(&text));

            if line.contains("\"role\":\"user\"") {
                if first_user_goal.is_none() {
                    first_user_goal = text.as_deref().and_then(sanitize_user_text);
                }
            } else if line.contains("\"role\":\"assistant\"")
                && let Some(text) = text.as_deref().and_then(sanitize_assistant_text)
            {
                last_assistant_outcome = Some(text);
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
    let attributed_repo_root =
        choose_attributed_repo_root(&repo_root, &cwd, &mentioned_repo_roots);

    Ok(Some(SessionSummary {
        id: session_id,
        started_at,
        cwd,
        repo_root,
        attributed_repo_root,
        mentioned_repo_roots,
        first_user_goal,
        last_assistant_outcome,
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

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
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
            "workspace_repo",
            normalize_path(&session.repo_root),
        ),
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

        if normalized_line.contains(&root_lower) || (!repo_name.is_empty() && normalized_line.contains(&repo_name)) {
            mentions.insert(root_string);
        }
    }
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
