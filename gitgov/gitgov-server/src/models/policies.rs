use super::*;

// POLICIES (gitgov.toml)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitGovConfig {
    #[serde(default)]
    pub branches: BranchConfig,
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
    #[serde(default)]
    pub admins: Vec<String>,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default)]
    pub checklist: ChecklistConfig,
    #[serde(default)]
    pub enforcement: EnforcementConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_gate_exception: Option<QualityGateExceptionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateExceptionConfig {
    #[serde(default)]
    pub enabled: bool,
    pub reason: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    pub expires_at: i64,
    #[serde(default)]
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchConfig {
    pub patterns: Vec<String>,
    pub protected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupConfig {
    pub members: Vec<String>,
    pub allowed_branches: Vec<String>,
    pub allowed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesConfig {
    #[serde(default)]
    pub require_pull_request: bool,
    #[serde(default)]
    pub min_approvals: u32,
    #[serde(default)]
    pub require_conventional_commits: bool,
    #[serde(default)]
    pub require_signed_commits: bool,
    #[serde(default)]
    pub max_files_per_commit: Option<u32>,
    #[serde(default)]
    pub require_linked_ticket: bool,
    #[serde(default)]
    pub block_force_push: bool,
    #[serde(default)]
    pub forbidden_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChecklistConfig {
    #[serde(default)]
    pub confirm: Vec<String>,
    #[serde(default)]
    pub auto_check: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnforcementConfig {
    #[serde(default)]
    pub pull_requests: EnforcementLevel,
    #[serde(default)]
    pub commits: EnforcementLevel,
    #[serde(default)]
    pub branches: EnforcementLevel,
    #[serde(default)]
    pub traceability: EnforcementLevel,
    #[serde(default)]
    pub quality_gates: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementLevel {
    #[default]
    Off,
    Warn,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub version: String,
    pub checksum: String,
    pub config: GitGovConfig,
    pub updated_at: i64,
}

// ============================================================================
