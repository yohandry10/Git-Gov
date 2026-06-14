// ============================================================================
// SHARED GOVERNANCE DECISION MODEL
// ============================================================================

const SHARED_GOVERNANCE_DECISION_CONTRACT: &str = "shared-governance-decision.v1";

fn governance_reason_code(reason: &str) -> String {
    let lowered = reason.to_ascii_lowercase();
    if lowered.contains("no valid release approval")
        || lowered.contains("release approval")
        || lowered.contains("accepted-risk")
    {
        "missing_release_approval".to_string()
    } else if lowered.contains("first governed repo") {
        "missing_first_governed_repo_setup".to_string()
    } else if lowered.contains("quorum") {
        "missing_quorum_approval".to_string()
    } else if lowered.contains("policy is configured") && lowered.contains("does not apply") {
        "policy_not_configured_for_environment".to_string()
    } else if lowered.contains("evidence packet") {
        "missing_or_mismatched_evidence_packet".to_string()
    } else if lowered.contains("ticket") {
        "missing_ticket_traceability".to_string()
    } else if lowered.contains("target sha") || lowered.contains("branch") {
        "missing_deployment_context".to_string()
    } else if lowered.contains("human approval") {
        "missing_human_approval".to_string()
    } else {
        "governance_warning".to_string()
    }
}

fn unique_sorted(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn release_governance_required_evidence(
    evaluation: &EnterpriseReleaseGovernanceEvaluationResponse,
) -> Vec<String> {
    let mut required = vec![
        "release_evidence_packet".to_string(),
        "deployment_context".to_string(),
    ];
    if evaluation.required_approval_count > 0 {
        required.push("release_approval".to_string());
        required.push("human_approval".to_string());
    }
    if evaluation.policy.quorum_enabled {
        required.push("quorum_approval".to_string());
    }
    unique_sorted(required)
}

fn release_governance_available_evidence(
    evaluation: &EnterpriseReleaseGovernanceEvaluationResponse,
    first_governed_repo_setup: Option<&FirstGovernedRepoSetupRecord>,
    break_glass_used: bool,
) -> Vec<String> {
    let mut available = vec![
        "release_evidence_packet".to_string(),
        "deployment_context".to_string(),
    ];
    if evaluation.valid_approval_count > 0 {
        available.push("release_approval".to_string());
        available.push("human_approval".to_string());
    }
    if first_governed_repo_setup.is_some() {
        available.push("first_governed_repo_setup".to_string());
    }
    if break_glass_used {
        available.push("break_glass_approval".to_string());
    }
    unique_sorted(available)
}

fn release_governance_missing_evidence(
    evaluation: &EnterpriseReleaseGovernanceEvaluationResponse,
    warnings: &[String],
) -> Vec<String> {
    let mut missing = Vec::new();
    if evaluation.required_approval_count > evaluation.valid_approval_count {
        missing.push("release_approval".to_string());
        missing.push("human_approval".to_string());
    }
    for issue in evaluation.issues.iter().chain(warnings.iter()) {
        match governance_reason_code(issue).as_str() {
            "missing_first_governed_repo_setup" => {
                missing.push("first_governed_repo_setup".to_string())
            }
            "missing_quorum_approval" => missing.push("quorum_approval".to_string()),
            "missing_or_mismatched_evidence_packet" => {
                missing.push("release_evidence_packet".to_string())
            }
            "policy_not_configured_for_environment" => {
                missing.push("environment_policy".to_string())
            }
            _ => {}
        }
    }
    unique_sorted(missing)
}

fn deployment_gate_shared_decision_state(
    legacy_decision: &str,
    evaluation: &EnterpriseReleaseGovernanceEvaluationResponse,
    warnings: &[String],
    break_glass_used: bool,
) -> String {
    if break_glass_used || legacy_decision == "approved" {
        return "allowed".to_string();
    }
    if evaluation.required_approval_count > evaluation.valid_approval_count
        && evaluation.required_approval_count > 0
    {
        return "requires_approval".to_string();
    }
    if legacy_decision == "blocked" {
        return "blocked".to_string();
    }
    if !warnings.is_empty() || !evaluation.policy.policy_applies {
        return "insufficient_evidence".to_string();
    }
    "allowed".to_string()
}

struct DeploymentGateGovernanceDecisionInput<'a> {
    payload: &'a DeploymentGateAuthorizationRequest,
    binding: &'a ReleaseEvidencePacketBinding,
    first_governed_repo_setup: Option<&'a FirstGovernedRepoSetupRecord>,
    evaluation: &'a EnterpriseReleaseGovernanceEvaluationResponse,
    legacy_decision: &'a str,
    policy_checksum: &'a str,
    warnings: &'a [String],
    blocked_by: &'a [String],
    break_glass_used: bool,
}

