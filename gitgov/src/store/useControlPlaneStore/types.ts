import type { CombinedEvent, ServerStats } from '@/lib/types'
import type {
  EnterpriseAdoptionProfile,
  FirstGovernedRepoSetupDraft,
  EnterpriseOnboardingChecklistTracking,
} from '@/components/control_plane/dashboard-helpers'

export interface ServerConfig {
  url: string
  api_key?: string
}

export interface DailyActivityPoint {
  day: string
  commits: number
  pushes: number
}

export interface CommitPipelineRun {
  pipeline_event_id: string
  pipeline_id: string
  job_name: string
  status: string
  branch?: string | null
  repo_full_name?: string | null
  duration_ms?: number | null
  triggered_by?: string | null
  ingested_at: number
}

export interface CommitPipelineCorrelation {
  commit_event_id: string
  commit_sha: string
  commit_message?: string | null
  commit_created_at: number
  user_login: string
  branch?: string | null
  repo_name?: string | null
  pipeline?: CommitPipelineRun | null
}

export interface PrMergeEvidenceEntry {
  id: string
  org_id?: string | null
  org_name?: string | null
  repo_id?: string | null
  repo_full_name?: string | null
  delivery_id: string
  pr_number: number
  pr_title?: string | null
  author_login?: string | null
  merged_by_login?: string | null
  approvers: string[]
  approvals_count: number
  head_sha?: string | null
  merge_commit_sha?: string | null
  base_branch?: string | null
  created_at: number
}

export type TicketCoverageItem = Record<string, unknown>

export interface TicketCoverageStats {
  org: string
  period: string
  total_commits: number
  commits_with_ticket: number
  coverage_percentage: number
  commits_without_ticket: TicketCoverageItem[]
  tickets_without_commits: TicketCoverageItem[]
}

export interface JiraCorrelateResponse {
  scanned_commits: number
  correlations_created: number
  correlated_tickets: string[]
}

export interface JiraTicketDetail {
  id: string
  org_id?: string | null
  ticket_id: string
  ticket_url?: string | null
  title?: string | null
  status?: string | null
  assignee?: string | null
  reporter?: string | null
  priority?: string | null
  ticket_type?: string | null
  related_commits: string[]
  related_prs: string[]
  related_branches: string[]
  created_at?: number | null
  updated_at?: number | null
  ingested_at: number
}

export interface JiraTicketDetailResponse {
  found: boolean
  ticket?: JiraTicketDetail | null
}

export interface EvidencePacketCompleteness {
  ticket_found: boolean
  commits: number
  pull_requests: number
  pipelines: number
  quality_gates: number
  missing: string[]
}

export interface EvidencePacketReconstructionFilters {
  org_name?: string | null
  repo_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  ticket_id: string
  hours: number
}

export interface EvidencePacketReconstructionSources {
  commit_correlations: number
  client_events: number
  pull_request_merge_commits: number
  pull_request_merges: number
  pipeline_events: number
  quality_gate_pipeline_events: number
  legacy_pipeline_scope_fallbacks: number
}

export interface EvidencePacketReconstruction {
  filters: EvidencePacketReconstructionFilters
  sources: EvidencePacketReconstructionSources
  warnings: string[]
}

export interface EvidencePacket {
  packet_type: string
  subject: string
  generated_at: number
  org_name?: string | null
  repo_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  release_id?: string | null
  environment?: string | null
  period: string
  ticket?: JiraTicketDetail | null
  commits: Record<string, unknown>[]
  pull_requests: PrMergeEvidenceEntry[]
  pipelines: CommitPipelineRun[]
  quality_gates: CommitPipelineRun[]
  reconstruction?: EvidencePacketReconstruction
  completeness: EvidencePacketCompleteness
  content_hash: string
}

export interface EvidencePacketResponse {
  found: boolean
  packet?: EvidencePacket | null
}

export interface EnterpriseAdoptionProfileRecord {
  org_id: string
  profile: EnterpriseAdoptionProfile
  updated_by: string
  created_at: number
  updated_at: number
}

export interface EnterpriseAdoptionProfileResponse {
  found: boolean
  profile?: EnterpriseAdoptionProfileRecord | null
}

export interface EnterpriseOnboardingChecklistTrackingRecord {
  org_id: string
  tracking: EnterpriseOnboardingChecklistTracking
  updated_by: string
  created_at: number
  updated_at: number
}

export interface EnterpriseOnboardingChecklistTrackingResponse {
  found: boolean
  tracking?: EnterpriseOnboardingChecklistTrackingRecord | null
}

export interface FirstGovernedRepoSetupRecord {
  run_id: string
  org_id: string
  status: FirstGovernedRepoSetupDraft['status']
  goal: FirstGovernedRepoSetupDraft['goal']
  repository_full_name: string
  default_branch: string
  selected_providers: FirstGovernedRepoSetupDraft['selected_providers']
  selected_modules: FirstGovernedRepoSetupDraft['selected_modules']
  policy_preset: FirstGovernedRepoSetupDraft['policy_preset']
  baseline: FirstGovernedRepoSetupDraft['baseline']
  created_by: string
  updated_by: string
  created_at: number
  updated_at: number
  completed_at?: number | null
}

export interface FirstGovernedRepoSetupResponse {
  found: boolean
  setup?: FirstGovernedRepoSetupRecord | null
}

export interface UpsertFirstGovernedRepoSetupRequest {
  org_name?: string | null
  status?: FirstGovernedRepoSetupDraft['status'] | null
  goal: FirstGovernedRepoSetupDraft['goal']
  repository_full_name: string
  default_branch: string
  selected_providers: FirstGovernedRepoSetupDraft['selected_providers']
  selected_modules: FirstGovernedRepoSetupDraft['selected_modules']
  policy_preset: FirstGovernedRepoSetupDraft['policy_preset']
  baseline: FirstGovernedRepoSetupDraft['baseline']
}

export type FirstGovernedRepoWizardActionRequest = UpsertFirstGovernedRepoSetupRequest

export interface FirstGovernedRepoWizardStateResponse {
  org_id: string
  found: boolean
  state: Record<string, unknown>
  setup?: FirstGovernedRepoSetupRecord | null
}

export interface FirstGovernedRepoWizardRunResponse {
  state: Record<string, unknown>
  setup: FirstGovernedRepoSetupRecord
}

export type EnterpriseReleaseApprovalDecision = 'approved' | 'rejected' | 'accepted-risk'
export type EnterpriseReleaseApprovalRiskSeverity = 'none' | 'low' | 'medium' | 'high' | 'critical'

export interface EnterpriseReleaseApprovalRecord {
  id: string
  org_id: string
  release_id: string
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  environment: string
  decision: EnterpriseReleaseApprovalDecision
  approver: string
  ticket_id?: string | null
  evidence_packet_hash?: string | null
  evidence_packet_uri?: string | null
  evidence_summary: Record<string, unknown>
  risk_severity: EnterpriseReleaseApprovalRiskSeverity
  risk_acceptance_reason?: string | null
  expires_at?: number | null
  approval_hash: string
  created_by: string
  created_at: number
}

export interface EnterpriseReleaseApprovalListResponse {
  items: EnterpriseReleaseApprovalRecord[]
  total: number
  limit: number
  offset: number
}

