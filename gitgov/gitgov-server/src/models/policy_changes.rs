use super::*;

// POLICY HISTORY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyHistory {
    pub id: String,
    pub repo_id: String,
    pub config: GitGovConfig,
    pub checksum: String,
    #[serde(default)]
    pub source: PolicySourceMetadata,
    pub changed_by: String,
    pub change_type: String,
    pub previous_checksum: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeRequestInput {
    pub config: GitGovConfig,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyChangeRequestDecisionInput {
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeRequestRecord {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: String,
    pub repo_name: String,
    pub requested_by: String,
    pub requested_checksum: String,
    pub requested_config: GitGovConfig,
    pub reason: Option<String>,
    pub status: String, // pending | approved | rejected
    pub decided_by: Option<String>,
    pub decision_note: Option<String>,
    pub created_at: i64,
    pub decided_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeRequestCreateResponse {
    pub accepted: bool,
    pub request_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeRequestListResponse {
    pub requests: Vec<PolicyChangeRequestRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
