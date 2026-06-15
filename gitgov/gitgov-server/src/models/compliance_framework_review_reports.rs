use super::*;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework_review_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_hash: Option<String>,
    pub format: String,
    pub artifact_hash: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportResponse {
    pub report: ComplianceFrameworkReviewReportRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkReviewReportPdfExportResponse {
    pub pdf_export: ComplianceFrameworkReviewReportPdfExportRecord,
    pub download_url: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportReviewRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub review_status: String,
    #[serde(default)]
    pub review_notes_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProfileRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub name: String,
    pub period_type: String,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub framework_owner_type: Option<String>,
    #[serde(default)]
    pub include_pdf: Option<bool>,
    #[serde(default)]
    pub include_manifest: Option<bool>,
    #[serde(default)]
    pub retention_days: Option<i32>,
    #[serde(default)]
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProfilePatchRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub period_type: Option<String>,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub framework_owner_type: Option<String>,
    #[serde(default)]
    pub include_pdf: Option<bool>,
    #[serde(default)]
    pub include_manifest: Option<bool>,
    #[serde(default)]
    pub retention_days: Option<i32>,
    #[serde(default)]
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProfileQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProfileRunRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub date_range_start: Option<i64>,
    #[serde(default)]
    pub date_range_end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProfileRecord {
    pub profile_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    pub updated_by_user_id: String,
    pub name: String,
    pub period_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework_owner_type: Option<String>,
    pub include_pdf: bool,
    pub include_manifest: bool,
    pub retention_days: i32,
    pub filters: serde_json::Value,
    pub status: String,
    pub run_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_period_report_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_pdf_export_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_manifest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProfileListResponse {
    pub items: Vec<CompliancePeriodReportProfileRecord>,
    pub count: usize,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProfileResponse {
    pub profile: CompliancePeriodReportProfileRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProfileRunResponse {
    pub profile: CompliancePeriodReportProfileRecord,
    pub period_report: CompliancePeriodReportRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_export: Option<CompliancePeriodReportPdfExportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<CompliancePeriodReportProvenanceManifestRecord>,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportRecord {
    pub period_report_id: String,
    pub org_id: String,
    pub created_by_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework_id: Option<String>,
    pub date_range_start: i64,
    pub date_range_end: i64,
    pub report_count: i32,
    pub source_report_ids: Vec<String>,
    pub format: String,
    pub status: String,
    pub artifact_hash: String,
    pub compliance_claim: bool,
    pub regulatory_claim: bool,
    pub requires_auditor_review: bool,
    pub certification: bool,
    pub review_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_notes_safe: Option<String>,
    pub created_at: i64,
    pub retention_status: String,
    pub retention_until: i64,
    pub download_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportResponse {
    pub period_report: CompliancePeriodReportRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportPdfExportResponse {
    pub pdf_export: CompliancePeriodReportPdfExportRecord,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProvenanceManifestRequest {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportProvenanceManifestQuery {
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProvenanceManifestRecord {
    pub manifest_id: String,
    pub org_id: String,
    pub period_report_id: String,
    pub generated_by_user_id: String,
    pub manifest_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_manifest_hash: Option<String>,
    pub signature_algorithm: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportProvenanceManifestResponse {
    pub manifest: CompliancePeriodReportProvenanceManifestRecord,
    pub download_url: String,
    pub artifact: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportSharePackageRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompliancePeriodReportSharePackageQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportSharePackageRecord {
    pub share_package_id: String,
    pub org_id: String,
    pub period_report_id: String,
    pub created_by_user_id: String,
    pub package_format: String,
    pub status: String,
    pub artifact_hash: String,
    pub period_report_artifact_hash: String,
    pub pdf_export_id: String,
    pub pdf_artifact_hash: String,
    pub manifest_id: String,
    pub manifest_hash: String,
    pub no_claims_snapshot: serde_json::Value,
    pub source_hashes: serde_json::Value,
    pub review_snapshot: serde_json::Value,
    pub retention_snapshot: serde_json::Value,
    pub download_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_by_user_id: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportSharePackageResponse {
    pub share_package: CompliancePeriodReportSharePackageRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePeriodReportSharePackageListResponse {
    pub items: Vec<CompliancePeriodReportSharePackageRecord>,
    pub count: usize,
    pub limit: i64,
}