export interface DeploymentGateAuthorizationRecord {
  id: string
  authorization_id: string
  org_id: string
  release_id: string
  repository_full_name: string
  branch: string
  target_sha: string
  environment: string
  deployer: string
  ticket_id?: string | null
  evidence_packet_hash: string
  evidence_packet_uri?: string | null
  decision: string
  approved: boolean
  blocking: boolean
  would_block: boolean
  reason: string
  blocked_by: string[]
  warnings: string[]
  policy_checksum: string
  break_glass_eligible: boolean
  break_glass_used: boolean
  break_glass_reason?: string | null
  break_glass_authorized_by?: string | null
  break_glass_expires_at?: number | null
  break_glass_approval_id?: string | null
  break_glass_approval_hash?: string | null
  evaluation: EnterpriseReleaseGovernanceEvaluationResponse
  governance_decision?: Record<string, unknown> | null
  details: Record<string, unknown>
  request_payload: Record<string, unknown>
  requested_by: string
  created_at: number
}

export interface DeploymentGateAuthorizationQuery {
  org_name?: string | null
  authorization_id?: string | null
  repository_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  release_id?: string | null
  environment?: string | null
  decision?: string | null
  deployer?: string | null
  limit?: number | null
  offset?: number | null
}

export interface DeploymentGateAuthorizationListResponse {
  items: DeploymentGateAuthorizationRecord[]
  total: number
  limit: number
  offset: number
}

export type ChangeRiskLevel = 'low' | 'medium' | 'high' | 'unknown' | string

export interface ChangeRiskEvaluationRecord {
  evaluation_id: string
  org_id: string
  repository_full_name: string
  branch: string
  environment: string
  change_id?: string | null
  deployment_gate_id?: string | null
  release_id?: string | null
  commit_sha?: string | null
  evidence_packet_hash?: string | null
  risk_level: ChangeRiskLevel
  risk_reasons: string[]
  missing_evidence: string[]
  blocking_gaps: string[]
  recommended_manual_actions: string[]
  advisory_only: boolean
  llm_used: boolean
  agent_governance_used: boolean
  compliance_claim: boolean
  certification: boolean
  evaluation: Record<string, unknown>
  request_payload: Record<string, unknown>
  created_by: string
  created_at: number
}

export interface ChangeRiskEvaluationRequest {
  org_name?: string | null
  repository_full_name: string
  branch: string
  environment: string
  change_id?: string | null
  deployment_gate_id?: string | null
  release_id?: string | null
  commit_sha?: string | null
  evidence_packet_hash?: string | null
  evidence_refs?: string[]
}

export interface ChangeRiskEvaluationQuery {
  org_name?: string | null
  evaluation_id?: string | null
  deployment_gate_id?: string | null
  release_id?: string | null
  repository_full_name?: string | null
  branch?: string | null
  environment?: string | null
  change_id?: string | null
  commit_sha?: string | null
  limit?: number | null
  offset?: number | null
}

export interface ChangeRiskEvaluationListResponse {
  items: ChangeRiskEvaluationRecord[]
  total: number
  limit: number
  offset: number
}

export interface ComplianceEvidenceExportRequest {
  org_name?: string | null
  scope: string
  deployment_gate_id?: string | null
  format?: string | null
  include_sections?: string[]
}

export interface ComplianceEvidenceExportQuery {
  org_name?: string | null
}

export interface ComplianceEvidenceExportRecord {
  export_id: string
  org_id: string
  created_by_user_id: string
  scope: string
  deployment_gate_id?: string | null
  release_id?: string | null
  status: string
  format: string
  artifact_hash: string
  policy_checksum?: string | null
  gate_decision?: string | null
  created_at: number
  completed_at?: number | null
  error_message_safe?: string | null
}

export interface ComplianceEvidenceExportResponse {
  export: ComplianceEvidenceExportRecord
  artifact?: Record<string, unknown> | null
}

export interface ComplianceControl {
  control_id: string
  title: string
  description: string
  required_evidence_types: string[]
  sort_order: number
}

export interface ComplianceControlFramework {
  framework_id: string
  org_id?: string | null
  name: string
  version: string
  description: string
  is_regulatory: boolean
  is_active: boolean
  owner_type: 'gitgov' | 'customer' | string
  owner_name?: string | null
  source: 'gitgov_owned' | 'customer_provided' | string
  is_gitgov_owned: boolean
  official_regulatory_mapping: boolean
  framework_pack_id?: string | null
  pack_hash?: string | null
  framework_pack_review_status?: string | null
  framework_pack_reviewed_by_user_id?: string | null
  framework_pack_reviewed_at?: number | null
  framework_pack_review_notes_safe?: string | null
  framework_pack_rejected_reason_safe?: string | null
  controls?: ComplianceControl[]
}

export interface ComplianceControlFrameworkListResponse {
  frameworks: ComplianceControlFramework[]
}

export interface ComplianceFrameworkPackImportRequest {
  org_name?: string | null
  format?: 'json' | 'yaml' | 'yml' | string | null
  pack?: Record<string, unknown> | null
  content?: string | null
}

export interface ComplianceFrameworkPackRecord {
  framework_pack_id: string
  org_id: string
  framework_id: string
  framework_name: string
  framework_version: string
  description: string
  owner_type: string
  owner_name: string
  source: string
  review_status: string
  schema_version: string
  pack_hash: string
  control_count: number
  compliance_claim: boolean
  regulatory_claim: boolean
  gitgov_certifies: boolean
  requires_auditor_review: boolean
  official_regulatory_mapping: boolean
  created_by_user_id: string
  created_at: number
  reviewed_by_user_id?: string | null
  reviewed_at?: number | null
  review_notes_safe?: string | null
  rejected_reason_safe?: string | null
  review_updated_at?: number | null
  archived_at?: number | null
}

export interface ComplianceFrameworkPackReviewRequest {
  org_name?: string | null
  review_status: 'needs_review' | 'reviewed' | 'needs_changes' | 'rejected' | 'archived' | string
  review_notes_safe?: string | null
  rejected_reason_safe?: string | null
}

export interface ComplianceFrameworkPackImportResponse {
  framework_pack: ComplianceFrameworkPackRecord
  framework: ComplianceControlFramework
}

export interface ComplianceFrameworkPackListResponse {
  framework_packs: ComplianceFrameworkPackRecord[]
}

export interface ComplianceFrameworkPackQuery {
  org_name?: string | null
}

export interface ComplianceFrameworkPackDiffQuery {
  org_name?: string | null
  base_pack_id: string
  target_pack_id: string
}

export interface ComplianceFrameworkPackDiffControlSide {
  title: string
  description: string
  required_evidence_types: string[]
}

export interface ComplianceFrameworkPackDiffControl {
  control_id: string
  change_type: 'added' | 'removed' | 'changed' | 'unchanged' | string
  base?: ComplianceFrameworkPackDiffControlSide | null
  target?: ComplianceFrameworkPackDiffControlSide | null
  changed_fields: string[]
}

export interface ComplianceFrameworkPackDiffSummary {
  added: number
  removed: number
  changed: number
  unchanged: number
}

export interface ComplianceFrameworkPackDiffResponse {
  base_pack: ComplianceFrameworkPackRecord
  target_pack: ComplianceFrameworkPackRecord
  original_framework_id: string
  same_original_framework: boolean
  summary: ComplianceFrameworkPackDiffSummary
  controls: ComplianceFrameworkPackDiffControl[]
  compliance_claim: boolean
  regulatory_claim: boolean
  gitgov_certifies: boolean
  official_regulatory_mapping: boolean
  requires_auditor_review: boolean
}

export interface ComplianceEvidenceMappingRequest {
  org_name?: string | null
  evidence_export_id: string
  framework_id: string
  framework_version?: string | null
}

export interface ComplianceEvidenceMappingQuery {
  org_name?: string | null
}

export interface ComplianceEvidenceMappingRecord {
  mapping_id: string
  org_id: string
  evidence_export_id: string
  evidence_export_hash: string
  framework_id: string
  framework_version: string
  created_by_user_id: string
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  created_at: number
}

