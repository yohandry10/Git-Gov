use super::common::*;

#[tokio::test]
async fn critical_drift_alert_is_dispatched_to_dedicated_webhook() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "drift-alert-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));

    let (probe_url, body_rx, probe_task) = spawn_webhook_probe().await;
    let app = build_test_app_with_alerts(db, None, vec![probe_url]);

    let repo_name = format!("org/repo-{}", uuid::Uuid::new_v4().simple());
    let payload = serde_json::json!({
        "action": "drift_snapshot",
        "repo_name": repo_name,
        "result": "observed",
        "metadata": {
            "drift_count": 4,
            "critical_count": 2
        }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "drift ingest failed: {}", body);

    let delivered_body = tokio::time::timeout(Duration::from_secs(2), body_rx)
        .await
        .expect("webhook delivery timeout")
        .expect("webhook body channel");
    assert!(
        delivered_body.contains("\"text\""),
        "expected slack-compatible text payload"
    );
    assert!(
        delivered_body.contains("Policy Drift"),
        "expected drift alert text in webhook payload"
    );
    assert!(
        delivered_body.contains("drift-alert-admin"),
        "expected actor login in webhook payload"
    );

    probe_task.await.expect("webhook probe task");
    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn critical_drift_alert_falls_back_to_generic_webhook() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "drift-fallback-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));

    let (probe_url, body_rx, probe_task) = spawn_webhook_probe().await;
    let app = build_test_app_with_alerts(db, Some(probe_url), vec![]);

    let repo_name = format!("org/repo-{}", uuid::Uuid::new_v4().simple());
    let payload = serde_json::json!({
        "action": "drift_snapshot",
        "repo_name": repo_name,
        "result": "observed",
        "metadata": {
            "drift_count": 3,
            "critical_count": 1
        }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "drift ingest failed: {}", body);

    let delivered_body = tokio::time::timeout(Duration::from_secs(2), body_rx)
        .await
        .expect("fallback webhook delivery timeout")
        .expect("fallback webhook body channel");
    assert!(
        delivered_body.contains("Policy Drift"),
        "expected drift alert text in fallback webhook payload"
    );
    assert!(
        delivered_body.contains("drift-fallback-admin"),
        "expected actor login in fallback webhook payload"
    );

    probe_task.await.expect("fallback webhook probe task");
    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn export_includes_policy_drift_and_policy_requests_in_json_and_csv() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "export-admin", "Admin").await;
    let _repo = insert_test_repo(&pool, "org/repo").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let events_payload = serde_json::json!({
        "events": [{
            "event_uuid": "export-drift-0001",
            "event_type": "commit",
            "repo_full_name": "org/repo",
            "user_login": "export-admin",
            "files": [],
            "status": "success",
            "branch": "main",
            "timestamp": 1700000010
        }],
        "client_version": "integration-test"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&events_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "event ingest failed: {}", body);

    let drift_payload = serde_json::json!({
        "action": "drift_snapshot",
        "repo_name": "org/repo",
        "result": "observed",
        "metadata": {
            "drift_count": 2,
            "critical_count": 1
        }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&drift_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "drift ingest failed: {}", body);

    let policy_request_payload = serde_json::json!({
        "config": {
            "branches": { "protected": ["main"], "patterns": ["feat/*"] },
            "rules": { "require_pull_request": true, "require_linked_ticket": true },
            "enforcement": { "pull_requests": "warn", "commits": "warn", "branches": "warn", "traceability": "warn" }
        },
        "reason": "Export coverage for policy requests"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/org%2Frepo/requests",
        Some(&policy_request_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "policy request ingest failed: {}",
        body
    );

    let json_export_payload = serde_json::json!({
        "export_type": "events"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/export",
        Some(&json_export_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "json export failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let data = parsed["data"].as_object().expect("data object");
    assert!(
        data.get("events")
            .and_then(|v| v.as_array())
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        "expected exported events array"
    );
    assert_eq!(
        data.get("policy_drift_events")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0),
        1
    );
    assert_eq!(
        data.get("policy_change_requests")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0),
        1
    );

    let csv_export_payload = serde_json::json!({
        "export_type": "events_csv"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/export",
        Some(&csv_export_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "csv export failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let csv_data = parsed["data"].as_str().unwrap_or_default();
    assert!(csv_data.contains("record_kind,id,source,event_type"));
    assert!(csv_data.contains("policy_drift"));
    assert!(csv_data.contains("policy_change_request"));
    assert!(csv_data.contains("org/repo"));

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_change_request_can_be_created_and_approved_by_admin() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let dev_key = insert_test_api_key(&pool, "policy-dev", "Developer").await;
    let _repo = insert_test_repo(&pool, "acme/repo").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let create_payload = serde_json::json!({
        "config": {
            "branches": { "protected": ["main"], "patterns": ["feat/*"] },
            "rules": { "require_pull_request": true, "min_approvals": 1 },
            "enforcement": { "pull_requests": "warn", "commits": "off", "branches": "warn", "traceability": "off" }
        },
        "reason": "Enable baseline protection"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/acme%2Frepo/requests",
        Some(&create_payload.to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create request failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"], true);
    assert_eq!(parsed["status"], "pending");
    let request_id = parsed["request_id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/approve", request_id),
        Some(&serde_json::json!({}).to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/approve", request_id),
        Some(&serde_json::json!({"note":"Looks good"}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "approve request failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["status"], "approved");
    assert_eq!(parsed["decided_by"], "policy-admin");

    let (status, body) =
        json_request(&app, "GET", "/policy/acme%2Frepo", None, Some(&admin_key)).await;
    assert_eq!(status, StatusCode::OK, "get policy failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["config"]["rules"]["require_pull_request"], true);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_change_request_rejects_self_approval() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let _repo = insert_test_repo(&pool, "acme/repo").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let create_payload = serde_json::json!({
        "config": { "rules": { "require_linked_ticket": true } },
        "reason": "Require ticket linkage"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/acme%2Frepo/requests",
        Some(&create_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create request failed: {}", body);
    let request_id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["request_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/approve", request_id),
        Some(&serde_json::json!({"note":"approve own request"}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_change_request_can_be_rejected_by_admin() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let dev_key = insert_test_api_key(&pool, "policy-dev", "Developer").await;
    let _repo = insert_test_repo(&pool, "acme/repo").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let create_payload = serde_json::json!({
        "config": {
            "branches": { "protected": ["main"], "patterns": ["feat/*"] },
            "rules": { "require_linked_ticket": true, "require_pull_request": true },
            "enforcement": { "pull_requests": "warn", "commits": "warn", "branches": "warn", "traceability": "warn" }
        },
        "reason": "Request stricter traceability"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/acme%2Frepo/requests",
        Some(&create_payload.to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create request failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"], true);
    let request_id = parsed["request_id"].as_str().unwrap().to_string();

    let reject_note = "Needs alignment with release policy";
    let (status, body) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/reject", request_id),
        Some(&serde_json::json!({"note": reject_note}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reject request failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["status"], "rejected");
    assert_eq!(parsed["decided_by"], "policy-admin");
    assert_eq!(parsed["decision_note"], reject_note);

    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?status=rejected&limit=10&offset=0",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "list rejected requests failed: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(parsed["total"].as_i64().unwrap_or_default() >= 1);
    let requests = parsed["requests"].as_array().cloned().unwrap_or_default();
    assert!(
        requests
            .iter()
            .any(|item| item["id"] == request_id && item["status"] == "rejected"),
        "expected rejected request id in filtered list"
    );

    let (status, _) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/approve", request_id),
        Some(&serde_json::json!({"note":"should conflict"}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_change_request_scope_is_enforced_for_multisession_listing() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let dev_a_key = insert_test_api_key(&pool, "policy-dev-a", "Developer").await;
    let dev_b_key = insert_test_api_key(&pool, "policy-dev-b", "Developer").await;
    let _repo = insert_test_repo(&pool, "acme/repo").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let create_payload_a = serde_json::json!({
        "config": {
            "branches": { "protected": ["main"], "patterns": ["feat/*"] },
            "rules": { "require_pull_request": true, "min_approvals": 1 },
            "enforcement": { "pull_requests": "warn", "commits": "off", "branches": "warn", "traceability": "off" }
        },
        "reason": "Developer A proposal"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/acme%2Frepo/requests",
        Some(&create_payload_a.to_string()),
        Some(&dev_a_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create request A failed: {}", body);
    let request_a_id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["request_id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_payload_b = serde_json::json!({
        "config": {
            "branches": { "protected": ["main"], "patterns": ["fix/*"] },
            "rules": { "require_linked_ticket": true },
            "enforcement": { "pull_requests": "off", "commits": "warn", "branches": "off", "traceability": "warn" }
        },
        "reason": "Developer B proposal"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/acme%2Frepo/requests",
        Some(&create_payload_b.to_string()),
        Some(&dev_b_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create request B failed: {}", body);
    let request_b_id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["request_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Developer A only sees own requests.
    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?limit=20&offset=0",
        None,
        Some(&dev_a_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "dev A list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["total"], 1);
    let requests = parsed["requests"].as_array().cloned().unwrap_or_default();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["id"], request_a_id);
    assert_eq!(requests[0]["requested_by"], "policy-dev-a");

    // Developer B only sees own requests.
    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?limit=20&offset=0",
        None,
        Some(&dev_b_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "dev B list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["total"], 1);
    let requests = parsed["requests"].as_array().cloned().unwrap_or_default();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["id"], request_b_id);
    assert_eq!(requests[0]["requested_by"], "policy-dev-b");

    // Admin sees all pending requests.
    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?status=pending&limit=20&offset=0",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin list pending failed: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(parsed["total"].as_i64().unwrap_or_default() >= 2);
    let requests = parsed["requests"].as_array().cloned().unwrap_or_default();
    assert!(
        requests.iter().any(|item| item["id"] == request_a_id),
        "expected request A in admin list"
    );
    assert!(
        requests.iter().any(|item| item["id"] == request_b_id),
        "expected request B in admin list"
    );

    // After admin approval of request A, developer B cannot see it under approved filter.
    let (status, body) = json_request(
        &app,
        "POST",
        &format!("/policy/requests/{}/approve", request_a_id),
        Some(&serde_json::json!({"note":"scope check approval"}).to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "approve request A failed: {}", body);

    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?status=approved&limit=20&offset=0",
        None,
        Some(&dev_b_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "dev B approved list failed: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        parsed["total"].as_i64().unwrap_or_default(),
        0,
        "developer B should not see approvals from developer A"
    );

    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/acme%2Frepo/requests?status=approved&limit=20&offset=0",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin approved list failed: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let requests = parsed["requests"].as_array().cloned().unwrap_or_default();
    assert!(
        requests.iter().any(|item| item["id"] == request_a_id),
        "expected approved request A in admin approved list"
    );

    teardown(&admin_pool, &schema).await;
}
