use super::*;

// TEAM MANAGEMENT (V1.5-B)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamOverviewQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub days: Option<i64>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamRepoSummary {
    pub repo_name: String,
    pub events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamDeveloperOverview {
    pub login: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    #[serde(default)]
    pub last_seen: Option<i64>,
    pub total_events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub repos_active_count: i64,
    #[serde(default)]
    pub repos: Vec<TeamRepoSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamOverviewResponse {
    pub entries: Vec<TeamDeveloperOverview>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamRepoOverview {
    pub repo_name: String,
    pub developers_active: i64,
    pub total_events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamReposResponse {
    pub entries: Vec<TeamRepoOverview>,
    pub total: i64,
}

// ============================================================================