export interface ComplianceEvidenceMappingItem {
  control_id: string
  control_title: string
  status: string
  evidence_refs: string[]
  missing_evidence: string[]
  notes_safe: string
}

export interface ComplianceEvidenceMappingResponse {
  mapping: ComplianceEvidenceMappingRecord
  items: ComplianceEvidenceMappingItem[]
}

export interface ComplianceReviewPackageRequest {
  org_name?: string | null
  mapping_id: string
  format?: string | null
  include_sections?: string[]
}

export interface ComplianceReviewPackageQuery {
  org_name?: string | null
}

export interface ComplianceReviewPackageRecord {
  review_package_id: string
  org_id: string
  created_by_user_id: string
  mapping_id: string
  evidence_export_id: string
  evidence_export_hash: string
  mapping_hash: string
  framework_id: string
  framework_version: string
  format: string
  artifact_hash: string
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  certification: boolean
  created_at: number
  downloaded_at?: number | null
  error_message_safe?: string | null
}

export interface ComplianceReviewPackageResponse {
  review_package: ComplianceReviewPackageRecord
  download_url: string
  artifact?: Record<string, unknown> | null
}

export interface ComplianceFrameworkReviewReportRequest {
  org_name?: string | null
  mapping_id: string
  review_package_id: string
  format?: string | null
}

export interface ComplianceFrameworkReviewReportQuery {
  org_name?: string | null
  framework_id?: string | null
  mapping_id?: string | null
  review_package_id?: string | null
  limit?: number | null
  assigned_to_me?: boolean | null
}

export interface ComplianceFrameworkReviewReportReviewRequest {
  org_name?: string | null
  review_status: 'needs_review' | 'reviewed' | 'needs_changes' | 'rejected' | string
  review_notes_safe?: string | null
}

export interface ComplianceFrameworkReviewReportRecord {
  report_id: string
  org_id: string
  created_by_user_id: string
  mapping_id: string
  review_package_id: string
  evidence_export_id: string
  evidence_export_hash: string
  mapping_hash: string
  review_package_hash: string
  framework_id: string
  framework_version: string
  framework_owner_type: string
  framework_review_status?: string | null
  pack_hash?: string | null
  format: string
  artifact_hash: string
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  certification: boolean
  review_status: string
  reviewed_by_user_id?: string | null
  reviewed_at?: number | null
  review_notes_safe?: string | null
  created_at: number
  downloaded_at?: number | null
  error_message_safe?: string | null
}

export interface ComplianceFrameworkReviewReportResponse {
  report: ComplianceFrameworkReviewReportRecord
  download_url: string
  artifact?: Record<string, unknown> | null
}

export interface ComplianceFrameworkReviewReportListResponse {
  items: ComplianceFrameworkReviewReportRecord[]
  count: number
  limit: number
}

export interface ComplianceFrameworkReviewReportAssignmentsRequest {
  org_name?: string | null
  auditor_client_ids: string[]
  assignment_notes_safe?: string | null
}

export interface ComplianceFrameworkReviewReportAssignmentQuery {
  org_name?: string | null
}

export interface ComplianceFrameworkReviewReportAssignmentRecord {
  id: string
  org_id: string
  report_id: string
  auditor_client_id: string
  assignment_status: string
  assigned_by_user_id: string
  assignment_notes_safe?: string | null
  created_at: number
  updated_at: number
}

export interface ComplianceFrameworkReviewReportAssignmentsResponse {
  assignments: ComplianceFrameworkReviewReportAssignmentRecord[]
  count: number
}

export interface ComplianceFrameworkReviewReportCommentRequest {
  org_name?: string | null
  comment_body_safe: string
  review_status_suggestion?: string | null
}

export interface ComplianceFrameworkReviewReportCommentsQuery {
  org_name?: string | null
}

export interface ComplianceFrameworkReviewReportCommentRecord {
  id: string
  org_id: string
  report_id: string
  commenter_client_id: string
  comment_body_safe: string
  review_status_suggestion?: string | null
  created_at: number
}

export interface ComplianceFrameworkReviewReportCommentsResponse {
  comments: ComplianceFrameworkReviewReportCommentRecord[]
  count: number
}

export interface ComplianceFrameworkReviewReportProvenanceManifestRequest {
  org_name?: string | null
}

export interface ComplianceFrameworkReviewReportProvenanceManifestRecord {
  manifest_id: string
  org_id: string
  report_id: string
  generated_by_user_id: string
  manifest_hash: string
  previous_manifest_hash?: string | null
  signature_algorithm: string
  created_at: number
}

export interface ComplianceFrameworkReviewReportProvenanceManifestResponse {
  manifest: ComplianceFrameworkReviewReportProvenanceManifestRecord
  download_url: string
  artifact: Record<string, unknown>
}

export interface ComplianceFrameworkReviewReportPdfExportRequest {
  org_name?: string | null
  manifest_id?: string | null
}

export interface ComplianceFrameworkReviewReportPdfExportQuery {
  org_name?: string | null
  pdf_export_id?: string | null
}

export interface ComplianceFrameworkReviewReportPdfExportRecord {
  pdf_export_id: string
  org_id: string
  report_id: string
  manifest_id: string
  created_by_user_id: string
  source_report_hash: string
  manifest_hash: string
  pdf_artifact_hash: string
  content_type: string
  page_count: number
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  certification: boolean
  created_at: number
  downloaded_at?: number | null
}

export interface ComplianceFrameworkReviewReportPdfExportResponse {
  pdf_export: ComplianceFrameworkReviewReportPdfExportRecord
  download_url: string
}

export interface ComplianceFrameworkReviewReportPdfDownloadResponse {
  pdf_export: ComplianceFrameworkReviewReportPdfExportRecord
  pdf_base64: string
}

export interface CompliancePeriodReportRequest {
  org_name?: string | null
  date_range_start: number
  date_range_end: number
  framework_id?: string | null
  format?: string | null
}

export interface CompliancePeriodReportQuery {
  org_name?: string | null
  framework_id?: string | null
  limit?: number | null
}

export interface CompliancePeriodReportReviewRequest {
  org_name?: string | null
  review_status: string
  review_notes_safe?: string | null
}

export interface CompliancePeriodReportProfileRequest {
  org_name?: string | null
  name: string
  period_type: 'monthly' | 'quarterly' | 'annual' | 'custom' | string
  framework_id?: string | null
  framework_owner_type?: 'gitgov_managed' | 'customer_provided' | string | null
  include_pdf?: boolean | null
  include_manifest?: boolean | null
  retention_days?: number | null
  filters?: Record<string, unknown> | null
}

export interface CompliancePeriodReportProfilePatchRequest {
  org_name?: string | null
  name?: string | null
  period_type?: 'monthly' | 'quarterly' | 'annual' | 'custom' | string | null
  framework_id?: string | null
  framework_owner_type?: 'gitgov_managed' | 'customer_provided' | string | null
  include_pdf?: boolean | null
  include_manifest?: boolean | null
  retention_days?: number | null
  filters?: Record<string, unknown> | null
}

export interface CompliancePeriodReportProfileQuery {
  org_name?: string | null
  framework_id?: string | null
  status?: 'active' | 'archived' | string | null
  limit?: number | null
}

export interface CompliancePeriodReportProfileRunRequest {
  org_name?: string | null
  date_range_start?: number | null
  date_range_end?: number | null
}

export interface CompliancePeriodReportProfileRecord {
  profile_id: string
  org_id: string
  created_by_user_id: string
  updated_by_user_id: string
  name: string
  period_type: string
  framework_id?: string | null
  framework_owner_type?: string | null
  include_pdf: boolean
  include_manifest: boolean
  retention_days: number
  filters: Record<string, unknown>
  status: string
  run_count: number
  last_run_at?: number | null
  last_period_report_id?: string | null
  last_pdf_export_id?: string | null
  last_manifest_id?: string | null
  archived_at?: number | null
  created_at: number
  updated_at: number
}

