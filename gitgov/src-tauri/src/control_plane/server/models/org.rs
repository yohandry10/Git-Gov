use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeyInfo {
    pub id: String,
    pub client_id: String,
    pub role: String,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub last_used: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub client_id: String,
    pub role: String,
    #[serde(default)]
    pub principal_type: Option<String>,
    #[serde(default)]
    pub platform_principal_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub requires_workspace_for_tenant_surfaces: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgSummary {
    pub id: String,
    #[serde(default)]
    pub github_id: Option<i64>,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamRepoSummary {
    pub repo_name: String,
    pub events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamDeveloperOverview {
    pub login: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    #[serde(default)]
    pub last_seen: Option<i64>,
    pub total_events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub repos_active_count: i64,
    #[serde(default)]
    pub repos: Vec<TeamRepoSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamOverviewResponse {
    pub entries: Vec<TeamDeveloperOverview>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamRepoOverview {
    pub repo_name: String,
    pub developers_active: i64,
    pub total_events: i64,
    pub commits: i64,
    pub pushes: i64,
    pub blocked_pushes: i64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamReposResponse {
    pub entries: Vec<TeamRepoOverview>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgUser {
    pub id: String,
    pub org_id: String,
    pub login: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub updated_by: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgRequest {
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgResponse {
    pub org_id: String,
    pub login: String,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgUserRequest {
    pub login: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrgUserResponse {
    pub user: OrgUser,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgUsersResponse {
    pub entries: Vec<OrgUser>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateOrgUserStatusRequest {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeyResponse {
    #[serde(default)]
    pub api_key: Option<String>,
    pub client_id: String,
    #[serde(default)]
    pub error: Option<String>,
}

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
