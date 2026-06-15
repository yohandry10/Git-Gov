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
    let developer =
        insert_test_api_key_for_org(&pool, "kan117-period-developer", "Developer", &org_id).await;
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
    assert_eq!(period_report["retention_status"], "active");
    assert_eq!(period_report["review_status"], "needs_review");
    assert!(period_report["reviewed_by_user_id"].is_null());
    assert!(period_report["reviewed_at"].is_null());
    assert!(period_report["review_notes_safe"].is_null());
    assert!(
        period_report["retention_until"]
            .as_i64()
            .unwrap_or_default()
            > period_report["created_at"].as_i64().unwrap_or_default()
    );
    assert_eq!(period_report["download_count"], 0);
    assert!(period_report["last_downloaded_at"].is_null());
    assert!(period_report["archived_at"].is_null());
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

    let initial_access_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("initial period access log count");
    assert_eq!(initial_access_log_count, 0);

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

    let (status, auditor_get) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor get failed: {auditor_get}");
    let auditor_get: serde_json::Value =
        serde_json::from_str(&auditor_get).expect("auditor get JSON");
    assert_eq!(
        auditor_get["period_report"]["period_report_id"],
        period_report_id
    );
    assert_eq!(
        auditor_get["period_report"]["review_status"],
        "needs_review"
    );

    let (status, developer_review) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(&json!({ "review_status": "reviewed" }).to_string()),
        Some(&developer),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(developer_review.contains("Admin or Auditor compliance review access required"));

    let (status, unassigned_review) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(&json!({ "review_status": "reviewed" }).to_string()),
        Some(&unassigned_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(unassigned_review.contains("not found"));

    let (status, unsafe_review_note) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes_safe": "token ghp_thisMustNotPersist"
            })
            .to_string(),
        ),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(unsafe_review_note.contains("cannot contain secrets"));

    let (status, missing_note_review) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(&json!({ "review_status": "needs_changes" }).to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(missing_note_review.contains("review_notes_safe is required"));

    let (status, review_get) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "review get failed: {review_get}");
    let review_get: serde_json::Value = serde_json::from_str(&review_get).expect("review get JSON");
    assert_eq!(review_get["period_report"]["review_status"], "needs_review");

    let (status, review_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes_safe": "KAN-117 manual reviewer sign-off for sharing package"
            })
            .to_string(),
        ),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "period review failed: {review_response}"
    );
    let review_response: serde_json::Value =
        serde_json::from_str(&review_response).expect("period review JSON");
    assert_eq!(
        review_response["period_report"]["review_status"],
        "reviewed"
    );
    assert_eq!(
        review_response["period_report"]["reviewed_by_user_id"],
        "kan113-period-auditor"
    );
    assert!(
        review_response["period_report"]["reviewed_at"]
            .as_i64()
            .unwrap_or_default()
            > 0
    );
    assert_eq!(
        review_response["period_report"]["review_notes_safe"],
        "KAN-117 manual reviewer sign-off for sharing package"
    );
    assert_eq!(
        review_response["period_report"]["artifact_hash"],
        artifact_hash
    );

    let period_hash_after_review: String = sqlx::query_scalar(
        "SELECT artifact_hash FROM compliance_period_reports WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period hash after review");
    assert_eq!(period_hash_after_review, artifact_hash);

    let review_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1 AND action = 'review_updated' AND artifact_type = 'review' AND artifact_hash = $2",
    )
    .bind(period_report_id)
    .bind(artifact_hash)
    .fetch_one(&pool)
    .await
    .expect("period review access log count");
    assert_eq!(review_log_count, 1);

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

    let download_custody: (i32, Option<i64>) = sqlx::query_as(
        r#"
        SELECT
            download_count,
            ROUND(EXTRACT(EPOCH FROM last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at
        FROM compliance_period_reports
        WHERE period_report_id = $1
        "#,
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period report JSON download custody");
    assert_eq!(download_custody.0, 1);
    assert!(download_custody.1.unwrap_or_default() > 0);

    let json_download_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1 AND action = 'downloaded_json' AND artifact_hash = $2",
    )
    .bind(period_report_id)
    .bind(artifact_hash)
    .fetch_one(&pool)
    .await
    .expect("period JSON download access log");
    assert_eq!(json_download_log_count, 1);

    let (status, _unassigned_pdf_create) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-reports/{period_report_id}/pdf-export"),
        Some(&json!({}).to_string()),
        Some(&unassigned_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, pdf_create) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-reports/{period_report_id}/pdf-export"),
        Some(&json!({}).to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "period PDF export failed: {pdf_create}"
    );
    let pdf_create: serde_json::Value = serde_json::from_str(&pdf_create).expect("period PDF JSON");
    let pdf_export = &pdf_create["pdf_export"];
    let pdf_export_id = pdf_export["pdf_export_id"].as_str().expect("pdf id");
    let pdf_artifact_hash = pdf_export["pdf_artifact_hash"].as_str().expect("pdf hash");
    assert!(pdf_export_id.starts_with("cprpdf_"));
    assert!(pdf_artifact_hash.starts_with("sha256:"));
    assert_eq!(pdf_export["period_report_id"], period_report_id);
    assert_eq!(pdf_export["source_period_report_hash"], artifact_hash);
    assert_eq!(pdf_export["content_type"], "application/pdf");
    assert_eq!(pdf_export["compliance_claim"], false);
    assert_eq!(pdf_export["regulatory_claim"], false);
    assert_eq!(pdf_export["certification"], false);
    assert_eq!(pdf_export["requires_auditor_review"], true);
    assert!(pdf_export["page_count"].as_i64().unwrap_or_default() >= 1);

    let (status, _other_pdf_download) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/period-reports/{period_report_id}/pdf-export/download?pdf_export_id={pdf_export_id}"
        ),
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, pdf_download) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/period-reports/{period_report_id}/pdf-export/download?pdf_export_id={pdf_export_id}"
        ),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "period PDF download failed: {pdf_download}"
    );
    assert!(pdf_download.starts_with("%PDF-1.4"));
    assert!(pdf_download.contains("GitGov Period Compliance Report"));
    assert!(pdf_download.contains("Not a certification"));
    assert!(pdf_download.contains(period_report_id));
    assert!(pdf_download.contains(&artifact_hash[..24]));
    assert!(!pdf_download.contains("must-not-report"));
    let recomputed_pdf_hash = format!("sha256:{:x}", Sha256::digest(pdf_download.as_bytes()));
    assert_eq!(recomputed_pdf_hash, pdf_artifact_hash);

    let pdf_download_custody: (i32, Option<i64>) = sqlx::query_as(
        r#"
        SELECT
            download_count,
            ROUND(EXTRACT(EPOCH FROM last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at
        FROM compliance_period_reports
        WHERE period_report_id = $1
        "#,
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period report PDF download custody");
    assert_eq!(pdf_download_custody.0, 2);
    assert!(pdf_download_custody.1.unwrap_or_default() > 0);

    let pdf_download_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1 AND action = 'downloaded_pdf' AND artifact_id = $2 AND artifact_hash = $3",
    )
    .bind(period_report_id)
    .bind(pdf_export_id)
    .bind(pdf_artifact_hash)
    .fetch_one(&pool)
    .await
    .expect("period PDF download access log");
    assert_eq!(pdf_download_log_count, 1);

    let pdf_downloaded_at: Option<i64> = sqlx::query_scalar(
        "SELECT ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT FROM compliance_period_report_pdf_exports WHERE pdf_export_id = $1",
    )
    .bind(pdf_export_id)
    .fetch_one(&pool)
    .await
    .expect("pdf downloaded_at");
    assert!(pdf_downloaded_at.unwrap_or_default() > 0);

    let period_hash_after_pdf: String = sqlx::query_scalar(
        "SELECT artifact_hash FROM compliance_period_reports WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period hash after pdf");
    assert_eq!(period_hash_after_pdf, artifact_hash);

    let (status, _unassigned_manifest_create) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-reports/{period_report_id}/provenance-manifests"),
        Some(&json!({}).to_string()),
        Some(&unassigned_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, manifest_create) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-reports/{period_report_id}/provenance-manifests"),
        Some(&json!({}).to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "period manifest failed: {manifest_create}"
    );
    let manifest_create: serde_json::Value =
        serde_json::from_str(&manifest_create).expect("period manifest JSON");
    let manifest = &manifest_create["manifest"];
    let manifest_artifact = &manifest_create["artifact"];
    let manifest_id = manifest["manifest_id"].as_str().expect("manifest id");
    let manifest_hash = manifest["manifest_hash"].as_str().expect("manifest hash");
    assert!(manifest_id.starts_with("cprm_"));
    assert!(manifest_hash.starts_with("sha256:"));
    assert!(manifest["previous_manifest_hash"].is_null());
    assert_eq!(
        manifest["signature_algorithm"],
        "sha256-period-report-provenance-manifest-v1"
    );
    assert_eq!(
        manifest_artifact["schema_version"],
        "gitgov_period_compliance_report_provenance_manifest.v1"
    );
    assert_eq!(
        manifest_artifact["period_report"]["period_report_id"],
        period_report_id
    );
    assert_eq!(
        manifest_artifact["period_report"]["artifact_hash"],
        artifact_hash
    );
    assert_eq!(
        manifest_artifact["period_report"]["review_status"],
        "reviewed"
    );
    assert_eq!(
        manifest_artifact["period_report"]["reviewed_by_user_id"],
        "kan113-period-auditor"
    );
    assert_eq!(
        manifest_artifact["period_report"]["has_review_notes_safe"],
        true
    );
    assert_eq!(
        manifest_artifact["period_artifact_summary"]["source_hashes"],
        artifact["source_hashes"]
    );
    assert_eq!(manifest_artifact["pdf_exports"]["count"], 1);
    assert_eq!(
        manifest_artifact["pdf_exports"]["items"][0]["pdf_artifact_hash"],
        pdf_artifact_hash
    );
    assert_eq!(manifest_artifact["claims"]["compliance_claim"], false);
    assert_eq!(manifest_artifact["claims"]["regulatory_claim"], false);
    assert_eq!(manifest_artifact["claims"]["certification"], false);
    assert_eq!(
        manifest_artifact["audit_metadata"]["agent_governance_required"],
        false
    );
    assert_eq!(
        manifest_artifact["audit_metadata"]["source_period_report_artifact_mutated"],
        false
    );
    assert_eq!(
        manifest_artifact["audit_metadata"]["source_period_report_review_mutated"],
        false
    );
    assert_eq!(
        manifest_artifact["audit_metadata"]["legal_attestation"],
        false
    );
    let manifest_action_counts = manifest_artifact["access_log"]["action_counts"]
        .as_array()
        .expect("manifest access counts");
    assert!(manifest_action_counts
        .iter()
        .any(|item| item["action"] == "downloaded_json" && item["count"] == 1));
    assert!(manifest_action_counts
        .iter()
        .any(|item| item["action"] == "downloaded_pdf" && item["count"] == 1));
    assert!(manifest_action_counts
        .iter()
        .any(|item| item["action"] == "viewed" && item["count"] == 1));
    assert!(manifest_action_counts
        .iter()
        .any(|item| item["action"] == "review_updated" && item["count"] == 1));

    let (status, _other_manifest_download) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/period-reports/{period_report_id}/provenance-manifests/{manifest_id}"
        ),
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, manifest_download) = json_request(
        &app,
        "GET",
        &format!(
            "/compliance/period-reports/{period_report_id}/provenance-manifests/{manifest_id}"
        ),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "period manifest download failed: {manifest_download}"
    );
    let manifest_download: serde_json::Value =
        serde_json::from_str(&manifest_download).expect("manifest download JSON");
    assert_eq!(manifest_download["manifest_id"], manifest_id);
    assert_eq!(
        manifest_download["hash_chain"]["manifest_hash"],
        manifest_hash
    );
    assert_eq!(
        manifest_download["period_report"]["artifact_hash"],
        artifact_hash
    );

    let (status, second_manifest_create) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-reports/{period_report_id}/provenance-manifests"),
        Some(&json!({}).to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "second period manifest failed: {second_manifest_create}"
    );
    let second_manifest_create: serde_json::Value =
        serde_json::from_str(&second_manifest_create).expect("second manifest JSON");
    let second_manifest = &second_manifest_create["manifest"];
    assert_ne!(second_manifest["manifest_id"], manifest_id);
    assert_eq!(second_manifest["previous_manifest_hash"], manifest_hash);
    assert_eq!(
        second_manifest_create["artifact"]["hash_chain"]["previous_manifest_hash"],
        manifest_hash
    );
    assert!(
        second_manifest_create["artifact"]["access_log"]["action_counts"]
            .as_array()
            .expect("second manifest access counts")
            .iter()
            .any(|item| item["action"] == "manifest_downloaded" && item["count"] == 1)
    );

    let manifest_row_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_manifests WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period manifest row count");
    assert_eq!(manifest_row_count, 2);

    let manifest_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1 AND action IN ('manifest_created', 'manifest_downloaded')",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("period manifest access log count");
    assert_eq!(manifest_log_count, 3);

    let downloaded_at: Option<i64> = sqlx::query_scalar(
        "SELECT ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT FROM compliance_period_reports WHERE period_report_id = $1",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("downloaded_at");
    assert!(downloaded_at.unwrap_or_default() > 0);

    let retention_until_future = chrono::Utc::now().timestamp_millis() + 365 * 24 * 60 * 60 * 1000;
    let (status, auditor_retention_update) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/retention"),
        Some(
            &json!({
                "retention_until": retention_until_future,
                "archive": false
            })
            .to_string(),
        ),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_retention_update.contains("Admin access required"));

    let retention_until_past = chrono::Utc::now().timestamp_millis() - 24 * 60 * 60 * 1000;
    let (status, expired_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/retention"),
        Some(
            &json!({
                "retention_until": retention_until_past,
                "archive": false
            })
            .to_string(),
        ),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "retention expiration update failed: {expired_response}"
    );
    let expired_response: serde_json::Value =
        serde_json::from_str(&expired_response).expect("expired response JSON");
    assert_eq!(
        expired_response["period_report"]["retention_status"],
        "retention_expired"
    );
    assert_eq!(expired_response["period_report"]["download_count"], 2);

    let (status, expired_download) = json_request(
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
        "expired report logical download failed: {expired_download}"
    );
    assert!(expired_download.contains(period_report_id));

    let (status, future_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/retention"),
        Some(
            &json!({
                "retention_until": retention_until_future,
                "archive": false
            })
            .to_string(),
        ),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "retention extension failed: {future_response}"
    );
    let future_response: serde_json::Value =
        serde_json::from_str(&future_response).expect("future retention response JSON");
    assert_eq!(
        future_response["period_report"]["retention_status"],
        "active"
    );

    let (status, archive_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/retention"),
        Some(&json!({ "archive": true }).to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "archive failed: {archive_response}");
    let archive_response: serde_json::Value =
        serde_json::from_str(&archive_response).expect("archive response JSON");
    assert_eq!(
        archive_response["period_report"]["retention_status"],
        "archived"
    );
    assert!(
        archive_response["period_report"]["archived_at"]
            .as_i64()
            .unwrap_or_default()
            > 0
    );

    let (status, archived_review_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-reports/{period_report_id}/review"),
        Some(
            &json!({
                "review_status": "needs_changes",
                "review_notes_safe": "Archived reports are immutable for new review decisions"
            })
            .to_string(),
        ),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(archived_review_response.contains("period_report_archived"));

    let (status, access_log_response) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/access-log?limit=20"),
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "access log failed: {access_log_response}"
    );
    let access_log_response: serde_json::Value =
        serde_json::from_str(&access_log_response).expect("access log JSON");
    let actions = access_log_response["items"]
        .as_array()
        .expect("access log items")
        .iter()
        .filter_map(|item| item["action"].as_str())
        .collect::<Vec<_>>();
    assert!(actions.contains(&"viewed"));
    assert!(actions.contains(&"downloaded_json"));
    assert!(actions.contains(&"downloaded_pdf"));
    assert!(actions.contains(&"retention_updated"));
    assert!(actions.contains(&"archived"));
    assert!(actions.contains(&"manifest_created"));
    assert!(actions.contains(&"manifest_downloaded"));
    assert!(actions.contains(&"review_updated"));
    assert_eq!(
        access_log_response["items"][0]["artifact_hash"],
        artifact_hash
    );

    let (status, other_access_log_response) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-reports/{period_report_id}/access-log"),
        None,
        Some(&other_auditor),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(other_access_log_response.contains("not found"));

    let archived_row: (String, Option<i64>, i32) = sqlx::query_as(
        r#"
        SELECT
            retention_status,
            ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at,
            download_count
        FROM compliance_period_reports
        WHERE period_report_id = $1
        "#,
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("archived report still exists");
    assert_eq!(archived_row.0, "archived");
    assert!(archived_row.1.unwrap_or_default() > 0);
    assert_eq!(archived_row.2, 3);

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

    let pdf_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.pdf_export_created'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count period PDF audit rows");
    assert_eq!(pdf_audit_count, 1);

    let manifest_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.provenance_manifest_created'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count period manifest audit rows");
    assert_eq!(manifest_audit_count, 2);

    let review_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.reviewed'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count period review audit rows");
    assert_eq!(review_audit_count, 1);

    let retention_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.retention_updated'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count retention audit rows");
    assert_eq!(retention_audit_count, 2);

    let archive_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action = 'compliance_period_report.archived'",
    )
    .bind(period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count archive audit rows");
    assert_eq!(archive_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn period_report_profiles_run_real_artifacts_and_enforce_manual_boundaries() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan118-period-report-profiles").await;
    let other_org_id = insert_test_org(&pool, "kan118-period-report-profiles-other").await;
    let admin = insert_test_api_key_for_org(&pool, "kan118-profile-admin", "Admin", &org_id).await;
    let auditor =
        insert_test_api_key_for_org(&pool, "kan118-profile-auditor", "Auditor", &org_id).await;
    let developer =
        insert_test_api_key_for_org(&pool, "kan118-profile-developer", "Developer", &org_id).await;
    let other_admin =
        insert_test_api_key_for_org(&pool, "kan118-profile-other-admin", "Admin", &other_org_id)
            .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let before_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations before profile runs");

    let report_a = create_reviewed_report_chain(&app, &pool, &org_id, &admin, "kan118-a").await;
    let report_b = create_reviewed_report_chain(&app, &pool, &org_id, &admin, "kan118-b").await;
    let other_report =
        create_reviewed_report_chain(&app, &pool, &other_org_id, &other_admin, "kan118-other")
            .await;

    let profile_body = json!({
        "name": "Monthly SOX-style evidence pack",
        "period_type": "monthly",
        "framework_id": BASELINE_FRAMEWORK_ID,
        "framework_owner_type": "gitgov_managed",
        "include_pdf": true,
        "include_manifest": true,
        "retention_days": 45,
        "filters": {
            "environment": "production",
            "manual_run_template": true
        }
    });
    let (status, auditor_create) = json_request(
        &app,
        "POST",
        "/compliance/period-report-profiles",
        Some(&profile_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_create.contains("Admin access required"));

    let (status, create_response) = json_request(
        &app,
        "POST",
        "/compliance/period-report-profiles",
        Some(&profile_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "profile create failed: {create_response}"
    );
    assert!(!create_response.contains("token"));
    assert!(!create_response.contains("secret"));
    let create_json: serde_json::Value =
        serde_json::from_str(&create_response).expect("profile create JSON");
    let profile_id = create_json["profile"]["profile_id"]
        .as_str()
        .expect("profile id")
        .to_string();
    assert!(profile_id.starts_with("cprprof_"));
    assert_eq!(create_json["profile"]["status"], "active");
    assert_eq!(create_json["profile"]["run_count"], 0);
    assert_eq!(create_json["profile"]["include_pdf"], true);
    assert_eq!(create_json["profile"]["include_manifest"], true);

    let (status, auditor_list) = json_request(
        &app,
        "GET",
        "/compliance/period-report-profiles?status=active",
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
        serde_json::from_str(&auditor_list).expect("auditor profile list JSON");
    assert_eq!(auditor_list["count"], 1);
    assert_eq!(auditor_list["items"][0]["profile_id"], profile_id);

    let (status, developer_list) = json_request(
        &app,
        "GET",
        "/compliance/period-report-profiles",
        None,
        Some(&developer),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(developer_list.contains("Admin or Auditor compliance review access required"));

    let (status, other_get) = json_request(
        &app,
        "GET",
        &format!("/compliance/period-report-profiles/{profile_id}"),
        None,
        Some(&other_admin),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(other_get.contains("profile not found"));

    let start = chrono::Utc::now().timestamp_millis() - 60 * 60 * 1000;
    let end = chrono::Utc::now().timestamp_millis() + 60 * 60 * 1000;
    let run_body = json!({
        "date_range_start": start,
        "date_range_end": end
    });
    let (status, auditor_run) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-report-profiles/{profile_id}/run"),
        Some(&run_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_run.contains("Admin access required"));

    let (status, run_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-report-profiles/{profile_id}/run"),
        Some(&run_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "profile run failed: {run_response}"
    );
    assert!(!run_response.contains("must-not-report"));
    let run_json: serde_json::Value =
        serde_json::from_str(&run_response).expect("profile run JSON");
    let period_report_id = run_json["period_report"]["period_report_id"]
        .as_str()
        .expect("run period report id")
        .to_string();
    let pdf_export_id = run_json["pdf_export"]["pdf_export_id"]
        .as_str()
        .expect("run pdf export id")
        .to_string();
    let manifest_id = run_json["manifest"]["manifest_id"]
        .as_str()
        .expect("run manifest id")
        .to_string();
    assert_eq!(run_json["profile"]["run_count"], 1);
    assert_eq!(
        run_json["profile"]["last_period_report_id"],
        period_report_id
    );
    assert_eq!(run_json["profile"]["last_pdf_export_id"], pdf_export_id);
    assert_eq!(run_json["profile"]["last_manifest_id"], manifest_id);
    assert_eq!(run_json["period_report"]["report_count"], 2);
    assert_eq!(run_json["period_report"]["review_status"], "needs_review");
    assert_eq!(run_json["period_report"]["retention_status"], "active");
    assert_eq!(run_json["period_report"]["compliance_claim"], false);
    assert_eq!(run_json["period_report"]["regulatory_claim"], false);
    assert_eq!(run_json["period_report"]["certification"], false);
    assert_eq!(run_json["pdf_export"]["content_type"], "application/pdf");
    assert_eq!(
        run_json["manifest"]["signature_algorithm"],
        "sha256-period-report-provenance-manifest-v1"
    );
    assert_eq!(
        run_json["download_url"],
        format!("/compliance/period-reports/{period_report_id}/download")
    );

    let generated_source_ids: Vec<String> = sqlx::query_scalar(
        "SELECT jsonb_array_elements_text(source_report_ids) FROM compliance_period_reports WHERE period_report_id = $1",
    )
    .bind(&period_report_id)
    .fetch_all(&pool)
    .await
    .expect("load profile run source ids");
    assert!(generated_source_ids.contains(&report_a));
    assert!(generated_source_ids.contains(&report_b));
    assert!(!generated_source_ids.contains(&other_report));

    let retention_delta_days: f64 = sqlx::query_scalar(
        r#"
        SELECT (EXTRACT(EPOCH FROM (retention_until - created_at)) / 86400.0)::float8
        FROM compliance_period_reports
        WHERE period_report_id = $1
        "#,
    )
    .bind(&period_report_id)
    .fetch_one(&pool)
    .await
    .expect("profile run retention delta");
    assert!(
        (44.0..=46.0).contains(&retention_delta_days),
        "retention delta should inherit 45 days, got {retention_delta_days}"
    );

    let pdf_row: (String, i32, Vec<u8>) = sqlx::query_as(
        r#"
        SELECT content_type, page_count, pdf_bytes
        FROM compliance_period_report_pdf_exports
        WHERE pdf_export_id = $1
        "#,
    )
    .bind(&pdf_export_id)
    .fetch_one(&pool)
    .await
    .expect("load profile PDF export");
    assert_eq!(pdf_row.0, "application/pdf");
    assert!(pdf_row.1 >= 1);
    assert!(pdf_row.2.starts_with(b"%PDF-1.4"));

    let manifest_payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload_json_redacted FROM compliance_period_report_manifests WHERE manifest_id = $1",
    )
    .bind(&manifest_id)
    .fetch_one(&pool)
    .await
    .expect("load profile manifest payload");
    assert_eq!(
        manifest_payload["schema_version"],
        "gitgov_period_compliance_report_provenance_manifest.v1"
    );
    assert_eq!(manifest_payload["claims"]["compliance_claim"], false);
    assert_eq!(
        manifest_payload["audit_metadata"]["agent_governance_required"],
        false
    );
    assert_eq!(manifest_payload["pdf_exports"]["count"], 1);
    assert_eq!(
        manifest_payload["period_report"]["review_status"],
        "needs_review"
    );

    let retention_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_access_log WHERE period_report_id = $1 AND action = 'retention_updated'",
    )
    .bind(&period_report_id)
    .fetch_one(&pool)
    .await
    .expect("count profile retention log");
    assert_eq!(retention_log_count, 1);

    let patch_body = json!({
        "name": "Monthly evidence pack without secondary artifacts",
        "include_pdf": false,
        "include_manifest": false,
        "retention_days": 31,
        "filters": {
            "environment": "production",
            "manual_run_template": true,
            "secondary_artifacts": false
        }
    });
    let (status, patch_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-report-profiles/{profile_id}"),
        Some(&patch_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "profile patch failed: {patch_response}"
    );
    let patch_json: serde_json::Value =
        serde_json::from_str(&patch_response).expect("profile patch JSON");
    assert_eq!(patch_json["profile"]["include_pdf"], false);
    assert_eq!(patch_json["profile"]["include_manifest"], false);
    assert_eq!(patch_json["profile"]["retention_days"], 31);

    let (status, second_run_response) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-report-profiles/{profile_id}/run"),
        Some(&run_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "second profile run failed: {second_run_response}"
    );
    let second_run_json: serde_json::Value =
        serde_json::from_str(&second_run_response).expect("second profile run JSON");
    let second_period_report_id = second_run_json["period_report"]["period_report_id"]
        .as_str()
        .expect("second period report id");
    assert_eq!(second_run_json["profile"]["run_count"], 2);
    assert!(second_run_json["pdf_export"].is_null());
    assert!(second_run_json["manifest"].is_null());
    let second_pdf_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_pdf_exports WHERE period_report_id = $1",
    )
    .bind(second_period_report_id)
    .fetch_one(&pool)
    .await
    .expect("second run pdf count");
    assert_eq!(second_pdf_count, 0);
    let second_manifest_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_period_report_manifests WHERE period_report_id = $1",
    )
    .bind(second_period_report_id)
    .fetch_one(&pool)
    .await
    .expect("second run manifest count");
    assert_eq!(second_manifest_count, 0);

    let (status, auditor_patch) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-report-profiles/{profile_id}"),
        Some(&patch_body.to_string()),
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(auditor_patch.contains("Admin access required"));

    let (status, archive_response) = json_request(
        &app,
        "PATCH",
        &format!("/compliance/period-report-profiles/{profile_id}/archive"),
        Some(&json!({}).to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "profile archive failed: {archive_response}"
    );
    let archive_json: serde_json::Value =
        serde_json::from_str(&archive_response).expect("profile archive JSON");
    assert_eq!(archive_json["profile"]["status"], "archived");
    assert!(archive_json["profile"]["archived_at"].as_i64().is_some());

    let (status, active_after_archive) = json_request(
        &app,
        "GET",
        "/compliance/period-report-profiles?status=active&limit=25",
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let active_after_archive_json: serde_json::Value =
        serde_json::from_str(&active_after_archive).expect("active profiles after archive JSON");
    assert!(
        active_after_archive_json["items"]
            .as_array()
            .expect("active profile items")
            .iter()
            .all(|item| item["profile_id"] != profile_id),
        "archived profile must not appear in status=active profile list"
    );

    let (status, archived_after_archive) = json_request(
        &app,
        "GET",
        "/compliance/period-report-profiles?status=archived&limit=25",
        None,
        Some(&auditor),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let archived_after_archive_json: serde_json::Value =
        serde_json::from_str(&archived_after_archive)
            .expect("archived profiles after archive JSON");
    assert!(
        archived_after_archive_json["items"]
            .as_array()
            .expect("archived profile items")
            .iter()
            .any(|item| item["profile_id"] == profile_id),
        "archived profile must appear in status=archived profile list"
    );

    let (status, archived_run) = json_request(
        &app,
        "POST",
        &format!("/compliance/period-report-profiles/{profile_id}/run"),
        Some(&run_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(archived_run.contains("period_report_profile_archived"));

    let profile_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE target_id = $1 AND action IN ('compliance_period_report_profile.created', 'compliance_period_report_profile.updated', 'compliance_period_report_profile.run', 'compliance_period_report_profile.archived')",
    )
    .bind(&profile_id)
    .fetch_one(&pool)
    .await
    .expect("count profile audit rows");
    assert_eq!(profile_audit_count, 5);

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after profile runs");
    assert_eq!(after_evaluations, before_evaluations);

    teardown(&admin_pool, &schema).await;
}
