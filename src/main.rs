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
            let sessions = scanner::scan_sessions()?;
            let repo_root = scanner::current_repo_root(repo.as_deref())?;
            let repo_key = normalize_path(&repo_root);
            let matching: Vec<_> = sessions
                .iter()
                .filter(|session| normalize_path(&session.attributed_repo_root) == repo_key)
                .collect();

            println!("ccx resume");
            println!("repo: {}", repo_root.display());

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
            let sessions = scanner::scan_sessions()?;
            let repo_root = match repo.as_deref() {
                Some(path) => Some(scanner::current_repo_root(Some(path))?),
                None => None,
            };
            let hits = scanner::search_sessions(&sessions, &query, repo_root.as_deref(), limit);

            println!("ccx find");
            println!("query: {query}");
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
            let sessions = scanner::scan_sessions()?;
            let left = scanner::find_session(&sessions, &session_a);
            let right = scanner::find_session(&sessions, &session_b);

            println!("ccx compare");
            println!("session_a: {session_a}");
            println!("session_b: {session_b}");

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
        Command::Pack { session } => {
            println!("ccx pack");
            println!("session: {}", session.as_deref().unwrap_or("auto"));
            println!("status: scaffolded");
        }
        Command::Sessions => {
            let sessions = scanner::scan_sessions()?;
            println!("ccx sessions");
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
            let sessions = scanner::scan_sessions()?;
            let projects = scanner::summarize_projects(&sessions);
            println!("ccx projects");
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
            let sessions = scanner::scan_sessions()?;
            println!("ccx index");
            println!("status: scanned");
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

fn display_excerpt(text: Option<&str>) -> String {
    text.map(|value| scanner::limit_text(value, 140))
        .unwrap_or_else(|| "no meaningful text extracted".to_owned())
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
