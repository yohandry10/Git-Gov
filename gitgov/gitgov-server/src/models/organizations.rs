use super::*;

// ============================================================================
// ORGANIZATIONS & REPOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Org {
    pub id: String,
    pub github_id: Option<i64>,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub tenant_type: String,
    #[serde(default)]
    pub lifecycle_status: String,
    #[serde(default)]
    pub provisioning_source: String,
    #[serde(default)]
    pub provisioned_by: Option<String>,
    #[serde(default)]
    pub platform_metadata: serde_json::Value,
    #[serde(default)]
    pub suspended_at: Option<i64>,
    #[serde(default)]
    pub archived_at: Option<i64>,
    #[serde(default)]
    pub deleted_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: String,
    pub org_id: Option<String>,
    pub github_id: Option<i64>,
    pub full_name: String,
    pub name: String,
    pub private: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub org_id: String,
    pub github_login: String,
    pub github_id: Option<i64>,
    pub role: UserRole,
    pub groups: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Architect,
    Developer,
    PM,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Architect => "Architect",
            UserRole::Developer => "Developer",
            UserRole::PM => "PM",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Admin" => UserRole::Admin,
            "Architect" => UserRole::Architect,
            "PM" => UserRole::PM,
            _ => UserRole::Developer,
        }
    }
}
