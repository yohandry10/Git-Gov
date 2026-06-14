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
