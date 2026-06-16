use super::common::*;

#[tokio::test]
async fn first_governed_repo_setup_rejects_secrets_and_upserts_idempotently() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "first-governed").await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "first-governed-admin", "Admin", &org_id).await;
    let auditor_key =
        insert_test_api_key_for_org(&pool, "first-governed-auditor", "Auditor", &org_id).await;
    let dev_key =
        insert_test_api_key_for_org(&pool, "first-governed-dev", "Developer", &org_id).await;
    let other_org_id = insert_test_org(&pool, "first-governed-other").await;
    let other_admin_key =
        insert_test_api_key_for_org(&pool, "first-governed-other-admin", "Admin", &other_org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let valid_payload = serde_json::json!({
        "status": "ready",
        "goal": "govern_release",
        "repository_full_name": "first-governed/app",
        "default_branch": "main",
        "selected_providers": ["github", "jira", "github"],
        "selected_modules": ["traceability", "release-readiness", "evidence-packets", "quality-gates"],
        "policy_preset": "strict",
        "baseline": {
            "policy_workflow_preview_acknowledged": true,
            "operator_note": "reviewed in PR before activation"
        }
    });

    let (status, body) = json_request(
        &app,
        "GET",
        "/enterprise/first-governed-repo-setup",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "initial get should work: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["found"], false);

    let (status, body) = json_request(
        &app,
        "PUT",
        "/enterprise/first-governed-repo-setup",
        Some(&valid_payload.to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not persist setup: {body}"
    );

    let unsafe_payload = serde_json::json!({
        "repository_full_name": "first-governed/app",
        "selected_providers": ["github"],
        "selected_modules": ["traceability"],
        "baseline": {
            "policy_workflow_preview_acknowledged": true,
            "notes": { "jira_api_token": "ATATT-example" }
        }
    });
    let (status, body) = json_request(
        &app,
        "PUT",
        "/enterprise/first-governed-repo-setup",
        Some(&unsafe_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "secret-looking baseline must be rejected: {body}"
    );

    let (status, body) = json_request(
        &app,
        "PUT",
        "/enterprise/first-governed-repo-setup",
        Some(&valid_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "valid setup should save: {body}");
    let first: serde_json::Value = serde_json::from_str(&body).unwrap();
    let run_id = first["run_id"].as_str().expect("run_id").to_string();
    assert_eq!(first["status"], "ready");
    assert_eq!(first["policy_preset"], "strict");
    assert_eq!(
        first["selected_providers"],
        serde_json::json!(["github", "jira"])
    );
    assert_eq!(first["baseline"]["gate_readiness"], "baseline_ready");
    assert_eq!(
        first["baseline"]["first_result"]["deployment_gate_mode"],
        "advisory"
    );
    assert_eq!(first["baseline"]["setup_summary"]["provider_count"], 2);
    assert!(first["baseline"]["action_center_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .all(|gap| gap.as_str() != Some("quality_gate_evidence")));

    let update_payload = serde_json::json!({
        "status": "completed",
        "goal": "generate_audit_evidence",
        "repository_full_name": "first-governed/app",
        "default_branch": "release",
        "selected_providers": ["github", "jira", "jenkins"],
        "selected_modules": ["traceability", "release-readiness", "evidence-packets", "quality-gates", "formal-approval"],
        "policy_preset": "moderate",
        "baseline": {
            "policy_workflow_preview_acknowledged": true,
            "operator_note": "same run updated after evidence review"
        }
    });
    let (status, body) = json_request(
        &app,
        "PUT",
        "/enterprise/first-governed-repo-setup",
        Some(&update_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "second upsert should save: {body}");
    let second: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(second["run_id"], run_id);
    assert_eq!(second["status"], "completed");
    assert_eq!(second["default_branch"], "release");
    assert!(second["completed_at"].as_i64().is_some());
    assert!(second["baseline"]["action_center_gaps"]
        .as_array()
        .unwrap()
        .is_empty());

    let (status, body) = json_request(
        &app,
        "GET",
        "/enterprise/first-governed-repo-setup",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "get after upsert should work: {body}"
    );
    let fetched: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(fetched["found"], true);
    assert_eq!(fetched["setup"]["run_id"], run_id);
    assert_eq!(
        fetched["setup"]["baseline"]["gate_readiness"],
        "baseline_ready"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'upsert_first_governed_repo_setup' AND target_id = $1",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert_eq!(audit_count, 2);

    let agent_body = serde_json::json!({
        "display_name": "KAN-120 setup agent",
        "description": "Must not access manual onboarding wizard",
        "environment": "staging",
        "allowed_actions": ["commit"]
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/agent-governance/agent-keys",
        Some(&agent_body.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "agent key create: {body}");
    let agent_key: serde_json::Value = serde_json::from_str(&body).unwrap();
    let agent_token = agent_key["token"].as_str().expect("agent token");

    let (status, body) = json_request(
        &app,
        "GET",
        "/onboarding/first-governed-repo/state",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor state read: {body}");
    let state_read: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        state_read["state"]["safety"]["agent_governance_required"],
        false
    );
    assert_eq!(state_read["state"]["safety"]["stores_secret_values"], false);

    let (status, body) = json_request(
        &app,
        "GET",
        "/onboarding/first-governed-repo/state",
        None,
        Some(&dev_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not read wizard state: {body}"
    );

    let (status, body) = json_request(
        &app,
        "GET",
        "/onboarding/first-governed-repo/state",
        None,
        Some(agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not read wizard state: {body}"
    );

    let wizard_payload = serde_json::json!({
        "status": "draft",
        "goal": "govern_release",
        "repository_full_name": "first-governed/app",
        "default_branch": "main",
        "selected_providers": ["github", "jira", "jenkins"],
        "selected_modules": ["traceability", "release-readiness", "evidence-packets", "quality-gates", "formal-approval"],
        "policy_preset": "moderate",
        "baseline": {
            "policy_workflow_preview_acknowledged": true
        }
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/onboarding/first-governed-repo/runs",
        Some(&wizard_payload.to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not create wizard run: {body}"
    );

    let (status, body) = json_request(
        &app,
        "POST",
        "/onboarding/first-governed-repo/runs",
        Some(&wizard_payload.to_string()),
        Some(agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not create wizard run: {body}"
    );

    let (status, body) = json_request(
        &app,
        "POST",
        "/onboarding/first-governed-repo/runs",
        Some(&wizard_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "existing setup should resume idempotently: {body}"
    );
    let resumed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(resumed["setup"]["run_id"], run_id);
    assert_eq!(resumed["state"]["safety"]["mutates_provider_state"], false);

    let validate_path = format!("/onboarding/first-governed-repo/runs/{run_id}/validate");
    let (status, body) = json_request(
        &app,
        "POST",
        &validate_path,
        Some(&serde_json::json!({}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "validate wizard run: {body}");
    let validated: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        validated["setup"]["baseline"]["provider_validation"]["stores_secret_values"],
        false
    );
    assert!(
        validated["state"]["provider_health"]
            .as_array()
            .unwrap()
            .len()
            >= 2
    );

    let plan_path = format!("/onboarding/first-governed-repo/runs/{run_id}/plan");
    let (status, body) = json_request(
        &app,
        "POST",
        &plan_path,
        Some(&serde_json::json!({}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "plan wizard run: {body}");
    let planned: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        planned["setup"]["baseline"]["baseline_plan"]["provider_mutation"],
        false
    );
    assert_eq!(
        planned["setup"]["baseline"]["baseline_plan"]["deployment_gate_mode"],
        "advisory"
    );

    let complete_path = format!("/onboarding/first-governed-repo/runs/{run_id}/complete");
    let (status, body) = json_request(
        &app,
        "POST",
        &complete_path,
        Some(&serde_json::json!({}).to_string()),
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "auditor must not complete setup: {body}"
    );

    let (status, body) = json_request(
        &app,
        "POST",
        &complete_path,
        Some(&serde_json::json!({}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "complete wizard run: {body}");
    let completed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(completed["setup"]["status"], "completed");
    assert_eq!(
        completed["setup"]["baseline"]["first_result_refs"]["safety"]["release_blocking_default"],
        false
    );
    assert_eq!(
        completed["setup"]["baseline"]["first_result_refs"]["safety"]["agent_governance_required"],
        false
    );
    assert_eq!(
        completed["setup"]["baseline"]["first_result_refs"]["safety"]["compliance_claim"],
        false
    );

    let (status, body) = json_request(
        &app,
        "GET",
        "/onboarding/first-governed-repo/state?org_name=first-governed",
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "other tenant admin must not read setup: {body}"
    );

    let wizard_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action IN ('onboarding_state_viewed', 'onboarding_run_resumed', 'onboarding_provider_validated', 'onboarding_baseline_planned', 'onboarding_completed') AND target_id = $1",
    )
    .bind(&run_id)
    .fetch_one(&pool)
    .await
    .expect("wizard audit count");
    assert!(
        wizard_audit_count >= 4,
        "wizard actions should be audited without secrets"
    );

    let agent_evaluations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_governance_evaluations")
            .fetch_one(&pool)
            .await
            .expect("agent evaluation count");
    assert_eq!(
        agent_evaluations, 0,
        "manual onboarding wizard must not create agent governance evaluations"
    );

    teardown(&admin_pool, &schema).await;
}
