use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub started_at: String,
    pub cwd: PathBuf,
    pub repo_root: PathBuf,
    pub attributed_repo_root: PathBuf,
    pub mentioned_repo_roots: Vec<PathBuf>,
    pub mentioned_files: Vec<String>,
    pub first_user_goal: Option<String>,
    pub last_assistant_outcome: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub repo_root: PathBuf,
    pub session_count: usize,
    pub latest_started_at: String,
    pub latest_goal: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub session: SessionSummary,
    pub score: usize,
    pub matched_terms: usize,
    pub why: Vec<String>,
}
