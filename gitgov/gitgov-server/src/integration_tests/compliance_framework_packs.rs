use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "yohandry10/Git-Gov";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const RELEASE_ID: &str = "KAN-103";
const ENVIRONMENT: &str = "production";
const TICKET_ID: &str = "KAN-103";

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

async fn seed_customer_framework_gate(pool: &sqlx::PgPool, org_id: &str, suffix: &str) -> String {
    let repo_id = insert_repo(pool, org_id).await;
    let authorization_id = format!("dga_kan103_{suffix}");

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
            '{"repo_full_name":"yohandry10/Git-Gov","token":"must-not-import"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(&repo_id)
    .bind(format!("kan103-client-event-{suffix}"))
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
            45000,
            'github-actions',
            '{"authorization":"must-not-import"}'::jsonb
        )
        "#,
    )
    .bind(org_id)
    .bind(format!("pipe-kan103-{suffix}"))
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
            'sha256:kan103',
            '/evidence/packets/tickets/KAN-103',
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
    .bind(format!("approval-kan103-{suffix}"))
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
            'sha256:kan103',
            '/evidence/packets/tickets/KAN-103',
            'advisory',
            TRUE,
            FALSE,
            FALSE,
            'KAN-103 customer framework fixture',
            '["sonar_quality_gate"]'::jsonb,
            '["PR review evidence not present in KAN-99 export"]'::jsonb,
            'policy-kan103',
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
                "policy":{"source":"repo-file","checksum":"policy-kan103"}
            }'::jsonb,
            '{"metadata":{"token":"must-not-import","source_code":"must-not-import"}}'::jsonb,
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

fn customer_pack() -> serde_json::Value {
    json!({
        "schema_version": "gitgov_customer_framework_pack.v1",
        "framework": {
            "id": "bank_internal_release_controls",
            "name": "Bank Internal Release Controls",
            "version": "2026.06",
            "description": "Customer-owned internal controls for release evidence review.",
            "owner_name": "Customer Security Office",
            "compliance_claim": false,
            "regulatory_claim": false,
            "gitgov_certifies": false,
            "official_regulatory_mapping": false
        },
        "controls": [
            {
                "control_id": "BRC-DEPLOY-01",
                "title": "Deployment decision captured",
                "description": "Deployment authorization evidence must include the gate decision.",
                "required_evidence_types": ["deployment_gate.decision"]
            },
            {
                "control_id": "BRC-CI-02",
                "title": "Build evidence captured",
                "description": "Release review must include CI or build execution evidence.",
                "required_evidence_types": ["ci_build_evidence"]
            },
            {
                "control_id": "BRC-QUALITY-03",
                "title": "Quality gate evidence captured",
                "description": "Release review must call out quality gate evidence or gaps.",
                "required_evidence_types": ["quality_gate_result"]
            },
            {
                "control_id": "BRC-NOAGENT-04",
                "title": "Manual-first release path is recorded",
                "description": "Release review must show whether Agent Governance was used.",
                "required_evidence_types": ["deployment_gate.agent_governance_used"]
            }
        ]
    })
}

