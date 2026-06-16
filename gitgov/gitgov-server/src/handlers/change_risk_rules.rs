fn change_risk_rule(
    rule_id: &str,
    title: &str,
    description: &str,
    severity: &str,
    evidence_inputs: &[&str],
    manual_action_hint: &str,
) -> ChangeRiskRuleDefinition {
    ChangeRiskRuleDefinition {
        rule_id: rule_id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        severity: severity.to_string(),
        evidence_inputs: evidence_inputs.iter().map(|value| value.to_string()).collect(),
        manual_action_hint: manual_action_hint.to_string(),
        enabled: true,
    }
}

fn change_risk_rule_catalog() -> Vec<ChangeRiskRuleDefinition> {
    vec![
        change_risk_rule(
            "missing_release_approval",
            "Missing release approval",
            "No human release approval or accepted-risk decision was available for the change.",
            "medium",
            &["deployment_gate", "release_approval", "governance_decision"],
            "Record a human release approval or accepted-risk decision before deployment.",
        ),
        change_risk_rule(
            "missing_ci_evidence",
            "Missing CI evidence",
            "The evaluation did not find CI, build, check-run, or pipeline evidence.",
            "medium",
            &["evidence_refs", "deployment_gate", "evidence_packet_hash"],
            "Attach CI or pipeline evidence to the change review.",
        ),
        change_risk_rule(
            "missing_code_review",
            "Missing code review evidence",
            "The evaluation did not find pull request, code review, or reviewer evidence.",
            "medium",
            &["evidence_refs", "deployment_gate", "change_id"],
            "Attach PR or reviewer evidence before release review.",
        ),
        change_risk_rule(
            "missing_change_link",
            "Missing change link",
            "The evaluation is not linked to a release or change identifier.",
            "medium",
            &["change_id", "release_id"],
            "Link the evaluation to the reviewed change or release.",
        ),
        change_risk_rule(
            "provider_unhealthy",
            "Provider health warning",
            "Provider health evidence reported a warning or unhealthy state.",
            "medium",
            &["deployment_gate", "governance_decision", "warnings"],
            "Review provider health and refresh evidence before release.",
        ),
        change_risk_rule(
            "policy_source_conflict",
            "Policy source conflict",
            "Policy evidence indicates a repo/config source conflict.",
            "medium",
            &["deployment_gate", "governance_decision", "warnings"],
            "Resolve the policy source conflict or document the manual decision.",
        ),
        change_risk_rule(
            "production_environment",
            "Production environment",
            "The target environment is production.",
            "medium",
            &["environment"],
            "Have a human reviewer confirm production impact and rollback readiness.",
        ),
        change_risk_rule(
            "break_glass_involved",
            "Break-glass involved",
            "The deployment gate used or reported break-glass handling.",
            "high",
            &["deployment_gate", "break_glass"],
            "Review the break-glass approval, expiry, and incident context manually.",
        ),
        change_risk_rule(
            "stale_evidence",
            "Stale or insufficient evidence",
            "The available evidence is stale, incomplete, or only advisory.",
            "medium",
            &["evidence_packet_hash", "deployment_gate", "evidence_refs"],
            "Refresh evidence and rerun the human review before relying on the advisory.",
        ),
        change_risk_rule(
            "gate_requires_approval",
            "Gate requires approval",
            "The deployment gate requires more valid approvals.",
            "medium",
            &["deployment_gate", "release_approval"],
            "Collect the required human approvals before deployment.",
        ),
        change_risk_rule(
            "gate_blocked",
            "Gate blocked",
            "The deployment gate returned a blocking decision.",
            "high",
            &["deployment_gate", "blocked_by"],
            "Resolve blocking governance gaps or document accepted risk.",
        ),
        change_risk_rule(
            "insufficient_evidence",
            "Insufficient evidence",
            "One or more required evidence inputs are missing.",
            "medium",
            &["missing_evidence", "evidence_refs", "deployment_gate"],
            "Attach the missing evidence and rerun the review.",
        ),
    ]
}

fn change_risk_catalog_hash() -> String {
    let payload = json!({
        "ruleset_version": CHANGE_RISK_RULESET_VERSION,
        "rules": change_risk_rule_catalog()
    });
    format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_string(&payload).unwrap_or_default().as_bytes())
    )
}

fn change_risk_evidence_refs_contain(
    payload: &ChangeRiskEvaluationRequest,
    needles: &[&str],
) -> bool {
    payload.evidence_refs.iter().any(|value| {
        let lower = value.to_ascii_lowercase();
        needles.iter().any(|needle| lower.contains(needle))
    })
}

