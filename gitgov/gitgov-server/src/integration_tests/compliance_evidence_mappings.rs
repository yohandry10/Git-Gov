use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "cccccccccccccccccccccccccccccccccccccccc";
const RELEASE_ID: &str = "KAN-100";
const ENVIRONMENT: &str = "production";
const TICKET_ID: &str = "KAN-100";
const FRAMEWORK_ID: &str = "gitgov_release_governance_baseline_v1";

async fn insert_repo(pool: &sqlx::PgPool, org_id: &str) -> String {
    let row = sqlx::query(
        r#"
        INSERT INTO repos (org_id, full_name, name, private)
        VALUES ($1::uuid, $2, 'Git-Gov', FALSE)
        ON CONFLICT (full_name) DO UPDATE SET org_id = EXCLUDED.org_id
        RETURNING id::text
        "#,
    )
    .bind(org_id)
    .bind(REPO_FULL_NAME)
    .fetch_one(pool)
    .await
    .expect("insert repo");
    row.get("id")
}

async fn seed_mapping_gate(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let repo_id = insert_repo(pool, org_id).await;
    let authorization_id = format!("dga_kan100_{suffix}");

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
            $3,
            'commit',
            'audited-engineer',
            $4,
            $5,
            'accepted',
            '{"repo_full_name":"yohandry10/Git-Gov","token":"must-not-map"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(format!("kan100-client-event-{suffix}"))
    .bind(BRANCH)
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
            triggered_by,
            payload
        )
        VALUES (
            $1::uuid,
            $2,
            'ci',
            'success',
            $3,
            $4,
            $5,
            42000,
            'github-actions',
            '{"authorization":"must-not-map"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(format!("pipe-kan100-{suffix}"))
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .bind(REPO_FULL_NAME)
    .execute(pool)
    .await
    .expect("insert pipeline event");

    sqlx::query(
        r#"
        INSERT INTO project_tickets (
            org_id,
            ticket_id,
            project_key,
            title,
            status,
            raw_payload
        )
        VALUES (
            $1::uuid,
            $2,
            'KAN',
            'Evidence-to-Control Mapping MVP',
            'Done',
            '{"api_key":"must-not-map"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(TICKET_ID)
    .execute(pool)
    .await
    .expect("insert project ticket");

    sqlx::query(
        r#"
        INSERT INTO enterprise_release_approvals (
            org_id,
            release_id,
            repository_full_name,
            branch,
            target_sha,
            environment,
            decision,
            approver,
            ticket_id,
            evidence_packet_hash,
            evidence_packet_uri,
            evidence_summary,
            risk_severity,
            approval_hash,
            created_by
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4,
            $5,
            $6,
            'approved',
            'release-manager',
            $7,
            'sha256:kan100',
            '/evidence/packets/tickets/KAN-100',
            '{"summary":"approved"}'::jsonb,
            'low',
            $8,
            'integration-test'
        )
        "#,
    )
    .bind(org_id)
    .bind(RELEASE_ID)
    .bind(REPO_FULL_NAME)
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .bind(ENVIRONMENT)
    .bind(TICKET_ID)
    .bind(format!("approval-kan100-{suffix}"))
    .execute(pool)
    .await
    .expect("insert release approval");

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
            evidence_packet_uri,
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
            $1,
            $2::uuid,
            $3,
            $4,
            $5,
            $6,
            $7,
            'github-actions',
            $8,
            'sha256:kan100',
            '/evidence/packets/tickets/KAN-100',
            'advisory',
            TRUE,
            FALSE,
            FALSE,
            'KAN-100 mapping fixture',
            '["sonar_quality_gate"]'::jsonb,
            '["PR review evidence not present in KAN-99 export"]'::jsonb,
            'policy-kan100',
            '{
                "status":"incomplete",
                "policy_satisfied":false,
                "blocking":false,
                "would_block":true,
                "valid_approval_count":1,
                "required_approval_count":1,
                "policy":{"mode":"warn","environment":"production","approval_required":true,"enforcement":"advisory","policy_applies":true,"quorum_enabled":false,"quorum_rules":[]},
                "approvals":[],
                "issues":["sonar evidence missing"],
                "next_steps":["collect sonar evidence"]
            }'::jsonb,
            '{
                "shared_governance_decision":{
                    "version":"shared-governance-decision.v1",
                    "consumer_type":"deployment_gate",
                    "decision":"insufficient_evidence",
                    "agent_governance_used":false,
                    "evidence":{"missing_evidence":["sonar_quality_gate"]}
                },
                "policy":{"source":"repo-file","checksum":"policy-kan100"}
            }'::jsonb,
            '{"metadata":{"token":"must-not-map","source_code":"must-not-map"}}'::jsonb,
            'integration-test'
        )
        "#,
    )
    .bind(&authorization_id)
    .bind(org_id)
    .bind(RELEASE_ID)
    .bind(REPO_FULL_NAME)
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .bind(ENVIRONMENT)
    .bind(TICKET_ID)
    .execute(pool)
    .await
    .expect("insert deployment gate");

    authorization_id
}

async fn create_export(app: &axum::Router, api_key: &str, gate_id: &str) -> (String, String) {
    let body = json!({
        "scope": "deployment_gate",
        "deployment_gate_id": gate_id,
        "format": "json",
        "include_sections": [
            "gate_decision",
            "policy",
            "readiness",
            "approvals",
            "evidence",
            "gaps",
            "audit"
        ]
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/compliance/evidence-exports",
        Some(&body.to_string()),
        Some(api_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "export failed: {response}");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("export JSON");
    (
        parsed["export"]["export_id"]
            .as_str()
            .expect("export id")
            .to_string(),
        parsed["export"]["artifact_hash"]
            .as_str()
            .expect("artifact hash")
            .to_string(),
    )
}

fn item_by_control<'a>(items: &'a [serde_json::Value], control_id: &str) -> &'a serde_json::Value {
    items
        .iter()
        .find(|item| item["control_id"] == control_id)
        .unwrap_or_else(|| panic!("missing control {control_id}"))
}

