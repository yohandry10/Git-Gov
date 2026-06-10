use super::*;

// JIRA / TICKET COVERAGE (V1.2-B groundwork)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTicket {
    pub id: String,
    pub org_id: Option<String>,
    pub ticket_id: String,
    pub ticket_url: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub reporter: Option<String>,
    pub priority: Option<String>,
    pub ticket_type: Option<String>,
    #[serde(default)]
    pub related_commits: Vec<String>,
    #[serde(default)]
    pub related_prs: Vec<String>,
    #[serde(default)]
    pub related_branches: Vec<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub ingested_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTicketCorrelation {
    pub id: String,
    pub org_id: Option<String>,
    pub commit_sha: String,
    pub ticket_id: String,
    pub correlation_source: String,
    pub confidence: f64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWebhookEvent {
    #[serde(default)]
    pub webhook_event: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub issue: Option<serde_json::Value>,
    #[serde(default)]
    pub user: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWebhookIngestResponse {
    pub accepted: bool,
    pub duplicate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraIntegrationStatusResponse {
    pub ok: bool,
    #[serde(default)]
    pub last_ingest_at: Option<i64>,
    #[serde(default)]
    pub recent_tickets_24h: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraTicketDetailResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<ProjectTicket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraTicketDetailQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketCoverageResponse {
    pub org: String,
    pub period: String,
    pub total_commits: i64,
    /// Commits whose detected ticket id was verified against an ingested Jira
    /// `project_tickets` row. Only verified commits count toward coverage.
    pub commits_with_ticket: i64,
    /// Verified coverage percentage (`commits_with_ticket / total_commits`).
    /// This is the value consumed by the release readiness gate.
    pub coverage_percentage: f64,
    /// Commits that carry a pattern-detected ticket reference that does NOT
    /// match any ingested Jira ticket. These are unverified and never raise
    /// the verified coverage; they are surfaced so operators can see the gap.
    #[serde(default)]
    pub detected_unverified_commits: i64,
    /// Percentage of commits with a pattern-detected but unverified ticket.
    #[serde(default)]
    pub unverified_coverage_percentage: f64,
    #[serde(default)]
    pub commits_without_ticket: Vec<serde_json::Value>,
    #[serde(default)]
    pub tickets_without_commits: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraCorrelateRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub hours: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraCorrelateResponse {
    pub scanned_commits: i64,
    #[serde(default)]
    pub scanned_prs: i64,
    pub correlations_created: i64,
    #[serde(default)]
    pub correlated_tickets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketCoverageQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrelationV2Query {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub hours: Option<i64>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    /// Server-side org scope binding. Never deserialized from the client so a
    /// scoped key cannot inject another org; set only after scope validation.
    #[serde(skip)]
    pub org_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketFlowCorrelation {
    pub ticket_id: String,
    pub ticket_status: Option<String>,
    pub correlation_source: Option<String>,
    pub correlation_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_source: Option<String>,
    pub commit_sha: String,
    pub branch: Option<String>,
    pub user_login: Option<String>,
    pub repo_name: Option<String>,
    pub commit_created_at: Option<i64>,
    pub pipeline: Option<CommitPipelineRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrelationV2Response {
    #[serde(default)]
    pub items: Vec<TicketFlowCorrelation>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
