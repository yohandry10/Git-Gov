use super::*;

// API KEYS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MeResponse {
    pub client_id: String,
    pub role: String,
    #[serde(default)]
    pub principal_type: String,
    #[serde(default)]
    pub platform_principal_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub requires_workspace_for_tenant_surfaces: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: UserRole,
    pub exp: usize,
}

// ============================================================================
