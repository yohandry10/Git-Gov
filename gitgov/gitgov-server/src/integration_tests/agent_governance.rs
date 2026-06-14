use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";

fn agent_payload(action: &str) -> serde_json::Value {
    json!({
        "agent_id": "codex-kan90",
        "agent_type": "codex",
        "actor": "engineer@example.com",
        "action": action,
        "repository_full_name": REPO_FULL_NAME,
        "branch": "feature/KAN-90-agent-governance-policy-api",
        "target_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "environment": "production",
        "ticket_id": "KAN-90",
        "operation_id": "op-kan-90",
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
            'integration opt-in',
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
        "description": "Integration test agent key",
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
async fn agent_governance_is_disabled_by_default_and_does_not_persist_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-disabled-default").await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-disabled-dev", "Developer", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(&api_key),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&response).expect("disabled response JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");
    assert_eq!(parsed["enabled"], false);
    assert_eq!(parsed["mode"], "manual_only");
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
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.evaluation_denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("denied audit count");
    assert_eq!(denied_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_allows_ticketed_commit_and_persists_audit_evidence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-allowed").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-dev", "Developer", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(&api_key),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("agent response JSON");
    assert!(parsed["evaluation_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("agv_")));
    assert_eq!(parsed["decision"], "allowed");
    assert_eq!(parsed["allowed"], true);
    assert_eq!(parsed["requires_approval"], false);
    assert_eq!(parsed["policy_id"], "agent-governance.v1");
    assert_eq!(parsed["evaluation"]["policy"]["llm_decision"], false);
    assert_eq!(
        parsed["request_payload"]["metadata"]["source"],
        "integration-test"
    );

    let persisted: (String, bool, bool, String) = sqlx::query_as(
        "SELECT decision, allowed, requires_approval, policy_id FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted evaluation");
    assert_eq!(persisted.0, "allowed");
    assert!(persisted.1);
    assert!(!persisted.2);
    assert_eq!(persisted.3, "agent-governance.v1");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_requires_approval_for_protected_branch_push() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-protected").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let mut payload = agent_payload("push");
    payload["branch"] = json!("main");

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
        StatusCode::CREATED,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("agent response JSON");
    assert_eq!(parsed["decision"], "requires_approval");
    assert_eq!(parsed["allowed"], false);
    assert_eq!(parsed["requires_approval"], true);
    assert_eq!(parsed["evaluation"]["protected_branch"], true);
    assert!(parsed["required_evidence"]
        .as_array()
        .expect("required evidence array")
        .iter()
        .any(|value| value == "human_approval"));

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_blocks_deploy_without_required_context() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-blocked").await;
    enable_agent_governance(&pool, &org_id).await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-block-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let mut payload = agent_payload("deploy");
    payload.as_object_mut().unwrap().remove("target_sha");
    payload.as_object_mut().unwrap().remove("operation_id");

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
        StatusCode::CREATED,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("agent response JSON");
    assert_eq!(parsed["decision"], "blocked");
    assert_eq!(parsed["allowed"], false);
    assert_eq!(parsed["requires_approval"], false);
    assert!(parsed["required_evidence"]
        .as_array()
        .expect("required evidence array")
        .iter()
        .any(|value| value == "target_sha"));
    assert!(parsed["required_evidence"]
        .as_array()
        .expect("required evidence array")
        .iter()
        .any(|value| value == "operation_id"));

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid AND decision = 'blocked'",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("blocked evaluation count");
    assert_eq!(count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_settings_admin_only_and_history_lists_minimized_payload() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-settings-history").await;
    let developer_key =
        insert_test_api_key_for_org(&pool, "agent-governance-settings-dev", "Developer", &org_id)
            .await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-governance-settings-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "GET",
        "/agent-governance/settings",
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not read settings: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PUT",
        "/agent-governance/settings",
        Some(
            &json!({
                "enabled": true,
                "reason": "Controlled KAN-92 integration opt-in"
            })
            .to_string(),
        ),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not update settings: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        "/agent-governance/settings",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unexpected settings: {response}");
    let default_settings: serde_json::Value =
        serde_json::from_str(&response).expect("default settings JSON");
    assert_eq!(default_settings["enabled"], false);
    assert_eq!(default_settings["mode"], "manual_only");
    assert_eq!(default_settings["payload_mode"], "minimized");

    let (status, response) = json_request(
        &app,
        "PUT",
        "/agent-governance/settings",
        Some(
            &json!({
                "enabled": true,
                "reason": "Controlled KAN-92 integration opt-in"
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected opt-in response: {response}"
    );
    let settings: serde_json::Value = serde_json::from_str(&response).expect("settings JSON");
    assert_eq!(settings["enabled"], true);
    assert_eq!(settings["mode"], "opt_in_enabled");
    assert_eq!(settings["payload_mode"], "minimized");
    assert_eq!(settings["reason"], "Controlled KAN-92 integration opt-in");

    let mut payload = agent_payload("commit");
    payload["agent_id"] = json!("codex-kan92");
    payload["metadata"] = json!({
        "source": "integration-test",
        "api_token": "Bearer must-not-persist",
        "nested": {
            "password": "also-secret",
            "note": "kept"
        }
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&payload.to_string()),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected evaluation response: {response}"
    );
    let created: serde_json::Value = serde_json::from_str(&response).expect("created JSON");
    let evaluation_id = created["evaluation_id"]
        .as_str()
        .expect("created evaluation_id");
    assert_eq!(
        created["request_payload"]["metadata"]["api_token"],
        "[REDACTED]"
    );
    assert_eq!(
        created["request_payload"]["metadata"]["nested"]["password"],
        "[REDACTED]"
    );
    assert_eq!(
        created["request_payload"]["metadata"]["nested"]["note"],
        "kept"
    );
    assert_eq!(created["request_payload"]["payload_mode"], "minimized");

    let history_path = format!("/agent-governance/evaluations?evaluation_id={evaluation_id}");
    let (status, response) =
        json_request(&app, "GET", &history_path, None, Some(&developer_key)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not read history: {response}"
    );

    let (status, response) = json_request(&app, "GET", &history_path, None, Some(&admin_key)).await;
    assert_eq!(status, StatusCode::OK, "unexpected history: {response}");
    let history: serde_json::Value = serde_json::from_str(&response).expect("history JSON");
    assert_eq!(history["total"], 1);
    assert_eq!(history["items"].as_array().expect("history array").len(), 1);
    assert_eq!(history["items"][0]["evaluation_id"], evaluation_id);
    assert_eq!(
        history["items"][0]["request_payload"]["metadata"]["api_token"],
        "[REDACTED]"
    );
    assert_eq!(
        history["items"][0]["request_payload"]["metadata"]["nested"]["password"],
        "[REDACTED]"
    );

    let enabled_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_governance.enabled'",
    )
    .fetch_one(&pool)
    .await
    .expect("enabled audit count");
    assert_eq!(enabled_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_rejects_unknown_action_and_cross_org_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-scope").await;
    let other_org_id = insert_test_org(&pool, "agent-governance-other").await;
    let api_key =
        insert_test_api_key_for_org(&pool, "agent-governance-scoped-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("delete_repo").to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response.contains("action must be one of"),
        "unexpected validation response: {response}"
    );

    let mut cross_org_payload = agent_payload("commit");
    cross_org_payload["org_name"] = json!("agent-governance-other");
    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&cross_org_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected response: {response}"
    );

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id IN ($1::uuid, $2::uuid)",
    )
    .bind(&org_id)
    .bind(&other_org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_governance_agent_keys_are_admin_only_and_token_is_one_time() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-agent-keys").await;
    let admin_key = insert_test_api_key_for_org(&pool, "agent-key-admin", "Admin", &org_id).await;
    let developer_key =
        insert_test_api_key_for_org(&pool, "agent-key-dev", "Developer", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/agent-keys",
        Some(
            &json!({
                "display_name": "codex-agent-staging",
                "allowed_actions": ["commit", "push"]
            })
            .to_string(),
        ),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not create agent keys: {response}"
    );

    let created = create_agent_key(
        &app,
        &admin_key,
        "codex-agent-staging",
        vec!["commit", "push", "open_pr", "merge_pr", "deploy"],
    )
    .await;
    let token = created["token"].as_str().expect("one-time token");
    assert!(token.starts_with("ggag_"));
    assert!(created["key_id"]
        .as_str()
        .expect("key id")
        .starts_with("agk_"));
    assert_eq!(created["scopes"], json!(["agent_governance:evaluate"]));
    assert!(!created["allowed_actions"]
        .as_array()
        .expect("allowed actions")
        .iter()
        .any(|value| value == "change_policy"));

    let stored: (String, String) =
        sqlx::query_as("SELECT token_hash, token_preview FROM agent_governance_agent_keys")
            .fetch_one(&pool)
            .await
            .expect("stored agent key");
    assert_ne!(stored.0, token);
    assert!(stored.1.starts_with("ggag_****"));
    assert!(token.ends_with(&stored.1[10..]));

    let (status, response) = json_request(
        &app,
        "GET",
        "/agent-governance/agent-keys",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unexpected list: {response}");
    let listed: serde_json::Value = serde_json::from_str(&response).expect("agent key list JSON");
    assert_eq!(listed["total"], 1);
    assert!(listed["items"][0].get("token").is_none());
    assert_eq!(listed["items"][0]["token_preview"], stored.1);

    let created_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("created audit count");
    assert_eq!(created_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_evaluates_when_enabled_and_records_agent_identity() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-enabled").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-enabled-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(
        &app,
        &admin_key,
        "codex-agent-enabled",
        vec!["commit", "push", "open_pr", "merge_pr", "deploy"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unexpected agent evaluation: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("evaluation JSON");
    assert_eq!(parsed["decision"], "allowed");
    assert_eq!(parsed["principal_type"], "agent");
    assert_eq!(parsed["agent_key_id"], key_id);
    assert_eq!(parsed["agent_display_name"], "codex-agent-enabled");
    assert_eq!(parsed["evaluation"]["principal"]["principal_type"], "agent");
    assert_eq!(
        parsed["evaluation"]["principal"]["scope"],
        "agent_governance:evaluate"
    );
    assert_eq!(parsed["evaluation"]["policy"]["llm_decision"], false);

    let persisted: (String, String, String) = sqlx::query_as(
        "SELECT principal_type, agent_key_id, agent_display_name FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("persisted agent identity");
    assert_eq!(persisted.0, "agent");
    assert_eq!(persisted.1, key_id);
    assert_eq!(persisted.2, "codex-agent-enabled");

    let used_audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.used'")
            .fetch_one(&pool)
            .await
            .expect("used audit count");
    assert_eq!(used_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_disabled_tenant_does_not_persist_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-disabled").await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-disabled-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(
        &app,
        &admin_key,
        "codex-agent-disabled",
        vec!["commit", "push", "open_pr", "merge_pr", "deploy"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected disabled response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("disabled JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_allowed_actions_revoke_and_tenant_scope_are_enforced() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-scope").await;
    let other_org_id = insert_test_org(&pool, "agent-key-other").await;
    enable_agent_governance(&pool, &org_id).await;
    enable_agent_governance(&pool, &other_org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-scope-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(&app, &admin_key, "codex-agent-scope", vec!["commit"]).await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    let (status, response) = json_request(
        &app,
        "GET",
        "/agent-governance/agent-keys",
        None,
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key should not list agent keys: {response}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&response).expect("invalid scope response JSON");
    assert_eq!(parsed["code"], "invalid_scope");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("change_policy").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "change_policy should be blocked by allowed_actions: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("action denied JSON");
    assert_eq!(parsed["code"], "action_not_allowed");

    let mut cross_org = agent_payload("commit");
    cross_org["org_name"] = json!("agent-key-other");
    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&cross_org.to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "cross-tenant agent key should be rejected: {response}"
    );

    let revoke_path = format!("/agent-governance/agent-keys/{key_id}");
    let (status, response) =
        json_request(&app, "DELETE", &revoke_path, None, Some(&admin_key)).await;
    assert_eq!(status, StatusCode::OK, "unexpected revoke: {response}");
    let revoked: serde_json::Value = serde_json::from_str(&response).expect("revoked JSON");
    assert_eq!(revoked["key_id"], key_id);
    assert!(revoked["revoked_at"].as_i64().is_some());

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "revoked key should be unauthorized: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("revoked response JSON");
    assert_eq!(parsed["code"], "revoked_key");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id IN ($1::uuid, $2::uuid)",
    )
    .bind(&org_id)
    .bind(&other_org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    let denied_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("denied audit count");
    assert!(denied_count >= 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn agent_key_expiration_is_enforced_before_evaluation_persistence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-key-expired").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-key-expired-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key(&app, &admin_key, "codex-agent-expired", vec!["commit"]).await;
    let token = created["token"].as_str().expect("agent token");
    let key_id = created["key_id"].as_str().expect("agent key id");

    sqlx::query(
        "UPDATE agent_governance_agent_keys SET expires_at = NOW() - INTERVAL '1 minute' WHERE key_id = $1",
    )
    .bind(key_id)
    .execute(&pool)
    .await
    .expect("expire agent key");

    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&agent_payload("commit").to_string()),
        Some(token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "expired key should be unauthorized: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("expired response JSON");
    assert_eq!(parsed["code"], "expired_key");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation count");
    assert_eq!(count, 0);

    let denied_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("denied audit count");
    assert_eq!(denied_count, 1);

    teardown(&admin_pool, &schema).await;
}
