use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";

fn attribution_payload(action: &str, correlation_id: Option<&str>) -> serde_json::Value {
    let mut attribution = json!({
        "session_id": "sess-kan96-001",
        "tool_name": "codex-cli",
        "tool_version": "1.0.0",
        "agent_name": "codex-kan96-agent",
        "external_run_id": "github-actions-run-123"
    });
    if let Some(correlation_id) = correlation_id {
        attribution["correlation_id"] = json!(correlation_id);
    }

    json!({
        "agent_id": "codex-kan96",
        "agent_type": "codex",
        "actor": "engineer@example.com",
        "action": action,
        "repository_full_name": REPO_FULL_NAME,
        "branch": "feature/KAN-96-agent-attribution-envelope",
        "target_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "environment": "production",
        "ticket_id": "KAN-96",
        "operation_id": "op-kan-96",
        "attribution": attribution,
        "metadata": {
            "source": "integration-test"
        }
    })
}

async fn enable_agent_governance(pool: &sqlx::PgPool, org_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO agent_governance_settings (
            org_id,
            enabled,
            mode,
            payload_mode,
            reason,
            updated_by
        )
        VALUES (
            $1::uuid,
            TRUE,
            'opt_in_enabled',
            'minimized',
            'attribution integration opt-in',
            'integration-test'
        )
        ON CONFLICT (org_id) DO UPDATE SET
            enabled = TRUE,
            mode = 'opt_in_enabled',
            payload_mode = 'minimized',
            reason = EXCLUDED.reason,
            updated_by = EXCLUDED.updated_by,
            updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .execute(pool)
    .await
    .expect("enable agent governance");
}

async fn create_agent_key(
    app: &axum::Router,
    admin_key: &str,
    display_name: &str,
    allowed_actions: Vec<&str>,
) -> serde_json::Value {
    let body = json!({
        "display_name": display_name,
        "description": "Integration test attribution agent key",
        "environment": "staging",
        "allowed_actions": allowed_actions
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/agent-governance/agent-keys",
        Some(&body.to_string()),
        Some(admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected agent key create response: {response}"
    );
    serde_json::from_str(&response).expect("agent key create JSON")
}

#[tokio::test]
async fn agent_governance_evaluate_persists_minimal_attribution_and_history_filter() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-attribution-evaluate").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-attribution-evaluate-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(&app, &admin_key, "codex-kan96-evaluate", vec!["commit"]).await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&attribution_payload("commit", Some("corr-kan96-evaluate")).to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected evaluate response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("evaluate JSON");
    assert_eq!(parsed["decision"], "allowed");
    assert_eq!(parsed["principal_type"], "agent");
    assert_eq!(parsed["agent_key_id"], key_id);
    assert_eq!(
        parsed["attribution"]["correlation_id"],
        "corr-kan96-evaluate"
    );
    assert_eq!(parsed["attribution"]["session_id"], "sess-kan96-001");
    assert_eq!(parsed["attribution"]["tool_name"], "codex-cli");
    assert_eq!(parsed["attribution"]["agent_key_id"], key_id);
    assert_eq!(parsed["attribution"]["consumer_type"], "agent_governance");
    assert_eq!(
        parsed["evaluation"]["attribution"]["correlation_id"],
        "corr-kan96-evaluate"
    );
    assert_eq!(parsed["evaluation"]["policy"]["llm_decision"], false);

    let persisted: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT attribution_id, correlation_id, session_id, tool_name
        FROM agent_governance_evaluations
        WHERE org_id = $1::uuid
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted attribution");
    assert!(persisted.0.starts_with("attr_"));
    assert_eq!(persisted.1, "corr-kan96-evaluate");
    assert_eq!(persisted.2, "sess-kan96-001");
    assert_eq!(persisted.3, "codex-cli");

    let (status, history) = json_request(
        &app,
        "GET",
        "/agent-governance/evaluations?correlation_id=corr-kan96-evaluate",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unexpected history: {history}");
    let history: serde_json::Value = serde_json::from_str(&history).expect("history JSON");
    assert_eq!(history["total"], 1);
    assert_eq!(
        history["items"][0]["attribution"]["correlation_id"],
        "corr-kan96-evaluate"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_dry_run_returns_attribution_without_formal_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-attribution-dry-run").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-attribution-dry-run-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(&app, &admin_key, "codex-kan96-dry-run", vec!["commit"]).await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&attribution_payload("commit", Some("corr-kan96-dry-run")).to_string()),
        Some(token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unexpected dry-run: {response}");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("dry-run JSON");
    assert!(parsed.get("evaluation_id").is_none());
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["would_persist_evaluation"], false);
    assert_eq!(parsed["would_authorize_execution"], false);
    assert_eq!(parsed["consumer_type"], "agent_dry_run");
    assert_eq!(
        parsed["attribution"]["correlation_id"],
        "corr-kan96-dry-run"
    );
    assert_eq!(parsed["attribution"]["agent_key_id"], key_id);
    assert_eq!(parsed["attribution"]["consumer_type"], "agent_dry_run");

    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation count");
    assert_eq!(persisted_count, 0);

    let audit_metadata: serde_json::Value = sqlx::query_scalar(
        "SELECT metadata FROM admin_audit_log WHERE action = 'agent_governance.dry_run_requested'",
    )
    .fetch_one(&pool)
    .await
    .expect("dry-run audit metadata");
    assert_eq!(audit_metadata["correlation_id"], "corr-kan96-dry-run");
    assert_eq!(audit_metadata["tool_name"], "codex-cli");
    assert_eq!(
        audit_metadata["attribution"]["consumer_type"],
        "agent_dry_run"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_generates_correlation_id_when_missing() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-attribution-generated-correlation").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key = insert_test_api_key_for_org(
        &pool,
        "agent-attribution-generated-correlation-dev",
        "Developer",
        &org_id,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&attribution_payload("commit", None).to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected evaluate response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("evaluate JSON");
    let correlation_id = parsed["attribution"]["correlation_id"]
        .as_str()
        .expect("generated correlation id");
    assert!(correlation_id.starts_with("agcorr_"));

    let stored: String = sqlx::query_scalar(
        "SELECT correlation_id FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("stored generated correlation id");
    assert_eq!(stored, correlation_id);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_manual_only_rejects_attributed_requests_without_persistence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-attribution-manual-only").await;
    let api_key = insert_test_api_key_for_org(
        &pool,
        "agent-attribution-manual-only-dev",
        "Developer",
        &org_id,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&attribution_payload("commit", Some("corr-kan96-manual")).to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected manual-only response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("manual-only JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");

    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation count");
    assert_eq!(persisted_count, 0);

    let audit_metadata: serde_json::Value = sqlx::query_scalar(
        "SELECT metadata FROM admin_audit_log WHERE action = 'agent_governance.evaluation_denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("denied audit metadata");
    assert_eq!(
        audit_metadata["attribution"]["correlation_id"],
        "corr-kan96-manual"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_rejects_unsafe_attribution_without_persistence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-attribution-invalid").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-attribution-invalid-dev", "Developer", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let mut payload = attribution_payload("commit", Some("corr-kan96-invalid"));
    payload["attribution"]["tool_name"] = json!("Bearer should-not-be-accepted");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unsafe attribution should be rejected: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("invalid JSON");
    assert_eq!(parsed["error"], "Invalid agent governance evaluation");
    assert!(parsed["details"]
        .as_array()
        .expect("details")
        .iter()
        .any(|detail| detail
            .as_str()
            .is_some_and(|text| text.contains("credential material"))));

    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation count");
    assert_eq!(persisted_count, 0);

    teardown(&admin_pool, &schema).await;
}