export interface CompliancePeriodReportProfileListResponse {
  items: CompliancePeriodReportProfileRecord[]
  count: number
  limit: number
}

export interface CompliancePeriodReportProfileResponse {
  profile: CompliancePeriodReportProfileRecord
}

export interface CompliancePeriodReportProfileRunResponse {
  profile: CompliancePeriodReportProfileRecord
  period_report: CompliancePeriodReportRecord
  pdf_export?: CompliancePeriodReportPdfExportRecord | null
  manifest?: CompliancePeriodReportProvenanceManifestRecord | null
  download_url: string
}

export interface CompliancePeriodReportRecord {
  period_report_id: string
  org_id: string
  created_by_user_id: string
  framework_id?: string | null
  date_range_start: number
  date_range_end: number
  report_count: number
  source_report_ids: string[]
  format: string
  status: string
  artifact_hash: string
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  certification: boolean
  review_status: 'needs_review' | 'reviewed' | 'needs_changes' | 'rejected' | string
  reviewed_by_user_id?: string | null
  reviewed_at?: number | null
  review_notes_safe?: string | null
  created_at: number
  downloaded_at?: number | null
  retention_status: 'active' | 'archived' | 'retention_expired' | string
  retention_until: number
  download_count: number
  last_downloaded_at?: number | null
  archived_at?: number | null
  error_message_safe?: string | null
}

export interface CompliancePeriodReportResponse {
  period_report: CompliancePeriodReportRecord
  download_url: string
  artifact?: Record<string, unknown> | null
}

export interface CompliancePeriodReportListResponse {
  items: CompliancePeriodReportRecord[]
  count: number
  limit: number
}

export interface CompliancePeriodReportRetentionRequest {
  org_name?: string | null
  retention_until?: number | null
  archive?: boolean
}

export interface CompliancePeriodReportAccessLogQuery {
  org_name?: string | null
  limit?: number | null
}

export interface CompliancePeriodReportAccessLogRecord {
  access_log_id: string
  org_id: string
  period_report_id: string
  actor_client_id: string
  action: string
  artifact_type: string
  artifact_id?: string | null
  artifact_hash?: string | null
  metadata: Record<string, unknown>
  created_at: number
}

export interface CompliancePeriodReportAccessLogResponse {
  items: CompliancePeriodReportAccessLogRecord[]
  count: number
  limit: number
}

export interface CompliancePeriodReportPdfExportRequest {
  org_name?: string | null
}

export interface CompliancePeriodReportPdfExportQuery {
  org_name?: string | null
  pdf_export_id?: string | null
}

export interface CompliancePeriodReportPdfExportRecord {
  pdf_export_id: string
  org_id: string
  period_report_id: string
  created_by_user_id: string
  source_period_report_hash: string
  pdf_artifact_hash: string
  content_type: string
  page_count: number
  compliance_claim: boolean
  regulatory_claim: boolean
  requires_auditor_review: boolean
  certification: boolean
  created_at: number
  downloaded_at?: number | null
}

export interface CompliancePeriodReportPdfExportResponse {
  pdf_export: CompliancePeriodReportPdfExportRecord
  download_url: string
}

export interface CompliancePeriodReportPdfDownloadResponse {
  pdf_export: CompliancePeriodReportPdfExportRecord
  pdf_base64: string
}

export interface CompliancePeriodReportProvenanceManifestRequest {
  org_name?: string | null
}

export interface CompliancePeriodReportProvenanceManifestQuery {
  org_name?: string | null
}

export interface CompliancePeriodReportProvenanceManifestRecord {
  manifest_id: string
  org_id: string
  period_report_id: string
  generated_by_user_id: string
  manifest_hash: string
  previous_manifest_hash?: string | null
  signature_algorithm: string
  created_at: number
}

export interface CompliancePeriodReportProvenanceManifestResponse {
  manifest: CompliancePeriodReportProvenanceManifestRecord
  download_url: string
  artifact: Record<string, unknown>
}

export interface CompliancePeriodReportSharePackageRequest {
  org_name?: string | null
}

export interface CompliancePeriodReportSharePackageQuery {
  org_name?: string | null
  status?: 'active' | 'revoked' | string | null
  limit?: number | null
}

export interface CompliancePeriodReportSharePackageRecord {
  share_package_id: string
  org_id: string
  period_report_id: string
  created_by_user_id: string
  period_report_artifact_hash: string
  pdf_export_id: string
  pdf_artifact_hash: string
  manifest_id: string
  manifest_hash: string
  artifact_hash: string
  package_format: string
  status: 'active' | 'revoked' | string
  no_claims_snapshot: Record<string, unknown>
  source_hashes: Record<string, unknown>
  download_count: number
  downloaded_at?: number | null
  last_downloaded_at?: number | null
  revoked_by_user_id?: string | null
  revoked_at?: number | null
  created_at: number
  error_message_safe?: string | null
}

export interface CompliancePeriodReportSharePackageResponse {
  share_package: CompliancePeriodReportSharePackageRecord
  download_url: string
  artifact?: Record<string, unknown> | null
}

export interface CompliancePeriodReportSharePackageListResponse {
  items: CompliancePeriodReportSharePackageRecord[]
  count: number
  limit: number
}

export interface EnterpriseReleaseApprovalQuery {
  org_name?: string | null
  repository_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  release_id?: string | null
  environment?: string | null
  decision?: EnterpriseReleaseApprovalDecision | '' | null
  evidence_packet_hash?: string | null
  limit?: number | null
  offset?: number | null
}

export type EnterpriseReleaseGovernanceEvaluationStatus =
  | 'recorded'
  | 'advisory-warning'
  | 'approved'
  | 'would-block'
  | 'blocked'
  | string

export interface EnterpriseReleaseGovernanceEvaluationQuery {
  org_name?: string | null
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  release_id: string
  environment: string
  evidence_packet_hash?: string | null
}

export interface EnterpriseReleaseGovernanceQuorumRuleSummary {
  role: string
  required: number
  observed: number
  satisfied: boolean
}

export interface EnterpriseReleaseGovernancePolicySummary {
  mode: string
  environment: string
  approval_required: boolean
  enforcement: string
  policy_applies: boolean
  quorum_enabled: boolean
  quorum_rules: EnterpriseReleaseGovernanceQuorumRuleSummary[]
}

export interface EnterpriseReleaseGovernanceApprovalSummary {
  id: string
  decision: string
  approver: string
  approver_role?: string | null
  risk_severity: string
  evidence_packet_hash?: string | null
  expires_at?: number | null
  created_at: number
  counts_toward_policy: boolean
}

export interface EnterpriseReleaseGovernanceEvaluationResponse {
  status: EnterpriseReleaseGovernanceEvaluationStatus
  policy_satisfied: boolean
  blocking: boolean
  would_block: boolean
  valid_approval_count: number
  required_approval_count: number
  policy: EnterpriseReleaseGovernancePolicySummary
  approvals: EnterpriseReleaseGovernanceApprovalSummary[]
  issues: string[]
  next_steps: string[]
}

export interface CreateEnterpriseReleaseApprovalRequest {
  org_name?: string | null
  release_id: string
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  environment: string
  decision: EnterpriseReleaseApprovalDecision
  approver: string
  ticket_id?: string | null
  evidence_packet_hash?: string | null
  evidence_packet_uri?: string | null
  evidence_summary?: Record<string, unknown>
  risk_severity?: EnterpriseReleaseApprovalRiskSeverity | null
  risk_acceptance_reason?: string | null
  expires_at?: number | null
}

export interface JiraCoverageFilters {
  hours: number
  repo_full_name: string
  branch: string
}

