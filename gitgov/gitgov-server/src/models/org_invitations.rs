use super::*;

// ORG INVITATIONS (V1.5-A)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgInvitation {
    pub id: String,
    pub org_id: String,
    #[serde(default)]
    pub invite_email: Option<String>,
    #[serde(default)]
    pub invite_login: Option<String>,
    pub role: String,
    pub status: String,
    pub invited_by: String,
    #[serde(default)]
    pub accepted_by: Option<String>,
    #[serde(default)]
    pub accepted_at: Option<i64>,
    #[serde(default)]
    pub revoked_by: Option<String>,
    #[serde(default)]
    pub revoked_at: Option<i64>,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgInvitationRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub invite_email: Option<String>,
    #[serde(default)]
    pub invite_login: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgInvitationResponse {
    pub invitation: OrgInvitation,
    pub invite_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgInvitationsQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgInvitationsResponse {
    pub entries: Vec<OrgInvitation>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResendOrgInvitationRequest {
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceptOrgInvitationRequest {
    pub token: String,
    #[serde(default)]
    pub login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceptOrgInvitationResponse {
    pub invitation: OrgInvitation,
    pub client_id: String,
    pub role: String,
    pub org_id: String,
    pub api_key: String,
}

// ============================================================================
