use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JenkinsCorrelationFilter {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub user_login: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DailyActivityFilter {
    pub days: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PrMergeEvidenceFilter {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub merged_by: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrMergeEvidenceEntry {
    pub id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    pub delivery_id: String,
    pub pr_number: i32,
    #[serde(default)]
    pub pr_title: Option<String>,
    #[serde(default)]
    pub author_login: Option<String>,
    #[serde(default)]
    pub merged_by_login: Option<String>,
    #[serde(default)]
    pub approvers: Vec<String>,
    pub approvals_count: i32,
    #[serde(default)]
    pub head_sha: Option<String>,
    #[serde(default)]
    pub base_branch: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPipelineRun {
    pub pipeline_event_id: String,
    pub pipeline_id: String,
    pub job_name: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub triggered_by: Option<String>,
    pub ingested_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPipelineCorrelation {
    pub commit_event_id: String,
    pub commit_sha: String,
    pub commit_message: Option<String>,
    pub commit_created_at: i64,
    pub user_login: String,
    pub branch: Option<String>,
    pub repo_name: Option<String>,
    pub pipeline: Option<CommitPipelineRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TicketCoverageQuery {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraCorrelateRequest {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub hours: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraCorrelateResponse {
    pub scanned_commits: i64,
    pub correlations_created: i64,
    #[serde(default)]
    pub correlated_tickets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketCoverageResponse {
    pub org: String,
    pub period: String,
    pub total_commits: i64,
    pub commits_with_ticket: i64,
    pub coverage_percentage: f64,
    #[serde(default)]
    pub commits_without_ticket: Vec<serde_json::Value>,
    #[serde(default)]
    pub tickets_without_commits: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraTicketDetailResponse {
    pub found: bool,
    pub ticket: Option<ProjectTicket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EvidencePacketQuery {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketCompleteness {
    pub ticket_found: bool,
    pub commits: i64,
    pub pull_requests: i64,
    pub pipelines: i64,
    pub quality_gates: i64,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacket {
    pub packet_type: String,
    pub subject: String,
    pub generated_at: i64,
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub period: String,
    pub ticket: Option<ProjectTicket>,
    pub commits: Vec<serde_json::Value>,
    pub pull_requests: Vec<PrMergeEvidenceEntry>,
    pub quality_gates: Vec<CommitPipelineRun>,
    pub completeness: EvidencePacketCompleteness,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketResponse {
    pub found: bool,
    pub packet: Option<EvidencePacket>,
}