export interface ActiveDev7dEntry {
  user_login: string
  events: number
  last_seen: number
  suspicious_test_data: boolean
  sample_repo_empty_count: number
}

export interface ApiKeyInfo {
  id: string
  client_id: string
  role: string
  org_id: string | null
  org_name?: string | null
  created_at: number
  last_used: number | null
  is_active: boolean
}

export interface MeResponse {
  client_id: string
  role: string
  principal_type?: string | null
  platform_principal_id?: string | null
  org_id: string | null
  org_name?: string | null
  requires_workspace_for_tenant_surfaces?: boolean
}

export interface PendingControlPlaneSession {
  client_id: string
  role: string
  org_id: string | null
  org_name: string | null
}

export interface OrgSummary {
  id: string
  github_id?: number | null
  login: string
  name?: string | null
  avatar_url?: string | null
  created_at: number
}

export interface RevokeApiKeyResponse {
  success: boolean
  message: string
}

export interface OrgUser {
  id: string
  org_id: string
  login: string
  display_name: string | null
  email: string | null
  role: string
  status: string
  created_by: string | null
  updated_by: string | null
  created_at: number
  updated_at: number
}

export interface OrgInvitation {
  id: string
  org_id: string
  invite_email: string | null
  invite_login: string | null
  role: string
  status: string
  invited_by: string
  accepted_by: string | null
  accepted_at: number | null
  revoked_by: string | null
  revoked_at: number | null
  expires_at: number
  created_at: number
  updated_at: number
}

export interface CreateOrgResponse {
  org_id: string
  login: string
  created: boolean
}

export interface CreateOrgUserResponse {
  user: OrgUser
  created: boolean
}

export interface OrgUsersResponse {
  entries: OrgUser[]
  total: number
}

export interface CreateOrgInvitationResponse {
  invitation: OrgInvitation
  invite_token: string
}

export interface OrgInvitationsResponse {
  entries: OrgInvitation[]
  total: number
}

export interface AcceptOrgInvitationResponse {
  invitation: OrgInvitation
  client_id: string
  role: string
  org_id: string
  api_key: string
}

export interface IssueOrgUserApiKeyResponse {
  api_key: string | null
  client_id: string
  error: string | null
}

export interface TeamRepoSummary {
  repo_name: string
  events: number
  commits: number
  pushes: number
  blocked_pushes: number
  last_seen: number
}

export interface TeamDeveloperOverview {
  login: string
  display_name: string | null
  email: string | null
  role: string
  status: string
  last_seen: number | null
  total_events: number
  commits: number
  pushes: number
  blocked_pushes: number
  repos_active_count: number
  repos: TeamRepoSummary[]
}

export interface TeamRepoOverview {
  repo_name: string
  developers_active: number
  total_events: number
  commits: number
  pushes: number
  blocked_pushes: number
  last_seen: number
}

export interface TeamOverviewResponse {
  entries: TeamDeveloperOverview[]
  total: number
}

export interface TeamReposResponse {
  entries: TeamRepoOverview[]
  total: number
}

export interface ExportResponse {
  id: string
  export_type: string
  record_count: number
  content_hash: string
  data?: unknown
  created_at: number
}

export interface ExportLogEntry {
  id: string
  org_id: string | null
  exported_by: string
  export_type: string
  date_range_start: number | null
  date_range_end: number | null
  filters: unknown
  record_count: number
  content_hash: string | null
  file_path: string | null
  created_at: number
}

// ── Chat interfaces ──────────────────────────────────────────────────────────

export interface ChatAskResponse {
  status: 'ok' | 'insufficient_data' | 'feature_not_available' | 'error'
  answer: string
  missing_capability?: string | null
  can_report_feature: boolean
  data_refs: string[]
  sources?: string[]
  entities_detected?: string[]
  time_range_used?: string | null
  actions_recommended?: string[]
  confidence?: number | null
  trace_id?: string | null
}

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  response?: ChatAskResponse
  timestamp: number
}

export interface GovernanceCopilotAskRequest {
  question: string
  org_name?: string | null
  repository_full_name?: string | null
  branch?: string | null
  ticket_id?: string | null
  release_id?: string | null
  environment?: string | null
  hours?: number | null
}

export interface GovernanceCopilotCitation {
  id: string
  label: string
  endpoint: string
  status: 'ok' | 'missing' | 'error' | 'skipped' | string
  httpStatus?: number | null
}

export interface GovernanceCopilotSource extends GovernanceCopilotCitation {
  summary?: unknown
}

export interface GovernanceCopilotResponse {
  success: boolean
  mode?: 'ai' | 'fallback' | string | null
  model?: string | null
  answer: string
  citations: GovernanceCopilotCitation[]
  sources: GovernanceCopilotSource[]
  warnings: string[]
}

export interface ChatSession {
  id: string
  title: string
  created_at: number
  updated_at: number
  messages: ChatMessage[]
}

export interface PolicyResponseData {
  version: string
  checksum: string
  config: import('@/lib/types').GitGovConfig
  source: import('@/lib/types').PolicySourceMetadata
  updated_at: number
}

export interface PolicyHistoryEntry {
  id: string
  repo_id: string
  config: import('@/lib/types').GitGovConfig
  checksum: string
  source?: import('@/lib/types').PolicySourceMetadata
  changed_by: string
  change_type: string
  previous_checksum: string | null
  created_at: number
}