async fn create_customer_framework_agent_key(app: &axum::Router, admin_key: &str) -> String {
    let body = json!({
        "display_name": "kan103-framework-import-agent",
        "description": "KAN-103 integration test agent key",
        "environment": "staging",
        "allowed_actions": ["commit"]
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
        "agent key setup failed: {response}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("agent key JSON");
    parsed["token"].as_str().expect("agent token").to_string()
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
async fn customer_framework_pack_import_maps_real_export_and_review_package_without_claims() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "kan103-tenant-a").await;
    let org_b = insert_test_org(&pool, "kan103-tenant-b").await;
    let gate_a = seed_customer_framework_gate(&pool, &org_a, "tenant_a").await;
    let admin_a = insert_test_api_key_for_org(&pool, "kan103-admin-a", "Admin", &org_a).await;
    let admin_b = insert_test_api_key_for_org(&pool, "kan103-admin-b", "Admin", &org_b).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let before_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_a)
    .fetch_one(&pool)
    .await
    .expect("count evaluations before");

    let import_body = json!({
        "format": "json",
        "pack": customer_pack()
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&import_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "import failed: {response}");
    assert!(!response.contains("must-not-import"));
    let imported: serde_json::Value = serde_json::from_str(&response).expect("import JSON");
    let framework_id = imported["framework"]["framework_id"]
        .as_str()
        .expect("framework id");
    let framework_pack_id = imported["framework_pack"]["framework_pack_id"]
        .as_str()
        .expect("framework pack id");
    let pack_hash = imported["framework_pack"]["pack_hash"]
        .as_str()
        .expect("pack hash");
    assert!(framework_id.starts_with("customer_bank_internal_release_controls_"));
    assert!(framework_pack_id.starts_with("cfp_"));
    assert!(pack_hash.starts_with("sha256:"));
    assert_eq!(imported["framework_pack"]["owner_type"], "customer");
    assert_eq!(imported["framework_pack"]["source"], "customer_provided");
    assert_eq!(imported["framework_pack"]["control_count"], 4);
    assert_eq!(imported["framework_pack"]["compliance_claim"], false);
    assert_eq!(imported["framework_pack"]["regulatory_claim"], false);
    assert_eq!(imported["framework_pack"]["gitgov_certifies"], false);
    assert_eq!(
        imported["framework_pack"]["official_regulatory_mapping"],
        false
    );
    assert_eq!(imported["framework_pack"]["requires_auditor_review"], true);

    let (status, framework_list) = json_request(
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
        "framework list failed: {framework_list}"
    );
    assert!(framework_list.contains("gitgov_release_governance_baseline_v1"));
    assert!(framework_list.contains(framework_id));

    let (status, other_framework_list) = json_request(
        &app,
        "GET",
        "/compliance/control-frameworks",
        None,
        Some(&admin_b),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!other_framework_list.contains(framework_id));

    let (export_id, export_hash) = create_export(&app, &admin_a, &gate_a).await;
    let mapping_body = json!({
        "evidence_export_id": export_id,
        "framework_id": framework_id,
        "framework_version": "2026.06"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&mapping_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "mapping failed: {response}");
    assert!(!response.contains("must-not-import"));
    let mapped: serde_json::Value = serde_json::from_str(&response).expect("mapping JSON");
    let mapping_id = mapped["mapping"]["mapping_id"]
        .as_str()
        .expect("mapping id");
    assert_eq!(mapped["mapping"]["evidence_export_hash"], export_hash);
    assert_eq!(mapped["mapping"]["framework_id"], framework_id);
    assert_eq!(mapped["mapping"]["framework_version"], "2026.06");
    assert_eq!(mapped["mapping"]["compliance_claim"], false);
    assert_eq!(mapped["mapping"]["regulatory_claim"], false);
    assert_eq!(mapped["mapping"]["requires_auditor_review"], true);
    let items = mapped["items"].as_array().expect("mapping items");
    assert_eq!(items.len(), 4);
    assert_eq!(
        item_by_control(items, "BRC-DEPLOY-01")["status"],
        "evidence_present"
    );
    assert_eq!(
        item_by_control(items, "BRC-CI-02")["status"],
        "evidence_present"
    );
    assert_eq!(
        item_by_control(items, "BRC-QUALITY-03")["status"],
        "missing"
    );
    assert_eq!(
        item_by_control(items, "BRC-NOAGENT-04")["status"],
        "evidence_present"
    );

    let package_body = json!({
        "mapping_id": mapping_id,
        "format": "json"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/review-packages",
        Some(&package_body.to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "review package failed: {response}"
    );
    let package: serde_json::Value = serde_json::from_str(&response).expect("package JSON");
    assert_eq!(package["artifact"]["framework"]["owner_type"], "customer");
    assert_eq!(
        package["artifact"]["framework"]["owner"],
        "Customer Security Office"
    );
    assert_eq!(
        package["artifact"]["framework"]["source"],
        "customer_provided"
    );
    assert_eq!(package["artifact"]["framework"]["customer_provided"], true);
    assert_eq!(
        package["artifact"]["framework"]["framework_pack_id"],
        framework_pack_id
    );
    assert_eq!(package["artifact"]["framework"]["pack_hash"], pack_hash);
    assert_eq!(
        package["artifact"]["framework"]["official_regulatory_mapping"],
        false
    );
    assert_eq!(package["artifact"]["claims"]["compliance_claim"], false);
    assert_eq!(package["artifact"]["claims"]["regulatory_claim"], false);
    assert_eq!(package["artifact"]["claims"]["certification"], false);
    assert_eq!(package["artifact"]["summary"]["total_controls"], 4);

    let cross_tenant_mapping = json!({
        "evidence_export_id": export_id,
        "framework_id": framework_id
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/evidence-mappings",
        Some(&cross_tenant_mapping.to_string()),
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-tenant customer framework must not be usable: {response}"
    );

    let after_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_a)
    .fetch_one(&pool)
    .await
    .expect("count evaluations after");
    assert_eq!(
        after_evaluations, before_evaluations,
        "customer framework import/mapping must not create Agent Governance evaluations"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn customer_framework_pack_import_rejects_claims_secrets_reserved_ids_and_non_admins() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "kan103-validation").await;
    let admin =
        insert_test_api_key_for_org(&pool, "kan103-validation-admin", "Admin", &org_id).await;
    let developer =
        insert_test_api_key_for_org(&pool, "kan103-validation-dev", "Developer", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let yaml_pack = r#"
schema_version: gitgov_customer_framework_pack.v1
framework:
  id: bank_internal_yaml_controls
  name: Bank Internal YAML Controls
  version: "2026.06"
  description: Customer-owned internal controls imported from YAML.
  owner_name: Customer Security Office
  compliance_claim: false
  regulatory_claim: false
  gitgov_certifies: false
  official_regulatory_mapping: false
controls:
  - control_id: BRC-YAML-01
    title: Deployment decision captured from YAML
    description: Deployment authorization evidence must include the gate decision.
    required_evidence_types:
      - deployment_gate.decision
"#;
    let yaml_body = json!({
        "format": "yaml",
        "content": yaml_pack
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&yaml_body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "YAML import failed: {response}"
    );
    let imported_yaml: serde_json::Value =
        serde_json::from_str(&response).expect("YAML import JSON");
    assert_eq!(imported_yaml["framework_pack"]["owner_type"], "customer");
    assert_eq!(
        imported_yaml["framework_pack"]["source"],
        "customer_provided"
    );
    assert_eq!(imported_yaml["framework_pack"]["compliance_claim"], false);
    assert_eq!(imported_yaml["framework_pack"]["regulatory_claim"], false);
    assert_eq!(imported_yaml["framework_pack"]["gitgov_certifies"], false);
    assert_eq!(
        imported_yaml["framework_pack"]["official_regulatory_mapping"],
        false
    );
    assert_eq!(
        imported_yaml["framework_pack"]["requires_auditor_review"],
        true
    );

    let valid_body = json!({
        "format": "json",
        "pack": customer_pack()
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&valid_body.to_string()),
        Some(&developer),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer should not import framework packs: {response}"
    );

    let agent_token = create_customer_framework_agent_key(&app, &admin).await;
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&valid_body.to_string()),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key should not import framework packs: {response}"
    );
    assert!(response.contains("Agent key scope does not allow this request"));

    let (status, response) = json_request(
        &app,
        "GET",
        "/compliance/framework-packs",
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key should not list framework packs: {response}"
    );

    let mut claimed_pack = customer_pack();
    claimed_pack["framework"]["compliance_claim"] = json!(true);
    let body = json!({ "format": "json", "pack": claimed_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("compliance_claim"));

    let mut reserved_pack = customer_pack();
    reserved_pack["framework"]["id"] = json!("soc2_cc_customer");
    let body = json!({ "format": "json", "pack": reserved_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("reserved"));

    let mut bad_evidence_pack = customer_pack();
    bad_evidence_pack["controls"][0]["required_evidence_types"] = json!(["soc2_cc_mapping"]);
    let body = json!({ "format": "json", "pack": bad_evidence_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("unsupported evidence type"));

    let mut duplicate_pack = customer_pack();
    duplicate_pack["controls"][1]["control_id"] =
        duplicate_pack["controls"][0]["control_id"].clone();
    let body = json!({ "format": "json", "pack": duplicate_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("duplicate control_id"));

    let mut oversized_pack = customer_pack();
    let mut controls = Vec::new();
    for index in 0..51 {
        controls.push(json!({
            "control_id": format!("BRC-LIMIT-{index:02}"),
            "title": format!("Limit control {index}"),
            "description": "Customer-owned limit test control.",
            "required_evidence_types": ["deployment_gate.decision"]
        }));
    }
    oversized_pack["controls"] = json!(controls);
    let body = json!({ "format": "json", "pack": oversized_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("controls cannot exceed"));

    let mut secret_pack = customer_pack();
    secret_pack["metadata"] = json!({ "api_key": "ghp_must_not_be_here" });
    let body = json!({ "format": "json", "pack": secret_pack });
    let (status, response) = json_request(
        &app,
        "POST",
        "/compliance/framework-packs/import",
        Some(&body.to_string()),
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.contains("secret-like"));

    teardown(&admin_pool, &schema).await;
}
