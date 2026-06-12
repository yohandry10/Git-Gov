use crate::{
    diff_policy_configs, parse_policy_str, policy_checksum, PolicyChangeSeverity, PolicyFormat,
    PolicySemanticChange, DEFAULT_POLICY_PATHS,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyGitValidationStatus {
    NotApplicable,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGitValidationResult {
    pub status: PolicyGitValidationStatus,
    pub allowed: bool,
    pub blocking: bool,
    pub changed_policy_paths: Vec<String>,
    pub base_checksum: Option<String>,
    pub head_checksum: Option<String>,
    pub changes: Vec<PolicySemanticChange>,
    pub errors: Vec<String>,
}

pub fn validate_git_policy_change(
    repo_path: impl AsRef<Path>,
    base_ref: &str,
    head_ref: &str,
    blocking: bool,
) -> PolicyGitValidationResult {
    let repo_path = repo_path.as_ref();
    let changed_files = match git_lines(
        repo_path,
        &["diff", "--name-only", &format!("{base_ref}..{head_ref}")],
    ) {
        Ok(files) => files,
        Err(error) => {
            return invalid(blocking, vec![error]);
        }
    };

    let changed_policy_paths = changed_files
        .into_iter()
        .filter(|path| {
            DEFAULT_POLICY_PATHS
                .iter()
                .any(|(candidate, _)| candidate == path)
        })
        .collect::<Vec<_>>();

    if changed_policy_paths.is_empty() {
        return PolicyGitValidationResult {
            status: PolicyGitValidationStatus::NotApplicable,
            allowed: true,
            blocking,
            changed_policy_paths,
            base_checksum: None,
            head_checksum: None,
            changes: vec![],
            errors: vec![],
        };
    }

    if changed_policy_paths.len() > 1 {
        return invalid(
            blocking,
            vec![format!(
                "Multiple policy files changed in one PR: {}",
                changed_policy_paths.join(", ")
            )],
        );
    }

    let policy_path = &changed_policy_paths[0];
    let Some(format) = PolicyFormat::from_path(policy_path) else {
        return invalid(
            blocking,
            vec![format!("Unsupported policy format for {policy_path}")],
        );
    };

    let Some(head_content) = git_show(repo_path, head_ref, policy_path) else {
        return invalid(
            blocking,
            vec![format!(
                "Policy file {policy_path} is missing at {head_ref}"
            )],
        );
    };

    let head_config = match parse_policy_str(&head_content, format, policy_path) {
        Ok(config) => config,
        Err(error) => return invalid(blocking, vec![error.to_string()]),
    };
    let head_checksum = match policy_checksum(&head_config) {
        Ok(checksum) => checksum,
        Err(error) => return invalid(blocking, vec![error.to_string()]),
    };

    let (base_checksum, changes) = match git_show(repo_path, base_ref, policy_path) {
        Some(base_content) => {
            let base_config = match parse_policy_str(&base_content, format, policy_path) {
                Ok(config) => config,
                Err(error) => return invalid(blocking, vec![error.to_string()]),
            };
            let checksum = match policy_checksum(&base_config) {
                Ok(checksum) => checksum,
                Err(error) => return invalid(blocking, vec![error.to_string()]),
            };
            (
                Some(checksum),
                diff_policy_configs(&base_config, &head_config),
            )
        }
        None => (
            None,
            vec![PolicySemanticChange {
                path: policy_path.clone(),
                before: None,
                after: Some("created".to_string()),
                severity: PolicyChangeSeverity::Info,
                message: "Policy file created".to_string(),
            }],
        ),
    };

    let has_risky_changes = changes
        .iter()
        .any(|change| change.severity == PolicyChangeSeverity::Risky);

    PolicyGitValidationResult {
        status: PolicyGitValidationStatus::Valid,
        allowed: !blocking || !has_risky_changes,
        blocking,
        changed_policy_paths,
        base_checksum,
        head_checksum: Some(head_checksum),
        changes,
        errors: vec![],
    }
}

fn invalid(blocking: bool, errors: Vec<String>) -> PolicyGitValidationResult {
    PolicyGitValidationResult {
        status: PolicyGitValidationStatus::Invalid,
        allowed: false,
        blocking,
        changed_policy_paths: vec![],
        base_checksum: None,
        head_checksum: None,
        changes: vec![],
        errors,
    }
}

fn git_lines(repo_path: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .map_err(|error| format!("failed to execute git: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn git_show(repo_path: &Path, git_ref: &str, path: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["show", &format!("{git_ref}:{path}")])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const STRICT_POLICY: &str = r#"
branches:
  protected: [main, release]
rules:
  require_pull_request: true
  require_linked_ticket: true
  min_approvals: 2
  block_force_push: true
enforcement:
  pull_requests: block
  traceability: block
  quality_gates: warn
"#;

    const WEAKER_POLICY: &str = r#"
branches:
  protected: [main]
rules:
  require_pull_request: false
  require_linked_ticket: false
  min_approvals: 1
  block_force_push: false
enforcement:
  pull_requests: warn
  traceability: off
  quality_gates: off
"#;

    #[test]
    fn validates_real_git_policy_change_and_blocks_risky_downgrade_when_enforced() {
        let repo = tempfile::tempdir().unwrap();
        run_git(repo.path(), &["init"]);
        run_git(repo.path(), &["config", "user.email", "dev@example.com"]);
        run_git(repo.path(), &["config", "user.name", "Dev"]);

        let policy_dir = repo.path().join(".gitgov");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("policy.yml"), STRICT_POLICY).unwrap();
        run_git(repo.path(), &["add", ".gitgov/policy.yml"]);
        run_git(repo.path(), &["commit", "-m", "KAN-1 strict policy"]);

        fs::write(policy_dir.join("policy.yml"), WEAKER_POLICY).unwrap();
        run_git(repo.path(), &["add", ".gitgov/policy.yml"]);
        run_git(repo.path(), &["commit", "-m", "KAN-1 weaken policy"]);

        let advisory = validate_git_policy_change(repo.path(), "HEAD~1", "HEAD", false);
        assert_eq!(advisory.status, PolicyGitValidationStatus::Valid);
        assert!(advisory.allowed);
        assert!(advisory
            .changes
            .iter()
            .any(|change| change.severity == PolicyChangeSeverity::Risky));

        let blocking = validate_git_policy_change(repo.path(), "HEAD~1", "HEAD", true);
        assert_eq!(blocking.status, PolicyGitValidationStatus::Valid);
        assert!(!blocking.allowed);
    }

    fn run_git(repo_path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(args)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
