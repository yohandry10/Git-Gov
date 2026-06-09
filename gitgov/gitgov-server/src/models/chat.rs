use super::*;

// ============================================================================
// CONVERSATIONAL CHAT (MVP)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAskRequest {
    pub question: String,
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAskResponse {
    /// "ok" | "insufficient_data" | "feature_not_available" | "error"
    pub status: String,
    pub answer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_capability: Option<String>,
    pub can_report_feature: bool,
    #[serde(default)]
    pub data_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities_detected: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range_used: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions_recommended: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRequestInput {
    pub question: String,
    #[serde(default)]
    pub missing_capability: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub user_login: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRequestRecord {
    pub id: String,
    pub org_id: Option<String>,
    pub requested_by: String,
    pub question: String,
    pub missing_capability: Option<String>,
    pub status: String,
    pub created_at: i64,
}
