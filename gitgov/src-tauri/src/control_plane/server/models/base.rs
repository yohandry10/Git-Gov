use crate::models::{AuditAction, AuditLogEntry, AuditStatus};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub event_type: String,
    pub timestamp: i64,
    pub developer_login: String,
    pub developer_name: String,
    pub action: AuditAction,
    pub branch: String,
    pub files: Vec<String>,
    pub commit_hash: Option<String>,
    pub status: AuditStatus,
    pub reason: Option<String>,
    pub repo_name: Option<String>,
    pub repo_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: String,
    pub received: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CombinedEvent {
    pub id: String,
    pub source: String,
    pub event_type: String,
    pub created_at: i64,
    pub user_login: Option<String>,
    pub repo_name: Option<String>,
    pub branch: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_json_object")]
    pub details: serde_json::Value,
}

fn default_json_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
    pub limit: usize,
    /// Legacy pagination for `/logs`; prefer keyset cursor (`before_created_at` + `before_id`).
    pub offset: usize,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            source: None,
            start_date: None,
            end_date: None,
            user_login: None,
            developer_login: None,
            event_type: None,
            action: None,
            status: None,
            branch: None,
            repo_full_name: None,
            repo_name: None,
            org_name: None,
            before_created_at: None,
            before_id: None,
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStats {
    pub github_events: GitHubEventStats,
    pub client_events: ClientEventStats,
    pub violations: ViolationStats,
    #[serde(default)]
    pub pipeline: PipelineHealthStats,
    pub active_devs_week: i64,
    pub active_repos: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyActivityPoint {
    pub day: String,
    pub commits: i64,
    pub pushes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubEventStats {
    pub total: i64,
    pub today: i64,
    pub pushes_today: i64,
    #[serde(default)]
    pub by_type: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientEventStats {
    pub total: i64,
    pub today: i64,
    pub blocked_today: i64,
    #[serde(default)]
    pub desktop_pushes_today: i64,
    #[serde(default)]
    pub by_type: std::collections::HashMap<String, i64>,
    #[serde(default)]
    pub by_status: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViolationStats {
    pub total: i64,
    pub unresolved: i64,
    pub critical: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineHealthStats {
    pub total_7d: i64,
    pub success_7d: i64,
    pub failure_7d: i64,
    pub aborted_7d: i64,
    pub unstable_7d: i64,
    pub avg_duration_ms_7d: i64,
    pub repos_with_failures_7d: i64,
}

impl EventPayload {
    pub fn from_audit_entry(
        entry: &AuditLogEntry,
        repo_name: Option<String>,
        repo_owner: Option<String>,
    ) -> Self {
        Self {
            event_type: "audit".to_string(),
            timestamp: entry.timestamp,
            developer_login: entry.developer_login.clone(),
            developer_name: entry.developer_name.clone(),
            action: entry.action,
            branch: entry.branch.clone(),
            files: entry.files.clone(),
            commit_hash: entry.commit_hash.clone(),
            status: entry.status,
            reason: entry.reason.clone(),
            repo_name,
            repo_owner,
        }
    }
}
