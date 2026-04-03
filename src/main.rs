use anyhow::Result;
use clap::{Parser, Subcommand};

mod model;
mod scanner;

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Resume { repo } => {
            let (sessions, source) = scanner::load_sessions()?;
            let repo_root = scanner::current_repo_root(repo.as_deref())?;
            let repo_key = normalize_path(&repo_root);
            let matching: Vec<_> = sessions
                .iter()
                .filter(|session| normalize_path(&session.attributed_repo_root) == repo_key)
                .collect();

            println!("ccx resume");
            println!("repo: {}", repo_root.display());
            println!("session_source: {}", session_source_label(source));

            if let Some(best) = matching.first() {
                println!("best_session: {}", best.id);
                println!("started_at: {}", best.started_at);
                println!("cwd: {}", best.cwd.display());
                println!("workspace_repo: {}", best.repo_root.display());
                println!("attributed_repo: {}", best.attributed_repo_root.display());
                println!(
                    "goal: {}",
                    display_excerpt(best.first_user_goal.as_deref())
                );
                println!(
                    "last_outcome: {}",
                    display_excerpt(best.last_assistant_outcome.as_deref())
                );
                println!("recent_sessions_in_repo: {}", matching.len());
                if !best.mentioned_repo_roots.is_empty() {
                    println!(
                        "mentioned_repos: {}",
                        best.mentioned_repo_roots
                            .iter()
                            .map(|path| path.display().to_string())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    );
                }
            } else {
                println!("status: no known Codex sessions for this repo yet");
                println!("scanned_sessions: {}", sessions.len());
            }
        }
        Command::Find { query, repo, limit } => {
            let (sessions, source) = scanner::load_sessions()?;
            let repo_root = match repo.as_deref() {
                Some(path) => Some(scanner::current_repo_root(Some(path))?),
                None => None,
            };
            let hits = scanner::search_sessions(&sessions, &query, repo_root.as_deref(), limit);

            println!("ccx find");
            println!("query: {query}");
            println!("session_source: {}", session_source_label(source));
            if let Some(repo_root) = repo_root.as_ref() {
                println!("repo_filter: {}", repo_root.display());
            }
            println!("results: {}", hits.len());

            if hits.is_empty() {
                println!("status: no matching sessions found");
            } else {
                for hit in hits {
                    println!(
                        "- score={} | terms={} | {} | {} | {}",
                        hit.score,
                        hit.matched_terms,
                        hit.session.started_at,
                        hit.session.id,
                        hit.session.attributed_repo_root.display()
                    );
                    println!(
                        "  goal: {}",
                        display_excerpt(hit.session.first_user_goal.as_deref())
                    );
                    println!(
                        "  outcome: {}",
                        display_excerpt(hit.session.last_assistant_outcome.as_deref())
                    );
                    println!("  why: {}", hit.why.join(" | "));
                }
            }
        }
        Command::Compare {
            session_a,
            session_b,
        } => {
            let (sessions, source) = scanner::load_sessions()?;
            let left = scanner::find_session(&sessions, &session_a);
            let right = scanner::find_session(&sessions, &session_b);

            println!("ccx compare");
            println!("session_a: {session_a}");
            println!("session_b: {session_b}");
            println!("session_source: {}", session_source_label(source));

            let (left, right) = match (left, right) {
                (Some(left), Some(right)) => (left, right),
                (None, Some(_)) => {
                    println!("status: session_a not found");
                    return Ok(());
                }
                (Some(_), None) => {
                    println!("status: session_b not found");
                    return Ok(());
                }
                (None, None) => {
                    println!("status: neither session was found");
                    return Ok(());
                }
            };

            let same_repo =
                normalize_path(&left.attributed_repo_root) == normalize_path(&right.attributed_repo_root);
            let same_workspace_repo =
                normalize_path(&left.repo_root) == normalize_path(&right.repo_root);
            let left_focus_files = session_focus_files(left);
            let right_focus_files = session_focus_files(right);
            let common_files = common_values(&left_focus_files, &right_focus_files);
            let only_in_a = left_only_values(&left_focus_files, &right_focus_files);
            let only_in_b = left_only_values(&right_focus_files, &left_focus_files);
            let common_repos = common_path_values(&left.mentioned_repo_roots, &right.mentioned_repo_roots);
            let relation = infer_session_relation(left, right, same_repo, same_workspace_repo);

            println!("relation: {relation}");
            println!("same_attributed_repo: {same_repo}");
            println!("same_workspace_repo: {same_workspace_repo}");
            println!();
            println!("session_a_summary:");
            println!("  started_at: {}", left.started_at);
            println!("  workspace_repo: {}", left.repo_root.display());
            println!("  attributed_repo: {}", left.attributed_repo_root.display());
            println!("  goal: {}", display_excerpt(left.first_user_goal.as_deref()));
            println!(
                "  outcome: {}",
                display_excerpt(left.last_assistant_outcome.as_deref())
            );
            println!("  files: {}", preview_values(&left_focus_files, 8));
            println!();
            println!("session_b_summary:");
            println!("  started_at: {}", right.started_at);
            println!("  workspace_repo: {}", right.repo_root.display());
            println!("  attributed_repo: {}", right.attributed_repo_root.display());
            println!("  goal: {}", display_excerpt(right.first_user_goal.as_deref()));
            println!(
                "  outcome: {}",
                display_excerpt(right.last_assistant_outcome.as_deref())
            );
            println!("  files: {}", preview_values(&right_focus_files, 8));
            println!();
            println!(
                "common_files: {}",
                preview_values(&common_files, 10)
            );
            println!(
                "only_in_a: {}",
                preview_values(&only_in_a, 10)
            );
            println!(
                "only_in_b: {}",
                preview_values(&only_in_b, 10)
            );
            println!(
                "common_mentioned_repos: {}",
                preview_values(&common_repos, 8)
            );
        }
        Command::Pack { session, repo } => {
            let (sessions, source) = scanner::load_sessions()?;
            println!("ccx pack");
            println!("session: {}", session.as_deref().unwrap_or("auto"));
            println!("repo: {}", repo.as_deref().unwrap_or("auto"));
            println!("session_source: {}", session_source_label(source));

            let source = if let Some(session_id) = session.as_deref() {
                match scanner::find_session(&sessions, session_id) {
                    Some(found) => found,
                    None => {
                        println!("status: session not found");
                        return Ok(());
                    }
                }
            } else {
                let repo_root = scanner::current_repo_root(repo.as_deref())?;
                let repo_key = normalize_path(&repo_root);
                match sessions
                    .iter()
                    .find(|candidate| normalize_path(&candidate.attributed_repo_root) == repo_key)
                {
                    Some(found) => found,
                    None => {
                        println!("status: no known session for repo");
                        return Ok(());
                    }
                }
            };

            let related = sessions
                .iter()
                .filter(|candidate| {
                    normalize_path(&candidate.attributed_repo_root)
                        == normalize_path(&source.attributed_repo_root)
                })
                .take(3)
                .collect::<Vec<_>>();
            let context_anchor = related
                .iter()
                .max_by_key(|session| context_score(session))
                .copied()
                .unwrap_or(source);
            let mut files_that_matter = dedupe_values(
                &related
                    .iter()
                    .flat_map(|session| session_focus_files(session))
                    .collect::<Vec<_>>(),
            );
            files_that_matter.sort_by_key(|value| pack_file_priority(value));

            println!("source_session: {}", source.id);
            println!("started_at: {}", source.started_at);
            println!("workspace_repo: {}", source.repo_root.display());
            println!("attributed_repo: {}", source.attributed_repo_root.display());
            println!("related_sessions: {}", related.len());
            println!("context_anchor_session: {}", context_anchor.id);
            println!();
            println!("BEGIN_CCX_RESUME_PACK");
            println!("Repo: {}", source.attributed_repo_root.display());
            println!("Latest session: {}", source.id);
            println!("Latest session started at: {}", source.started_at);
            println!("Context anchor session: {}", context_anchor.id);
            println!(
                "Current goal: {}",
                excerpt_or_default(source.first_user_goal.as_deref(), 320, "No meaningful user goal extracted.")
            );
            println!(
                "Best continuity summary: {}",
                excerpt_or_default(
                    context_anchor.last_assistant_outcome.as_deref(),
                    700,
                    "No meaningful assistant outcome extracted."
                )
            );
            println!("Recent related sessions:");
            for session in &related {
                println!(
                    "- {} | {} | {}",
                    session.started_at,
                    session.id,
                    excerpt_or_default(
                        session.first_user_goal.as_deref(),
                        110,
                        "no meaningful user goal extracted"
                    )
                );
            }
            println!("Files that mattered:");
            for file in files_that_matter.iter().take(10) {
                println!("- {file}");
            }
            println!("Suggested resume prompt:");
            println!(
                "Continue work on `{}`. Use latest session `{}` as the most recent checkpoint and context anchor session `{}` as the richest historical summary. Review the listed files before making new changes.",
                source.attributed_repo_root.display(),
                source.id,
                context_anchor.id
            );
            println!("END_CCX_RESUME_PACK");
        }
        Command::Sessions => {
            let (sessions, source) = scanner::load_sessions()?;
            println!("ccx sessions");
            println!("session_source: {}", session_source_label(source));
            println!("count: {}", sessions.len());

            for session in sessions.iter().take(10) {
                println!(
                    "- {} | {} | {} | {} | {}",
                    session.started_at,
                    session.id,
                    session.attributed_repo_root.display(),
                    session.repo_root.display(),
                    session
                        .first_user_goal
                        .as_deref()
                        .map(|text| scanner::limit_text(text, 140))
                        .unwrap_or_else(|| "no meaningful user goal extracted".to_owned())
                );
            }
        }
        Command::Projects => {
            let (sessions, source) = scanner::load_sessions()?;
            let projects = scanner::summarize_projects(&sessions);
            println!("ccx projects");
            println!("session_source: {}", session_source_label(source));
            println!("count: {}", projects.len());

            for project in projects.iter().take(10) {
                println!(
                    "- {} | sessions={} | {} | {}",
                    project.latest_started_at,
                    project.session_count,
                    project.repo_root.display(),
                    project
                        .latest_goal
                        .as_deref()
                        .map(|text| scanner::limit_text(text, 140))
                        .unwrap_or_else(|| "no meaningful user goal extracted".to_owned())
                );
            }
        }
        Command::Index => {
            let sessions = scanner::rebuild_session_index()?;
            println!("ccx index");
            println!("status: rebuilt");
            println!("sessions_indexed: {}", sessions.len());
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    name = "ccx",
    version,
    about = "Codex Continuity OS CLI",
    long_about = "Local-first continuity layer for Codex sessions."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Recover the best resume context for the current repo.
    Resume {
        /// Optional repo path to inspect instead of the current directory.
        #[arg(long)]
        repo: Option<String>,
    },
    /// Search Codex session history.
    Find {
        /// Text query to search for.
        query: String,
        /// Optional repo path to limit search to one project.
        #[arg(long)]
        repo: Option<String>,
        /// Maximum number of matches to return.
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },
    /// Compare two Codex sessions.
    Compare {
        /// First session id.
        session_a: String,
        /// Second session id.
        session_b: String,
    },
    /// Generate a compact resume pack.
    Pack {
        /// Optional session id to force a specific pack source.
        #[arg(long)]
        session: Option<String>,
        /// Optional repo path to build a pack for when session is not provided.
        #[arg(long)]
        repo: Option<String>,
    },
    /// List known sessions.
    Sessions,
    /// List known projects.
    Projects,
    /// Refresh the local index.
    Index,
}

