use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAdoptionProfileRecord {
    pub org_id: String,
    pub profile: serde_json::Value,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseAdoptionProfileResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<EnterpriseAdoptionProfileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseAdoptionProfileQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertEnterpriseAdoptionProfileRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub profile: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseOnboardingChecklistTrackingRecord {
    pub org_id: String,
    pub tracking: serde_json::Value,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseOnboardingChecklistTrackingResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<EnterpriseOnboardingChecklistTrackingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseOnboardingChecklistTrackingQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertEnterpriseOnboardingChecklistTrackingRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub tracking: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoSetupResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<FirstGovernedRepoSetupRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoSetupQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertFirstGovernedRepoSetupRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub repository_full_name: String,
    #[serde(default)]
    pub default_branch: String,
    #[serde(default)]
    pub selected_providers: Vec<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    #[serde(default)]
    pub policy_preset: String,
    #[serde(default)]
    pub baseline: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseReleaseApprovalRecord {
    pub id: String,
    pub org_id: String,
    pub release_id: String,
    pub repository_full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha: Option<String>,
    pub environment: String,
    pub decision: String,
    pub approver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_packet_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_packet_uri: Option<String>,
    pub evidence_summary: serde_json::Value,
    pub risk_severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_acceptance_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub approval_hash: String,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseApprovalQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repository_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub evidence_packet_hash: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseReleaseGovernanceEvaluationQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    pub repository_full_name: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    pub release_id: String,
    pub environment: String,
    #[serde(default)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approver_role: Option<String>,
    pub risk_severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_packet_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    pub evidence_packet_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub evaluation: EnterpriseReleaseGovernanceEvaluationResponse,
    #[serde(default)]
    pub details: serde_json::Value,
    pub request_payload: serde_json::Value,
    pub requested_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateAuthorizationRequest {
    #[serde(default)]
    pub org_name: Option<String>,
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
    #[serde(default)]
    pub requested_by: Option<String>,
    #[serde(default)]
    pub deployment_run_id: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateAuthorizationQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub authorization_id: Option<String>,
    #[serde(default)]
    pub repository_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub deployer: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
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
pub struct CreateEnterpriseReleaseApprovalRequest {
    #[serde(default)]
    pub org_name: Option<String>,
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
    #[serde(default)]
    pub risk_severity: Option<String>,
    #[serde(default)]
    pub risk_acceptance_reason: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

pub const RELEVANT_AUDIT_ACTIONS: &[&str] = &[
    "protected_branch.create",
    "protected_branch.destroy",
    "protected_branch.update_name",
    "protected_branch.update_admin_enforced",
    "protected_branch.update_pull_request_reviews_enforcement_level",
    "protected_branch.update_required_pull_request_reviews",
    "protected_branch.update_required_status_checks",
    "protected_branch.update_required_approving_review_count",
    "protected_branch.update_signature_requirement_enforcement_level",
    "protected_branch.update_strict_required_status_checks_policy",
    "repository_ruleset.create",
    "repository_ruleset.destroy",
    "repository_ruleset.update",
    "repository_ruleset.clear_custom_properties",
    "repo.access",
    "repo.permissions_granted",
    "repo.permissions_revoked",
    "team.add_repository",
    "team.remove_repository",
    "team.update_repository_permission",
    "org.update_member_repository_creation_permission",
    "org.update_default_repository_permission",
];

// ============================================================================