fn build_deployment_gate_governance_decision(
    input: DeploymentGateGovernanceDecisionInput<'_>,
) -> serde_json::Value {
    let decision = deployment_gate_shared_decision_state(
        input.legacy_decision,
        input.evaluation,
        input.warnings,
        input.break_glass_used,
    );
    let reasons = input
        .evaluation
        .issues
        .iter()
        .chain(input.warnings.iter())
        .cloned()
        .collect::<Vec<_>>();
    let reason_codes = unique_sorted(
        reasons
            .iter()
            .map(|reason| governance_reason_code(reason))
            .chain(input.blocked_by.iter().map(|reason| governance_reason_code(reason))),
    );
    let missing_evidence = release_governance_missing_evidence(input.evaluation, input.warnings);
    let action_center_items = missing_evidence
        .iter()
        .map(|item| {
            json!({
                "kind": "missing_evidence",
                "key": item,
                "status": "open"
            })
        })
        .collect::<Vec<_>>();

    json!({
        "contract_version": SHARED_GOVERNANCE_DECISION_CONTRACT,
        "consumer_type": "deployment_gate",
        "actor_type": "system",
        "action": "deploy",
        "decision": decision,
        "legacy_decision": input.legacy_decision,
        "approved": input.legacy_decision != "blocked",
        "manual_approval_required": input.evaluation.required_approval_count > input.evaluation.valid_approval_count,
        "agent_governance_used": false,
        "break_glass_used": input.break_glass_used,
        "policy": {
            "policy_id": "release-governance.v1",
            "policy_checksum": input.policy_checksum,
            "policy_source": "resolved",
            "mode": input.evaluation.policy.mode,
            "environment": input.evaluation.policy.environment,
            "enforcement": input.evaluation.policy.enforcement,
            "policy_applies": input.evaluation.policy.policy_applies
        },
        "operation": {
            "release_id": input.payload.release_id,
            "repository_full_name": input.payload.repository_full_name,
            "branch": input.payload.branch,
            "target_sha": input.payload.target_sha,
            "environment": input.payload.environment,
            "ticket_id": input.payload.ticket_id,
            "deployer": input.payload.deployer,
            "deployment_run_id": input.payload.deployment_run_id
        },
        "evidence": {
            "required_evidence": release_governance_required_evidence(input.evaluation),
            "available_evidence": release_governance_available_evidence(
                input.evaluation,
                input.first_governed_repo_setup,
                input.break_glass_used
            ),
            "missing_evidence": missing_evidence,
            "evidence_packet_hash": input.binding.evidence_packet_hash,
            "evidence_packet_uri": input.payload
                .evidence_packet_uri
                .as_deref()
                .unwrap_or(input.binding.evidence_packet_uri.as_str()),
            "valid_approval_count": input.evaluation.valid_approval_count,
            "required_approval_count": input.evaluation.required_approval_count
        },
        "reason_codes": reason_codes,
        "reasons": reasons,
        "action_center_items": action_center_items
    })
}

struct AgentGovernanceDecisionInput<'a> {
    payload: &'a AgentGovernanceEvaluationRequest,
    decision: &'a str,
    allowed: bool,
    requires_approval: bool,
    reasons: &'a [String],
    required_evidence: &'a [String],
    policy_checksum: &'a str,
    protected_branch: bool,
}

fn build_agent_governance_decision(input: AgentGovernanceDecisionInput<'_>) -> serde_json::Value {
    let reason_codes = unique_sorted(
        input
            .reasons
            .iter()
            .map(|reason| governance_reason_code(reason)),
    );
    let missing_evidence = input.required_evidence.to_vec();

    json!({
        "contract_version": SHARED_GOVERNANCE_DECISION_CONTRACT,
        "consumer_type": "agent_governance",
        "actor_type": "agent",
        "action": input.payload.action,
        "decision": input.decision,
        "legacy_decision": input.decision,
        "approved": input.allowed,
        "manual_approval_required": input.requires_approval,
        "agent_governance_used": true,
        "policy": {
            "policy_id": AGENT_GOVERNANCE_POLICY_ID,
            "policy_checksum": input.policy_checksum,
            "policy_source": "resolved",
            "mode": "opt_in_enabled",
            "environment": input.payload.environment,
            "enforcement": "advisory",
            "policy_applies": true
        },
        "operation": {
            "repository_full_name": input.payload.repository_full_name,
            "branch": input.payload.branch,
            "target_sha": input.payload.target_sha,
            "environment": input.payload.environment,
            "ticket_id": input.payload.ticket_id,
            "operation_id": input.payload.operation_id,
            "protected_branch": input.protected_branch
        },
        "evidence": {
            "required_evidence": input.required_evidence,
            "available_evidence": [],
            "missing_evidence": missing_evidence
        },
        "reason_codes": reason_codes,
        "reasons": input.reasons,
        "action_center_items": []
    })
}
