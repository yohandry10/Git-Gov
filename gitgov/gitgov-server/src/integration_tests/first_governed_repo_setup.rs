use super::common::*;

#[tokio::test]
async fn first_governed_repo_setup_rejects_secrets_and_upserts_idempotently() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "first-governed").await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "first-governed-admin", "Admin", &org_id).await;
    let dev_key =
        insert_test_api_key_for_org(&pool, "first-governed-dev", "Developer", &org_id).await;
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

    teardown(&admin_pool, &schema).await;
}
