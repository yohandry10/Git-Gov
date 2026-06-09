use crate::models::GitGovConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub version: String,
    pub checksum: String,
    pub config: GitGovConfig,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyHistoryEntry {
    pub id: String,
    pub repo_id: String,
    pub config: GitGovConfig,
    pub checksum: String,
    pub changed_by: String,
    pub change_type: String,
    pub previous_checksum: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckRequest {
    pub repo: String,
    #[serde(default)]
    pub commit: Option<String>,
    pub branch: String,
    #[serde(default)]
    pub user_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyCheckResponse {
    pub advisory: bool,
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub evaluated_rules: Vec<String>,
    #[serde(default)]
    pub enforcement_applied: String,
    #[serde(default)]
    pub violations: Vec<RuleViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule: String,
    pub category: String,
    pub enforcement: String,
    pub message: String,
}
