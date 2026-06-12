use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod diff;
pub mod git_validation;

pub use diff::*;
pub use git_validation::*;

pub const DEFAULT_POLICY_PATHS: &[(&str, PolicyFormat)] = &[
    (".gitgov/policy.yml", PolicyFormat::Yaml),
    (".gitgov/policy.yaml", PolicyFormat::Yaml),
    (".gitgov/policy.json", PolicyFormat::Json),
    ("gitgov.toml", PolicyFormat::Toml),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyFormat {
    Toml,
    Yaml,
    Json,
}

impl PolicyFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())?;

        match extension.as_str() {
            "toml" => Some(Self::Toml),
            "yml" | "yaml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicySourceMode {
    ControlPlaneManaged,
    RepoPolicyAsCode,
    HybridAdvisory,
}

impl Default for PolicySourceMode {
    fn default() -> Self {
        Self::ControlPlaneManaged
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyDriftStatus {
    InSync,
    Drifted,
    OverrideActive,
    Unknown,
}

impl Default for PolicyDriftStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicySourceMetadata {
    #[serde(default)]
    pub source_mode: PolicySourceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_format: Option<PolicyFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default)]
    pub reviewers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_checksum: Option<String>,
    #[serde(default)]
    pub drift_status: PolicyDriftStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emergency_override: Option<PolicyEmergencyOverride>,
}

