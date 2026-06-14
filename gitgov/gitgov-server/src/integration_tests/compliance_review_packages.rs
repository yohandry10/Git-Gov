use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "dddddddddddddddddddddddddddddddddddddddd";
const RELEASE_ID: &str = "KAN-101";
const ENVIRONMENT: &str = "production";
const TICKET_ID: &str = "KAN-101";
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

async fn seed_review_package_gate(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let repo_id = insert_repo(pool, org_id).await;
    let authorization_id = format!("dga_kan101_{suffix}");

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
            '{"repo_full_name":"yohandry10/Git-Gov","token":"must-not-package"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(format!("kan101-client-event-{suffix}"))
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
            44000,
            'github-actions',
            '{"authorization":"must-not-package"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(format!("pipe-kan101-{suffix}"))
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
            'Control Mapping Review Package',
            'Done',
            '{"api_key":"must-not-package"}'::jsonb
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
            'sha256:kan101',
            '/evidence/packets/tickets/KAN-101',
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
    .bind(format!("approval-kan101-{suffix}"))
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
            'sha256:kan101',
            '/evidence/packets/tickets/KAN-101',
            'advisory',
            TRUE,
            FALSE,
            FALSE,
            'KAN-101 review package fixture',
            '["sonar_quality_gate"]'::jsonb,
            '["PR review evidence not present in KAN-99 export"]'::jsonb,
            'policy-kan101',
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
                "policy":{"source":"repo-file","checksum":"policy-kan101"}
            }'::jsonb,
            '{"metadata":{"token":"must-not-package","source_code":"must-not-package"}}'::jsonb,
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

