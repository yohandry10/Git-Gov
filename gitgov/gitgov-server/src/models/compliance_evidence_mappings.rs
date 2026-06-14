use super::*;

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
    pub name: String,
    pub version: String,
    pub description: String,
    pub is_regulatory: bool,
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<ComplianceControl>,
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
