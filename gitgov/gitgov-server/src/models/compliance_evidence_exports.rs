use super::*;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_gate_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    pub status: String,
    pub format: String,
    pub artifact_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_decision: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_safe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceExportResponse {
    pub export: ComplianceEvidenceExportRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<serde_json::Value>,
}
