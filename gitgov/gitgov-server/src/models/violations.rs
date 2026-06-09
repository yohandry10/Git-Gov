use super::*;

// VIOLATIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub github_event_id: Option<String>,
    pub client_event_id: Option<String>,
    pub violation_type: ViolationType,
    pub severity: Severity,
    pub user_login: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub details: serde_json::Value,
    pub resolved: bool,
    pub resolved_at: Option<i64>,
    pub resolved_by: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationType {
    UnauthorizedPush,
    BranchProtection,
    NamingViolation,
    PathViolation,
    CommitMessageViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

// ============================================================================
