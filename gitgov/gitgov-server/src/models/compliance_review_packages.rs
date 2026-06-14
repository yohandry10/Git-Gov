use super::*;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReviewPackageResponse {
    pub review_package: ComplianceReviewPackageRecord,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}
