use super::*;

// PIPELINE EVENTS (V1.2-A Jenkins Integration)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEvent {
    pub id: String,
    pub org_id: Option<String>,
    pub pipeline_id: String,
    pub job_name: String,
    pub status: PipelineStatus,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub repo_full_name: Option<String>,
    pub duration_ms: Option<i64>,
    pub triggered_by: Option<String>,
    #[serde(default)]
    pub stages: Vec<PipelineStage>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub ingested_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Success,
    Failure,
    Aborted,
    Unstable,
}

impl PipelineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipelineStatus::Success => "success",
            PipelineStatus::Failure => "failure",
            PipelineStatus::Aborted => "aborted",
            PipelineStatus::Unstable => "unstable",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "success" => Some(PipelineStatus::Success),
            "failure" => Some(PipelineStatus::Failure),
            "aborted" => Some(PipelineStatus::Aborted),
            "unstable" => Some(PipelineStatus::Unstable),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PipelineStage {
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JenkinsPipelineEventInput {
    #[serde(default)]
    pub org_name: Option<String>,
    pub pipeline_id: String,
    pub job_name: String,
    pub status: String,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub triggered_by: Option<String>,
    #[serde(default)]
    pub stages: Vec<PipelineStage>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JenkinsPipelineEventResponse {
    pub accepted: bool,
    pub duplicate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JenkinsIntegrationStatusResponse {
    pub ok: bool,
    #[serde(default)]
    pub last_ingest_at: Option<i64>,
    #[serde(default)]
    pub recent_events_24h: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JenkinsCorrelationFilter {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub user_login: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPipelineRun {
    pub pipeline_event_id: String,
    pub pipeline_id: String,
    pub job_name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_full_name: Option<String>,
    pub duration_ms: Option<i64>,
    pub triggered_by: Option<String>,
    pub ingested_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JenkinsCorrelationsResponse {
    pub correlations: Vec<CommitPipelineCorrelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PolicyCheckRequest {
    pub repo: String,
    #[serde(default)]
    pub commit: Option<String>,
    pub branch: String,
    #[serde(default)]
    pub user_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct PolicyCheckResponse {
    pub advisory: bool,
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub evaluated_rules: Vec<String>,
    #[serde(default)]
    pub enforcement_applied: String,
    #[serde(default)]
    pub violations: Vec<RuleViolation>,
    #[serde(default)]
    pub external_decisions: Vec<ExternalPolicyDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RuleViolation {
    pub rule: String,
    pub category: String,
    pub enforcement: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct ExternalPolicyDecision {
    pub adapter: String,
    pub status: String,
    pub allowed: Option<bool>,
    pub enforcement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    pub decision_path: String,
    pub latency_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
