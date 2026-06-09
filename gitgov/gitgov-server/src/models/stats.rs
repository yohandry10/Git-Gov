use super::*;

// FILTERS & STATS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventFilter {
    pub source: Option<String>,
    pub event_type: Option<String>,
    pub user_login: Option<String>,
    pub branch: Option<String>,
    pub repo_full_name: Option<String>,
    pub org_name: Option<String>,
    /// UUID string — set internally by handlers to scope by API key org without a DB roundtrip.
    /// Takes precedence over org_name when org_name is absent.
    #[serde(default)]
    pub org_id: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    /// Keyset cursor timestamp (ms epoch) for `/logs` pagination.
    #[serde(default)]
    pub before_created_at: Option<i64>,
    /// Keyset cursor tie-breaker (event id as text UUID) for `/logs` pagination.
    #[serde(default)]
    pub before_id: Option<String>,
    #[serde(default)]
    pub limit: usize,
    /// Offset pagination is legacy for `/logs` and kept for backward compatibility.
    /// Prefer keyset cursor (`before_created_at` + `before_id`) for high-volume paths.
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct AuditStats {
    pub github_events: GitHubEventStats,
    pub client_events: ClientEventStats,
    pub violations: ViolationStats,
    #[serde(default)]
    pub pipeline: PipelineHealthStats,
    pub active_devs_week: i64,
    pub active_repos: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct DailyActivityPoint {
    pub day: String,
    pub commits: i64,
    pub pushes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyActivityQuery {
    #[serde(default)]
    pub days: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct GitHubEventStats {
    pub total: i64,
    pub today: i64,
    pub pushes_today: i64,
    #[serde(default)]
    pub by_type: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct ClientEventStats {
    pub total: i64,
    pub today: i64,
    pub blocked_today: i64,
    #[serde(default)]
    pub desktop_pushes_today: i64,
    #[serde(default)]
    pub by_type: HashMap<String, i64>,
    #[serde(default)]
    pub by_status: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct ViolationStats {
    pub total: i64,
    pub unresolved: i64,
    pub critical: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct PipelineHealthStats {
    pub total_7d: i64,
    pub success_7d: i64,
    pub failure_7d: i64,
    pub aborted_7d: i64,
    pub unstable_7d: i64,
    pub avg_duration_ms_7d: i64,
    pub repos_with_failures_7d: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CombinedEvent {
    pub id: String,
    pub source: String,
    pub event_type: String,
    pub created_at: i64,
    pub user_login: Option<String>,
    pub repo_name: Option<String>,
    pub branch: Option<String>,
    pub status: Option<String>,
    pub details: serde_json::Value,
}

// ============================================================================
