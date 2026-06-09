use super::*;

// GDPR — T2
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraseUserResponse {
    pub user_login: String,
    pub client_events_erased: i64,
    pub github_events_erased: i64,
    pub erased_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserResponse {
    pub user_login: String,
    pub events: Vec<CombinedEvent>,
    pub total: usize,
    pub exported_at: i64,
}
