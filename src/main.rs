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
                    best.first_user_goal
                        .as_deref()
                        .unwrap_or("no meaningful user goal extracted")
                );
                println!(
                    "last_outcome: {}",
                    best.last_assistant_outcome
                        .as_deref()
                        .unwrap_or("no assistant outcome extracted")
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
        Command::Find { query } => {
            println!("ccx find");
            println!("query: {query}");
            println!("status: scaffolded");
        }
        Command::Compare {
            session_a,
            session_b,
        } => {
            println!("ccx compare");
            println!("session_a: {session_a}");
            println!("session_b: {session_b}");
            println!("status: scaffolded");
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
                        .unwrap_or("no meaningful user goal extracted")
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
                        .unwrap_or("no meaningful user goal extracted")
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
