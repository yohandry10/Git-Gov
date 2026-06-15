use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "ffffffffffffffffffffffffffffffffffffffff";
const RELEASE_ID: &str = "KAN-105";
const ENVIRONMENT: &str = "production";
const TICKET_ID: &str = "KAN-105";
const BASELINE_FRAMEWORK_ID: &str = "gitgov_release_governance_baseline_v1";

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

async fn seed_framework_report_gate(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let repo_id = insert_repo(pool, org_id).await;
    let authorization_id = format!("dga_kan105_{suffix}");

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
            '{"repo_full_name":"yohandry10/Git-Gov","token":"must-not-report"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(format!("kan105-client-event-{suffix}"))
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
            41000,
            'github-actions',
            '{"authorization":"must-not-report"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(format!("pipe-kan105-{suffix}"))
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .bind(REPO_FULL_NAME)
    .execute(pool)
    .await
    .expect("insert pipeline event");

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
            'sha256:kan105',
            '/evidence/packets/tickets/KAN-105',
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
    .bind(format!("approval-kan105-{suffix}"))
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
            'sha256:kan105',
            '/evidence/packets/tickets/KAN-105',
            'advisory',
            TRUE,
            FALSE,
            FALSE,
            'KAN-105 framework report fixture',
            '["sonar_quality_gate"]'::jsonb,
            '["quality gate evidence not present in KAN-99 export"]'::jsonb,
            'policy-kan105',
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
                "policy":{"source":"repo-file","checksum":"policy-kan105"}
            }'::jsonb,
            '{"metadata":{"token":"must-not-report","source_code":"must-not-report"}}'::jsonb,
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
        parsed["export"]["export_id"].as_str().unwrap().to_string(),
        parsed["export"]["artifact_hash"]
            .as_str()
            .unwrap()
            .to_string(),
    )
}

