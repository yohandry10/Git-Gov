#[cfg(test)]
mod deployment_gate_tests {
    use super::*;

    fn valid_authorization() -> DeploymentGateAuthorizationRequest {
        DeploymentGateAuthorizationRequest {
            org_name: Some("yohandry10".to_string()),
            release_id: "release-2026.06.13".to_string(),
            repository_full_name: "yohandry10/Git-Gov".to_string(),
            branch: "main".to_string(),
            target_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            environment: "production".to_string(),
            deployer: "github-actions".to_string(),
            ticket_id: Some("KAN-83".to_string()),
            evidence_packet_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            evidence_packet_uri: Some("/evidence/packets/tickets/KAN-83".to_string()),
            requested_by: Some("release-bot".to_string()),
            deployment_run_id: Some("run-123".to_string()),
            metadata: json!({ "workflow": "deploy-production" }),
            break_glass: None,
        }
    }

    #[test]
    fn deployment_gate_authorization_validation_requires_full_sha() {
        let mut payload = valid_authorization();
        payload.target_sha = "abcdef1".to_string();

        let errors = normalize_and_validate_deployment_gate_authorization(&mut payload).unwrap_err();

        assert!(errors.contains(
            &"target_sha must be a full 40 or 64 character hexadecimal commit SHA.".to_string()
        ));
    }

    #[test]
    fn deployment_gate_authorization_checksum_is_policy_stable() {
        let evaluation = EnterpriseReleaseGovernanceEvaluationResponse {
            policy: EnterpriseReleaseGovernancePolicySummary {
                mode: "record-only".to_string(),
                environment: "production".to_string(),
                approval_required: false,
                enforcement: "disabled".to_string(),
                policy_applies: true,
                quorum_enabled: false,
                quorum_rules: Vec::new(),
            },
            ..EnterpriseReleaseGovernanceEvaluationResponse::default()
        };

        assert_eq!(
            deployment_gate_policy_checksum(&evaluation),
            deployment_gate_policy_checksum(&evaluation)
        );
    }
}

