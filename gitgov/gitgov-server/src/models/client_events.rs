use super::*;

// ============================================================================
// CLIENT EVENTS (Telemetry from Desktop App)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEvent {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub event_uuid: String,
    pub event_type: ClientEventType,
    pub user_login: String,
    pub user_name: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub files: Vec<String>,
    pub status: EventStatus,
    pub reason: Option<String>,
    pub metadata: serde_json::Value,
    pub client_version: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClientEventType {
    AttemptPush,
    BlockedPush,
    SuccessfulPush,
    PushFailed,
    GovernanceBlockedPush,
    GovernanceWarnedPush,
    CliCommand,
    CliCommandCompleted,
    Heartbeat,
    CreateBranch,
    BlockedBranch,
    StageFiles,
    Commit,
    CheckoutBranch,
    Login,
    Logout,
}

impl ClientEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientEventType::AttemptPush => "attempt_push",
            ClientEventType::BlockedPush => "blocked_push",
            ClientEventType::SuccessfulPush => "successful_push",
            ClientEventType::PushFailed => "push_failed",
            ClientEventType::GovernanceBlockedPush => "governance_blocked_push",
            ClientEventType::GovernanceWarnedPush => "governance_warned_push",
            ClientEventType::CliCommand => "cli_command",
            ClientEventType::CliCommandCompleted => "cli_command_completed",
            ClientEventType::Heartbeat => "heartbeat",
            ClientEventType::CreateBranch => "create_branch",
            ClientEventType::BlockedBranch => "blocked_branch",
            ClientEventType::StageFiles => "stage_files",
            ClientEventType::Commit => "commit",
            ClientEventType::CheckoutBranch => "checkout_branch",
            ClientEventType::Login => "login",
            ClientEventType::Logout => "logout",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "attempt_push" => ClientEventType::AttemptPush,
            "blocked_push" => ClientEventType::BlockedPush,
            "successful_push" => ClientEventType::SuccessfulPush,
            "push_failed" => ClientEventType::PushFailed,
            "governance_blocked_push" => ClientEventType::GovernanceBlockedPush,
            "governance_warned_push" => ClientEventType::GovernanceWarnedPush,
            "cli_command" => ClientEventType::CliCommand,
            "cli_command_completed" => ClientEventType::CliCommandCompleted,
            "heartbeat" => ClientEventType::Heartbeat,
            "create_branch" => ClientEventType::CreateBranch,
            "blocked_branch" => ClientEventType::BlockedBranch,
            "stage_files" => ClientEventType::StageFiles,
            "commit" => ClientEventType::Commit,
            "checkout_branch" => ClientEventType::CheckoutBranch,
            "login" => ClientEventType::Login,
            "logout" => ClientEventType::Logout,
            _ => return None,
        }
        .into()
    }

    pub fn from_db_str(s: &str) -> Self {
        Self::parse(s).unwrap_or(ClientEventType::AttemptPush)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Success,
    Blocked,
    Failed,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Success => "success",
            EventStatus::Blocked => "blocked",
            EventStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "success" => EventStatus::Success,
            "blocked" => EventStatus::Blocked,
            "failed" => EventStatus::Failed,
            _ => return None,
        }
        .into()
    }

    pub fn from_db_str(s: &str) -> Self {
        Self::parse(s).unwrap_or(EventStatus::Failed)
    }
}

// ============================================================================
// BATCH INGEST FROM CLIENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClientEventBatch {
    pub events: Vec<ClientEventInput>,
    pub client_id: Option<String>,
    pub client_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClientEventInput {
    pub event_uuid: String,
    pub event_type: String,
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub user_login: String,
    pub user_name: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub files: Vec<String>,
    pub status: String,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClientEventResponse {
    pub accepted: Vec<String>,
    pub duplicates: Vec<String>,
    pub errors: Vec<EventError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventError {
    pub event_uuid: String,
    pub error: String,
}

// ============================================================================
