use super::*;

// PULL REQUEST MERGES (V1.3-A)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrMergeRecord {
    pub id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    pub delivery_id: String,
    pub pr_number: i32,
    #[serde(default)]
    pub pr_title: Option<String>,
    #[serde(default)]
    pub author_login: Option<String>,
    #[serde(default)]
    pub merged_by_login: Option<String>,
    #[serde(default)]
    pub head_sha: Option<String>,
    #[serde(default)]
    pub base_branch: Option<String>,
    pub payload: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrMergeEvidenceEntry {
    pub id: String,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    pub delivery_id: String,
    pub pr_number: i32,
    #[serde(default)]
    pub pr_title: Option<String>,
    #[serde(default)]
    pub author_login: Option<String>,
    #[serde(default)]
    pub merged_by_login: Option<String>,
    #[serde(default)]
    pub approvers: Vec<String>,
    pub approvals_count: i32,
    #[serde(default)]
    pub head_sha: Option<String>,
    #[serde(default)]
    pub merge_commit_sha: Option<String>,
    #[serde(default)]
    pub base_branch: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrMergeEvidenceResponse {
    pub entries: Vec<PrMergeEvidenceEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrMergeEvidenceQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub merged_by: Option<String>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

// ============================================================================
