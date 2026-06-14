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

#[tokio::test]
async fn agent_governance_allows_ticketed_commit_and_persists_audit_evidence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-governance-allowed").await;
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
