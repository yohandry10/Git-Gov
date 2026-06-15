use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RELEASE_ID: &str = "KAN-113";
const ENVIRONMENT: &str = "production";
const TICKET_ID: &str = "KAN-113";
const BASELINE_FRAMEWORK_ID: &str = "gitgov_release_governance_baseline_v1";

async fn insert_repo(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let full_name = format!("{REPO_FULL_NAME}-{suffix}");
    let row = sqlx::query(
        r#"
        INSERT INTO repos (org_id, full_name, name, private)
        VALUES ($1::uuid, $2, $3, FALSE)
        ON CONFLICT (full_name) DO UPDATE SET org_id = EXCLUDED.org_id
        RETURNING id::text
        "#,
    )
    .bind(org_id)
    .bind(&full_name)
    .bind(format!("Git-Gov-{suffix}"))
    .fetch_one(pool)
    .await
    .expect("insert repo");
    row.get("id")
}

async fn seed_period_report_gate(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let repo_id = insert_repo(pool, org_id, suffix).await;
    let authorization_id = format!("dga_kan113_{suffix}");
    let target_sha = format!("{}{:0>4}", &TARGET_SHA[..36], suffix.len());

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
            '{"repo_full_name":"period-report-fixture","token":"must-not-report"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(format!("kan113-client-event-{suffix}"))
    .bind(BRANCH)
    .bind(&target_sha)
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
    .bind(format!("pipe-kan113-{suffix}"))
    .bind(BRANCH)
    .bind(&target_sha)
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
            'sha256:kan113',
            '/evidence/packets/tickets/KAN-113',
            '{"summary":"approved"}'::jsonb,
            'low',
            $8,
            'integration-test'
        )
        "#,
    )
    .bind(org_id)
    .bind(format!("{RELEASE_ID}-{suffix}"))
    .bind(REPO_FULL_NAME)
    .bind(BRANCH)
    .bind(&target_sha)
    .bind(ENVIRONMENT)
    .bind(TICKET_ID)
    .bind(format!("approval-kan113-{suffix}"))
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
            'sha256:kan113',
            '/evidence/packets/tickets/KAN-113',
            'advisory',
            TRUE,
            FALSE,
            FALSE,
            'KAN-113 period report fixture',
            '["sonar_quality_gate"]'::jsonb,
            '["quality gate evidence not present in KAN-99 export"]'::jsonb,
            'policy-kan113',
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
                "policy":{"source":"repo-file","checksum":"policy-kan113"}
            }'::jsonb,
            '{"metadata":{"token":"must-not-report","source_code":"must-not-report"}}'::jsonb,
            'integration-test'
        )
        "#,
    )
    .bind(&authorization_id)
    .bind(org_id)
    .bind(format!("{RELEASE_ID}-{suffix}"))
    .bind(REPO_FULL_NAME)
    .bind(BRANCH)
    .bind(&target_sha)
    .bind(ENVIRONMENT)
    .bind(TICKET_ID)
    .execute(pool)
    .await
    .expect("insert deployment gate");

    authorization_id
}

