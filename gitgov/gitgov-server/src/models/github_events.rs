use super::*;

// ============================================================================
// GITHUB EVENTS (Source of Truth - from webhooks)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub delivery_id: String,
    pub event_type: String,
    pub actor_login: Option<String>,
    pub actor_id: Option<i64>,
    pub ref_name: Option<String>,
    pub ref_type: Option<String>,
    pub before_sha: Option<String>,
    pub after_sha: Option<String>,
    pub commit_shas: Vec<String>,
    pub commits_count: i32,
    pub payload: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub delivery_id: String,
    pub event_type: String,
    pub signature: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEvent {
    pub r#ref: String,
    pub before: String,
    pub after: String,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
    pub commits: Vec<GitHubCommit>,
    /// GitHub sets this to true when the push rewrites history (git push --force / --force-with-lease).
    #[serde(default)]
    pub forced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvent {
    pub r#ref: String,
    pub ref_type: String,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub private: bool,
    pub organization: Option<GitHubOrganization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOrganization {
    pub login: String,
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub id: String,
    pub message: String,
    pub author: GitHubCommitAuthor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommitAuthor {
    pub name: String,
    pub email: String,
}
