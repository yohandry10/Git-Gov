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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoWizardActionRequest {
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
    #[serde(default)]
    pub current_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstGovernedRepoWizardStateResponse {
    pub org_id: String,
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<FirstGovernedRepoSetupRecord>,
    pub state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstGovernedRepoWizardRunResponse {
    pub setup: FirstGovernedRepoSetupRecord,
    pub state: serde_json::Value,
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
    #[serde(default)]
    pub break_glass_used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_glass_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_glass_authorized_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_glass_expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_glass_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_glass_approval_hash: Option<String>,
    pub evaluation: EnterpriseReleaseGovernanceEvaluationResponse,
    #[serde(default)]
    pub governance_decision: serde_json::Value,
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
    #[serde(default)]
    pub break_glass: Option<DeploymentGateBreakGlassRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateBreakGlassRequest {
    #[serde(default)]
    pub requested: bool,
    pub reason: String,
    #[serde(default)]
    pub approval_id: Option<String>,
    #[serde(default)]
    pub authorized_by: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGateBreakGlassApprovalRecord {
    pub approval_id: String,
    pub org_id: String,
    pub release_id: String,
    pub repository_full_name: String,
    pub branch: String,
    pub target_sha: String,
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    pub evidence_packet_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_packet_uri: Option<String>,
    pub reason: String,
    pub approver: String,
    pub approver_role: String,
    pub expires_at: i64,
    pub approval_hash: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateDeploymentGateBreakGlassApprovalRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub release_id: String,
    pub repository_full_name: String,
    pub branch: String,
    pub target_sha: String,
    pub environment: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    pub evidence_packet_hash: String,
    #[serde(default)]
    pub evidence_packet_uri: Option<String>,
    pub reason: String,
    pub approver: String,
    #[serde(default)]
    pub approver_role: Option<String>,
    pub expires_at: i64,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateBreakGlassApprovalQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub approval_id: Option<String>,
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
    pub evidence_packet_hash: Option<String>,
    #[serde(default)]
    pub approver: Option<String>,
    #[serde(default)]
    pub active_only: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentGateBreakGlassApprovalListResponse {
    #[serde(default)]
    pub items: Vec<DeploymentGateBreakGlassApprovalRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGateRiskContextResponse {
    pub deployment_gate_id: String,
    pub authorization: DeploymentGateAuthorizationRecord,
    #[serde(default)]
    pub change_risk_evaluations: Vec<ChangeRiskEvaluationRecord>,
    #[serde(default)]
    pub cab_packets: Vec<ChangeRiskCabPacketRecord>,
    #[serde(default)]
    pub cab_decision_manifests: Vec<ChangeRiskCabDecisionManifestRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_risk_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_review_status: Option<String>,
    pub triggered_rules_count: usize,
    pub advisory_only: bool,
    pub enforcement_used: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiRepoExecutiveGovernanceQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub posture: Option<String>,
    #[serde(default)]
    pub gate_decision: Option<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub review_status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepoExecutiveGovernanceRepository {
    pub repository_full_name: String,
    pub posture: String,
    pub gate_count: i64,
    pub blocked_gate_count: i64,
    pub advisory_gate_count: i64,
    pub break_glass_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_gate_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_gate_decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_gate_created_at: Option<i64>,
    pub change_risk_count: i64,
    pub high_risk_count: i64,
    pub needs_review_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_risk_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_review_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_risk_created_at: Option<i64>,
    pub cab_packet_count: i64,
    pub cab_manifest_count: i64,
    pub active_manifest_count: i64,
    pub revoked_manifest_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_manifest_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_manifest_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_manifest_created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiRepoExecutiveGovernanceTotals {
    pub repositories: i64,
    pub gate_count: i64,
    pub blocked_gate_count: i64,
    pub advisory_gate_count: i64,
    pub break_glass_count: i64,
    pub change_risk_count: i64,
    pub high_risk_count: i64,
    pub needs_review_count: i64,
    pub cab_packet_count: i64,
    pub cab_manifest_count: i64,
    pub active_manifest_count: i64,
    pub revoked_manifest_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepoExecutiveGovernanceResponse {
    pub org_id: String,
    pub generated_at: i64,
    #[serde(default)]
    pub repositories: Vec<MultiRepoExecutiveGovernanceRepository>,
    pub totals: MultiRepoExecutiveGovernanceTotals,
    pub limit: i64,
    pub offset: i64,
    pub advisory_only: bool,
    pub enforcement_used: bool,
    pub deployment_execution: bool,
    pub provider_mutation: bool,
    pub repository_mutation: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveGovernanceSnapshotRecord {
    pub snapshot_id: String,
    pub org_id: String,
    pub name: String,
    #[serde(default)]
    pub filters: serde_json::Value,
    pub artifact_hash: String,
    pub repository_count: i64,
    pub status: String,
    pub created_by_user_id: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    pub download_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveGovernanceSnapshotRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub name: String,
    #[serde(default)]
    pub filters: MultiRepoExecutiveGovernanceQuery,
    #[serde(default = "default_true")]
    pub include_repository_rows: bool,
    #[serde(default = "default_true")]
    pub include_summary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveGovernanceSnapshotQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveGovernanceSnapshotResponse {
    pub snapshot: ExecutiveGovernanceSnapshotRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveGovernanceSnapshotListResponse {
    #[serde(default)]
    pub items: Vec<ExecutiveGovernanceSnapshotRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskEvaluationRecord {
    pub evaluation_id: String,
    pub org_id: String,
    pub repository_full_name: String,
    pub branch: String,
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_gate_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_packet_hash: Option<String>,
    pub risk_level: String,
    pub ruleset_version: String,
    #[serde(default)]
    pub risk_reasons: Vec<String>,
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    #[serde(default)]
    pub blocking_gaps: Vec<String>,
    #[serde(default)]
    pub recommended_manual_actions: Vec<String>,
    #[serde(default)]
    pub triggered_rules: Vec<String>,
    #[serde(default)]
    pub non_triggered_rules: Vec<String>,
    #[serde(default)]
    pub evaluation_trace: serde_json::Value,
    pub trace_hash: String,
    pub advisory_only: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
    #[serde(default)]
    pub evaluation: serde_json::Value,
    #[serde(default)]
    pub request_payload: serde_json::Value,
    pub created_by: String,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_updated_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskEvaluationRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub repository_full_name: String,
    pub branch: String,
    pub environment: String,
    #[serde(default)]
    pub change_id: Option<String>,
    #[serde(default)]
    pub deployment_gate_id: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub evidence_packet_hash: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskEvaluationQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub evaluation_id: Option<String>,
    #[serde(default)]
    pub repository_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub change_id: Option<String>,
    #[serde(default)]
    pub deployment_gate_id: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub review_status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskEvaluationListResponse {
    #[serde(default)]
    pub items: Vec<ChangeRiskEvaluationRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskRuleDefinition {
    pub rule_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    #[serde(default)]
    pub evidence_inputs: Vec<String>,
    pub manual_action_hint: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskRuleCatalogResponse {
    pub ruleset_version: String,
    pub catalog_hash: String,
    #[serde(default)]
    pub rules: Vec<ChangeRiskRuleDefinition>,
    pub advisory_only: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskEvaluationTraceResponse {
    pub evaluation_id: String,
    pub org_id: String,
    pub ruleset_version: String,
    #[serde(default)]
    pub triggered_rules: Vec<String>,
    #[serde(default)]
    pub non_triggered_rules: Vec<String>,
    #[serde(default)]
    pub evaluation_trace: serde_json::Value,
    pub trace_hash: String,
    pub advisory_only: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskEvaluationReviewRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub review_status: String,
    #[serde(default)]
    pub review_notes: Option<String>,
    #[serde(default)]
    pub mitigation_notes: Option<String>,
    #[serde(default)]
    pub decision_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskEvaluationReviewResponse {
    pub evaluation_id: String,
    pub org_id: String,
    pub risk_level: String,
    pub ruleset_version: String,
    pub trace_hash: String,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_updated_at: Option<i64>,
    pub advisory_only: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub compliance_claim: bool,
    pub certification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskCabPacketRecord {
    pub packet_id: String,
    pub org_id: String,
    pub name: String,
    #[serde(default)]
    pub filters: serde_json::Value,
    #[serde(default)]
    pub evaluation_ids: Vec<String>,
    pub artifact_hash: String,
    pub status: String,
    pub created_by_user_id: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    pub download_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_by_user_id: Option<String>,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason_safe: Option<String>,
    pub follow_up_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_owner_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabPacketRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub repository_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub review_status: Option<String>,
    #[serde(default)]
    pub date_range_start: Option<i64>,
    #[serde(default)]
    pub date_range_end: Option<i64>,
    #[serde(default)]
    pub evaluation_ids: Vec<String>,
    #[serde(default)]
    pub deployment_gate_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabPacketQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskCabPacketResponse {
    pub packet: ChangeRiskCabPacketRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabPacketListResponse {
    #[serde(default)]
    pub items: Vec<ChangeRiskCabPacketRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabPacketReviewRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub review_status: String,
    #[serde(default)]
    pub review_notes: Option<String>,
    #[serde(default)]
    pub mitigation_notes: Option<String>,
    #[serde(default)]
    pub decision_reason: Option<String>,
    #[serde(default)]
    pub follow_up_required: bool,
    #[serde(default)]
    pub follow_up_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskCabPacketReviewResponse {
    pub packet_id: String,
    pub org_id: String,
    pub artifact_hash: String,
    pub packet_status: String,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation_notes_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason_safe: Option<String>,
    pub follow_up_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_owner_safe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_updated_at: Option<i64>,
    pub manual_cab_disposition_only: bool,
    pub advisory_only: bool,
    pub llm_used: bool,
    pub agent_governance_used: bool,
    pub release_blocking: bool,
    pub deployment_execution: bool,
    pub compliance_claim: bool,
    pub certification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskCabDecisionManifestRecord {
    pub manifest_id: String,
    pub org_id: String,
    pub cab_packet_id: String,
    pub cab_packet_hash: String,
    pub manifest_hash: String,
    pub review_status_snapshot: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    pub created_by_user_id: String,
    pub created_at: i64,
    pub download_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabDecisionManifestRequest {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabDecisionManifestQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskCabDecisionManifestResponse {
    pub manifest: ChangeRiskCabDecisionManifestRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeRiskCabDecisionManifestListResponse {
    #[serde(default)]
    pub items: Vec<ChangeRiskCabDecisionManifestRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceEvaluationRecord {
    pub id: String,
    pub evaluation_id: String,
    pub org_id: String,
    pub agent_id: String,
    pub agent_type: String,
    pub actor: String,
    pub action: String,
    pub repository_full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    pub decision: String,
    pub allowed: bool,
    pub requires_approval: bool,
    pub reason: String,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    pub policy_id: String,
    pub policy_checksum: String,
    #[serde(default)]
    pub evaluation: serde_json::Value,
    #[serde(default)]
    pub request_payload: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<AgentGovernanceAttributionEnvelope>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceAgentKeyRecord {
    pub id: String,
    pub key_id: String,
    pub org_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    pub token_preview: String,
    pub status: String,
    pub expiring_soon: bool,
    pub no_expiry: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotated_from_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_reason: Option<String>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_by: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentGovernanceAgentKeyResponse {
    #[serde(flatten)]
    pub record: AgentGovernanceAgentKeyRecord,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateAgentGovernanceAgentKeyResponse {
    pub replacement: AgentGovernanceAgentKeyRecord,
    pub replaced: AgentGovernanceAgentKeyRecord,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceAgentKeyQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateAgentGovernanceAgentKeyRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub no_expiry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotateAgentGovernanceAgentKeyRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub grace_period_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceAgentKeyListResponse {
    #[serde(default)]
    pub items: Vec<AgentGovernanceAgentKeyRecord>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceSettingsRecord {
    pub org_id: String,
    pub enabled: bool,
    pub mode: String,
    pub payload_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceSettingsQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertAgentGovernanceSettingsRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceEvaluationQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub evaluation_id: Option<String>,
    #[serde(default)]
    pub repository_full_name: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceEvaluationListResponse {
    #[serde(default)]
    pub items: Vec<AgentGovernanceEvaluationRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceReadContextQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    pub repository_full_name: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceReadContextResponse {
    pub context_id: String,
    pub org_id: String,
    pub repository_full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub read_only: bool,
    pub will_authorize_execution: bool,
    pub mcp_surface: bool,
    pub generated_at: i64,
    pub principal: serde_json::Value,
    pub branch_status: serde_json::Value,
    pub policy_compliance: serde_json::Value,
    pub pipeline_state: serde_json::Value,
    pub risk_score: serde_json::Value,
    pub recent_activity: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGovernanceEvaluationRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub agent_id: String,
    #[serde(default)]
    pub agent_type: Option<String>,
    pub actor: String,
    pub action: String,
    pub repository_full_name: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub attribution: Option<AgentGovernanceAttributionInput>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AgentGovernanceAttributionInput {
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub parent_correlation_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_version: Option<String>,
    #[serde(default)]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub external_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceAttributionEnvelope {
    pub attribution_id: String,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_run_id: Option<String>,
    pub principal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_display_name: Option<String>,
    pub consumer_type: String,
    pub action: String,
    pub decision: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGovernanceDryRunResponse {
    pub dry_run: bool,
    pub would_persist_evaluation: bool,
    pub would_authorize_execution: bool,
    pub org_id: String,
    pub agent_id: String,
    pub agent_type: String,
    pub actor: String,
    pub action: String,
    pub repository_full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    pub decision: String,
    pub allowed: bool,
    pub requires_approval: bool,
    pub reason: String,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    pub policy_id: String,
    pub policy_checksum: String,
    #[serde(default)]
    pub evaluation: serde_json::Value,
    #[serde(default)]
    pub request_payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_display_name: Option<String>,
    pub consumer_type: String,
    pub attribution: AgentGovernanceAttributionEnvelope,
    pub previewed_at: i64,
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
