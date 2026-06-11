use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportResponse {
    pub id: String,
    pub export_type: String,
    pub record_count: i32,
    pub content_hash: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportLogEntry {
    pub id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    pub exported_by: String,
    pub export_type: String,
    #[serde(default)]
    pub date_range_start: Option<i64>,
    #[serde(default)]
    pub date_range_end: Option<i64>,
    pub filters: serde_json::Value,
    pub record_count: i32,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
    pub created_at: i64,
}

// ============================================================================
// CHAT STRUCTS (must mirror server models.rs ChatAskRequest / ChatAskResponse)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatAskRequest {
    pub question: String,
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatAskResponse {
    pub status: String,
    pub answer: String,
    #[serde(default)]
    pub missing_capability: Option<String>,
    pub can_report_feature: bool,
    #[serde(default)]
    pub data_refs: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub entities_detected: Vec<String>,
    #[serde(default)]
    pub time_range_used: Option<String>,
    #[serde(default)]
    pub actions_recommended: Vec<String>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureRequestInput {
    pub question: String,
    #[serde(default)]
    pub missing_capability: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub user_login: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureRequestCreated {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliCommandInput {
    #[serde(default)]
    pub org_name: Option<String>,
    pub command: String,
    pub origin: String,
    pub branch: String,
    #[serde(default)]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliCommandResponse {
    pub accepted: bool,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliCommandRecord {
    pub id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    pub user_login: String,
    pub command: String,
    pub origin: String,
    pub branch: String,
    #[serde(default)]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliCommandListResponse {
    #[serde(default)]
    pub commands: Vec<CliCommandRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