export interface ControlPlaneState {
  serverConfig: ServerConfig | null
  serverStats: ServerStats | null
  serverLogs: CombinedEvent[]
  activeDevs7d: ActiveDev7dEntry[]
  activeDevs7dUpdatedAt: number | null
  logsPage: number
  logsPageSize: number
  jenkinsCorrelations: CommitPipelineCorrelation[]
  prMergeEvidence: PrMergeEvidenceEntry[]
  dailyActivity: DailyActivityPoint[]
  ticketCoverage: TicketCoverageStats | null
  jiraCoverageFilters: JiraCoverageFilters
  jiraTicketDetails: Record<string, JiraTicketDetail | null>
  jiraTicketDetailFetchedAt: Record<string, number>
  jiraTicketDetailLoading: Record<string, boolean>
  evidencePacket: EvidencePacket | null
  evidencePacketTicketId: string
  isEvidencePacketLoading: boolean
  enterpriseAdoptionProfile: EnterpriseAdoptionProfile | null
  enterpriseAdoptionProfileUpdatedAt: number | null
  isEnterpriseAdoptionProfileLoading: boolean
  isEnterpriseAdoptionProfileSaving: boolean
  enterpriseAdoptionProfileError: string | null
  enterpriseOnboardingChecklistTracking: EnterpriseOnboardingChecklistTracking | null
  enterpriseOnboardingChecklistTrackingUpdatedAt: number | null
  isEnterpriseOnboardingChecklistTrackingLoading: boolean
  isEnterpriseOnboardingChecklistTrackingSaving: boolean
  enterpriseOnboardingChecklistTrackingError: string | null
  firstGovernedRepoSetup: FirstGovernedRepoSetupRecord | null
  firstGovernedRepoSetupUpdatedAt: number | null
  isFirstGovernedRepoSetupLoading: boolean
  isFirstGovernedRepoSetupSaving: boolean
  firstGovernedRepoSetupError: string | null
  firstGovernedRepoWizardState: Record<string, unknown> | null
  isFirstGovernedRepoWizardLoading: boolean
  isFirstGovernedRepoWizardActionRunning: boolean
  firstGovernedRepoWizardError: string | null
  releaseApprovals: EnterpriseReleaseApprovalRecord[]
  releaseApprovalsTotal: number
  releaseApprovalsFilters: EnterpriseReleaseApprovalQuery
  deploymentGateAuthorizations: DeploymentGateAuthorizationRecord[]
  deploymentGateAuthorizationsTotal: number
  deploymentGateAuthorizationsFilters: DeploymentGateAuthorizationQuery
  deploymentGateAuthorizationsUpdatedAt: number | null
  changeRiskEvaluations: ChangeRiskEvaluationRecord[]
  changeRiskEvaluationsTotal: number
  changeRiskEvaluationsFilters: ChangeRiskEvaluationQuery
  changeRiskSelectedEvaluation: ChangeRiskEvaluationRecord | null
  isChangeRiskEvaluationsLoading: boolean
  isChangeRiskEvaluationCreating: boolean
  changeRiskError: string | null
  complianceEvidenceSelectedDeploymentGateId: string | null
  complianceControlFrameworks: ComplianceControlFramework[]
  complianceFrameworkPacks: ComplianceFrameworkPackRecord[]
  selectedComplianceFrameworkId: string
  complianceFrameworkImportResponse: ComplianceFrameworkPackImportResponse | null
  complianceFrameworkPackDiff: ComplianceFrameworkPackDiffResponse | null
  complianceEvidenceExport: ComplianceEvidenceExportResponse | null
  complianceEvidenceMapping: ComplianceEvidenceMappingResponse | null
  complianceReviewPackage: ComplianceReviewPackageResponse | null
  complianceReviewPackageArtifact: Record<string, unknown> | null
  complianceFrameworkReviewReport: ComplianceFrameworkReviewReportResponse | null
  complianceFrameworkReviewReports: ComplianceFrameworkReviewReportListResponse | null
  assignedComplianceFrameworkReviewReports: ComplianceFrameworkReviewReportListResponse | null
  complianceFrameworkReviewReportAssignments: ComplianceFrameworkReviewReportAssignmentsResponse | null
  complianceFrameworkReviewReportComments: ComplianceFrameworkReviewReportCommentsResponse | null
  complianceFrameworkReviewReportArtifact: Record<string, unknown> | null
  complianceFrameworkReviewReportProvenanceManifest: ComplianceFrameworkReviewReportProvenanceManifestResponse | null
  complianceFrameworkReviewReportPdfExport: ComplianceFrameworkReviewReportPdfExportResponse | null
  compliancePeriodReport: CompliancePeriodReportResponse | null
  compliancePeriodReports: CompliancePeriodReportListResponse | null
  compliancePeriodReportProfiles: CompliancePeriodReportProfileListResponse | null
  compliancePeriodReportProfile: CompliancePeriodReportProfileResponse | null
  compliancePeriodReportProfileRun: CompliancePeriodReportProfileRunResponse | null
  compliancePeriodReportArtifact: Record<string, unknown> | null
  compliancePeriodReportAccessLog: CompliancePeriodReportAccessLogResponse | null
  compliancePeriodReportPdfExport: CompliancePeriodReportPdfExportResponse | null
  compliancePeriodReportProvenanceManifest: CompliancePeriodReportProvenanceManifestResponse | null
  compliancePeriodReportSharePackages: CompliancePeriodReportSharePackageListResponse | null
  compliancePeriodReportSharePackage: CompliancePeriodReportSharePackageResponse | null
  compliancePeriodReportSharePackageArtifact: Record<string, unknown> | null
  releaseGovernanceEvaluation: EnterpriseReleaseGovernanceEvaluationResponse | null
  isReleaseGovernanceEvaluating: boolean
  isReleaseApprovalsLoading: boolean
  isDeploymentGateAuthorizationsLoading: boolean
  isComplianceFrameworksLoading: boolean
  isComplianceFrameworkPackImporting: boolean
  isComplianceFrameworkPackReviewing: boolean
  isComplianceFrameworkPackDiffLoading: boolean
  isComplianceEvidenceExportCreating: boolean
  isComplianceEvidenceMappingCreating: boolean
  isComplianceReviewPackageCreating: boolean
  isComplianceReviewPackageDownloading: boolean
  isComplianceFrameworkReviewReportCreating: boolean
  isComplianceFrameworkReviewReportsLoading: boolean
  isAssignedComplianceFrameworkReviewReportsLoading: boolean
  isComplianceFrameworkReviewReportAssignmentsLoading: boolean
  isComplianceFrameworkReviewReportAssignmentsSaving: boolean
  isComplianceFrameworkReviewReportCommentsLoading: boolean
  isComplianceFrameworkReviewReportCommenting: boolean
  isComplianceFrameworkReviewReportReviewing: boolean
  isComplianceFrameworkReviewReportDownloading: boolean
  isComplianceFrameworkReviewReportProvenanceManifestCreating: boolean
  isComplianceFrameworkReviewReportPdfExportCreating: boolean
  isComplianceFrameworkReviewReportPdfExportDownloading: boolean
  isCompliancePeriodReportCreating: boolean
  isCompliancePeriodReportsLoading: boolean
  isCompliancePeriodReportProfileCreating: boolean
  isCompliancePeriodReportProfilesLoading: boolean
  isCompliancePeriodReportProfileUpdating: boolean
  isCompliancePeriodReportProfileArchiving: boolean
  isCompliancePeriodReportProfileRunning: boolean
  isCompliancePeriodReportDownloading: boolean
  isCompliancePeriodReportReviewing: boolean
  isCompliancePeriodReportRetentionUpdating: boolean
  isCompliancePeriodReportAccessLogLoading: boolean
  isCompliancePeriodReportPdfExportCreating: boolean
  isCompliancePeriodReportPdfExportDownloading: boolean
  isCompliancePeriodReportProvenanceManifestCreating: boolean
  isCompliancePeriodReportProvenanceManifestDownloading: boolean
  isCompliancePeriodReportSharePackageCreating: boolean
  isCompliancePeriodReportSharePackagesLoading: boolean
  isCompliancePeriodReportSharePackageDownloading: boolean
  isCompliancePeriodReportSharePackageRevoking: boolean
  isReleaseApprovalSubmitting: boolean
  releaseApprovalError: string | null
  complianceEvidenceError: string | null
  userRole: string | null
  userClientId: string | null
  userOrgId: string | null
  controlPlaneAuthConfirmed: boolean
  pendingControlPlaneSession: PendingControlPlaneSession | null
  selectedOrgName: string
  selectedOrgValidated: boolean
  availableOrgs: OrgSummary[]
  isLoadingOrgs: boolean
  orgUsers: OrgUser[]
  orgUsersTotal: number
  orgInvitations: OrgInvitation[]
  orgInvitationsTotal: number
  lastGeneratedInviteToken: string | null
  teamOverview: TeamDeveloperOverview[]
  teamOverviewTotal: number
  teamRepos: TeamRepoOverview[]
  teamReposTotal: number
  teamWindowDays: number
  teamStatusFilter: '' | 'active' | 'disabled'
  apiKeys: ApiKeyInfo[]
  isLoadingApiKeys: boolean
  exportLogs: ExportLogEntry[]
  connectionStatus: 'connected' | 'disconnected' | 'maintenance' | 'checking'
  maintenanceDetectedAt: number | null
  isConnected: boolean
  isLoading: boolean
  isRefreshingDashboard: boolean
  error: string | null
  chatSessions: ChatSession[]
  activeChatSessionId: string | null
  chatMessages: ChatMessage[]
  isChatLoading: boolean
  governanceCopilotResponse: GovernanceCopilotResponse | null
  isGovernanceCopilotLoading: boolean
  governanceCopilotError: string | null
  displayTimezone: string
  policyData: PolicyResponseData | null
  policyHistory: PolicyHistoryEntry[]
  isPolicyLoading: boolean
  isPolicySaving: boolean
  policyError: string | null
  sseConnected: boolean
}

