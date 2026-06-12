use super::common::*;
use axum::{routing::post, Json, Router};

async fn start_mock_opa(result: serde_json::Value) -> String {
    let app = Router::new().route(
        "/v1/data/gitgov/allow",
        post(move |Json(_payload): Json<serde_json::Value>| {
            let result = result.clone();
            async move {
                Json(serde_json::json!({
                    "decision_id": "opa-test-decision",
                    "result": result
                }))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock opa");
    let addr = listener.local_addr().expect("mock opa addr");
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::warn!(error = %e, "mock OPA server stopped");
        }
    });
    format!("http://{}", addr)
}

#[tokio::test]
async fn policy_check_is_advisory_by_default_even_when_not_allowed() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut config = crate::models::GitGovConfig::default();
    config.branches.patterns = vec!["feature/*".to_string()];
    config.enforcement.branches = crate::models::EnforcementLevel::Block;
    let config_json = serde_json::to_value(config).expect("serialize policy config");
    insert_test_policy(&pool, &repo_id, config_json).await;

    let payload = serde_json::json!({
        "repo": "acme/repo",
        "branch": "main",
        "user_login": "policy-admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/check",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "expected advisory 200: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("parse policy check body");
    assert_eq!(parsed["allowed"], false);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_check_returns_conflict_when_block_scope_matches_org_and_branch() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app_with_policy_check_scopes(
        db,
        vec![PolicyCheckBlockingScope::new(
            "acme".to_string(),
            "main".to_string(),
        )],
    );

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut config = crate::models::GitGovConfig::default();
    config.branches.patterns = vec!["feature/*".to_string()];
    config.enforcement.branches = crate::models::EnforcementLevel::Block;
    let config_json = serde_json::to_value(config).expect("serialize policy config");
    insert_test_policy(&pool, &repo_id, config_json).await;

    let payload = serde_json::json!({
        "repo": "acme/repo",
        "branch": "main",
        "user_login": "policy-admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/check",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected blocking 409: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("parse policy check body");
    assert_eq!(parsed["allowed"], false);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_check_blocks_when_required_opa_denies() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let opa_url = start_mock_opa(serde_json::json!({
        "allow": false,
        "reasons": ["release window is closed"],
        "warnings": ["opa evaluated gitgov input"]
    }))
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app_with_policy_check_scopes(
        db,
        vec![PolicyCheckBlockingScope::new(
            "acme".to_string(),
            "main".to_string(),
        )],
    );

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut config = crate::models::GitGovConfig::default();
    config.adapters.opa.enabled = true;
    config.adapters.opa.base_url = Some(opa_url);
    config.adapters.opa.effect = crate::models::ExternalPolicyEffect::Required;
    config.adapters.opa.failure_mode = crate::models::ExternalPolicyFailureMode::FailClosed;
    config.enforcement.external_policy = crate::models::EnforcementLevel::Block;
    insert_test_policy(
        &pool,
        &repo_id,
        serde_json::to_value(config).expect("serialize opa policy"),
    )
    .await;

    let payload = serde_json::json!({
        "repo": "acme/repo",
        "branch": "main",
        "commit": "abc123",
        "user_login": "policy-admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/check",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "expected OPA block: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("parse policy body");
    assert_eq!(parsed["allowed"], false);
    assert_eq!(parsed["external_decisions"][0]["adapter"], "opa");
    assert_eq!(parsed["external_decisions"][0]["status"], "denied");
    assert_eq!(
        parsed["external_decisions"][0]["decision_id"],
        "opa-test-decision"
    );
    assert_eq!(
        parsed["violations"][0]["category"],
        serde_json::Value::String("external_policy".to_string())
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_check_records_opa_fail_open_without_blocking() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut config = crate::models::GitGovConfig::default();
    config.adapters.opa.enabled = true;
    config.adapters.opa.base_url = Some("http://127.0.0.1:9".to_string());
    config.adapters.opa.effect = crate::models::ExternalPolicyEffect::Required;
    config.adapters.opa.failure_mode = crate::models::ExternalPolicyFailureMode::FailOpen;
    config.enforcement.external_policy = crate::models::EnforcementLevel::Block;
    insert_test_policy(
        &pool,
        &repo_id,
        serde_json::to_value(config).expect("serialize opa policy"),
    )
    .await;

    let payload = serde_json::json!({
        "repo": "acme/repo",
        "branch": "main",
        "commit": "abc123",
        "user_login": "policy-admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/check",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "expected fail-open 200: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("parse policy body");
    assert_eq!(parsed["allowed"], true);
    assert_eq!(parsed["external_decisions"][0]["status"], "error-fail-open");
    assert_eq!(parsed["external_decisions"][0]["allowed"], true);
    assert!(
        parsed["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str().unwrap_or_default().contains("failed open")),
        "expected fail-open warning: {}",
        body
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_override_rejects_quality_gate_downgrade_without_exception() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut existing = crate::models::GitGovConfig::default();
    existing.enforcement.quality_gates = crate::models::EnforcementLevel::Block;
    insert_test_policy(
        &pool,
        &repo_id,
        serde_json::to_value(existing).expect("serialize existing policy"),
    )
    .await;

    let downgrade_payload = serde_json::json!({
        "enforcement": {
            "quality_gates": "off"
        }
    });
    let (status, body) = json_request(
        &app,
        "PUT",
        "/policy/acme%2Frepo/override",
        Some(&downgrade_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unexpected status: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        parsed["error"]
            .as_str()
            .unwrap_or_default()
            .contains("quality gate enforcement downgrade requires active quality_gate_exception"),
        "expected governed override error, got {}",
        body
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_override_accepts_governed_exception_for_quality_gate_downgrade() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (_, repo_id) = insert_test_repo(&pool, "acme/repo").await;
    let mut existing = crate::models::GitGovConfig::default();
    existing.enforcement.quality_gates = crate::models::EnforcementLevel::Block;
    insert_test_policy(
        &pool,
        &repo_id,
        serde_json::to_value(existing).expect("serialize existing policy"),
    )
    .await;

    let expires_at = chrono::Utc::now().timestamp_millis() + 3_600_000;
    let governed_payload = serde_json::json!({
        "config": {
            "enforcement": {
                "quality_gates": "warn"
            }
        },
        "quality_gate_exception": {
            "reason": "Hotfix release window",
            "ticket_id": "OPS-777",
            "approved_by": "security-admin",
            "expires_at": expires_at
        }
    });
    let (status, body) = json_request(
        &app,
        "PUT",
        "/policy/acme%2Frepo/override",
        Some(&governed_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "override failed: {}", body);

    let (status, body) =
        json_request(&app, "GET", "/policy/acme%2Frepo", None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "get policy failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        parsed["config"]["enforcement"]["quality_gates"],
        serde_json::Value::String("warn".to_string())
    );
    assert_eq!(
        parsed["config"]["quality_gate_exception"]["reason"],
        serde_json::Value::String("Hotfix release window".to_string())
    );
    assert_eq!(
        parsed["config"]["quality_gate_exception"]["ticket_id"],
        serde_json::Value::String("OPS-777".to_string())
    );
    assert_eq!(
        parsed["config"]["quality_gate_exception"]["approved_by"],
        serde_json::Value::String("security-admin".to_string())
    );
    assert_eq!(
        parsed["config"]["quality_gate_exception"]["enabled"],
        serde_json::Value::Bool(true)
    );
    assert!(
        parsed["config"]["quality_gate_exception"]["expires_at"]
            .as_i64()
            .unwrap_or_default()
            >= expires_at
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_drift_events_require_auth() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let payload = serde_json::json!({
        "action": "sync_local",
        "repo_name": "acme-repo",
        "result": "success"
    });

    let (status, _) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&payload.to_string()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = json_request(&app, "GET", "/policy/drift-events", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_drift_ingest_and_list_for_admin() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let payload = serde_json::json!({
        "action": "sync_local",
        "repo_name": "acme-repo",
        "result": "success",
        "before_checksum": "abc",
        "after_checksum": "def",
        "duration_ms": 42,
        "metadata": { "source": "integration-test" }
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"], true);
    assert!(parsed["id"].is_string());

    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/drift-events?limit=10&offset=0",
        None,
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert_eq!(parsed["total"].as_i64().unwrap(), 1);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["user_login"], "policy-admin");
    assert_eq!(events[0]["action"], "sync_local");
    assert_eq!(events[0]["repo_name"], "acme-repo");
    assert_eq!(events[0]["result"], "success");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_drift_rejects_invalid_payload() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "policy-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let invalid_action = serde_json::json!({
        "action": "invalid",
        "repo_name": "acme-repo",
        "result": "success"
    });
    let (status, _) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&invalid_action.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let empty_repo = serde_json::json!({
        "action": "sync_local",
        "repo_name": "",
        "result": "success"
    });
    let (status, _) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&empty_repo.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn policy_drift_scope_is_enforced_for_developer() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "admin-user", "Admin").await;
    let dev_key = insert_test_api_key(&pool, "dev-user", "Developer").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let admin_event = serde_json::json!({
        "action": "push_local",
        "repo_name": "repo-admin",
        "result": "success"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&admin_event.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "admin ingest failed: {}", body);

    let dev_event = serde_json::json!({
        "action": "sync_local",
        "repo_name": "repo-dev",
        "result": "failed"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/policy/drift-events",
        Some(&dev_event.to_string()),
        Some(&dev_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "dev ingest failed: {}", body);

    // Developer cannot expand scope through query params.
    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/drift-events?limit=50&offset=0&user_login=admin-user",
        None,
        Some(&dev_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "dev list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["user_login"], "dev-user");
    assert_eq!(events[0]["repo_name"], "repo-dev");

    // Admin can filter explicitly.
    let (status, body) = json_request(
        &app,
        "GET",
        "/policy/drift-events?limit=50&offset=0&user_login=admin-user",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "admin list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["user_login"], "admin-user");
    assert_eq!(events[0]["repo_name"], "repo-admin");

    teardown(&admin_pool, &schema).await;
}
