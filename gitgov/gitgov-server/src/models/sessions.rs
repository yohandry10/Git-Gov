use super::*;

// ============================================================================
// CLIENT SESSIONS — T3.A
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientSession {
    pub client_id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    pub last_seen_at: i64,
    #[serde(default)]
    pub device_metadata: serde_json::Value,
    pub created_at: i64,
    /// true if last_seen_at is within the last 24 hours
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientSessionsResponse {
    pub sessions: Vec<ClientSession>,
    pub total: usize,
}

// ============================================================================