impl PolicySourceMetadata {
    pub fn control_plane_managed(actor: impl Into<String>, checksum: impl Into<String>) -> Self {
        let checksum = checksum.into();
        Self {
            source_mode: PolicySourceMode::ControlPlaneManaged,
            actor: Some(actor.into()),
            source_checksum: Some(checksum.clone()),
            active_checksum: Some(checksum),
            drift_status: PolicyDriftStatus::InSync,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEmergencyOverride {
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    pub actor: String,
    pub expires_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySourceSettings {
    pub source_mode: PolicySourceMode,
    pub policy_path: String,
    pub policy_format: PolicyFormat,
    pub activation_branch: String,
    #[serde(default)]
    pub allow_emergency_override: bool,
    #[serde(default)]
    pub require_policy_pr_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyAdaptersConfig {
    #[serde(default)]
    pub opa: OpaAdapterConfig,
}

impl PolicyAdaptersConfig {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpaAdapterConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default = "default_opa_decision_path")]
    pub decision_path: String,
    #[serde(default)]
    pub effect: ExternalPolicyEffect,
    #[serde(default)]
    pub failure_mode: ExternalPolicyFailureMode,
    #[serde(default = "default_opa_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_opa_input_profile")]
    pub input_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env_var: Option<String>,
    #[serde(default)]
    pub result_mapping: OpaResultMapping,
}

impl Default for OpaAdapterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            connection: None,
            base_url: None,
            decision_path: default_opa_decision_path(),
            effect: ExternalPolicyEffect::Advisory,
            failure_mode: ExternalPolicyFailureMode::FailOpen,
            timeout_ms: default_opa_timeout_ms(),
            input_profile: default_opa_input_profile(),
            token_env_var: None,
            result_mapping: OpaResultMapping::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalPolicyEffect {
    #[default]
    Advisory,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalPolicyFailureMode {
    #[default]
    FailOpen,
    FailClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpaResultMapping {
    #[serde(default = "default_opa_allowed_key")]
    pub allowed_key: String,
    #[serde(default = "default_opa_reasons_key")]
    pub reasons_key: String,
    #[serde(default = "default_opa_warnings_key")]
    pub warnings_key: String,
    #[serde(default = "default_opa_message_key")]
    pub message_key: String,
    #[serde(default = "default_opa_decision_id_key")]
    pub decision_id_key: String,
}

impl Default for OpaResultMapping {
    fn default() -> Self {
        Self {
            allowed_key: default_opa_allowed_key(),
            reasons_key: default_opa_reasons_key(),
            warnings_key: default_opa_warnings_key(),
            message_key: default_opa_message_key(),
            decision_id_key: default_opa_decision_id_key(),
        }
    }
}

fn default_opa_decision_path() -> String {
    "/v1/data/gitgov/allow".to_string()
}

fn default_opa_timeout_ms() -> u64 {
    1500
}

fn default_opa_input_profile() -> String {
    "policy-check-v1".to_string()
}

fn default_opa_allowed_key() -> String {
    "allow".to_string()
}

fn default_opa_reasons_key() -> String {
    "reasons".to_string()
}

fn default_opa_warnings_key() -> String {
    "warnings".to_string()
}

fn default_opa_message_key() -> String {
    "message".to_string()
}

fn default_opa_decision_id_key() -> String {
    "decision_id".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredPolicyFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub format: PolicyFormat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedPolicyFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub format: PolicyFormat,
    pub config: GitGovConfig,
    pub canonical_json: String,
    pub checksum: String,
}

#[derive(Debug, Error)]
pub enum PolicyFileError {
    #[error("policy file not found")]
    NotFound,
    #[error("multiple policy files found: {0}")]
    Ambiguous(String),
    #[error("unsupported policy file format for {0}")]
    UnsupportedFormat(String),
    #[error("failed to read policy file {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse {format} policy {path}: {message}")]
    Parse {
        format: &'static str,
        path: String,
        message: String,
    },
    #[error("failed to canonicalize policy: {0}")]
    Canonicalize(String),
    #[error("invalid policy config: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
    #[serde(default, skip_serializing_if = "PolicyAdaptersConfig::is_default")]
    pub adapters: PolicyAdaptersConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BranchConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub protected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GroupConfig {
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub allowed_branches: Vec<String>,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ChecklistConfig {
    #[serde(default)]
    pub confirm: Vec<String>,
    #[serde(default)]
    pub auto_check: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
    #[serde(default, skip_serializing_if = "EnforcementLevel::is_off")]
    pub external_policy: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementLevel {
    #[default]
    Off,
    Warn,
    Block,
}

impl EnforcementLevel {
    pub fn is_off(&self) -> bool {
        matches!(self, Self::Off)
    }
}

pub fn discover_policy_file(
    repo_path: impl AsRef<Path>,
) -> Result<DiscoveredPolicyFile, PolicyFileError> {
    let repo_path = repo_path.as_ref();
    let mut matches = Vec::new();

    for (relative_path, format) in DEFAULT_POLICY_PATHS {
        let absolute_path = repo_path.join(relative_path);
        if absolute_path.exists() {
            matches.push(DiscoveredPolicyFile {
                relative_path: (*relative_path).to_string(),
                absolute_path,
                format: *format,
            });
        }
    }

    match matches.len() {
        0 => Err(PolicyFileError::NotFound),
        1 => Ok(matches.remove(0)),
        _ => Err(PolicyFileError::Ambiguous(
            matches
                .iter()
                .map(|item| item.relative_path.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )),
    }
}

pub fn load_policy_from_repo(
    repo_path: impl AsRef<Path>,
) -> Result<ParsedPolicyFile, PolicyFileError> {
    let discovered = discover_policy_file(repo_path)?;
    parse_policy_file(
        &discovered.absolute_path,
        &discovered.relative_path,
        discovered.format,
    )
}

pub fn parse_policy_file(
    absolute_path: impl AsRef<Path>,
    relative_path: &str,
    format: PolicyFormat,
) -> Result<ParsedPolicyFile, PolicyFileError> {
    let absolute_path = absolute_path.as_ref().to_path_buf();
    let content =
        std::fs::read_to_string(&absolute_path).map_err(|source| PolicyFileError::Read {
            path: absolute_path.to_string_lossy().to_string(),
            source,
        })?;
    let config = parse_policy_str(&content, format, relative_path)?;
    let canonical_json = canonical_policy_json(&config)?;
    let checksum = sha256_hex(canonical_json.as_bytes());

    Ok(ParsedPolicyFile {
        relative_path: relative_path.to_string(),
        absolute_path,
        format,
        config,
        canonical_json,
        checksum,
    })
}

pub fn parse_policy_path(path: impl AsRef<Path>) -> Result<ParsedPolicyFile, PolicyFileError> {
    let path = path.as_ref();
    let format = PolicyFormat::from_path(path)
        .ok_or_else(|| PolicyFileError::UnsupportedFormat(path.to_string_lossy().to_string()))?;
    let relative_path = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("policy")
        .to_string();
    parse_policy_file(path, &relative_path, format)
}

pub fn parse_policy_str(
    content: &str,
    format: PolicyFormat,
    path_for_errors: &str,
) -> Result<GitGovConfig, PolicyFileError> {
    let config = match format {
        PolicyFormat::Toml => toml::from_str(content).map_err(|err| PolicyFileError::Parse {
            format: format.as_str(),
            path: path_for_errors.to_string(),
            message: err.to_string(),
        }),
        PolicyFormat::Yaml => serde_yaml::from_str(content).map_err(|err| PolicyFileError::Parse {
            format: format.as_str(),
            path: path_for_errors.to_string(),
            message: err.to_string(),
        }),
        PolicyFormat::Json => serde_json::from_str(content).map_err(|err| PolicyFileError::Parse {
            format: format.as_str(),
            path: path_for_errors.to_string(),
            message: err.to_string(),
        }),
    }?;
    validate_policy_config(&config)?;
    Ok(config)
}

pub fn validate_policy_config(config: &GitGovConfig) -> Result<(), PolicyFileError> {
    validate_opa_adapter_config(&config.adapters.opa)
}

fn validate_opa_adapter_config(config: &OpaAdapterConfig) -> Result<(), PolicyFileError> {
    if !config.enabled {
        return Ok(());
    }

    let decision_path = config.decision_path.trim();
    if decision_path.is_empty() {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.decision_path is required when OPA is enabled".to_string(),
        ));
    }
    if !decision_path.starts_with("/v1/data/") || decision_path.contains("://") {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.decision_path must be an OPA Data API path such as /v1/data/gitgov/allow"
                .to_string(),
        ));
    }

    if config.timeout_ms == 0 || config.timeout_ms > 30_000 {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.timeout_ms must be between 1 and 30000".to_string(),
        ));
    }

    if config.input_profile.trim().is_empty() {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.input_profile cannot be empty".to_string(),
        ));
    }

    validate_opa_result_mapping(&config.result_mapping)?;

    if let Some(base_url) = config.base_url.as_deref() {
        validate_opa_base_url(base_url)?;
    }

    if let Some(token_env_var) = config.token_env_var.as_deref() {
        validate_secret_env_var_name(token_env_var)?;
    }

    Ok(())
}

fn validate_opa_result_mapping(mapping: &OpaResultMapping) -> Result<(), PolicyFileError> {
    for (field, value) in [
        ("allowed_key", &mapping.allowed_key),
        ("reasons_key", &mapping.reasons_key),
        ("warnings_key", &mapping.warnings_key),
        ("message_key", &mapping.message_key),
        ("decision_id_key", &mapping.decision_id_key),
    ] {
        if value.trim().is_empty() {
            return Err(PolicyFileError::InvalidConfig(format!(
                "adapters.opa.result_mapping.{} cannot be empty",
                field
            )));
        }
    }
    Ok(())
}

pub fn validate_opa_base_url(base_url: &str) -> Result<(), PolicyFileError> {
    let trimmed = base_url.trim();
    let lowered = trimmed.to_ascii_lowercase();
    if trimmed.is_empty() {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.base_url cannot be empty".to_string(),
        ));
    }
    if !(lowered.starts_with("https://") || lowered.starts_with("http://")) {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.base_url must use https://, or http:// only for loopback OPA sidecars"
                .to_string(),
        ));
    }
    if contains_url_credentials(trimmed) || contains_secret_query(trimmed) {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.base_url must not contain inline credentials, tokens, or secret query parameters"
                .to_string(),
        ));
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.base_url must not include query strings or fragments".to_string(),
        ));
    }
    let host = extract_url_host(trimmed).ok_or_else(|| {
        PolicyFileError::InvalidConfig(
            "adapters.opa.base_url must include a valid host".to_string(),
        )
    })?;
    validate_url_port(trimmed)?;
    if lowered.starts_with("http://") && !is_loopback_host(host) {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.base_url may use http:// only for localhost/loopback OPA sidecars"
                .to_string(),
        ));
    }
    Ok(())
}

