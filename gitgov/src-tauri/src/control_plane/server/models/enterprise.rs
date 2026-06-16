use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseAdoptionProfileRecord {
    pub org_id: String,
    #[serde(default)]
    pub profile: serde_json::Value,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseAdoptionProfileResponse {
    pub found: bool,
    #[serde(default)]
    pub profile: Option<EnterpriseAdoptionProfileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertEnterpriseAdoptionProfileRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub profile: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseOnboardingChecklistTrackingRecord {
    pub org_id: String,
    #[serde(default)]
    pub tracking: serde_json::Value,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseOnboardingChecklistTrackingResponse {
    pub found: bool,
    #[serde(default)]
    pub tracking: Option<EnterpriseOnboardingChecklistTrackingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertEnterpriseOnboardingChecklistTrackingRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub tracking: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoSetupRecord {
    pub run_id: String,
    pub org_id: String,
    pub status: String,
    pub goal: String,
    pub repository_full_name: String,
    pub default_branch: String,
    #[serde(default)]
    pub selected_providers: Vec<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    pub policy_preset: String,
    #[serde(default)]
    pub baseline: serde_json::Value,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoSetupResponse {
    pub found: bool,
    #[serde(default)]
    pub setup: Option<FirstGovernedRepoSetupRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UpsertFirstGovernedRepoSetupRequest {
    pub org_name: Option<String>,
    pub status: Option<String>,
    pub goal: String,
    pub repository_full_name: String,
    pub default_branch: String,
    pub selected_providers: Vec<String>,
    pub selected_modules: Vec<String>,
    pub policy_preset: String,
    pub baseline: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FirstGovernedRepoWizardActionRequest {
    pub org_name: Option<String>,
    pub status: Option<String>,
    pub goal: String,
    pub repository_full_name: String,
    pub default_branch: String,
    pub selected_providers: Vec<String>,
    pub selected_modules: Vec<String>,
    pub policy_preset: String,
    pub baseline: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoWizardStateResponse {
    pub org_id: String,
    pub found: bool,
    #[serde(default)]
    pub state: serde_json::Value,
    #[serde(default)]
    pub setup: Option<FirstGovernedRepoSetupRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoWizardRunResponse {
    #[serde(default)]
    pub state: serde_json::Value,
    pub setup: FirstGovernedRepoSetupRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseApprovalRecord {
    pub id: String,
    pub org_id: String,
    pub release_id: String,
    pub repository_full_name: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    pub environment: String,
    pub decision: String,
    pub approver: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub evidence_packet_hash: Option<String>,
    #[serde(default)]
    pub evidence_packet_uri: Option<String>,
    #[serde(default)]
    pub evidence_summary: serde_json::Value,
    pub risk_severity: String,
    #[serde(default)]
    pub risk_acceptance_reason: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    pub approval_hash: String,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterpriseReleaseApprovalQuery {
    pub org_name: Option<String>,
    pub repository_full_name: Option<String>,
    pub branch: Option<String>,
    pub target_sha: Option<String>,
    pub release_id: Option<String>,
    pub environment: Option<String>,
    pub decision: Option<String>,
    pub evidence_packet_hash: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterpriseReleaseGovernanceEvaluationQuery {
    pub org_name: Option<String>,
    pub repository_full_name: String,
    pub branch: Option<String>,
    pub target_sha: Option<String>,
    pub release_id: String,
    pub environment: String,
    pub evidence_packet_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseGovernanceQuorumRuleSummary {
    pub role: String,
    pub required: i64,
    pub observed: i64,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseGovernancePolicySummary {
    pub mode: String,
    pub environment: String,
    pub approval_required: bool,
    pub enforcement: String,
    pub policy_applies: bool,
    pub quorum_enabled: bool,
    #[serde(default)]
    pub quorum_rules: Vec<EnterpriseReleaseGovernanceQuorumRuleSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseGovernanceApprovalSummary {
    pub id: String,
    pub decision: String,
    pub approver: String,
    #[serde(default)]
    pub approver_role: Option<String>,
    pub risk_severity: String,
    #[serde(default)]
    pub evidence_packet_hash: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub counts_toward_policy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseGovernanceEvaluationResponse {
    pub status: String,
    pub policy_satisfied: bool,
    pub blocking: bool,
    pub would_block: bool,
    pub valid_approval_count: i64,
    pub required_approval_count: i64,
    pub policy: EnterpriseReleaseGovernancePolicySummary,
    #[serde(default)]
    pub approvals: Vec<EnterpriseReleaseGovernanceApprovalSummary>,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseApprovalListResponse {
    #[serde(default)]
    pub items: Vec<EnterpriseReleaseApprovalRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateAuthorizationRecord {
    pub id: String,
    pub authorization_id: String,
    pub org_id: String,
    pub release_id: String,
    pub repository_full_name: String,
    pub branch: String,
    pub target_sha: String,
    pub environment: String,
    pub deployer: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    pub evidence_packet_hash: String,
    #[serde(default)]
    pub evidence_packet_uri: Option<String>,
    pub decision: String,
    pub approved: bool,
    pub blocking: bool,
    pub would_block: bool,
    pub reason: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub policy_checksum: String,
    pub break_glass_eligible: bool,
    #[serde(default)]
    pub break_glass_used: bool,
    #[serde(default)]
    pub break_glass_reason: Option<String>,
    #[serde(default)]
    pub break_glass_authorized_by: Option<String>,
    #[serde(default)]
    pub break_glass_expires_at: Option<i64>,
    #[serde(default)]
    pub break_glass_approval_id: Option<String>,
    #[serde(default)]
    pub break_glass_approval_hash: Option<String>,
    pub evaluation: EnterpriseReleaseGovernanceEvaluationResponse,
    #[serde(default)]
    pub governance_decision: serde_json::Value,
    #[serde(default)]
    pub details: serde_json::Value,
    #[serde(default)]
    pub request_payload: serde_json::Value,
    pub requested_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DeploymentGateAuthorizationQuery {
    pub org_name: Option<String>,
    pub authorization_id: Option<String>,
    pub repository_full_name: Option<String>,
    pub branch: Option<String>,
    pub target_sha: Option<String>,
    pub release_id: Option<String>,
    pub environment: Option<String>,
    pub decision: Option<String>,
    pub deployer: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateAuthorizationListResponse {
    #[serde(default)]
    pub items: Vec<DeploymentGateAuthorizationRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CreateEnterpriseReleaseApprovalRequest {
    pub org_name: Option<String>,
    pub release_id: String,
    pub repository_full_name: String,
    pub branch: Option<String>,
    pub target_sha: Option<String>,
    pub environment: String,
    pub decision: String,
    pub approver: String,
    pub ticket_id: Option<String>,
    pub evidence_packet_hash: Option<String>,
    pub evidence_packet_uri: Option<String>,
    pub evidence_summary: serde_json::Value,
    pub risk_severity: Option<String>,
    pub risk_acceptance_reason: Option<String>,
    pub expires_at: Option<i64>,
}