async fn create_mapping(
    app: &axum::Router,
    api_key: &str,
    export_id: &str,
    framework_id: &str,
    framework_version: Option<&str>,
) -> String {
    let body = json!({
        "evidence_export_id": export_id,
        "framework_id": framework_id,
        "framework_version": framework_version
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
    parsed["mapping"]["mapping_id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_review_package(app: &axum::Router, api_key: &str, mapping_id: &str) -> String {
    let body = json!({
        "mapping_id": mapping_id,
        "format": "json"
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/compliance/review-packages",
        Some(&body.to_string()),
        Some(api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "review package failed: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("package JSON");
    parsed["review_package"]["review_package_id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_framework_report(
    app: &axum::Router,
    api_key: &str,
    mapping_id: &str,
    package_id: &str,
) -> serde_json::Value {
    let body = json!({
        "mapping_id": mapping_id,
        "review_package_id": package_id,
        "format": "json"
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/compliance/framework-review-reports",
        Some(&body.to_string()),
        Some(api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "framework report failed: {response}"
    );
    assert!(
        !response.contains("must-not-report"),
        "framework report response leaked secret-like fixture payload"
    );
    serde_json::from_str(&response).expect("framework report JSON")
}

fn customer_pack() -> serde_json::Value {
    json!({
        "schema_version": "gitgov_customer_framework_pack.v1",
        "framework": {
            "id": "bank_framework_report_controls",
            "name": "Bank Framework Report Controls",
            "version": "2026.06",
            "description": "Customer-owned controls for framework report validation.",
            "owner_name": "Customer Audit Office",
            "compliance_claim": false,
            "regulatory_claim": false,
            "gitgov_certifies": false,
            "official_regulatory_mapping": false
        },
        "controls": [
            {
                "control_id": "BFR-DEPLOY-01",
                "title": "Deployment decision captured",
                "description": "Deployment authorization evidence must include the gate decision.",
                "required_evidence_types": ["deployment_gate.decision"]
            },
            {
                "control_id": "BFR-QUALITY-02",
                "title": "Quality gap captured",
                "description": "Release review must call out quality evidence or gaps.",
                "required_evidence_types": ["quality_gate_result"]
            }
        ]
    })
}

async fn review_framework_pack(
    app: &axum::Router,
    api_key: &str,
    framework_pack_id: &str,
    review_status: &str,
) -> serde_json::Value {
    let body = json!({
        "review_status": review_status,
        "review_notes_safe": format!("KAN-105 review status {review_status}"),
        "rejected_reason_safe": if review_status == "rejected" {
            Some("KAN-105 rejected after report package".to_string())
        } else {
            None
        }
    });
    let (status, response) = json_request(
        app,
        "PATCH",
        &format!("/compliance/framework-packs/{framework_pack_id}/review"),
        Some(&body.to_string()),
        Some(api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "framework pack review failed: {response}"
    );
    serde_json::from_str(&response).expect("framework pack review JSON")
}

fn canonical_json_hash(value: &serde_json::Value) -> String {
    let content = serde_json::to_string(value).expect("canonical JSON string");
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

#[tokio::test]
async fn framework_review_report_exports_baseline_mapping_with_source_hashes_and_no_claims() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan105-baseline-report").await;
    let other_org_id = insert_test_org(&pool, "kan106-report-history-other").await;
    let gate_id = seed_framework_report_gate(&pool, &org_id, "baseline").await;
    let admin = insert_test_api_key_for_org(&pool, "kan105-report-admin", "Admin", &org_id).await;
    let auditor =
        insert_test_api_key_for_org(&pool, "kan108-report-auditor", "Auditor", &org_id).await;
    let second_auditor = insert_test_api_key_for_org(
        &pool,
        "kan109-report-unassigned-auditor",
        "Auditor",
        &org_id,
    )
    .await;
    let developer =
        insert_test_api_key_for_org(&pool, "kan107-report-dev", "Developer", &org_id).await;
    let other_auditor = insert_test_api_key_for_org(
        &pool,
        "kan108-report-history-other-auditor",
        "Auditor",
        &other_org_id,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let before_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations before");

    let (export_id, export_hash) = create_export(&app, &admin, &gate_id).await;
    let mapping_id = create_mapping(&app, &admin, &export_id, BASELINE_FRAMEWORK_ID, None).await;
    let package_id = create_review_package(&app, &admin, &mapping_id).await;
    let report = create_framework_report(&app, &admin, &mapping_id, &package_id).await;

    let record = &report["report"];
    let report_id = record["report_id"].as_str().expect("report id");
    let artifact_hash = record["artifact_hash"].as_str().expect("artifact hash");
    assert!(report_id.starts_with("frr_"));
    assert_eq!(record["mapping_id"], mapping_id);
    assert_eq!(record["review_package_id"], package_id);
    assert_eq!(record["evidence_export_id"], export_id);
    assert_eq!(record["evidence_export_hash"], export_hash);
    assert_eq!(record["framework_id"], BASELINE_FRAMEWORK_ID);
    assert_eq!(record["framework_owner_type"], "gitgov");
    assert_eq!(record["compliance_claim"], false);
    assert_eq!(record["regulatory_claim"], false);
    assert_eq!(record["certification"], false);
    assert_eq!(record["requires_auditor_review"], true);
    assert_eq!(record["review_status"], "needs_review");

    let artifact = &report["artifact"];
    assert_eq!(
        artifact["schema_version"],
        "gitgov_framework_review_report.v1"
    );
    assert_eq!(artifact["framework"]["owner_type"], "gitgov");
    assert_eq!(artifact["framework"]["is_regulatory"], false);
    assert_eq!(
        artifact["source_hashes"]["evidence_export_hash"],
        export_hash
    );
    assert_eq!(artifact["source_hashes"]["mapping_id"], mapping_id);
    assert_eq!(artifact["source_hashes"]["review_package_id"], package_id);
    assert_eq!(artifact["claims"]["compliance_claim"], false);
    assert_eq!(artifact["claims"]["regulatory_claim"], false);
    assert_eq!(artifact["claims"]["certification"], false);
    assert_eq!(artifact["claims"]["requires_auditor_review"], true);
    assert_eq!(artifact["summary"]["total_controls"], 10);
    assert_eq!(artifact["controls"].as_array().expect("controls").len(), 10);
    assert!(artifact["missing_evidence"]
        .as_array()
        .expect("missing evidence")
        .iter()
        .any(|value| value == "sonar_quality_gate"));

    let (status, auditor_export_metadata) = json_request(
        &app,
        "GET",
        &format!("/compliance/evidence-exports/{export_id}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should read export metadata: {auditor_export_metadata}"
    );
    let (status, auditor_export_download) = json_request(
        &app,
        "GET",
        &format!("/compliance/evidence-exports/{export_id}/download"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should download export artifact: {auditor_export_download}"
    );
    assert!(
        !auditor_export_download.contains("must-not-report"),
        "auditor export download leaked secret-like fixture payload"
    );

    let (status, auditor_mapping) = json_request(
        &app,
        "GET",
        &format!("/compliance/evidence-mappings/{mapping_id}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should read evidence mapping: {auditor_mapping}"
    );

    let (status, auditor_package_metadata) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{package_id}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should read review package metadata: {auditor_package_metadata}"
    );
    let (status, auditor_package_download) = json_request(
        &app,
        "GET",
        &format!("/compliance/review-packages/{package_id}/download"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should download review package: {auditor_package_download}"
    );

    let (status, auditor_frameworks) = json_request(
        &app,
        "GET",
        "/compliance/control-frameworks",
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should list control frameworks: {auditor_frameworks}"
    );
    let (status, auditor_framework) = json_request(
        &app,
        "GET",
        &format!("/compliance/control-frameworks/{BASELINE_FRAMEWORK_ID}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor should read control framework: {auditor_framework}"
    );

    let forbidden_export_body = json!({
        "scope": "deployment_gate",
        "deployment_gate_id": gate_id,
        "format": "json"
    });
    let (status, auditor_export_create) = json_request(
        &app,
        "POST",
        "/compliance/evidence-exports",
        Some(&forbidden_export_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_export_create.contains("Admin access required"));

    let forbidden_mapping_body = json!({
        "evidence_export_id": export_id,
        "framework_id": BASELINE_FRAMEWORK_ID
    });
    let (status, auditor_mapping_create) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&forbidden_mapping_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_mapping_create.contains("Admin access required"));

    let forbidden_package_body = json!({ "mapping_id": mapping_id, "format": "json" });
    let (status, auditor_package_create) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&forbidden_package_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_package_create.contains("Admin access required"));

    let forbidden_report_body = json!({
        "mapping_id": mapping_id,
        "review_package_id": package_id,
        "format": "json"
    });
    let (status, auditor_report_create) = json_request(
        &app,
        "POST",
        "/compliance/framework-review-reports",
        Some(&forbidden_report_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_report_create.contains("Admin access required"));

    let forbidden_pack_import_body = json!({
        "framework_id": "customer_kan108_forbidden",
        "name": "KAN-108 Forbidden",
        "version": "1",
        "format": "json",
        "content": "{\"controls\":[]}"
    });
    let (status, auditor_pack_import) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&forbidden_pack_import_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_pack_import.contains("Admin access required"));

    let forbidden_gate_body = json!({
        "release_id": "KAN-108",
        "repository_full_name": "yohandry10/Git-Gov",
        "branch": "main",
        "target_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "environment": "production",
        "deployer": "auditor",
        "ticket_id": "KAN-108",
        "evidence_packet_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    let (status, auditor_gate_authorize) = json_request(
        &app,
        "POST",
        "/deployment-gates/authorize",
        Some(&forbidden_gate_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_gate_authorize.contains("Admin access required"));

    let (status, auditor_agent_settings) = json_request(
        &app,
        "GET",
        "/agent-governance/settings",
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_agent_settings.contains("Admin access required"));

    let forbidden_key_body = json!({
        "client_id": "kan108-auditor-should-not-create-keys",
        "role": "Auditor"
    });
    let (status, auditor_api_key_create) = json_request(
        &app,
        "POST",
        "/api-keys",
        Some(&forbidden_key_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_api_key_create.contains("Admin access required"));

    let (status, metadata) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get report failed: {metadata}");
    let metadata: serde_json::Value = serde_json::from_str(&metadata).expect("report metadata");
    assert!(metadata.get("artifact").is_none());
    assert_eq!(metadata["report"]["artifact_hash"], artifact_hash);
    assert_eq!(metadata["report"]["review_status"], "needs_review");

    let invalid_assignment_body = json!({
        "auditor_client_ids": ["kan107-report-dev"],
        "assignment_notes_safe": "Developer cannot be assigned as Auditor"
    });
    let (status, invalid_assignment_response) = json_request(
        &app,
        "PUT",
        &format!("/compliance/framework-review-reports/{report_id}/assignments"),
        Some(&invalid_assignment_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "invalid assignment response: {invalid_assignment_response}"
    );
    assert!(invalid_assignment_response.contains("active tenant Auditor"));

    let secret_assignment_body = json!({
        "auditor_client_ids": ["kan108-report-auditor"],
        "assignment_notes_safe": "bearer ghp_should_not_be_here"
    });
    let (status, secret_assignment_response) = json_request(
        &app,
        "PUT",
        &format!("/compliance/framework-review-reports/{report_id}/assignments"),
        Some(&secret_assignment_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(secret_assignment_response.contains("plain text"));

    let assignment_body = json!({
        "auditor_client_ids": ["kan108-report-auditor"],
        "assignment_notes_safe": "KAN-109 assigned to primary Auditor for manual review"
    });
    let (status, assignments_response) = json_request(
        &app,
        "PUT",
        &format!("/compliance/framework-review-reports/{report_id}/assignments"),
        Some(&assignment_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "assignment update failed: {assignments_response}"
    );
    let assignments_response: serde_json::Value =
        serde_json::from_str(&assignments_response).expect("assignments JSON");
    assert_eq!(assignments_response["count"], 1);
    assert_eq!(
        assignments_response["assignments"][0]["auditor_client_id"],
        "kan108-report-auditor"
    );
    assert_eq!(
        assignments_response["assignments"][0]["assignment_status"],
        "active"
    );
    assert_eq!(
        assignments_response["assignments"][0]["assignment_notes_safe"],
        "KAN-109 assigned to primary Auditor for manual review"
    );

    let (status, auditor_assignments) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}/assignments"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "assigned Auditor should read assignments: {auditor_assignments}"
    );
    let auditor_assignments: serde_json::Value =
        serde_json::from_str(&auditor_assignments).expect("auditor assignments JSON");
    assert_eq!(auditor_assignments["count"], 1);

    let (status, unassigned_assignments) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}/assignments"),
        None,
        Some(&second_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(unassigned_assignments.contains("auditor_not_assigned"));

    let (status, auditor_assigned_list) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/framework-review-reports/assigned-to-me?framework_id={BASELINE_FRAMEWORK_ID}&mapping_id={mapping_id}&review_package_id={package_id}&limit=500"
        ),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "assigned report list failed: {auditor_assigned_list}"
    );
    let auditor_assigned_list: serde_json::Value =
        serde_json::from_str(&auditor_assigned_list).expect("assigned report list JSON");
    assert_eq!(auditor_assigned_list["count"], 1);
    assert_eq!(auditor_assigned_list["items"][0]["report_id"], report_id);
    assert!(auditor_assigned_list["items"][0].get("artifact").is_none());
    assert!(auditor_assigned_list["items"][0]
        .get("payload_json_redacted")
        .is_none());

    let (status, second_assigned_list) = json_request(
        &app,
        "GET",
        "/compliance/framework-review-reports/assigned-to-me",
        None,
        Some(&second_auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unassigned Auditor list failed: {second_assigned_list}"
    );
    let second_assigned_list: serde_json::Value =
        serde_json::from_str(&second_assigned_list).expect("second assigned report list JSON");
    assert_eq!(second_assigned_list["count"], 0);

    let (status, other_assigned_list) = json_request(
        &app,
        "GET",
        "/compliance/framework-review-reports/assigned-to-me",
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "other tenant assigned list failed: {other_assigned_list}"
    );
    let other_assigned_list: serde_json::Value =
        serde_json::from_str(&other_assigned_list).expect("other assigned report list JSON");
    assert_eq!(other_assigned_list["count"], 0);

    let unassigned_comment_body = json!({
        "comment_body_safe": "Unassigned same-tenant Auditor should not comment."
    });
    let (status, unassigned_comment) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/comments"),
        Some(&unassigned_comment_body.to_string()),
        Some(&second_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(unassigned_comment.contains("auditor_not_assigned"));

    let comment_body = json!({
        "comment_body_safe": "Evidence owner sign-off is missing; deployment gate evidence is present.",
        "review_status_suggestion": "needs_changes"
    });
    let (status, comment_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/comments"),
        Some(&comment_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "assigned Auditor comment failed: {comment_response}"
    );
    let comment_response: serde_json::Value =
        serde_json::from_str(&comment_response).expect("comment JSON");
    assert_eq!(
        comment_response["commenter_client_id"],
        "kan108-report-auditor"
    );
    assert_eq!(
        comment_response["review_status_suggestion"],
        "needs_changes"
    );

    let secret_comment_body = json!({
        "comment_body_safe": "please check bearer ghp_should_not_be_here"
    });
    let (status, secret_comment_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/comments"),
        Some(&secret_comment_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(secret_comment_response.contains("plain text"));

    let (status, comments_response) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}/comments"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "assigned Auditor should list comments: {comments_response}"
    );
    let comments_response: serde_json::Value =
        serde_json::from_str(&comments_response).expect("comments JSON");
    assert_eq!(comments_response["count"], 1);
    assert_eq!(
        comments_response["comments"][0]["comment_body_safe"],
        "Evidence owner sign-off is missing; deployment gate evidence is present."
    );

    let (status, unassigned_comments) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}/comments"),
        None,
        Some(&second_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(unassigned_comments.contains("auditor_not_assigned"));

    let unassigned_review_body = json!({
        "review_status": "reviewed",
        "review_notes_safe": "Unassigned Auditor cannot review."
    });
    let (status, unassigned_review) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&unassigned_review_body.to_string()),
        Some(&second_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(unassigned_review.contains("auditor_not_assigned"));

    let review_body = json!({
        "review_status": " needs_changes ",
        "review_notes_safe": "Auditor needs evidence owner sign-off before this can be accepted."
    });
    let (status, reviewed_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&review_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "review report failed: {reviewed_response}"
    );
    let reviewed_response: serde_json::Value =
        serde_json::from_str(&reviewed_response).expect("review response JSON");
    let reviewed = &reviewed_response["report"];
    assert_eq!(reviewed["review_status"], "needs_changes");
    assert_eq!(reviewed["reviewed_by_user_id"], "kan108-report-auditor");
    assert!(reviewed["reviewed_at"].as_i64().unwrap_or_default() > 0);
    assert_eq!(
        reviewed["review_notes_safe"],
        "Auditor needs evidence owner sign-off before this can be accepted."
    );
    assert_eq!(reviewed["artifact_hash"], artifact_hash);
    assert_eq!(reviewed["compliance_claim"], false);
    assert_eq!(reviewed["regulatory_claim"], false);
    assert_eq!(reviewed["certification"], false);
    assert_eq!(reviewed["requires_auditor_review"], true);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_framework_review_report.reviewed'",
    )
    .bind(report_id)
    .fetch_one(&pool)
    .await
    .expect("count review audit rows");
    assert_eq!(audit_count, 1);

    let assignment_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_framework_review_report.assignments_updated'",
    )
    .bind(report_id)
    .fetch_one(&pool)
    .await
    .expect("count assignment audit rows");
    assert_eq!(assignment_audit_count, 1);

    let comment_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_framework_review_report.comment_created'",
    )
    .bind(report_id)
    .fetch_one(&pool)
    .await
    .expect("count comment audit rows");
    assert_eq!(comment_audit_count, 1);

    let manifest_body = json!({});
    let (status, manifest_before_review) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&manifest_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(manifest_before_review.contains("report_not_reviewed"));

    let invalid_status = json!({
        "review_status": "approved",
        "review_notes_safe": "valid plain text"
    });
    let (status, invalid_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&invalid_status.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(invalid_response.contains("review_status"));

    let missing_notes = json!({ "review_status": "rejected" });
    let (status, missing_notes_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&missing_notes.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(missing_notes_response.contains("review_notes_safe"));

    let secret_notes = json!({
        "review_status": "needs_changes",
        "review_notes_safe": "bearer ghp_should_not_be_here"
    });
    let (status, secret_notes_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&secret_notes.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(secret_notes_response.contains("plain text"));

    let reviewed_body = json!({ "review_status": "reviewed" });
    let (status, developer_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&reviewed_body.to_string()),
        Some(&developer),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(developer_response.contains("Admin or Auditor compliance review access required"));

    let (status, other_tenant_review) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&reviewed_body.to_string()),
        Some(&other_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(other_tenant_review.contains("not found"));

    let (status, final_review_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&reviewed_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "final reviewed status failed: {final_review_response}"
    );
    let final_review_response: serde_json::Value =
        serde_json::from_str(&final_review_response).expect("final review JSON");
    assert_eq!(final_review_response["report"]["review_status"], "reviewed");
    assert_eq!(
        final_review_response["report"]["artifact_hash"],
        artifact_hash
    );

    let (status, unassigned_manifest) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&manifest_body.to_string()),
        Some(&second_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(unassigned_manifest.contains("auditor_not_assigned"));

    let (status, developer_manifest) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&manifest_body.to_string()),
        Some(&developer),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(developer_manifest.contains("Admin or Auditor compliance review access required"));

    let (status, first_manifest_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&manifest_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "first provenance manifest failed: {first_manifest_response}"
    );
    assert!(
        !first_manifest_response.contains("must-not-report"),
        "provenance manifest leaked secret-like fixture payload"
    );
    let first_manifest: serde_json::Value =
        serde_json::from_str(&first_manifest_response).expect("first manifest JSON");
    let first_manifest_hash = first_manifest["manifest"]["manifest_hash"]
        .as_str()
        .expect("first manifest hash");
    assert!(first_manifest["manifest"]["manifest_id"]
        .as_str()
        .expect("manifest id")
        .starts_with("frrm_"));
    assert_eq!(
        first_manifest["manifest"]["signature_algorithm"],
        "sha256-provenance-manifest-v1"
    );
    assert_eq!(
        first_manifest["manifest"]["previous_manifest_hash"],
        serde_json::Value::Null
    );
    assert_eq!(
        first_manifest["artifact"]["schema_version"],
        "gitgov_framework_review_report_provenance_manifest.v1"
    );
    assert_eq!(
        first_manifest["artifact"]["source_hashes"]["report_artifact_hash"],
        artifact_hash
    );
    assert_eq!(
        first_manifest["artifact"]["report"]["review_status"],
        "reviewed"
    );
    assert_eq!(
        first_manifest["artifact"]["hash_chain"]["manifest_hash"],
        first_manifest_hash
    );
    assert_eq!(
        first_manifest["artifact"]["hash_chain"]["previous_manifest_hash"],
        serde_json::Value::Null
    );
    assert_eq!(
        first_manifest["artifact"]["claims"]["compliance_claim"],
        false
    );
    assert_eq!(
        first_manifest["artifact"]["claims"]["regulatory_claim"],
        false
    );
    assert_eq!(first_manifest["artifact"]["claims"]["certification"], false);
    assert_eq!(
        first_manifest["artifact"]["audit_metadata"]["source_report_artifact_mutated"],
        false
    );
    assert_eq!(
        first_manifest["artifact"]["audit_metadata"]["agent_governance_required"],
        false
    );
    assert_eq!(
        first_manifest["artifact"]["collaboration"]["active_assignment_count"],
        1
    );
    assert_eq!(
        first_manifest["artifact"]["collaboration"]["comment_count"],
        1
    );
    let first_manifest_id = first_manifest["manifest"]["manifest_id"]
        .as_str()
        .expect("first manifest id");
    let (status, first_manifest_download) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/framework-review-reports/{report_id}/provenance-manifests/{first_manifest_id}"
        ),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "manifest download failed: {first_manifest_download}"
    );
    let first_manifest_download: serde_json::Value =
        serde_json::from_str(&first_manifest_download).expect("manifest download JSON");
    assert_eq!(
        first_manifest_download["hash_chain"]["manifest_hash"],
        first_manifest_hash
    );

    let (status, second_manifest_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&manifest_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "second provenance manifest failed: {second_manifest_response}"
    );
    let second_manifest: serde_json::Value =
        serde_json::from_str(&second_manifest_response).expect("second manifest JSON");
    assert_eq!(
        second_manifest["manifest"]["previous_manifest_hash"],
        first_manifest_hash
    );
    assert_eq!(
        second_manifest["artifact"]["hash_chain"]["previous_manifest_hash"],
        first_manifest_hash
    );
    assert_ne!(
        second_manifest["manifest"]["manifest_hash"],
        first_manifest["manifest"]["manifest_hash"]
    );

    let manifest_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_framework_review_report.provenance_manifest_created'",
    )
    .bind(report_id)
    .fetch_one(&pool)
    .await
    .expect("count provenance manifest audit rows");
    assert_eq!(manifest_audit_count, 2);

    let (status, list) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/framework-review-reports?framework_id={BASELINE_FRAMEWORK_ID}&mapping_id={mapping_id}&review_package_id={package_id}&limit=500"
        ),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list report failed: {list}");
    let list: serde_json::Value = serde_json::from_str(&list).expect("report list JSON");
    assert_eq!(list["count"], 1);
    assert_eq!(list["limit"], 100);
    assert_eq!(list["items"][0]["report_id"], report_id);
    assert_eq!(list["items"][0]["framework_id"], BASELINE_FRAMEWORK_ID);
    assert_eq!(list["items"][0]["review_status"], "reviewed");
    assert!(list["items"][0].get("artifact").is_none());
    assert!(list["items"][0].get("payload_json_redacted").is_none());

    let (status, other_list) = json_request(
        &app,
        "GET",
        "/compliance/framework-review-reports",
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "other tenant list failed: {other_list}"
    );
    let other_list: serde_json::Value =
        serde_json::from_str(&other_list).expect("other report list JSON");
    assert_eq!(other_list["count"], 0);

    let (status, invalid_query) = json_request(
        &app,
        "GET",
        "/compliance/framework-review-reports?mapping_id=bad",
        None,
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(invalid_query.contains("mapping_id"));

    let (status, download) = json_request(
        &app,
        "GET",
        &format!("/compliance/framework-review-reports/{report_id}/download"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "download failed: {download}");
    assert!(
        !download.contains("must-not-report"),
        "downloaded framework report leaked secret-like fixture payload"
    );
    let downloaded: serde_json::Value =
        serde_json::from_str(&download).expect("downloaded report JSON");
    assert_eq!(canonical_json_hash(&downloaded), artifact_hash);

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after");
    assert_eq!(
        after_evaluations, before_evaluations,
        "framework reports must not create Agent Governance evaluations"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn framework_review_report_requires_customer_pack_to_remain_reviewed() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan105-customer-report").await;
    let gate_id = seed_framework_report_gate(&pool, &org_id, "customer").await;
    let admin = insert_test_api_key_for_org(&pool, "kan105-customer-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let import_body = json!({
        "format": "json",
        "pack": customer_pack()
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&import_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "import failed: {response}");
    let imported: serde_json::Value = serde_json::from_str(&response).expect("import JSON");
    let framework_id = imported["framework"]["framework_id"].as_str().unwrap();
    let framework_pack_id = imported["framework_pack"]["framework_pack_id"]
        .as_str()
        .unwrap();
    let pack_hash = imported["framework_pack"]["pack_hash"].as_str().unwrap();

    review_framework_pack(&app, &admin, framework_pack_id, "reviewed").await;
    let (export_id, _) = create_export(&app, &admin, &gate_id).await;
    let mapping_id = create_mapping(&app, &admin, &export_id, framework_id, Some("2026.06")).await;
    let package_id = create_review_package(&app, &admin, &mapping_id).await;
    let report = create_framework_report(&app, &admin, &mapping_id, &package_id).await;

    assert_eq!(report["artifact"]["framework"]["owner_type"], "customer");
    assert_eq!(
        report["artifact"]["framework"]["source"],
        "customer_provided"
    );
    assert_eq!(report["artifact"]["framework"]["pack_hash"], pack_hash);
    assert_eq!(report["artifact"]["framework"]["review_status"], "reviewed");
    assert_eq!(report["artifact"]["claims"]["compliance_claim"], false);
    assert_eq!(report["artifact"]["claims"]["regulatory_claim"], false);
    assert_eq!(report["artifact"]["claims"]["certification"], false);

    review_framework_pack(&app, &admin, framework_pack_id, "rejected").await;
    let body = json!({
        "mapping_id": mapping_id,
        "review_package_id": package_id,
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-review-reports",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "rejected customer pack must block new reports: {response}"
    );
    assert!(response.contains("framework_pack_rejected"));

    teardown(&admin_pool, &schema).await;
}
