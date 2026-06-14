use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const TARGET_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

async fn insert_repo_for_org(pool: &sqlx::PgPool, org_id: &str, full_name: &str) -> String {
    let name = full_name
        .rsplit('/')
        .next()
        .expect("repo name segment")
        .to_string();
    sqlx::query_scalar(
        r#"
        INSERT INTO repos (org_id, full_name, name, private)
        VALUES ($1::uuid, $2, $3, FALSE)
        ON CONFLICT (full_name) DO UPDATE SET
            org_id = EXCLUDED.org_id,
            name = EXCLUDED.name,
            updated_at = NOW()
        RETURNING id::text
        "#,
    )
    .bind(org_id)
    .bind(full_name)
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("insert repo")
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
            'read context integration opt-in',
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

async fn create_agent_key_with_scopes(
    app: &axum::Router,
    admin_key: &str,
    display_name: &str,
    scopes: Vec<&str>,
) -> serde_json::Value {
    let body = json!({
        "display_name": display_name,
        "description": "Integration test read context agent key",
        "environment": "staging",
        "scopes": scopes,
        "allowed_actions": ["commit", "push"]
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

async fn seed_read_context_evidence(pool: &sqlx::PgPool, org_id: &str) {
    let repo_id = insert_repo_for_org(pool, org_id, REPO_FULL_NAME).await;
    sqlx::query(
        r#"
        INSERT INTO policies (org_id, repo_id, config, checksum, source_metadata)
        VALUES (
            $1::uuid,
            $2::uuid,
            '{"mode":"warn","required_approvals":1}'::jsonb,
            'policy-kan98',
            '{"source_mode":"repo-file","drift_status":"synced"}'::jsonb
        )
        ON CONFLICT (repo_id) DO UPDATE SET
            config = EXCLUDED.config,
            checksum = EXCLUDED.checksum,
            source_metadata = EXCLUDED.source_metadata,
            updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .execute(pool)
    .await
    .expect("insert policy");

    sqlx::query(
        r#"
        INSERT INTO client_events (
            org_id,
            repo_id,
            event_uuid,
            event_type,
            user_login,
            branch,
            commit_sha,
            status,
            metadata
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'kan98-context-commit',
            'commit',
            'engineer',
            'main',
            $3,
            'accepted',
            '{"repo_full_name":"yohandry10/Git-Gov"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(TARGET_SHA)
    .execute(pool)
    .await
    .expect("insert client event");

    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            org_id,
            pipeline_id,
            job_name,
            status,
            branch,
            commit_sha,
            repo_full_name,
            duration_ms,
            triggered_by
        )
        VALUES (
            $1::uuid,
            'pipe-kan98',
            'ci',
            'success',
            'main',
            $2,
            $3,
            42000,
            'github-actions'
        )
        "#,
    )
    .bind(org_id)
    .bind(TARGET_SHA)
    .bind(REPO_FULL_NAME)
    .execute(pool)
    .await
    .expect("insert pipeline event");

    sqlx::query(
        r#"
        INSERT INTO deployment_gate_authorizations (
            authorization_id,
            org_id,
            release_id,
            repository_full_name,
            branch,
            target_sha,
            environment,
            deployer,
            ticket_id,
            evidence_packet_hash,
            decision,
            approved,
            blocking,
            would_block,
            reason,
            blocked_by,
            warnings,
            policy_checksum,
            evaluation,
            details,
            request_payload,
            requested_by
        )
        VALUES (
            'dga_kan98_context',
            $1::uuid,
            'KAN-98',
            $2,
            'main',
            $3,
            'production',
            'github-actions',
            'KAN-98',
            'sha256:kan98',
            'approved',
            TRUE,
            FALSE,
            FALSE,
            'read context seeded gate approval',
            '[]'::jsonb,
            '[]'::jsonb,
            'policy-kan98',
            '{"readiness_score": 95}'::jsonb,
            '{"shared_governance_decision":{"consumer_type":"deployment_gate","agent_governance_used":false}}'::jsonb,
            '{}'::jsonb,
            'integration-test'
        )
        "#,
    )
    .bind(org_id)
    .bind(REPO_FULL_NAME)
    .bind(TARGET_SHA)
    .execute(pool)
    .await
    .expect("insert deployment gate authorization");
}

#[tokio::test]
async fn read_only_agent_key_can_load_context_without_persisting_evaluation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-read-context").await;
    enable_agent_governance(&pool, &org_id).await;
    seed_read_context_evidence(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-read-context-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key_with_scopes(
        &app,
        &admin_key,
        "kan98-read-agent",
        vec!["agent_governance:read"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let before_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations before context");

    let path = format!(
        "/agent-governance/context?repository_full_name=yohandry10%2FGit-Gov&branch=main&target_sha={TARGET_SHA}&environment=production"
    );
    let (status, response) = json_request(&app, "GET", &path, None, Some(token)).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected read context response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("context JSON");
    assert_eq!(parsed["read_only"], true);
    assert_eq!(parsed["will_authorize_execution"], false);
    assert_eq!(parsed["mcp_surface"], false);
    assert_eq!(parsed["principal"]["principal_type"], "agent");
    assert_eq!(parsed["branch_status"]["protected_branch"], true);
    assert_eq!(parsed["branch_status"]["commit_events_count"], 1);
    assert_eq!(parsed["policy_compliance"]["policy_found"], true);
    assert_eq!(parsed["policy_compliance"]["llm_decision"], false);
    assert_eq!(parsed["pipeline_state"]["latest"]["status"], "success");
    assert_eq!(
        parsed["recent_activity"]["latest_deployment_gate"]["decision"],
        "approved"
    );
    assert_eq!(
        parsed["recent_activity"]["latest_deployment_gate"]["policy_checksum"],
        "policy-kan98"
    );
    assert_eq!(parsed["risk_score"]["level"], "low");

    let after_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after context");
    assert_eq!(
        after_count, before_count,
        "read-only context must not persist evaluations"
    );

    let read_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.used' AND metadata->'metadata'->>'scope' = 'agent_governance:read'",
    )
    .fetch_one(&pool)
    .await
    .expect("count read audit");
    assert_eq!(read_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn evaluate_only_agent_key_cannot_load_read_context() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-read-context-scope").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-read-context-scope-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key_with_scopes(
        &app,
        &admin_key,
        "kan98-evaluate-agent",
        vec!["agent_governance:evaluate"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let path =
        format!("/agent-governance/context?repository_full_name=yohandry10%2FGit-Gov&branch=main");
    let (status, response) = json_request(&app, "GET", &path, None, Some(token)).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("error JSON");
    assert_eq!(parsed["code"], "invalid_scope");

    let invalid_scope_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'agent_key.invalid_scope'",
    )
    .fetch_one(&pool)
    .await
    .expect("count invalid scope audit");
    assert_eq!(invalid_scope_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn read_only_agent_context_respects_manual_only_tenant_boundary() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-read-context-disabled").await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-read-context-disabled-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key_with_scopes(
        &app,
        &admin_key,
        "kan98-disabled-read-agent",
        vec!["agent_governance:read"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let path =
        format!("/agent-governance/context?repository_full_name=yohandry10%2FGit-Gov&branch=main");
    let (status, response) = json_request(&app, "GET", &path, None, Some(token)).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("disabled JSON");
    assert_eq!(parsed["code"], "agent_governance_disabled");
    assert_eq!(parsed["manual_governance_available"], true);
    assert_eq!(parsed["read_only"], true);
    assert_eq!(parsed["will_authorize_execution"], false);

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

#[tokio::test]
async fn read_only_agent_key_cannot_call_evaluate() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "agent-read-context-no-eval").await;
    enable_agent_governance(&pool, &org_id).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "agent-read-context-no-eval-admin", "Admin", &org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let created = create_agent_key_with_scopes(
        &app,
        &admin_key,
        "kan98-read-no-eval-agent",
        vec!["agent_governance:read"],
    )
    .await;
    let token = created["token"].as_str().expect("agent token");

    let body = json!({
        "agent_id": "kan98-read-agent",
        "agent_type": "codex",
        "actor": "engineer@example.com",
        "action": "commit",
        "repository_full_name": REPO_FULL_NAME,
        "branch": "main",
        "target_sha": TARGET_SHA,
        "metadata": {}
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/agent-governance/evaluate",
        Some(&body.to_string()),
        Some(token),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "unexpected response: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("error JSON");
    assert_eq!(parsed["code"], "invalid_scope");

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