pub fn validate_opa_token_env_var_name(name: &str) -> Result<(), PolicyFileError> {
    validate_secret_env_var_name(name)
}

fn extract_url_host(value: &str) -> Option<&str> {
    let (_, remainder) = value.split_once("://")?;
    let authority = remainder
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|item| !item.is_empty())?;
    if authority.contains('@') {
        return None;
    }
    if let Some(without_bracket) = authority.strip_prefix('[') {
        let end = without_bracket.find(']')?;
        let rest = &without_bracket[end + 1..];
        if !rest.is_empty() && !rest.starts_with(':') {
            return None;
        }
        return Some(&without_bracket[..end]);
    }
    authority
        .split(':')
        .next()
        .map(str::trim)
        .filter(|item| !item.is_empty())
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .map(|addr| addr.is_loopback())
            .unwrap_or(false)
}

fn validate_url_port(value: &str) -> Result<(), PolicyFileError> {
    let (_, remainder) = value.split_once("://").ok_or_else(|| {
        PolicyFileError::InvalidConfig("adapters.opa.base_url must include a scheme".to_string())
    })?;
    let authority = remainder
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| {
            PolicyFileError::InvalidConfig(
                "adapters.opa.base_url must include a valid host".to_string(),
            )
        })?;

    let port = if let Some(without_bracket) = authority.strip_prefix('[') {
        let end = without_bracket.find(']').ok_or_else(|| {
            PolicyFileError::InvalidConfig(
                "adapters.opa.base_url must include a valid IPv6 host".to_string(),
            )
        })?;
        let rest = &without_bracket[end + 1..];
        rest.strip_prefix(':')
    } else {
        let mut parts = authority.split(':');
        let _host = parts.next();
        let port = parts.next();
        if parts.next().is_some() {
            return Err(PolicyFileError::InvalidConfig(
                "adapters.opa.base_url must bracket IPv6 hosts".to_string(),
            ));
        }
        port
    };

    if let Some(port) = port {
        if port.is_empty() || !port.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(PolicyFileError::InvalidConfig(
                "adapters.opa.base_url port must be numeric".to_string(),
            ));
        }
        let parsed = port.parse::<u16>().map_err(|_| {
            PolicyFileError::InvalidConfig(
                "adapters.opa.base_url port must be between 1 and 65535".to_string(),
            )
        })?;
        if parsed == 0 {
            return Err(PolicyFileError::InvalidConfig(
                "adapters.opa.base_url port must be between 1 and 65535".to_string(),
            ));
        }
    }

    Ok(())
}

