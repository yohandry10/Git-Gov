use crate::{EnforcementLevel, GitGovConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyChangeSeverity {
    Info,
    Risky,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySemanticChange {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub severity: PolicyChangeSeverity,
    pub message: String,
}

pub fn diff_policy_configs(base: &GitGovConfig, head: &GitGovConfig) -> Vec<PolicySemanticChange> {
    let mut changes = Vec::new();

    compare_enforcement(
        &mut changes,
        "enforcement.pull_requests",
        &base.enforcement.pull_requests,
        &head.enforcement.pull_requests,
    );
    compare_enforcement(
        &mut changes,
        "enforcement.commits",
        &base.enforcement.commits,
        &head.enforcement.commits,
    );
    compare_enforcement(
        &mut changes,
        "enforcement.branches",
        &base.enforcement.branches,
        &head.enforcement.branches,
    );
    compare_enforcement(
        &mut changes,
        "enforcement.traceability",
        &base.enforcement.traceability,
        &head.enforcement.traceability,
    );
    compare_enforcement(
        &mut changes,
        "enforcement.quality_gates",
        &base.enforcement.quality_gates,
        &head.enforcement.quality_gates,
    );

    if head.rules.min_approvals < base.rules.min_approvals {
        changes.push(PolicySemanticChange {
            path: "rules.min_approvals".to_string(),
            before: Some(base.rules.min_approvals.to_string()),
            after: Some(head.rules.min_approvals.to_string()),
            severity: PolicyChangeSeverity::Risky,
            message: "Minimum PR approvals decreased".to_string(),
        });
    } else if head.rules.min_approvals > base.rules.min_approvals {
        changes.push(PolicySemanticChange {
            path: "rules.min_approvals".to_string(),
            before: Some(base.rules.min_approvals.to_string()),
            after: Some(head.rules.min_approvals.to_string()),
            severity: PolicyChangeSeverity::Info,
            message: "Minimum PR approvals increased".to_string(),
        });
    }

    compare_required_bool(
        &mut changes,
        "rules.require_pull_request",
        base.rules.require_pull_request,
        head.rules.require_pull_request,
        "Pull request requirement disabled",
        "Pull request requirement enabled",
    );
    compare_required_bool(
        &mut changes,
        "rules.require_linked_ticket",
        base.rules.require_linked_ticket,
        head.rules.require_linked_ticket,
        "Linked-ticket requirement disabled",
        "Linked-ticket requirement enabled",
    );
    compare_required_bool(
        &mut changes,
        "rules.require_signed_commits",
        base.rules.require_signed_commits,
        head.rules.require_signed_commits,
        "Signed-commit requirement disabled",
        "Signed-commit requirement enabled",
    );
    compare_required_bool(
        &mut changes,
        "rules.block_force_push",
        base.rules.block_force_push,
        head.rules.block_force_push,
        "Force-push block disabled",
        "Force-push block enabled",
    );

    let base_protected = base
        .branches
        .protected
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let head_protected = head
        .branches
        .protected
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    for branch in base_protected.difference(&head_protected) {
        changes.push(PolicySemanticChange {
            path: "branches.protected".to_string(),
            before: Some(branch.clone()),
            after: None,
            severity: PolicyChangeSeverity::Risky,
            message: format!("Protected branch removed: {}", branch),
        });
    }
    for branch in head_protected.difference(&base_protected) {
        changes.push(PolicySemanticChange {
            path: "branches.protected".to_string(),
            before: None,
            after: Some(branch.clone()),
            severity: PolicyChangeSeverity::Info,
            message: format!("Protected branch added: {}", branch),
        });
    }

    changes
}

fn compare_enforcement(
    changes: &mut Vec<PolicySemanticChange>,
    path: &str,
    before: &EnforcementLevel,
    after: &EnforcementLevel,
) {
    if before == after {
        return;
    }

    let downgraded = enforcement_rank(after) < enforcement_rank(before);
    changes.push(PolicySemanticChange {
        path: path.to_string(),
        before: Some(format_enforcement(before)),
        after: Some(format_enforcement(after)),
        severity: if downgraded {
            PolicyChangeSeverity::Risky
        } else {
            PolicyChangeSeverity::Info
        },
        message: if downgraded {
            format!("{} enforcement decreased", path)
        } else {
            format!("{} enforcement increased", path)
        },
    });
}

fn compare_required_bool(
    changes: &mut Vec<PolicySemanticChange>,
    path: &str,
    before: bool,
    after: bool,
    disabled_message: &str,
    enabled_message: &str,
) {
    if before == after {
        return;
    }

    changes.push(PolicySemanticChange {
        path: path.to_string(),
        before: Some(before.to_string()),
        after: Some(after.to_string()),
        severity: if before && !after {
            PolicyChangeSeverity::Risky
        } else {
            PolicyChangeSeverity::Info
        },
        message: if before && !after {
            disabled_message.to_string()
        } else {
            enabled_message.to_string()
        },
    });
}

fn enforcement_rank(level: &EnforcementLevel) -> u8 {
    match level {
        EnforcementLevel::Off => 0,
        EnforcementLevel::Warn => 1,
        EnforcementLevel::Block => 2,
    }
}

fn format_enforcement(level: &EnforcementLevel) -> String {
    match level {
        EnforcementLevel::Off => "off",
        EnforcementLevel::Warn => "warn",
        EnforcementLevel::Block => "block",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BranchConfig, EnforcementConfig, RulesConfig};

    #[test]
    fn diff_detects_risky_policy_downgrades() {
        let base = GitGovConfig {
            branches: BranchConfig {
                protected: vec!["main".to_string(), "release".to_string()],
                patterns: vec![],
            },
            rules: RulesConfig {
                require_pull_request: true,
                require_linked_ticket: true,
                min_approvals: 2,
                block_force_push: true,
                ..RulesConfig::default()
            },
            enforcement: EnforcementConfig {
                pull_requests: EnforcementLevel::Block,
                traceability: EnforcementLevel::Block,
                quality_gates: EnforcementLevel::Warn,
                ..EnforcementConfig::default()
            },
            ..GitGovConfig::default()
        };
        let head = GitGovConfig {
            branches: BranchConfig {
                protected: vec!["main".to_string()],
                patterns: vec![],
            },
            rules: RulesConfig {
                require_pull_request: false,
                require_linked_ticket: false,
                min_approvals: 1,
                block_force_push: false,
                ..RulesConfig::default()
            },
            enforcement: EnforcementConfig {
                pull_requests: EnforcementLevel::Warn,
                traceability: EnforcementLevel::Off,
                quality_gates: EnforcementLevel::Off,
                ..EnforcementConfig::default()
            },
            ..GitGovConfig::default()
        };

        let changes = diff_policy_configs(&base, &head);
        let risky = changes
            .iter()
            .filter(|change| change.severity == PolicyChangeSeverity::Risky)
            .count();

        assert!(risky >= 7, "expected risky downgrades, got {changes:#?}");
        assert!(changes
            .iter()
            .any(|change| change.message == "Protected branch removed: release"));
    }
}
