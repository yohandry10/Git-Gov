use super::*;

// ADMIN AUDIT LOG (V1.3-A)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditLogEntry {
    pub id: String,
    pub actor_client_id: String,
    pub action: String,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminAuditLogResponse {
    pub entries: Vec<AdminAuditLogEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminAuditLogQuery {
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

// ============================================================================
