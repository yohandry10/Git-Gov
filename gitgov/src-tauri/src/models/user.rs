use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub login: String,
    pub name: String,
    pub avatar_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub is_admin: bool,
}
