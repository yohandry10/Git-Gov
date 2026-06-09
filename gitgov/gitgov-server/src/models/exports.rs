use super::*;

// EXPORT LOGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportLog {
    pub id: String,
    pub org_id: Option<String>,
    pub exported_by: String,
    pub export_type: String,
    pub date_range_start: Option<i64>,
    pub date_range_end: Option<i64>,
    pub filters: serde_json::Value,
    pub record_count: i32,
    pub content_hash: Option<String>,
    pub file_path: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub export_type: String,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub filters: Option<serde_json::Value>,
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub id: String,
    pub export_type: String,
    pub record_count: i32,
    pub content_hash: String,
    pub data: Option<serde_json::Value>,
    pub created_at: i64,
}

// ============================================================================