async fn create_export(app: &axum::Router, api_key: &str, gate_id: &str) -> String {
    let body = json!({
        "scope": "deployment_gate",
        "deployment_gate_id": gate_id,
        "format": "json",
        "include_sections": ["gate_decision", "policy", "readiness", "approvals", "evidence", "gaps", "audit"]
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
    parsed["export"]["export_id"].as_str().unwrap().to_string()
}

async fn create_mapping(app: &axum::Router, api_key: &str, export_id: &str) -> String {
    let body = json!({
        "evidence_export_id": export_id,
        "framework_id": BASELINE_FRAMEWORK_ID
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
    let body = json!({ "mapping_id": mapping_id, "format": "json" });
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
) -> String {
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
    assert!(!response.contains("must-not-report"));
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("report JSON");
    parsed["report"]["report_id"].as_str().unwrap().to_string()
}

async fn create_reviewed_report_chain(
    app: &axum::Router,
    pool: &sqlx::PgPool,
    org_id: &str,
    admin_key: &str,
    suffix: &str,
) -> String {
    let gate_id = seed_period_report_gate(pool, org_id, suffix).await;
    let export_id = create_export(app, admin_key, &gate_id).await;
    let mapping_id = create_mapping(app, admin_key, &export_id).await;
    let package_id = create_review_package(app, admin_key, &mapping_id).await;
    let report_id = create_framework_report(app, admin_key, &mapping_id, &package_id).await;
    let body = json!({ "review_status": "reviewed" });
    let (status, response) = json_request(
        app,
        "PATCH",
        &format!("/compliance/framework-review-reports/{report_id}/review"),
        Some(&body.to_string()),
        Some(admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "review report failed: {response}");
    let (status, response) = json_request(
        app,
        "POST",
        &format!("/compliance/framework-review-reports/{report_id}/provenance-manifests"),
        Some(&json!({}).to_string()),
        Some(admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "manifest failed: {response}");
    report_id
}

#[tokio::test]
async fn period_compliance_report_aggregates_reviewed_reports_without_claims() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan113-period-report").await;
    let other_org_id = insert_test_org(&pool, "kan113-period-report-other").await;
    let admin = insert_test_api_key_for_org(&pool, "kan113-period-admin", "Admin", &org_id).await;
    let auditor =
        insert_test_api_key_for_org(&pool, "kan113-period-auditor", "Auditor", &org_id).await;
    let unassigned_auditor = insert_test_api_key_for_org(
        &pool,
        "kan113-period-unassigned-auditor",
        "Auditor",
        &org_id,
    )
    .await;
    let other_admin =
        insert_test_api_key_for_org(&pool, "kan113-period-other-admin", "Admin", &other_org_id)
            .await;
    let other_auditor = insert_test_api_key_for_org(
        &pool,
        "kan113-period-other-auditor",
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

    let report_a = create_reviewed_report_chain(&app, &pool, &org_id, &admin, "inside-a").await;
    let report_b = create_reviewed_report_chain(&app, &pool, &org_id, &admin, "inside-b").await;
    let outside_report =
        create_reviewed_report_chain(&app, &pool, &org_id, &admin, "outside").await;
    let other_report =
        create_reviewed_report_chain(&app, &pool, &other_org_id, &other_admin, "other").await;

    sqlx::query(
        "UPDATE compliance_framework_review_reports SET created_at = NOW() - INTERVAL '10 days' WHERE report_id = $1",
    )
    .bind(&outside_report)
    .execute(&pool)
    .await
    .expect("move outside report outside period");

    let assignment_body = json!({
        "auditor_client_ids": ["kan113-period-auditor"],
        "assignment_notes_safe": "KAN-113 period report visibility source assignment"
    });
    let (status, assignment_response) = json_request(
        &app,
        "PUT",
        &format!("/compliance/framework-review-reports/{report_a}/assignments"),
        Some(&assignment_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "assignment failed: {assignment_response}"
    );

    let start = chrono::Utc::now().timestamp_millis() - 60 * 60 * 1000;
    let end = chrono::Utc::now().timestamp_millis() + 60 * 60 * 1000;
    let period_body = json!({
        "date_range_start": start,
        "date_range_end": end,
        "framework_id": BASELINE_FRAMEWORK_ID,
        "format": "json"
    });

    let (status, auditor_create) = json_request(
        &app,
        "POST",
        "/compliance/period-reports",
        Some(&period_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_create.contains("Admin access required"));

    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/period-reports",
        Some(&period_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "period report failed: {response}"
    );
    assert!(!response.contains("must-not-report"));
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("period report JSON");
    let period_report = &parsed["period_report"];
    let artifact = &parsed["artifact"];
    let period_report_id = period_report["period_report_id"]
        .as_str()
        .expect("period report id");
    let artifact_hash = period_report["artifact_hash"]
        .as_str()
        .expect("artifact hash");

    assert!(period_report_id.starts_with("cpr_"));
    assert!(artifact_hash.starts_with("sha256:"));
    assert_eq!(period_report["report_count"], 2);
    assert_eq!(period_report["compliance_claim"], false);
    assert_eq!(period_report["regulatory_claim"], false);
    assert_eq!(period_report["certification"], false);
    assert_eq!(period_report["requires_auditor_review"], true);
    assert_eq!(
        artifact["schema_version"],
        "gitgov_period_compliance_report.v1"
    );
    assert_eq!(artifact["summary"]["report_count"], 2);
    assert_eq!(artifact["summary"]["reviewed_report_count"], 2);
    assert_eq!(artifact["summary"]["reports_with_manifest_count"], 2);
    assert_eq!(artifact["summary"]["reports_missing_manifest_count"], 0);
    assert_eq!(artifact["claims"]["compliance_claim"], false);
    assert_eq!(artifact["claims"]["regulatory_claim"], false);
    assert_eq!(artifact["claims"]["certification"], false);
    assert_eq!(
        artifact["audit_metadata"]["agent_governance_required"],
        false
    );
    assert_eq!(artifact["audit_metadata"]["policy_mutation"], false);
    assert_eq!(artifact["audit_metadata"]["provider_mutation"], false);
    assert_eq!(artifact["audit_metadata"]["gate_mutation"], false);

    let reports = artifact["reports"].as_array().expect("artifact reports");
    let source_ids: Vec<&str> = reports
        .iter()
        .map(|item| item["report_id"].as_str().unwrap())
        .collect();
    assert!(source_ids.contains(&report_a.as_str()));
    assert!(source_ids.contains(&report_b.as_str()));
    assert!(!source_ids.contains(&outside_report.as_str()));
    assert!(!source_ids.contains(&other_report.as_str()));
    assert!(reports.iter().all(|item| item["latest_manifest_hash"]
        .as_str()
        .unwrap()
        .starts_with("sha256:")));
    assert!(artifact["source_hashes"]["report_hashes"]
        .as_array()
        .expect("report hashes")
        .iter()
        .all(|hash| hash.as_str().unwrap().starts_with("sha256:")));
    assert!(artifact["missing_evidence_summary"]
        .as_array()
        .expect("missing evidence")
        .iter()
        .any(|item| item["evidence_type"] == "sonar_quality_gate"));

    let recomputed_hash = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_string(artifact).unwrap().as_bytes())
    );
    assert_eq!(recomputed_hash, artifact_hash);

    let (status, auditor_list) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports?framework_id={BASELINE_FRAMEWORK_ID}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor list failed: {auditor_list}"
    );
    let auditor_list: serde_json::Value =
        serde_json::from_str(&auditor_list).expect("auditor list JSON");
    assert_eq!(auditor_list["count"], 1);
    assert_eq!(
        auditor_list["items"][0]["period_report_id"],
        period_report_id
    );

    let (status, unassigned_list) = json_request(
        &app,
        "GET",
        "/compliance/period-reports",
        None,
        Some(&unassigned_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let unassigned_list: serde_json::Value =
        serde_json::from_str(&unassigned_list).expect("unassigned list JSON");
    assert_eq!(unassigned_list["count"], 0);

    let (status, unassigned_download) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/download"),
        None,
        Some(&unassigned_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(unassigned_download.contains("not found"));

    let (status, other_download) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/download"),
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(other_download.contains("not found"));

    let (status, download) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/download"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor download failed: {download}"
    );
    assert!(!download.contains("must-not-report"));
    let downloaded: serde_json::Value = serde_json::from_str(&download).expect("download JSON");
    assert_eq!(downloaded["period_report_id"], period_report_id);
    assert_eq!(downloaded["source_hashes"], artifact["source_hashes"]);
    assert_eq!(downloaded["claims"]["requires_auditor_review"], true);

    let downloaded_at: Option<i64> = sqlx::query_scalar(
        "SELECT ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT FROM compliance_period_reports WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("downloaded_at");
    assert!(downloaded_at.unwrap_or_default() > 0);

    let report_hash_after_period: String = sqlx::query_scalar(
        "SELECT artifact_hash FROM compliance_framework_review_reports WHERE report_id = $1",
    )
    .bind(&report_a)
    .fetch_one(&pool)
    .await
    .expect("report hash after period");
    let report_a_artifact_hash = reports
        .iter()
        .find(|item| item["report_id"] == report_a)
        .and_then(|item| item["artifact_hash"].as_str())
        .expect("report A artifact hash");
    assert_eq!(report_hash_after_period, report_a_artifact_hash);

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after");
    assert_eq!(after_evaluations, before_evaluations);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.created'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count period audit rows");
    assert_eq!(audit_count, 1);

    teardown(&admin_pool, &schema).await;
}