async fn create_mapping(
    app: &axum::Router,
    api_key: &str,
    export_id: &str,
) -> (String, serde_json::Value) {
    let body = json!({
        "evidence_export_id": export_id,
        "framework_id": FRAMEWORK_ID
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&body.to_string()),
        Some(api_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "mapping failed: {response}");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("mapping JSON");
    (
        parsed["mapping"]["mapping_id"]
            .as_str()
            .expect("mapping id")
            .to_string(),
        parsed,
    )
}

fn canonical_json_hash(value: &serde_json::Value) -> String {
    let content = serde_json::to_string(value).expect("canonical JSON string");
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

#[tokio::test]
async fn review_package_downloads_hashable_no_claim_artifact_from_real_mapping() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan101-review-package").await;
    let gate_id = seed_review_package_gate(&pool, &org_id, "primary").await;
    let api_key =
        insert_test_api_key_for_org(&pool, "kan101-package-admin", "Admin", &org_id).await;
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
    let (mapping_id, mapping_response) = create_mapping(&app, &api_key, &export_id).await;
    assert_eq!(
        mapping_response["mapping"]["evidence_export_hash"],
        export_hash
    );

    let body = json!({
        "mapping_id": mapping_id,
        "format": "json",
        "include_sections": [
            "summary",
            "source_hashes",
            "framework",
            "control_matrix",
            "missing_evidence",
            "no_claims",
            "audit_metadata"
        ]
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&body.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "review package failed: {response}"
    );
    assert!(
        !response.contains("must-not-package"),
        "review package response leaked secret-like fixture payload"
    );

    let parsed: serde_json::Value = serde_json::from_str(&response).expect("package JSON");
    let package = &parsed["review_package"];
    let package_id = package["review_package_id"]
        .as_str()
        .expect("review package id");
    let artifact_hash = package["artifact_hash"].as_str().expect("artifact hash");
    let mapping_hash = package["mapping_hash"].as_str().expect("mapping hash");
    assert!(package_id.starts_with("crp_"));
    assert!(mapping_hash.starts_with("sha256:"));
    assert_eq!(package["mapping_id"], mapping_id);
    assert_eq!(package["evidence_export_id"], export_id);
    assert_eq!(package["evidence_export_hash"], export_hash);
    assert_eq!(package["framework_id"], FRAMEWORK_ID);
    assert_eq!(package["framework_version"], "1.0.0");
    assert_eq!(package["format"], "json");
    assert_eq!(package["compliance_claim"], false);
    assert_eq!(package["regulatory_claim"], false);
    assert_eq!(package["requires_auditor_review"], true);
    assert_eq!(package["certification"], false);
    assert_eq!(
        parsed["download_url"],
        format!("/compliance/review-packages/{package_id}/download")
    );

    let artifact = &parsed["artifact"];
    assert_eq!(
        artifact["schema_version"],
        "gitgov_control_review_package.v1"
    );
    assert_eq!(artifact["review_package_id"], package_id);
    assert_eq!(artifact["source"]["evidence_export_id"], export_id);
    assert_eq!(artifact["source"]["evidence_export_hash"], export_hash);
    assert_eq!(artifact["source"]["mapping_id"], mapping_id);
    assert_eq!(artifact["source"]["mapping_hash"], mapping_hash);
    assert_eq!(artifact["claims"]["compliance_claim"], false);
    assert_eq!(artifact["claims"]["regulatory_claim"], false);
    assert_eq!(artifact["claims"]["requires_auditor_review"], true);
    assert_eq!(artifact["claims"]["certification"], false);
    assert_eq!(artifact["framework"]["is_regulatory"], false);
    assert_eq!(artifact["summary"]["total_controls"], 10);
    assert_eq!(
        artifact["summary"]["controls_requiring_customer_or_auditor_review"],
        10
    );
    assert_eq!(artifact["controls"].as_array().expect("controls").len(), 10);
    assert!(artifact["missing_evidence"]
        .as_array()
        .expect("missing evidence")
        .iter()
        .any(|value| value == "sonar_quality_gate"));
    assert_eq!(artifact["audit_metadata"]["artifact_redacted"], true);
    assert_eq!(artifact["audit_metadata"]["raw_payload_included"], false);
    assert_eq!(
        artifact["audit_metadata"]["agent_governance_required"],
        false
    );
    assert_eq!(artifact["audit_metadata"]["llm_decision"], false);
    assert_eq!(artifact["audit_metadata"]["provider_mutation"], false);

    let (status, metadata) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{package_id}"),
        None,
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get package failed: {metadata}");
    let metadata: serde_json::Value = serde_json::from_str(&metadata).expect("package metadata");
    assert!(metadata.get("artifact").is_none());
    assert_eq!(metadata["review_package"]["artifact_hash"], artifact_hash);

    let (status, download) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{package_id}/download"),
        None,
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "download package failed: {download}"
    );
    assert!(
        !download.contains("must-not-package"),
        "downloaded review package leaked secret-like fixture payload"
    );
    let downloaded: serde_json::Value =
        serde_json::from_str(&download).expect("downloaded review package JSON");
    assert_eq!(canonical_json_hash(&downloaded), artifact_hash);
    assert_eq!(downloaded["source"]["evidence_export_hash"], export_hash);
    assert_eq!(downloaded["source"]["mapping_hash"], mapping_hash);
    assert_eq!(downloaded["claims"]["certification"], false);

    let (status, repeat_response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&body.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "repeat package failed: {repeat_response}"
    );
    let repeat: serde_json::Value =
        serde_json::from_str(&repeat_response).expect("repeat package JSON");
    assert_eq!(repeat["review_package"]["review_package_id"], package_id);
    assert_eq!(repeat["review_package"]["artifact_hash"], artifact_hash);

    let db_flags = sqlx::query(
        r#"
        SELECT compliance_claim, regulatory_claim, requires_auditor_review, certification
        FROM compliance_review_packages
        WHERE review_package_id = $1
        "#,
    )
    .bind(package_id)
    .fetch_one(&pool)
    .await
    .expect("load package flags");
    assert!(!db_flags.get::<bool, _>("compliance_claim"));
    assert!(!db_flags.get::<bool, _>("regulatory_claim"));
    assert!(db_flags.get::<bool, _>("requires_auditor_review"));
    assert!(!db_flags.get::<bool, _>("certification"));

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after");
    assert_eq!(
        after_evaluations, before_evaluations,
        "review packages must not create Agent Governance evaluations"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn review_package_enforces_admin_scope_validation_and_tenant_isolation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "kan101-tenant-a").await;
    let org_b = insert_test_org(&pool, "kan101-tenant-b").await;
    let gate_a = seed_review_package_gate(&pool, &org_a, "tenant_a").await;
    let gate_b = seed_review_package_gate(&pool, &org_b, "tenant_b").await;
    let admin_a = insert_test_api_key_for_org(&pool, "kan101-admin-a", "Admin", &org_a).await;
    let admin_b = insert_test_api_key_for_org(&pool, "kan101-admin-b", "Admin", &org_b).await;
    let developer_a =
        insert_test_api_key_for_org(&pool, "kan101-developer-a", "Developer", &org_a).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (export_a, _) = create_export(&app, &admin_a, &gate_a).await;
    let (export_b, _) = create_export(&app, &admin_b, &gate_b).await;
    let (mapping_a, _) = create_mapping(&app, &admin_a, &export_a).await;
    let (mapping_b, _) = create_mapping(&app, &admin_b, &export_b).await;

    let developer_body = json!({
        "mapping_id": mapping_a,
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&developer_body.to_string()),
        Some(&developer_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not create review package: {response}"
    );

    let cross_tenant_body = json!({
        "mapping_id": mapping_b,
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&cross_tenant_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant mapping must not be visible: {response}"
    );

    let own_body = json!({
        "mapping_id": mapping_a,
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&own_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "package failed: {response}");
    let own_package: serde_json::Value = serde_json::from_str(&response).expect("own package JSON");
    let own_package_id = own_package["review_package"]["review_package_id"]
        .as_str()
        .expect("own package id");

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{own_package_id}"),
        None,
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant package metadata must not be visible: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{own_package_id}/download"),
        None,
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant package download must not be visible: {response}"
    );

    let invalid_id = json!({
        "mapping_id": "bad",
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&invalid_id.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("cem_"));

    let invalid_format = json!({
        "mapping_id": mapping_a,
        "format": "pdf"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&invalid_format.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("format must be json"));

    let invalid_section = json!({
        "mapping_id": mapping_a,
        "format": "json",
        "include_sections": ["summary", "auditor_signature"]
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&invalid_section.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("unsupported include section"));

    teardown(&admin_pool, &schema).await;
}