fn normalize_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn session_source_label(source: scanner::SessionSource) -> &'static str {
    match source {
        scanner::SessionSource::Cache => "cache",
        scanner::SessionSource::Scan => "scan",
    }
}

fn display_excerpt(text: Option<&str>) -> String {
    excerpt_or_default(text, 140, "no meaningful text extracted")
}

fn preview_values(values: &[String], limit: usize) -> String {
    if values.is_empty() {
        return "none".to_owned();
    }

    let mut preview = values.iter().take(limit).cloned().collect::<Vec<_>>();
    if values.len() > limit {
        preview.push(format!("+{} more", values.len() - limit));
    }
    preview.join(" | ")
}

fn common_values(left: &[String], right: &[String]) -> Vec<String> {
    let right_set = right
        .iter()
        .map(|value| value.to_lowercase())
        .collect::<std::collections::BTreeSet<_>>();

    left.iter()
        .filter(|value| right_set.contains(&value.to_lowercase()))
        .cloned()
        .collect()
}

fn left_only_values(left: &[String], right: &[String]) -> Vec<String> {
    let right_set = right
        .iter()
        .map(|value| value.to_lowercase())
        .collect::<std::collections::BTreeSet<_>>();

    left.iter()
        .filter(|value| !right_set.contains(&value.to_lowercase()))
        .cloned()
        .collect()
}