fn contains_url_credentials(value: &str) -> bool {
    value
        .split_once("://")
        .and_then(|(_, remainder)| remainder.split(['/', '?', '#']).next())
        .map(|authority| authority.contains('@'))
        .unwrap_or(false)
}

fn contains_secret_query(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    [
        "token=",
        "password=",
        "secret=",
        "api_key=",
        "apikey=",
        "access_token=",
        "bearer=",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn validate_secret_env_var_name(name: &str) -> Result<(), PolicyFileError> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        || trimmed.chars().next().is_some_and(|ch| ch.is_ascii_digit())
    {
        return Err(PolicyFileError::InvalidConfig(
            "adapters.opa.token_env_var must be an uppercase environment variable name, not a secret value"
                .to_string(),
        ));
    }
    Ok(())
}

pub fn canonical_policy_json(config: &GitGovConfig) -> Result<String, PolicyFileError> {
    let mut value = serde_json::to_value(config)
        .map_err(|err| PolicyFileError::Canonicalize(err.to_string()))?;
    sort_json_value(&mut value);
    serde_json::to_string(&value).map_err(|err| PolicyFileError::Canonicalize(err.to_string()))
}

pub fn policy_checksum(config: &GitGovConfig) -> Result<String, PolicyFileError> {
    let canonical_json = canonical_policy_json(config)?;
    Ok(sha256_hex(canonical_json.as_bytes()))
}

fn sort_json_value(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                sort_json_value(item);
            }
        }
        Value::Object(object) => {
            let mut sorted = BTreeMap::new();
            for (key, mut child) in std::mem::take(object) {
                sort_json_value(&mut child);
                sorted.insert(key, child);
            }

            let mut replacement = Map::new();
            for (key, child) in sorted {
                replacement.insert(key, child);
            }
            *object = replacement;
        }
        _ => {}
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const TOML_POLICY: &str = r#"
admins = ["alice"]