fn change_risk_gate_text_contains(
    gate: Option<&DeploymentGateAuthorizationRecord>,
    needles: &[&str],
) -> bool {
    let Some(gate) = gate else {
        return false;
    };
    let mut values = vec![gate.reason.as_str(), gate.decision.as_str()];
    values.extend(gate.warnings.iter().map(String::as_str));
    values.extend(gate.blocked_by.iter().map(String::as_str));
    let governance_text = gate.governance_decision.to_string().to_ascii_lowercase();
    values.iter().any(|value| {
        let lower = value.to_ascii_lowercase();
        needles.iter().any(|needle| lower.contains(needle))
    }) || needles
        .iter()
        .any(|needle| governance_text.contains(needle))
}

fn build_change_risk_rule_trace(
    payload: &ChangeRiskEvaluationRequest,
    gate: Option<&DeploymentGateAuthorizationRecord>,
    risk_level: &str,
    reasons: &[String],
    missing: &[String],
    blocking: &[String],
) -> (Vec<String>, Vec<String>, serde_json::Value, String) {
    let catalog = change_risk_rule_catalog();
    let has_ci_ref = change_risk_evidence_refs_contain(
        payload,
        &[
            "ci",
            "pipeline",
            "check",
            "jenkins",
            "github/actions",
            "github_actions",
        ],
    );
    let has_review_ref = change_risk_evidence_refs_contain(
        payload,
        &["pull", "pr/", "review", "reviewer", "merge"],
    );
    let has_gate = gate.is_some();
    let has_packet = payload.evidence_packet_hash.is_some();

    let mut triggered_rules = Vec::new();
    let mut entries = Vec::new();
    for rule in &catalog {
        let triggered = match rule.rule_id.as_str() {
            "missing_release_approval" => missing.iter().any(|item| item == "release_approval"),
            "missing_ci_evidence" => {
                missing.iter().any(|item| {
                    matches!(
                        item.as_str(),
                        "ci_evidence" | "pipeline_evidence" | "check_run"
                    )
                }) || (!has_gate && !has_packet && !has_ci_ref)
            }
            "missing_code_review" => !has_gate && !has_review_ref,
            "missing_change_link" => payload.change_id.is_none() && payload.release_id.is_none(),
            "provider_unhealthy" => change_risk_gate_text_contains(
                gate,
                &["provider_unhealthy", "provider health", "unhealthy"],
            ),
            "policy_source_conflict" => change_risk_gate_text_contains(
                gate,
                &["policy_source_conflict", "policy source conflict"],
            ),
            "production_environment" => reasons.iter().any(|item| item == "production_environment"),
            "break_glass_involved" => reasons.iter().any(|item| item == "break_glass_involved"),
            "stale_evidence" => reasons
                .iter()
                .any(|item| item == "stale_or_insufficient_evidence"),
            "gate_requires_approval" => reasons
                .iter()
                .any(|item| item == "deployment_gate_requires_approval"),
            "gate_blocked" => reasons.iter().any(|item| item == "deployment_gate_blocked"),
            "insufficient_evidence" => !missing.is_empty() || !blocking.is_empty(),
            _ => false,
        };
        if triggered {
            triggered_rules.push(rule.rule_id.clone());
        }
        entries.push(json!({
            "rule_id": &rule.rule_id,
            "title": &rule.title,
            "severity": &rule.severity,
            "triggered": triggered,
            "evidence_inputs": &rule.evidence_inputs,
            "manual_action_hint": &rule.manual_action_hint,
        }));
    }

    triggered_rules = unique_change_risk_values(triggered_rules);
    let non_triggered_rules = catalog
        .iter()
        .map(|rule| rule.rule_id.clone())
        .filter(|rule_id| !triggered_rules.iter().any(|triggered| triggered == rule_id))
        .collect::<Vec<_>>();

    let trace = json!({
        "schema_version": "change-risk-evaluation-trace.v1",
        "ruleset_version": CHANGE_RISK_RULESET_VERSION,
        "catalog_hash": change_risk_catalog_hash(),
        "deterministic": true,
        "risk_level": risk_level,
        "advisory_only": true,
        "llm_used": false,
        "agent_governance_used": false,
        "compliance_claim": false,
        "certification": false,
        "input_summary": {
            "repository_full_name": payload.repository_full_name,
            "branch": payload.branch,
            "environment": payload.environment,
            "has_change_id": payload.change_id.is_some(),
            "has_deployment_gate": has_gate,
            "has_release_id": payload.release_id.is_some(),
            "has_commit_sha": payload.commit_sha.is_some(),
            "has_evidence_packet_hash": has_packet,
            "evidence_ref_count": payload.evidence_refs.len(),
            "has_ci_ref": has_ci_ref,
            "has_code_review_ref": has_review_ref
        },
        "risk_reasons": reasons,
        "missing_evidence": missing,
        "blocking_gaps": blocking,
        "triggered_rules": &triggered_rules,
        "non_triggered_rules": &non_triggered_rules,
        "rules": entries
    });
    let trace_hash = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_string(&trace).unwrap_or_default().as_bytes())
    );
    (triggered_rules, non_triggered_rules, trace, trace_hash)
}
