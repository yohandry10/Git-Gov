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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceEvidenceMappingRequest {
    #[serde(default)]
    pub org_name: Option<String>,
    pub evidence_export_id: String,
    pub framework_id: String,
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