[branches]
patterns = ["feature/*", "KAN-*"]
protected = ["main", "release"]

[rules]
require_pull_request = true
min_approvals = 2
require_conventional_commits = true
require_signed_commits = false
max_files_per_commit = 25
require_linked_ticket = true
block_force_push = true
forbidden_patterns = ["*.pem", "secrets/**"]

[checklist]
confirm = ["tests passed", "security reviewed"]
auto_check = ["lint", "typecheck"]

[enforcement]
pull_requests = "block"
commits = "warn"
branches = "block"
traceability = "block"
quality_gates = "warn"

[groups.backend]
members = ["alice", "bob"]
allowed_branches = ["feature/*", "KAN-*"]
allowed_paths = ["gitgov/gitgov-server/**", "docs/**"]
"#;

    const YAML_POLICY: &str = r#"
groups:
  backend:
    allowed_paths:
      - gitgov/gitgov-server/**
      - docs/**
    allowed_branches:
      - feature/*
      - KAN-*
    members:
      - alice
      - bob
enforcement:
  quality_gates: warn
  traceability: block
  branches: block
  commits: warn
  pull_requests: block
checklist:
  auto_check:
    - lint
    - typecheck
  confirm:
    - tests passed
    - security reviewed
rules:
  forbidden_patterns:
    - "*.pem"
    - secrets/**
  block_force_push: true
  require_linked_ticket: true
  max_files_per_commit: 25
  require_signed_commits: false
  require_conventional_commits: true
  min_approvals: 2
  require_pull_request: true
branches:
  protected:
    - main
    - release
  patterns:
    - feature/*
    - KAN-*
admins:
  - alice
"#;

    const JSON_POLICY: &str = r#"
{
  "admins": ["alice"],
  "branches": {
    "protected": ["main", "release"],
    "patterns": ["feature/*", "KAN-*"]
  },
  "rules": {
    "require_pull_request": true,
    "min_approvals": 2,
    "require_conventional_commits": true,
    "require_signed_commits": false,
    "max_files_per_commit": 25,
    "require_linked_ticket": true,
    "block_force_push": true,
    "forbidden_patterns": ["*.pem", "secrets/**"]
  },
  "checklist": {
    "confirm": ["tests passed", "security reviewed"],
    "auto_check": ["lint", "typecheck"]
  },
  "enforcement": {
    "pull_requests": "block",
    "commits": "warn",
    "branches": "block",
    "traceability": "block",
    "quality_gates": "warn"
  },
  "groups": {
    "backend": {
      "members": ["alice", "bob"],
      "allowed_branches": ["feature/*", "KAN-*"],
      "allowed_paths": ["gitgov/gitgov-server/**", "docs/**"]
    }
  }
}
"#;

    #[test]
    fn equivalent_toml_yaml_and_json_have_same_config_and_checksum() {
        let toml_config = parse_policy_str(TOML_POLICY, PolicyFormat::Toml, "gitgov.toml").unwrap();
        let yaml_config =
            parse_policy_str(YAML_POLICY, PolicyFormat::Yaml, ".gitgov/policy.yml").unwrap();
        let json_config =
            parse_policy_str(JSON_POLICY, PolicyFormat::Json, ".gitgov/policy.json").unwrap();

        assert_eq!(toml_config, yaml_config);
        assert_eq!(toml_config, json_config);

        let toml_checksum = policy_checksum(&toml_config).unwrap();
        assert_eq!(toml_checksum, policy_checksum(&yaml_config).unwrap());
        assert_eq!(toml_checksum, policy_checksum(&json_config).unwrap());
    }

    #[test]
    fn canonical_checksum_ignores_json_key_order_and_whitespace() {
        let first = parse_policy_str(
            r#"{"admins":["alice"],"branches":{"patterns":["feature/*"],"protected":["main"]}}"#,
            PolicyFormat::Json,
            ".gitgov/policy.json",
        )
        .unwrap();
        let second = parse_policy_str(
            r#"
            {
              "branches": {
                "protected": ["main"],
                "patterns": ["feature/*"]
              },
              "admins": ["alice"]
            }
            "#,
            PolicyFormat::Json,
            ".gitgov/policy.json",
        )
        .unwrap();

        assert_eq!(
            policy_checksum(&first).unwrap(),
            policy_checksum(&second).unwrap()
        );
    }

    #[test]
    fn legacy_policy_defaults_opa_adapter_off_without_checksum_churn() {
        let config = parse_policy_str(
            r#"{"admins":["alice"],"enforcement":{"branches":"warn"}}"#,
            PolicyFormat::Json,
            ".gitgov/policy.json",
        )
        .unwrap();

        assert!(!config.adapters.opa.enabled);
        assert_eq!(config.enforcement.external_policy, EnforcementLevel::Off);
        let canonical = canonical_policy_json(&config).unwrap();
        assert!(!canonical.contains("adapters"));
        assert!(!canonical.contains("external_policy"));
    }

    #[test]
    fn opa_required_adapter_parses_from_toml() {
        let config = parse_policy_str(
            r#"
            [enforcement]
            external_policy = "block"

            [adapters.opa]
            enabled = true
            connection = "default"
            base_url = "http://127.0.0.1:8181"
            decision_path = "/v1/data/gitgov/allow"
            effect = "required"
            failure_mode = "fail-closed"
            timeout_ms = 1000
            input_profile = "policy-check-v1"
            token_env_var = "OPA_AUTH_TOKEN"
            "#,
            PolicyFormat::Toml,
            "gitgov.toml",
        )
        .unwrap();

        assert!(config.adapters.opa.enabled);
        assert_eq!(config.adapters.opa.effect, ExternalPolicyEffect::Required);
        assert_eq!(
            config.adapters.opa.failure_mode,
            ExternalPolicyFailureMode::FailClosed
        );
        assert_eq!(config.enforcement.external_policy, EnforcementLevel::Block);
        assert_eq!(config.adapters.opa.result_mapping.allowed_key, "allow");
    }

    #[test]
    fn opa_adapter_rejects_inline_secret_material() {
        let error = parse_policy_str(
            r#"
            [adapters.opa]
            enabled = true
            base_url = "https://opa.example.com/v1?token=do-not-commit"
            "#,
            PolicyFormat::Toml,
            "gitgov.toml",
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("must not contain inline credentials"));

        let error = parse_policy_str(
            r#"
            [adapters.opa]
            enabled = true
            base_url = "https://opa.example.com"
            token_env_var = "ghp_raw_secret_value"
            "#,
            PolicyFormat::Toml,
            "gitgov.toml",
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("uppercase environment variable name"));
    }

    #[test]
    fn opa_adapter_rejects_empty_profile_or_mapping_keys() {
        let error = parse_policy_str(
            r#"
            [adapters.opa]
            enabled = true
            input_profile = ""
            "#,
            PolicyFormat::Toml,
            "gitgov.toml",
        )
        .unwrap_err();

        assert!(error.to_string().contains("input_profile cannot be empty"));

        let error = parse_policy_str(
            r#"
            [adapters.opa]
            enabled = true

            [adapters.opa.result_mapping]
            allowed_key = ""
            "#,
            PolicyFormat::Toml,
            "gitgov.toml",
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("result_mapping.allowed_key cannot be empty"));
    }

    #[test]
    fn opa_adapter_rejects_spoofed_loopback_http_hosts() {
        for base_url in [
            "http://localhost.example.com:8181",
            "http://127.0.0.1.example.com:8181",
            "http://[::1].example.com:8181",
            "http://opa.example.com:8181",
            "http://127.0.0.1:abc",
            "http://127.0.0.1:0",
        ] {
            let error = parse_policy_str(
                &format!(
                    r#"
                    [adapters.opa]
                    enabled = true
                    base_url = "{}"
                    "#,
                    base_url
                ),
                PolicyFormat::Toml,
                "gitgov.toml",
            )
            .unwrap_err();

            let message = error.to_string();
            assert!(
                message.contains("localhost/loopback")
                    || message.contains("valid host")
                    || message.contains("port must"),
                "unexpected error for {}: {}",
                base_url,
                message
            );
        }

        for base_url in [
            "https://opa.example.com?tenant=acme",
            "https://opa.example.com#fragment",
        ] {
            let error = parse_policy_str(
                &format!(
                    r#"
                    [adapters.opa]
                    enabled = true
                    base_url = "{}"
                    "#,
                    base_url
                ),
                PolicyFormat::Toml,
                "gitgov.toml",
            )
            .unwrap_err();

            assert!(
                error.to_string().contains("query strings or fragments"),
                "unexpected error for {}: {}",
                base_url,
                error
            );
        }

        for base_url in [
            "http://localhost:8181",
            "http://127.0.0.1:8181",
            "http://127.0.0.2:8181",
            "http://[::1]:8181",
        ] {
            validate_opa_base_url(base_url).expect("loopback OPA URL should be allowed");
        }
    }

    #[test]
    fn discovery_loads_real_policy_file_from_temp_repo() {
        let repo = tempfile::tempdir().unwrap();
        let policy_dir = repo.path().join(".gitgov");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("policy.yml"), YAML_POLICY).unwrap();

        let parsed = load_policy_from_repo(repo.path()).unwrap();

        assert_eq!(parsed.relative_path, ".gitgov/policy.yml");
        assert_eq!(parsed.format, PolicyFormat::Yaml);
        assert_eq!(parsed.config.rules.min_approvals, 2);
        assert_eq!(parsed.checksum, policy_checksum(&parsed.config).unwrap());
    }

    #[test]
    fn discovery_rejects_ambiguous_policy_files() {
        let repo = tempfile::tempdir().unwrap();
        let policy_dir = repo.path().join(".gitgov");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("policy.yml"), YAML_POLICY).unwrap();
        fs::write(repo.path().join("gitgov.toml"), TOML_POLICY).unwrap();

        let err = discover_policy_file(repo.path()).unwrap_err();

        assert!(matches!(err, PolicyFileError::Ambiguous(_)));
        assert!(err.to_string().contains(".gitgov/policy.yml"));
        assert!(err.to_string().contains("gitgov.toml"));
    }

    #[test]
    fn invalid_policy_reports_format_and_path() {
        let err =
            parse_policy_str("not: [valid", PolicyFormat::Yaml, ".gitgov/policy.yml").unwrap_err();

        let message = err.to_string();
        assert!(message.contains("yaml"));
        assert!(message.contains(".gitgov/policy.yml"));
    }
}
