use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceEvidenceExportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub scope: String,
    #[serde(default)]
    pub deployment_gate_id: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub include_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceEvidenceExportQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceExportRecord {
    pub export_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    pub scope: String,
    #[serde(default)]
    pub deployment_gate_id: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    pub status: String,
    pub format: String,
    pub artifact_hash: String,
    #[serde(default)]
    pub policy_checksum: Option<String>,
    #[serde(default)]
    pub gate_decision: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceExportResponse {
    pub export: ComplianceEvidenceExportRecord,
    #[serde(default)]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControl {
    pub control_id: String,
    pub title: String,
    pub description: String,
    pub required_evidence_types: Vec<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControlFramework {
    pub framework_id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    pub name: String,
    pub version: String,
    pub description: String,
    pub is_regulatory: bool,
    pub is_active: bool,
    pub owner_type: String,
    #[serde(default)]
    pub owner_name: Option<String>,
    pub source: String,
    pub is_gitgov_owned: bool,
    pub official_regulatory_mapping: bool,
    #[serde(default)]
    pub framework_pack_id: Option<String>,
    #[serde(default)]
    pub pack_hash: Option<String>,
    #[serde(default)]
    pub framework_pack_review_status: Option<String>,
    #[serde(default)]
    pub framework_pack_reviewed_by_user_id: Option<String>,
    #[serde(default)]
    pub framework_pack_reviewed_at: Option<i64>,
    #[serde(default)]
    pub framework_pack_review_notes_safe: Option<String>,
    #[serde(default)]
    pub framework_pack_rejected_reason_safe: Option<String>,
    #[serde(default)]
    pub controls: Vec<ComplianceControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkPackImportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub pack: Option<serde_json::Value>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackRecord {
    pub framework_pack_id: String,
    pub org_id: String,
    pub framework_id: String,
    pub framework_name: String,
    pub framework_version: String,
    pub description: String,
    pub owner_type: String,
    pub owner_name: String,
    pub source: String,
    pub review_status: String,
    pub schema_version: String,
    pub pack_hash: String,
    pub control_count: i32,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub gitgov_certifies: bool,
    pub requires_auditor_review: bool,
    pub official_regulatory_mapping: bool,
    pub created_by_user_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub reviewed_by_user_id: Option<String>,
    #[serde(default)]
    pub reviewed_at: Option<i64>,
    #[serde(default)]
    pub review_notes_safe: Option<String>,
    #[serde(default)]
    pub rejected_reason_safe: Option<String>,
    #[serde(default)]
    pub review_updated_at: Option<i64>,
    #[serde(default)]
    pub archived_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkPackReviewRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub review_status: String,
    #[serde(default)]
    pub review_notes_safe: Option<String>,
    #[serde(default)]
    pub rejected_reason_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackImportResponse {
    pub framework_pack: ComplianceFrameworkPackRecord,
    pub framework: ComplianceControlFramework,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackListResponse {
    pub framework_packs: Vec<ComplianceFrameworkPackRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControlFrameworkListResponse {
    pub frameworks: Vec<ComplianceControlFramework>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkPackQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkPackDiffQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    pub base_pack_id: String,
    pub target_pack_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackDiffControlSide {
    pub title: String,
    pub description: String,
    pub required_evidence_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackDiffControl {
    pub control_id: String,
    pub change_type: String,
    #[serde(default)]
    pub base: Option<ComplianceFrameworkPackDiffControlSide>,
    #[serde(default)]
    pub target: Option<ComplianceFrameworkPackDiffControlSide>,
    #[serde(default)]
    pub changed_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackDiffSummary {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkPackDiffResponse {
    pub base_pack: ComplianceFrameworkPackRecord,
    pub target_pack: ComplianceFrameworkPackRecord,
    pub original_framework_id: String,
    pub same_original_framework: bool,
    pub summary: ComplianceFrameworkPackDiffSummary,
    pub controls: Vec<ComplianceFrameworkPackDiffControl>,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub gitgov_certifies: bool,
    pub official_regulatory_mapping: bool,
    pub requires_auditor_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceEvidenceMappingRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub evidence_export_id: String,
    pub framework_id: String,
    #[serde(default)]
    pub framework_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceEvidenceMappingQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceMappingRecord {
    pub mapping_id: String,
    pub org_id: String,
    pub evidence_export_id: String,
    pub evidence_export_hash: String,
    pub framework_id: String,
    pub framework_version: String,
    pub created_by_user_id: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceMappingItem {
    pub control_id: String,
    pub control_title: String,
    pub status: String,
    pub evidence_refs: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub notes_safe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceMappingResponse {
    pub mapping: ComplianceEvidenceMappingRecord,
    pub items: Vec<ComplianceEvidenceMappingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceReviewPackageRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub mapping_id: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub include_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceReviewPackageQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReviewPackageRecord {
    pub review_package_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    pub mapping_id: String,
    pub evidence_export_id: String,
    pub evidence_export_hash: String,
    pub mapping_hash: String,
    pub framework_id: String,
    pub framework_version: String,
    pub format: String,
    pub artifact_hash: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub created_at: i64,
    pub retention_status: String,
    pub retention_until: i64,
    pub download_count: i32,
    #[serde(default)]
    pub last_downloaded_at: Option<i64>,
    #[serde(default)]
    pub archived_at: Option<i64>,
    #[serde(default)]
    pub downloaded_at: Option<i64>,
    #[serde(default)]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReviewPackageResponse {
    pub review_package: ComplianceReviewPackageRecord,
    pub download_url: String,
    #[serde(default)]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub mapping_id: String,
    pub review_package_id: String,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub mapping_id: Option<String>,
    #[serde(default)]
    pub review_package_id: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub assigned_to_me: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportReviewRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub review_status: String,
    #[serde(default)]
    pub review_notes_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportRecord {
    pub report_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    pub mapping_id: String,
    pub review_package_id: String,
    pub evidence_export_id: String,
    pub evidence_export_hash: String,
    pub mapping_hash: String,
    pub review_package_hash: String,
    pub framework_id: String,
    pub framework_version: String,
    pub framework_owner_type: String,
    #[serde(default)]
    pub framework_review_status: Option<String>,
    #[serde(default)]
    pub pack_hash: Option<String>,
    pub format: String,
    pub artifact_hash: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub review_status: String,
    #[serde(default)]
    pub reviewed_by_user_id: Option<String>,
    #[serde(default)]
    pub reviewed_at: Option<i64>,
    #[serde(default)]
    pub review_notes_safe: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub downloaded_at: Option<i64>,
    #[serde(default)]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportResponse {
    pub report: ComplianceFrameworkReviewReportRecord,
    pub download_url: String,
    #[serde(default)]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportListResponse {
    pub items: Vec<ComplianceFrameworkReviewReportRecord>,
    pub count: usize,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportAssignmentsRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub auditor_client_ids: Vec<String>,
    #[serde(default)]
    pub assignment_notes_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportAssignmentQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportAssignmentRecord {
    pub id: String,
    pub org_id: String,
    pub report_id: String,
    pub auditor_client_id: String,
    pub assignment_status: String,
    pub assigned_by_user_id: String,
    #[serde(default)]
    pub assignment_notes_safe: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportAssignmentsResponse {
    pub assignments: Vec<ComplianceFrameworkReviewReportAssignmentRecord>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportCommentRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub comment_body_safe: String,
    #[serde(default)]
    pub review_status_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportCommentsQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportCommentRecord {
    pub id: String,
    pub org_id: String,
    pub report_id: String,
    pub commenter_client_id: String,
    pub comment_body_safe: String,
    #[serde(default)]
    pub review_status_suggestion: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportCommentsResponse {
    pub comments: Vec<ComplianceFrameworkReviewReportCommentRecord>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportProvenanceManifestRequest {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportProvenanceManifestRecord {
    pub manifest_id: String,
    pub org_id: String,
    pub report_id: String,
    pub generated_by_user_id: String,
    pub manifest_hash: String,
    #[serde(default)]
    pub previous_manifest_hash: Option<String>,
    pub signature_algorithm: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportProvenanceManifestResponse {
    pub manifest: ComplianceFrameworkReviewReportProvenanceManifestRecord,
    pub download_url: String,
    pub artifact: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportPdfExportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub manifest_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkReviewReportPdfExportQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub pdf_export_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportPdfExportRecord {
    pub pdf_export_id: String,
    pub org_id: String,
    pub report_id: String,
    pub manifest_id: String,
    pub created_by_user_id: String,
    pub source_report_hash: String,
    pub manifest_hash: String,
    pub pdf_artifact_hash: String,
    pub content_type: String,
    pub page_count: i32,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub created_at: i64,
    #[serde(default)]
    pub downloaded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportPdfExportResponse {
    pub pdf_export: ComplianceFrameworkReviewReportPdfExportRecord,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportPdfDownloadResponse {
    pub pdf_export: ComplianceFrameworkReviewReportPdfExportRecord,
    pub pdf_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub date_range_start: i64,
    pub date_range_end: i64,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportRecord {
    pub period_report_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    #[serde(default)]
    pub framework_id: Option<String>,
    pub date_range_start: i64,
    pub date_range_end: i64,
    pub report_count: i32,
    #[serde(default)]
    pub source_report_ids: Vec<String>,
    pub format: String,
    pub status: String,
    pub artifact_hash: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub created_at: i64,
    #[serde(default)]
    pub downloaded_at: Option<i64>,
    #[serde(default)]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportResponse {
    pub period_report: CompliancePeriodReportRecord,
    pub download_url: String,
    #[serde(default)]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportListResponse {
    pub items: Vec<CompliancePeriodReportRecord>,
    pub count: usize,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportRetentionRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub retention_until: Option<i64>,
    #[serde(default)]
    pub archive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportAccessLogQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportAccessLogRecord {
    pub access_log_id: String,
    pub org_id: String,
    pub period_report_id: String,
    pub actor_client_id: String,
    pub action: String,
    pub artifact_type: String,
    #[serde(default)]
    pub artifact_id: Option<String>,
    #[serde(default)]
    pub artifact_hash: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportAccessLogResponse {
    pub items: Vec<CompliancePeriodReportAccessLogRecord>,
    pub count: usize,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportPdfExportRequest {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportPdfExportQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub pdf_export_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportPdfExportRecord {
    pub pdf_export_id: String,
    pub org_id: String,
    pub period_report_id: String,
    pub created_by_user_id: String,
    pub source_period_report_hash: String,
    pub pdf_artifact_hash: String,
    pub content_type: String,
    pub page_count: i32,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub created_at: i64,
    #[serde(default)]
    pub downloaded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportPdfExportResponse {
    pub pdf_export: CompliancePeriodReportPdfExportRecord,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportPdfDownloadResponse {
    pub pdf_export: CompliancePeriodReportPdfExportRecord,
    pub pdf_base64: String,
}
