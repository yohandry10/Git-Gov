#[cfg(test)]
mod agent_governance_tests {
    use super::*;

    fn base_payload(action: &str) -> AgentGovernanceEvaluationRequest {
        AgentGovernanceEvaluationRequest {
            org_name: Some("example".to_string()),
            agent_id: "codex-agent-1".to_string(),
            agent_type: Some("codex".to_string()),
            actor: "dev@example.com".to_string(),
            action: action.to_string(),
            repository_full_name: "owner/repo".to_string(),
            branch: Some("feature/KAN-90-agent-api".to_string()),
            target_sha: Some("a".repeat(40)),
            environment: Some("production".to_string()),
            ticket_id: Some("KAN-90".to_string()),
            operation_id: Some("op-123".to_string()),
            metadata: json!({}),
        }
    }

    #[test]
    fn agent_governance_allows_ticketed_commit() {
        let payload = base_payload("commit");
        let (decision, allowed, requires_approval, _, _, required, evaluation) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "allowed");
        assert!(allowed);
        assert!(!requires_approval);
        assert!(required.is_empty());
        assert_eq!(evaluation["policy"]["llm_decision"], false);
        assert_eq!(
            evaluation["shared_governance_decision"]["consumer_type"],
            "agent_governance"
        );
        assert_eq!(
            evaluation["shared_governance_decision"]["agent_governance_used"],
            true
        );
        assert_eq!(evaluation["shared_governance_decision"]["decision"], "allowed");
    }

    #[test]
    fn agent_governance_blocks_deploy_without_context() {
        let mut payload = base_payload("deploy");
        payload.target_sha = None;
        payload.operation_id = None;
        let (decision, allowed, requires_approval, _, reasons, required, evaluation) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "blocked");
        assert!(!allowed);
        assert!(!requires_approval);
        assert!(reasons[0].contains("Deploy requires"));
        assert!(required.contains(&"target_sha".to_string()));
        assert!(required.contains(&"operation_id".to_string()));
        assert_eq!(
            evaluation["shared_governance_decision"]["evidence"]["missing_evidence"],
            serde_json::json!(["operation_id", "target_sha"])
        );
    }

    #[test]
    fn agent_governance_requires_approval_for_protected_branch_push() {
        let mut payload = base_payload("push");
        payload.branch = Some("main".to_string());
        let (decision, allowed, requires_approval, _, _, required, evaluation) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "requires_approval");
        assert!(!allowed);
        assert!(requires_approval);
        assert!(required.contains(&"human_approval".to_string()));
        assert_eq!(evaluation["protected_branch"], true);
    }

    #[test]
    fn agent_governance_minimizes_secret_like_metadata() {
        let mut payload = base_payload("commit");
        payload.metadata = json!({
            "source": "unit-test",
            "api_token": "Bearer should-not-persist",
            "nested": {
                "password": "secret-value",
                "note": "safe"
            }
        });

        let minimized = minimized_agent_governance_request_payload(&payload);

        assert_eq!(minimized["metadata"]["source"], "unit-test");
        assert_eq!(minimized["metadata"]["api_token"], REDACTED_VALUE);
        assert_eq!(minimized["metadata"]["nested"]["password"], REDACTED_VALUE);
        assert_eq!(minimized["metadata"]["nested"]["note"], "safe");
        assert_eq!(minimized["payload_mode"], "minimized");
    }
}