export interface ControlPlaneActions {
  initFromEnv: () => Promise<void>
  setServerConfig: (config: ServerConfig) => void
  applyEnvApiKey: () => Promise<boolean>
  applyApiKey: (apiKey: string, url?: string) => Promise<boolean>
  markControlPlaneSessionValidated: (session: PendingControlPlaneSession) => void
  confirmControlPlaneSession: () => void
  resetControlPlaneAuthGate: () => void
  checkConnection: (options?: { background?: boolean }) => Promise<void>
  refreshDashboardData: (params?: { logLimit?: number; forceHeavy?: boolean }) => Promise<void>
  loadStats: () => Promise<void>
  loadDailyActivity: (days?: number) => Promise<void>
  loadLogs: (limit?: number, offset?: number) => Promise<void>
  loadLogsIncremental: (limit?: number) => Promise<void>
  loadActiveDevs7d: () => Promise<void>
  setLogsPage: (page: number) => void
  loadJenkinsCorrelations: (limit?: number) => Promise<void>
  loadPrMergeEvidence: (limit?: number) => Promise<void>
  loadTicketCoverage: (params?: { hours?: number; repo_full_name?: string; branch?: string; org_name?: string }) => Promise<void>
  applyTicketCoverageFilters: (filters: Partial<JiraCoverageFilters>) => Promise<void>
  correlateJiraTickets: (params?: { hours?: number; limit?: number; repo_full_name?: string; org_name?: string }) => Promise<JiraCorrelateResponse | null>
  loadJiraTicketDetail: (ticketId: string) => Promise<JiraTicketDetail | null>
  loadTicketEvidencePacket: (ticketId: string, params?: { hours?: number; repo_full_name?: string; branch?: string; target_sha?: string; release_id?: string; environment?: string; org_name?: string }) => Promise<EvidencePacket | null>
  loadEnterpriseAdoptionProfile: (orgName?: string) => Promise<EnterpriseAdoptionProfile | null>
  saveEnterpriseAdoptionProfile: (profile: EnterpriseAdoptionProfile, orgName?: string) => Promise<boolean>
  loadEnterpriseOnboardingChecklistTracking: (orgName?: string) => Promise<EnterpriseOnboardingChecklistTracking | null>
  saveEnterpriseOnboardingChecklistTracking: (tracking: EnterpriseOnboardingChecklistTracking, orgName?: string) => Promise<boolean>
  loadFirstGovernedRepoSetup: (orgName?: string) => Promise<FirstGovernedRepoSetupRecord | null>
  saveFirstGovernedRepoSetup: (payload: UpsertFirstGovernedRepoSetupRequest, orgName?: string) => Promise<FirstGovernedRepoSetupRecord | null>
  loadFirstGovernedRepoWizardState: (orgName?: string) => Promise<FirstGovernedRepoWizardStateResponse | null>
  createFirstGovernedRepoWizardRun: (payload: FirstGovernedRepoWizardActionRequest, orgName?: string) => Promise<FirstGovernedRepoWizardRunResponse | null>
  updateFirstGovernedRepoWizardRun: (runId: string, payload: FirstGovernedRepoWizardActionRequest, orgName?: string) => Promise<FirstGovernedRepoWizardRunResponse | null>
  validateFirstGovernedRepoWizardRun: (runId: string, payload: FirstGovernedRepoWizardActionRequest, orgName?: string) => Promise<FirstGovernedRepoWizardRunResponse | null>
  planFirstGovernedRepoWizardRun: (runId: string, payload: FirstGovernedRepoWizardActionRequest, orgName?: string) => Promise<FirstGovernedRepoWizardRunResponse | null>
  completeFirstGovernedRepoWizardRun: (runId: string, payload: FirstGovernedRepoWizardActionRequest, orgName?: string) => Promise<FirstGovernedRepoWizardRunResponse | null>
  loadEnterpriseReleaseApprovals: (query?: EnterpriseReleaseApprovalQuery) => Promise<EnterpriseReleaseApprovalListResponse | null>
  loadDeploymentGateAuthorizations: (query?: DeploymentGateAuthorizationQuery) => Promise<DeploymentGateAuthorizationListResponse | null>
  loadChangeRiskEvaluations: (query?: ChangeRiskEvaluationQuery) => Promise<ChangeRiskEvaluationListResponse | null>
  getChangeRiskEvaluation: (evaluationId: string, query?: ChangeRiskEvaluationQuery) => Promise<ChangeRiskEvaluationRecord | null>
  createChangeRiskEvaluation: (payload: ChangeRiskEvaluationRequest) => Promise<ChangeRiskEvaluationRecord | null>
  loadComplianceFrameworks: () => Promise<ComplianceControlFramework[]>
  importComplianceFrameworkPack: (content: string, format?: 'json' | 'yaml' | 'yml') => Promise<ComplianceFrameworkPackImportResponse | null>
  reviewComplianceFrameworkPack: (
    frameworkPackId: string,
    reviewStatus: ComplianceFrameworkPackReviewRequest['review_status'],
    notes?: { review_notes_safe?: string | null; rejected_reason_safe?: string | null },
  ) => Promise<ComplianceFrameworkPackRecord | null>
  loadComplianceFrameworkPackDiff: (
    basePackId: string,
    targetPackId: string,
  ) => Promise<ComplianceFrameworkPackDiffResponse | null>
  selectComplianceFramework: (frameworkId: string) => void
  createComplianceEvidenceExport: (deploymentGateId: string) => Promise<ComplianceEvidenceExportResponse | null>
  createComplianceEvidenceMapping: (exportId: string, frameworkId?: string) => Promise<ComplianceEvidenceMappingResponse | null>
  createComplianceReviewPackage: (mappingId: string) => Promise<ComplianceReviewPackageResponse | null>
  downloadComplianceReviewPackage: (reviewPackageId: string) => Promise<Record<string, unknown> | null>
  createComplianceFrameworkReviewReport: (mappingId: string, reviewPackageId: string) => Promise<ComplianceFrameworkReviewReportResponse | null>
  loadComplianceFrameworkReviewReports: (filters?: Omit<ComplianceFrameworkReviewReportQuery, 'org_name'>) => Promise<ComplianceFrameworkReviewReportListResponse | null>
  loadAssignedComplianceFrameworkReviewReports: (filters?: Omit<ComplianceFrameworkReviewReportQuery, 'org_name' | 'assigned_to_me'>) => Promise<ComplianceFrameworkReviewReportListResponse | null>
  loadComplianceFrameworkReviewReportAssignments: (reportId: string) => Promise<ComplianceFrameworkReviewReportAssignmentsResponse | null>
  saveComplianceFrameworkReviewReportAssignments: (
    reportId: string,
    auditorClientIds: string[],
    assignmentNotesSafe?: string | null,
  ) => Promise<ComplianceFrameworkReviewReportAssignmentsResponse | null>
  loadComplianceFrameworkReviewReportComments: (reportId: string) => Promise<ComplianceFrameworkReviewReportCommentsResponse | null>
  createComplianceFrameworkReviewReportComment: (
    reportId: string,
    commentBodySafe: string,
    reviewStatusSuggestion?: string | null,
  ) => Promise<ComplianceFrameworkReviewReportCommentRecord | null>
  reviewComplianceFrameworkReviewReport: (
    reportId: string,
    reviewStatus: ComplianceFrameworkReviewReportReviewRequest['review_status'],
    reviewNotesSafe?: string | null,
  ) => Promise<ComplianceFrameworkReviewReportResponse | null>
  downloadComplianceFrameworkReviewReport: (reportId: string) => Promise<Record<string, unknown> | null>
  createComplianceFrameworkReviewReportProvenanceManifest: (reportId: string) => Promise<ComplianceFrameworkReviewReportProvenanceManifestResponse | null>
  createComplianceFrameworkReviewReportPdfExport: (reportId: string, manifestId?: string | null) => Promise<ComplianceFrameworkReviewReportPdfExportResponse | null>
  downloadComplianceFrameworkReviewReportPdfExport: (reportId: string, pdfExportId?: string | null) => Promise<ComplianceFrameworkReviewReportPdfDownloadResponse | null>
  createCompliancePeriodReport: (
    dateRangeStart: number,
    dateRangeEnd: number,
    frameworkId?: string | null,
  ) => Promise<CompliancePeriodReportResponse | null>
  loadCompliancePeriodReports: (filters?: Omit<CompliancePeriodReportQuery, 'org_name'>) => Promise<CompliancePeriodReportListResponse | null>
  createCompliancePeriodReportProfile: (
    payload: Omit<CompliancePeriodReportProfileRequest, 'org_name'>,
  ) => Promise<CompliancePeriodReportProfileResponse | null>
  loadCompliancePeriodReportProfiles: (
    filters?: Omit<CompliancePeriodReportProfileQuery, 'org_name'>,
  ) => Promise<CompliancePeriodReportProfileListResponse | null>
  updateCompliancePeriodReportProfile: (
    profileId: string,
    payload: Omit<CompliancePeriodReportProfilePatchRequest, 'org_name'>,
  ) => Promise<CompliancePeriodReportProfileResponse | null>
  archiveCompliancePeriodReportProfile: (profileId: string) => Promise<CompliancePeriodReportProfileResponse | null>
  runCompliancePeriodReportProfile: (
    profileId: string,
    payload?: Omit<CompliancePeriodReportProfileRunRequest, 'org_name'>,
  ) => Promise<CompliancePeriodReportProfileRunResponse | null>
  downloadCompliancePeriodReport: (periodReportId: string) => Promise<Record<string, unknown> | null>
  reviewCompliancePeriodReport: (
    periodReportId: string,
    reviewStatus: string,
    reviewNotesSafe?: string | null,
  ) => Promise<CompliancePeriodReportResponse | null>
  updateCompliancePeriodReportRetention: (
    periodReportId: string,
    payload: Omit<CompliancePeriodReportRetentionRequest, 'org_name'>,
  ) => Promise<CompliancePeriodReportResponse | null>
  loadCompliancePeriodReportAccessLog: (
    periodReportId: string,
    query?: Omit<CompliancePeriodReportAccessLogQuery, 'org_name'>,
  ) => Promise<CompliancePeriodReportAccessLogResponse | null>
  createCompliancePeriodReportPdfExport: (periodReportId: string) => Promise<CompliancePeriodReportPdfExportResponse | null>
  downloadCompliancePeriodReportPdfExport: (periodReportId: string, pdfExportId?: string | null) => Promise<CompliancePeriodReportPdfDownloadResponse | null>
  createCompliancePeriodReportProvenanceManifest: (periodReportId: string) => Promise<CompliancePeriodReportProvenanceManifestResponse | null>
  downloadCompliancePeriodReportProvenanceManifest: (periodReportId: string, manifestId: string) => Promise<Record<string, unknown> | null>
  createCompliancePeriodReportSharePackage: (periodReportId: string) => Promise<CompliancePeriodReportSharePackageResponse | null>
  loadCompliancePeriodReportSharePackages: (
    periodReportId: string,
    query?: Omit<CompliancePeriodReportSharePackageQuery, 'org_name'>,
  ) => Promise<CompliancePeriodReportSharePackageListResponse | null>
  downloadCompliancePeriodReportSharePackage: (sharePackageId: string) => Promise<Record<string, unknown> | null>
  revokeCompliancePeriodReportSharePackage: (sharePackageId: string) => Promise<CompliancePeriodReportSharePackageResponse | null>
  resetComplianceEvidenceFlow: () => void
  evaluateEnterpriseReleaseGovernance: (query: EnterpriseReleaseGovernanceEvaluationQuery) => Promise<EnterpriseReleaseGovernanceEvaluationResponse | null>
  createEnterpriseReleaseApproval: (payload: CreateEnterpriseReleaseApprovalRequest) => Promise<EnterpriseReleaseApprovalRecord | null>
  loadMe: () => Promise<boolean>
  loadOrgs: () => Promise<OrgSummary[]>
  validateOrgName: (orgName: string) => Promise<OrgSummary | null>
  activateOrgName: (orgName: string) => Promise<OrgSummary | null>
  createOrg: (payload: { login: string; name?: string }) => Promise<CreateOrgResponse | null>
  setSelectedOrgName: (orgName: string) => void
  loadOrgUsers: (params?: { orgName?: string; status?: string; limit?: number; offset?: number }) => Promise<void>
  upsertOrgUser: (payload: {
    orgName?: string
    login: string
    email?: string
    displayName?: string
    role?: string
    status?: string
  }) => Promise<OrgUser | null>
  updateOrgUserStatus: (userId: string, status: 'active' | 'disabled') => Promise<OrgUser | null>
  issueApiKeyForOrgUser: (userId: string) => Promise<IssueOrgUserApiKeyResponse | null>
  loadOrgInvitations: (params?: { orgName?: string; status?: string; limit?: number; offset?: number }) => Promise<void>
  createOrgInvitation: (payload: {
    orgName?: string
    inviteEmail?: string
    inviteLogin?: string
    role?: string
    expiresInDays?: number
  }) => Promise<CreateOrgInvitationResponse | null>
  resendOrgInvitation: (invitationId: string, expiresInDays?: number) => Promise<CreateOrgInvitationResponse | null>
  revokeOrgInvitation: (invitationId: string) => Promise<boolean>
  previewOrgInvitation: (token: string) => Promise<OrgInvitation | null>
  acceptOrgInvitation: (payload: { token: string; login?: string }) => Promise<AcceptOrgInvitationResponse | null>
  setTeamFilters: (filters: { days?: number; status?: '' | 'active' | 'disabled' }) => void
  loadTeamOverview: (params?: { orgName?: string; days?: number; status?: '' | 'active' | 'disabled'; limit?: number; offset?: number; append?: boolean }) => Promise<void>
  loadTeamRepos: (params?: { orgName?: string; days?: number; limit?: number; offset?: number; append?: boolean }) => Promise<void>
  refreshForCurrentRole: (options?: { forceHeavy?: boolean }) => Promise<void>
  loadApiKeys: (params?: { orgName?: string | null; global?: boolean }) => Promise<void>
  revokeApiKey: (keyId: string, params?: { orgName?: string | null; global?: boolean }) => Promise<boolean>
  exportAuditData: (params: { exportType?: string; startDate?: number; endDate?: number; orgName?: string }) => Promise<ExportResponse | null>
  loadExportLogs: () => Promise<void>
  clearError: () => void
  disconnect: () => void
  chatAsk: (question: string, orgName?: string) => Promise<ChatAskResponse | null>
  askGovernanceCopilot: (request: GovernanceCopilotAskRequest) => Promise<GovernanceCopilotResponse | null>
  reportFeature: (question: string, missingCapability?: string) => Promise<boolean>
  clearChatMessages: () => void
  createChatSession: () => void
  setActiveChatSession: (sessionId: string) => void
  closeChatSession: (sessionId: string) => void
  refreshChatMessagesForActiveUser: () => void
  setDisplayTimezone: (tz: string) => void
  loadPolicy: (repoName: string) => Promise<void>
  savePolicy: (repoName: string, config: import('@/lib/types').GitGovConfig) => Promise<boolean>
  loadPolicyHistory: (repoName: string) => Promise<void>
  connectSse: () => Promise<void>
  disconnectSse: () => void
}