fn common_path_values(
    left: &[std::path::PathBuf],
    right: &[std::path::PathBuf],
) -> Vec<String> {
    let right_set = right
        .iter()
        .map(|path| normalize_path(path))
        .collect::<std::collections::BTreeSet<_>>();

    left.iter()
        .filter(|path| right_set.contains(&normalize_path(path)))
        .map(|path| path.display().to_string())
        .collect()
}

fn infer_session_relation(
    left: &model::SessionSummary,
    right: &model::SessionSummary,
    same_repo: bool,
    same_workspace_repo: bool,
) -> &'static str {
    if same_repo {
        if left.started_at <= right.started_at {
            "same attributed repo; session_b looks like the later continuation"
        } else {
            "same attributed repo; session_a looks like the later continuation"
        }
    } else if same_workspace_repo {
        "same workspace repo, but attributed to different downstream contexts"
    } else {
        "different repos or contexts"
    }
}

fn dedupe_values(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered = Vec::new();

    for value in values {
        let key = value.to_lowercase();
        if seen.insert(key) {
            ordered.push(value.clone());
        }
    }

    ordered
}

fn excerpt_or_default(text: Option<&str>, max_len: usize, fallback: &str) -> String {
    text.map(|value| scanner::limit_text(value, max_len))
        .unwrap_or_else(|| fallback.to_owned())
}

