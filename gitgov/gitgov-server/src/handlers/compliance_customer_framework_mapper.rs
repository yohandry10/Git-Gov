#[derive(Debug, Clone)]
enum CustomerEvidenceCheck {
    Present(&'static str),
    Missing(&'static str),
    NotApplicable(&'static str),
}

fn customer_evidence_check(
    artifact: &serde_json::Value,
    evidence_type: &str,
) -> CustomerEvidenceCheck {
    match evidence_type {
        "deployment_gate.decision" => {
            if json_string_exists(artifact, "/deployment_gate/decision") {
                CustomerEvidenceCheck::Present("deployment_gate.decision")
            } else {
                CustomerEvidenceCheck::Missing("deployment_gate.decision")
            }
        }
        "policy.checksum" => {
            if json_string_exists(artifact, "/policy/checksum") {
                CustomerEvidenceCheck::Present("policy.checksum")
            } else {
                CustomerEvidenceCheck::Missing("policy.checksum")
            }
        }
        "policy.source" => {
            if artifact.pointer("/policy/source").is_some() {
                CustomerEvidenceCheck::Present("policy.source")
            } else {
                CustomerEvidenceCheck::Missing("policy.source")
            }
        }
        "release_approval" => {
            let required = json_i64(artifact, "/readiness/required_approval_count");
            let valid = json_i64(artifact, "/readiness/valid_approval_count");
            if required <= 0 {
                CustomerEvidenceCheck::NotApplicable("release_approval")
            } else if valid >= required {
                CustomerEvidenceCheck::Present("approvals.release_governance_approvals")
            } else {
                CustomerEvidenceCheck::Missing("release_approval")
            }
        }
        "ci_build_evidence" => {
            if json_i64(artifact, "/evidence/counts/pipeline_events") > 0 {
                CustomerEvidenceCheck::Present("evidence.counts.pipeline_events")
            } else {
                CustomerEvidenceCheck::Missing("ci_build_evidence")
            }
        }
        "code_change_evidence" => {
            if json_i64(artifact, "/evidence/counts/client_events") > 0 {
                CustomerEvidenceCheck::Present("evidence.counts.client_events")
            } else {
                CustomerEvidenceCheck::Missing("code_change_evidence")
            }
        }
        "pr_review_evidence" => CustomerEvidenceCheck::Missing("pr_review_evidence"),
        "quality_gate_result" => {
            if json_array_contains_string(artifact, "/gaps/missing_evidence", "sonar_quality_gate")
                || json_array_contains_string(
                    artifact,
                    "/readiness/missing_evidence",
                    "sonar_quality_gate",
                )
            {
                CustomerEvidenceCheck::Missing("sonar_quality_gate")
            } else if artifact.pointer("/evidence/sonar").is_some() {
                CustomerEvidenceCheck::Present("evidence.sonar")
            } else {
                CustomerEvidenceCheck::Missing("quality_gate_result")
            }
        }
        "deployment_target" => {
            let complete = [
                "/deployment_gate/repository_full_name",
                "/deployment_gate/branch",
                "/deployment_gate/target_sha",
                "/deployment_gate/environment",
            ]
            .iter()
            .all(|pointer| json_string_exists(artifact, pointer));
            if complete {
                CustomerEvidenceCheck::Present("deployment_gate.target")
            } else {
                CustomerEvidenceCheck::Missing("deployment_target")
            }
        }
        "missing_evidence" => {
            if json_array_exists(artifact, "/gaps/missing_evidence")
                || json_array_exists(artifact, "/readiness/missing_evidence")
            {
                CustomerEvidenceCheck::Present("gaps.missing_evidence")
            } else {
                CustomerEvidenceCheck::Missing("missing_evidence")
            }
        }
        "audit_trail" => {
            let has_generated = artifact.pointer("/audit/export_generated_at").is_some();
            let redacted = json_bool(artifact, "/audit/artifact_redacted").unwrap_or(false);
            let raw_payload_included =
                json_bool(artifact, "/audit/raw_payload_included").unwrap_or(true);
            if has_generated && redacted && !raw_payload_included {
                CustomerEvidenceCheck::Present("audit")
            } else {
                CustomerEvidenceCheck::Missing("audit_trail")
            }
        }
        "deployment_gate.agent_governance_used" => {
            if artifact
                .pointer("/deployment_gate/agent_governance_used")
                .and_then(|value| value.as_bool())
                .is_some()
            {
                CustomerEvidenceCheck::Present("deployment_gate.agent_governance_used")
            } else {
                CustomerEvidenceCheck::Missing("deployment_gate.agent_governance_used")
            }
        }
        _ => CustomerEvidenceCheck::Missing("unsupported_evidence_type"),
    }
}

fn map_customer_framework_control(
    artifact: &serde_json::Value,
    control: &ComplianceControl,
) -> ComplianceEvidenceMappingItem {
    let mut evidence_refs = Vec::new();
    let mut missing_evidence = Vec::new();
    let mut not_applicable = Vec::new();

    for evidence_type in &control.required_evidence_types {
        match customer_evidence_check(artifact, evidence_type) {
            CustomerEvidenceCheck::Present(reference) => {
                if !evidence_refs.contains(&reference) {
                    evidence_refs.push(reference);
                }
            }
            CustomerEvidenceCheck::Missing(reference) => {
                if !missing_evidence.contains(&reference) {
                    missing_evidence.push(reference);
                }
            }
            CustomerEvidenceCheck::NotApplicable(reference) => {
                if !not_applicable.contains(&reference) {
                    not_applicable.push(reference);
                }
            }
        }
    }

    let status = if missing_evidence.is_empty() && !evidence_refs.is_empty() {
        "evidence_present"
    } else if missing_evidence.is_empty() && evidence_refs.is_empty() && !not_applicable.is_empty() {
        "not_applicable"
    } else if evidence_refs.is_empty() {
        "missing"
    } else {
        "partial"
    };

    let notes = match status {
        "evidence_present" => {
            "Customer-owned control evidence types were found in the KAN-99 export."
        }
        "not_applicable" => {
            "Customer-owned control evidence type was not applicable for this release export."
        }
        "partial" => {
            "Some customer-owned control evidence types were found; missing items require customer/auditor review."
        }
        _ => "Customer-owned control evidence was not found in the KAN-99 export.",
    };

    evidence_mapping_status(status, evidence_refs, missing_evidence, notes, control)
}

fn build_customer_framework_mapping_items(
    artifact: &serde_json::Value,
    framework: &ComplianceControlFramework,
) -> Vec<ComplianceEvidenceMappingItem> {
    framework
        .controls
        .iter()
        .map(|control| map_customer_framework_control(artifact, control))
        .collect()
}

