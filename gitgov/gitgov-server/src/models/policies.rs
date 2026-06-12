use super::*;

pub use gitgov_policy_core::{
    EnforcementLevel, GitGovConfig, PolicySourceMetadata, QualityGateExceptionConfig,
};

// Compatibility reexports for the historical `models::*` policy contract. Some of these types are
// consumed by downstream clients/tests even when this crate does not construct them directly.
#[allow(unused_imports)]
pub use gitgov_policy_core::{
    BranchConfig, ChecklistConfig, EnforcementConfig, ExternalPolicyEffect,
    ExternalPolicyFailureMode, GroupConfig, OpaAdapterConfig, OpaResultMapping,
    PolicyAdaptersConfig, PolicyDriftStatus, PolicyEmergencyOverride, PolicyFormat,
    PolicySourceMode, PolicySourceSettings, RulesConfig,
};

// POLICIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub version: String,
    pub checksum: String,
    pub config: GitGovConfig,
    #[serde(default)]
    pub source: PolicySourceMetadata,
    pub updated_at: i64,
}

// ============================================================================
