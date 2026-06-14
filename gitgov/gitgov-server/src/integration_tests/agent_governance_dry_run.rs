use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";

fn agent_payload(action: &str) -> serde_json::Value {
    json!({
        "agent_id": "codex-kan95",
        "agent_type": "codex",
        "actor": "engineer@example.com",
        "action": action,
        "repository_full_name": REPO_FULL_NAME,
        "branch": "feature/KAN-95-agent-dry-run",
        "target_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "environment": "production",
        "ticket_id": "KAN-95",
        "operation_id": "op-kan-95",
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
            'dry-run integration opt-in',
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
        "description": "Integration test dry-run agent key",
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
async fn agent_governance_dry_run_disabled_by_default_and_does_not_persist_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-dry-run-disabled").await;
    let api_key = insert_test_api_key_for_org(
        &pool,
        "agent-governance-dry-run-disabled-dev",
        "Developer",
        &org_id,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&agent_payload("commit").to_string()),
        Some(&api_key),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected dry-run disabled response: {response}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&response).expect("disabled dry-run response JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["would_persist_evaluation"], false);
    assert_eq!(parsed["manual_governance_available"], true);

    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation count");
    assert_eq!(persisted_count, 0);

    let denied_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.dry_run_denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("dry-run denied audit count");
    assert_eq!(denied_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_dry_run_previews_without_persisting_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-dry-run-preview").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-dry-run-dev", "Developer", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&agent_payload("deploy").to_string()),
        Some(&api_key),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "unexpected dry-run: {response}");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("dry-run JSON");
    assert!(parsed.get("evaluation_id").is_none());
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["would_persist_evaluation"], false);
    assert_eq!(parsed["would_authorize_execution"], false);
    assert_eq!(parsed["decision"], "requires_approval");
    assert_eq!(parsed["requires_approval"], true);
    assert_eq!(parsed["policy_id"], "agent-governance.v1");
    assert_eq!(parsed["evaluation"]["policy"]["llm_decision"], false);
    assert_eq!(parsed["evaluation"]["dry_run"]["dry_run"], true);
    assert_eq!(
        parsed["evaluation"]["shared_governance_decision"]["consumer_type"],
        "agent_governance"
    );
    assert!(parsed["required_evidence"]
        .as_array()
        .expect("required evidence")
        .iter()
        .any(|value| value == "deployment_gate_authorization"));

    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation count");
    assert_eq!(persisted_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.dry_run_requested'",
    )
    .fetch_one(&pool)
    .await
    .expect("dry-run audit count");
    assert_eq!(audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_dry_run_records_identity_without_persisting_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-dry-run-enabled").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-dry-run-enabled-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(&app, &admin_key, "codex-agent-dry-run", vec!["commit"]).await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected agent key dry-run: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("dry-run JSON");
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["would_persist_evaluation"], false);
    assert_eq!(parsed["decision"], "allowed");
    assert_eq!(parsed["principal_type"], "agent");
    assert_eq!(parsed["agent_key_id"], key_id);
    assert_eq!(parsed["agent_display_name"], "codex-agent-dry-run");
    assert_eq!(parsed["evaluation"]["principal"]["principal_type"], "agent");
    assert_eq!(
        parsed["evaluation"]["principal"]["scope"],
        "agent_governance:evaluate"
    );

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&agent_payload("change_policy").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "change_policy dry-run should be blocked by allowed_actions: {response}"
    );
    let denied: serde_json::Value =
        serde_json::from_str(&response).expect("action denied response JSON");
    assert_eq!(denied["code"], "action_not_allowed");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    let used_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.used'")
            .fetch_one(&pool)
            .await
            .expect("agent key used audit count");
    assert_eq!(used_count, 2);

    let dry_run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.dry_run_requested'",
    )
    .fetch_one(&pool)
    .await
    .expect("dry-run audit count");
    assert_eq!(dry_run_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_dry_run_disabled_tenant_does_not_persist_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-dry-run-disabled").await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-dry-run-disabled-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(
        &app,
        &admin_key,
        "codex-agent-dry-run-disabled",
        vec!["commit"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/dry-run",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected disabled dry-run response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("disabled JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["would_persist_evaluation"], false);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    let denied_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.dry_run_denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("dry-run denied audit count");
    assert_eq!(denied_count, 1);

    teardown(&admin_pool, &schema).await;
}
