use super::*;

// ============================================================================
// POLICY DRIFT AUDIT — dedicated audit trail for policy sync/push/drift snapshot
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDriftEventInput {
    /// "sync_local" | "push_local" | "drift_snapshot"
    pub action: String,
    pub repo_name: String,
    /// "success" | "failed" | "observed"
    pub result: String,
    #[serde(default)]
    pub before_checksum: Option<String>,
    #[serde(default)]
    pub after_checksum: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDriftEventResponse {
    pub accepted: bool,
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDriftEventRecord {
    pub id: String,
    pub org_id: Option<String>,
    pub user_login: String,
    pub action: String,
    pub repo_name: String,
    pub result: String,
    pub before_checksum: Option<String>,
    pub after_checksum: Option<String>,
    pub duration_ms: Option<i64>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}
