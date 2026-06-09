use super::*;

// ============================================================================
// CLI COMMAND AUDIT — embedded terminal audit trail
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandInput {
    pub command: String,
    pub origin: String, // "button_click" | "manual_input"
    pub branch: String,
    pub repo_name: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandResponse {
    pub accepted: bool,
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandRecord {
    pub id: String,
    pub org_id: Option<String>,
    pub user_login: String,
    pub command: String,
    pub origin: String,
    pub branch: String,
    pub repo_name: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}
