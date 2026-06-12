pub mod validator;

pub use validator::*;

use crate::models::GitGovConfig;
use gitgov_policy_core::{load_policy_from_repo, PolicyFileError};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(String),
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("Policy file error: {0}")]
    PolicyFile(String),
}

pub fn load_config(repo_path: &str) -> Result<GitGovConfig, ConfigError> {
    load_policy_from_repo(repo_path)
        .map(|parsed| parsed.config)
        .map_err(map_policy_file_error)
}

fn map_policy_file_error(error: PolicyFileError) -> ConfigError {
    match error {
        PolicyFileError::NotFound => {
            ConfigError::FileNotFound(Path::new("gitgov.toml").to_string_lossy().to_string())
        }
        PolicyFileError::Parse { message, .. } => ConfigError::ParseError(message),
        other => ConfigError::PolicyFile(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const YAML_POLICY: &str = r#"
branches:
  patterns:
    - feature/*
  protected:
    - main
groups:
  backend:
    members:
      - alice
    allowed_branches:
      - feature/*
    allowed_paths:
      - gitgov/**
admins:
  - alice
rules:
  require_pull_request: true
  min_approvals: 1
  require_conventional_commits: true
  require_linked_ticket: true
enforcement:
  pull_requests: block
  branches: block
  commits: warn
  traceability: block
  quality_gates: warn
"#;

    const TOML_POLICY: &str = r#"
admins = ["alice"]

[branches]
patterns = ["feature/*"]
protected = ["main"]
"#;

    #[test]
    fn load_config_reads_yaml_policy_file_from_real_repo_path() {
        let repo = tempfile::tempdir().unwrap();
        let policy_dir = repo.path().join(".gitgov");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("policy.yml"), YAML_POLICY).unwrap();

        let config = load_config(repo.path().to_string_lossy().as_ref()).unwrap();

        assert_eq!(config.admins, vec!["alice"]);
        assert_eq!(config.rules.min_approvals, 1);
        assert_eq!(
            config.enforcement.pull_requests,
            crate::models::EnforcementLevel::Block
        );
    }

    #[test]
    fn load_config_rejects_ambiguous_policy_files() {
        let repo = tempfile::tempdir().unwrap();
        let policy_dir = repo.path().join(".gitgov");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("policy.yml"), YAML_POLICY).unwrap();
        fs::write(repo.path().join("gitgov.toml"), TOML_POLICY).unwrap();

        let error = load_config(repo.path().to_string_lossy().as_ref()).unwrap_err();

        assert!(error.to_string().contains("multiple policy files found"));
    }
}
