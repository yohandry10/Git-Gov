use super::*;

// GOVERNANCE EVENTS (Audit Log Streaming)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub delivery_id: String,
    pub event_type: String,
    pub actor_login: Option<String>,
    pub target: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub payload: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAuditLogEntry {
    #[serde(rename = "@timestamp")]
    pub timestamp: i64,
    pub action: String,
    pub actor: Option<String>,
    pub actor_location: Option<GitHubAuditActorLocation>,
    pub org: Option<String>,
    pub repo: Option<String>,
    pub repository: Option<String>,
    pub repository_id: Option<i64>,
    pub user: Option<String>,
    pub team: Option<String>,
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAuditActorLocation {
    pub country_code: Option<String>,
    pub country_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStreamBatch {
    pub entries: Vec<GitHubAuditLogEntry>,
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStreamResponse {
    pub accepted: i32,
    pub filtered: i32,
    pub errors: Vec<String>,
}

// ============================================================================
