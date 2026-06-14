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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    pub name: String,
    pub version: String,
    pub description: String,
    pub is_regulatory: bool,
    pub is_active: bool,
    pub owner_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    pub source: String,
    pub is_gitgov_owned: bool,
    pub official_regulatory_mapping: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_pack_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceFrameworkPackQuery {
    #[serde(default)]
    pub org_name: Option<String>,
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
