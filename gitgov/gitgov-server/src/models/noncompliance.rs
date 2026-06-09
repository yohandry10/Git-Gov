use super::*;

// NONCOMPLIANCE SIGNALS (NO binario - confidence levels)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoncomplianceSignal {
    pub id: String,
    pub org_id: Option<String>,
    pub repo_id: Option<String>,
    pub github_event_id: Option<String>,
    pub client_event_id: Option<String>,
    pub signal_type: String,
    pub confidence: String,
    pub actor_login: String,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub evidence: serde_json::Value,
    pub context: serde_json::Value,
    pub status: String,
    pub investigated_by: Option<String>,
    pub investigated_at: Option<i64>,
    pub investigation_notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    UntrustedPath,
    MissingTelemetry,
    PolicyViolation,
    CorrelationMismatch,
    CommitNoTicket,
    TicketNoCoverage,
    PipelineFailureStreak,
    StaleInProgress,
    DoneNotDeployed,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::UntrustedPath => "untrusted_path",
            SignalType::MissingTelemetry => "missing_telemetry",
            SignalType::PolicyViolation => "policy_violation",
            SignalType::CorrelationMismatch => "correlation_mismatch",
            SignalType::CommitNoTicket => "commit_no_ticket",
            SignalType::TicketNoCoverage => "ticket_no_coverage",
            SignalType::PipelineFailureStreak => "pipeline_failure_streak",
            SignalType::StaleInProgress => "stale_in_progress",
            SignalType::DoneNotDeployed => "done_not_deployed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "untrusted_path" => SignalType::UntrustedPath,
            "missing_telemetry" => SignalType::MissingTelemetry,
            "policy_violation" => SignalType::PolicyViolation,
            "commit_no_ticket" => SignalType::CommitNoTicket,
            "ticket_no_coverage" => SignalType::TicketNoCoverage,
            "pipeline_failure_streak" => SignalType::PipelineFailureStreak,
            "stale_in_progress" => SignalType::StaleInProgress,
            "done_not_deployed" => SignalType::DoneNotDeployed,
            _ => SignalType::CorrelationMismatch,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

impl ConfidenceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfidenceLevel::High => "high",
            ConfidenceLevel::Medium => "medium",
            ConfidenceLevel::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "high" => ConfidenceLevel::High,
            "medium" => ConfidenceLevel::Medium,
            _ => ConfidenceLevel::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SignalStatus {
    Pending,
    Investigating,
    Confirmed,
    Dismissed,
}

impl SignalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalStatus::Pending => "pending",
            SignalStatus::Investigating => "investigating",
            SignalStatus::Confirmed => "confirmed",
            SignalStatus::Dismissed => "dismissed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "investigating" => SignalStatus::Investigating,
            "confirmed" => SignalStatus::Confirmed,
            "dismissed" => SignalStatus::Dismissed,
            _ => SignalStatus::Pending,
        }
    }
}

// ============================================================================