#[tokio::test]
async fn evidence_mapping_generates_non_regulatory_matrix_from_real_export() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan100-evidence-mapping").await;
    let gate_id = seed_mapping_gate(&pool, &org_id, "primary").await;
    let api_key =
        insert_test_api_key_for_org(&pool, "kan100-mapping-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let before_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations before");

    let (export_id, export_hash) = create_export(&app, &api_key, &gate_id).await;
    let body = json!({
        "evidence_export_id": export_id,
        "framework_id": FRAMEWORK_ID
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&body.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "mapping failed: {response}");
    assert!(
        !response.contains("must-not-map"),
        "mapping response leaked secret-like fixture payload"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("mapping JSON");
    let mapping_id = parsed["mapping"]["mapping_id"]
        .as_str()
        .expect("mapping id");
    assert!(mapping_id.starts_with("cem_"));
    assert_eq!(parsed["mapping"]["evidence_export_id"], export_id);
    assert_eq!(parsed["mapping"]["evidence_export_hash"], export_hash);
    assert_eq!(parsed["mapping"]["framework_id"], FRAMEWORK_ID);
    assert_eq!(parsed["mapping"]["framework_version"], "1.0.0");
    assert_eq!(parsed["mapping"]["compliance_claim"], false);
    assert_eq!(parsed["mapping"]["regulatory_claim"], false);
    assert_eq!(parsed["mapping"]["requires_auditor_review"], true);

    let items = parsed["items"].as_array().expect("mapping items");
    assert_eq!(items.len(), 10);
    assert_eq!(
        item_by_control(items, "GG-RG-01")["status"],
        "evidence_present"
    );
    assert_eq!(
        item_by_control(items, "GG-RG-03")["status"],
        "evidence_present"
    );
    assert_eq!(
        item_by_control(items, "GG-RG-04")["status"],
        "evidence_present"
    );
    assert_eq!(item_by_control(items, "GG-RG-05")["status"], "partial");
    assert_eq!(item_by_control(items, "GG-RG-06")["status"], "missing");
    assert!(item_by_control(items, "GG-RG-06")["missing_evidence"]
        .as_array()
        .expect("missing evidence")
        .iter()
        .any(|value| value == "sonar_quality_gate"));
    assert_eq!(
        item_by_control(items, "GG-RG-10")["status"],
        "evidence_present"
    );

    let (status, metadata) = json_request(
        &app,
        "GET",
        &format!("/compliance/evidence-mappings/{mapping_id}"),
        None,
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get mapping failed: {metadata}");
    let metadata: serde_json::Value = serde_json::from_str(&metadata).expect("mapping get JSON");
    assert_eq!(metadata["mapping"]["evidence_export_hash"], export_hash);
    assert_eq!(metadata["items"].as_array().expect("items").len(), 10);

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after");
    assert_eq!(
        after_evaluations, before_evaluations,
        "mapping must not create Agent Governance evaluations"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn evidence_mapping_enforces_admin_tenant_scope_and_framework_limits() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "kan100-tenant-a").await;
    let org_b = insert_test_org(&pool, "kan100-tenant-b").await;
    let gate_a = seed_mapping_gate(&pool, &org_a, "tenant_a").await;
    let gate_b = seed_mapping_gate(&pool, &org_b, "tenant_b").await;
    let admin_a = insert_test_api_key_for_org(&pool, "kan100-admin-a", "Admin", &org_a).await;
    let admin_b = insert_test_api_key_for_org(&pool, "kan100-admin-b", "Admin", &org_b).await;
    let developer_a =
        insert_test_api_key_for_org(&pool, "kan100-developer-a", "Developer", &org_a).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (export_a, _) = create_export(&app, &admin_a, &gate_a).await;
    let (export_b, _) = create_export(&app, &admin_b, &gate_b).await;

    let developer_body = json!({
        "evidence_export_id": export_a,
        "framework_id": FRAMEWORK_ID
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&developer_body.to_string()),
        Some(&developer_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not create mapping: {response}"
    );

    let cross_tenant_body = json!({
        "evidence_export_id": export_b,
        "framework_id": FRAMEWORK_ID
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&cross_tenant_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant export must not be visible: {response}"
    );

    let invalid_framework = json!({
        "evidence_export_id": export_a,
        "framework_id": "soc2_cc_fake"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&invalid_framework.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains(FRAMEWORK_ID));

    let (status, frameworks) = json_request(
        &app,
        "GET",
        "/compliance/control-frameworks",
        None,
        Some(&admin_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "framework list failed: {frameworks}"
    );
    assert!(frameworks.contains(FRAMEWORK_ID));
    assert!(frameworks.contains("\"is_regulatory\":false"));

    let (status, framework) = json_request(
        &app,
        "GET",
        &format!("/compliance/control-frameworks/{FRAMEWORK_ID}"),
        None,
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "framework get failed: {framework}");
    let framework: serde_json::Value = serde_json::from_str(&framework).expect("framework JSON");
    assert_eq!(
        framework["controls"].as_array().expect("controls").len(),
        10
    );

    teardown(&admin_pool, &schema).await;
}
