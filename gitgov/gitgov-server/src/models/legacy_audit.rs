use super::*;

// LEGACY SUPPORT (keep for backward compatibility)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
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
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum AuditAction {
    Push,
    BranchCreate,
    StageFile,
    Commit,
    BlockedPush,
    BlockedBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum AuditStatus {
    Success,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub developer_login: Option<String>,
    pub action: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub repo_name: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

// ============================================================================