fn session_focus_files(session: &model::SessionSummary) -> Vec<String> {
    let attributed_root = normalize_path(&session.attributed_repo_root);
    let workspace_root = normalize_path(&session.repo_root);

    let focused = session
        .mentioned_files
        .iter()
        .filter(|value| is_repo_file_candidate(value, &attributed_root, &workspace_root))
        .cloned()
        .collect::<Vec<_>>();

    if focused.is_empty() {
        session
            .mentioned_files
            .iter()
            .take(20)
            .cloned()
            .collect()
    } else {
        focused
    }
}

fn is_repo_file_candidate(value: &str, attributed_root: &str, workspace_root: &str) -> bool {
    let normalized = value.replace('\\', "/").to_lowercase();

    if normalized.contains("/.agents/skills/")
        || normalized.contains("/.codex/skills/")
        || normalized.contains("/.codex/memories/")
    {
        return false;
    }

    if normalized.contains(attributed_root) || normalized.contains(workspace_root) {
        return true;
    }

    normalized.starts_with("./")
        || normalized.starts_with("src/")
        || normalized.starts_with("docs/")
        || normalized.starts_with("backend/")
        || normalized.starts_with("frontend/")
        || normalized.starts_with("scripts/")
        || normalized.starts_with("app/")
        || matches!(
            normalized.as_str(),
            "readme.md" | "agents.md" | "cargo.toml" | "cargo.lock" | "continuity.md"
        )
}

fn context_score(session: &model::SessionSummary) -> usize {
    session
        .last_assistant_outcome
        .as_deref()
        .map(|text| text.len())
        .unwrap_or(0)
        + session.mentioned_files.len() * 4
}

fn pack_file_priority(value: &str) -> (usize, String) {
    let lower = value.replace('\\', "/").to_lowercase();
    let bucket = if lower.contains("/backend/")
        || lower.contains("/frontend/")
        || lower.contains("/src/")
    {
        0
    } else if lower.ends_with("/state_of_play.md")
        || lower.ends_with("/prompt_profiles.md")
        || lower.ends_with("/agents.md")
        || lower.ends_with("/architecture.md")
        || lower.ends_with("/continuity.md")
    {
        1
    } else if lower.contains("/docs/") {
        2
    } else if lower.contains("/.agent/compare/") {
        3
    } else if lower.contains("/.agent/e2e/") {
        4
    } else if lower.contains("/.agent/history/") {
        6
    } else if lower.starts_with("./scripts/") {
        7
    } else {
        5
    };

    (bucket, lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_session(files: Vec<&str>) -> model::SessionSummary {
        model::SessionSummary {
            id: "session".to_owned(),
            started_at: "2026-04-04T00:00:00.000Z".to_owned(),
            cwd: PathBuf::from(r"D:\saas-workspace\products\roompilot-ai"),
            repo_root: PathBuf::from(r"D:\saas-workspace\templates\saas-mvp-template"),
            attributed_repo_root: PathBuf::from(r"D:\saas-workspace\products\roompilot-ai"),
            mentioned_repo_roots: vec![PathBuf::from(r"D:\saas-workspace\products\roompilot-ai")],
            mentioned_files: files.into_iter().map(str::to_owned).collect(),
            first_user_goal: Some("Goal".to_owned()),
            last_assistant_outcome: Some("Outcome".to_owned()),
        }
    }

    #[test]
    fn session_focus_files_drops_global_skill_noise() {
        let session = sample_session(vec![
            "C:/Users/AKR/.agents/skills/agent-change-walkthrough/SKILL.md",
            "D:/saas-workspace/products/roompilot-ai/backend/app/core/config.py",
            "D:/saas-workspace/products/roompilot-ai/frontend/src/App.tsx",
        ]);

        let focused = session_focus_files(&session);

        assert_eq!(focused.len(), 2);
        assert!(focused.iter().all(|value| !value.contains("/.agents/skills/")));
    }

    #[test]
    fn dedupe_values_preserves_first_seen_order() {
        let values = vec![
            "README.md".to_owned(),
            "src/main.rs".to_owned(),
            "readme.md".to_owned(),
        ];

        let deduped = dedupe_values(&values);

        assert_eq!(deduped, vec!["README.md".to_owned(), "src/main.rs".to_owned()]);
    }

    #[test]
    fn pack_file_priority_prefers_code_before_history_noise() {
        let code_priority =
            pack_file_priority("D:/saas-workspace/products/roompilot-ai/backend/app/core/config.py");
        let history_priority = pack_file_priority(
            "D:/saas-workspace/products/roompilot-ai/.agent/history/api/index.jsonl",
        );

        assert!(code_priority < history_priority);
    }
}
