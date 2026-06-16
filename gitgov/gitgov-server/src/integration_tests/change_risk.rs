use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

const REPO_FULL_NAME: &str = "risk-org/repo";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const EVIDENCE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

async fn insert_repo_for_org(pool: &sqlx::PgPool, org_id: &str, full_name: &str) {
    let repo_id = uuid::Uuid::new_v4().to_string();
    let repo_name = full_name
        .split('/')
        .next_back()
        .filter(|name| !name.is_empty())
        .unwrap_or(full_name);
    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(org_id)
    .bind(full_name)
    .bind(repo_name)
    .execute(pool)
    .await
    .expect("insert change risk repo");
}

async fn insert_gate(
    pool: &sqlx::PgPool,
    org_id: &str,
    authorization_id: &str,
    release_id: &str,
    environment: &str,
    decision: &str,
    break_glass_used: bool,
) {
    let (blocking, would_block, approved, blocked_by, warnings, required, valid, shared_decision) =
        match decision {
            "blocked" => (
                true,
                true,
                false,
                json!(["No valid release approval or accepted-risk record was found."]),
                json!([]),
                1,
                0,
                "requires_approval",
            ),
            "advisory" => (
                false,
                true,
                true,
                json!([]),
                json!(["No valid release approval or accepted-risk record was found."]),
                1,
                0,
                "requires_approval",
            ),
            "break_glass" => (false, true, true, json!([]), json!([]), 1, 0, "allowed"),
            _ => (false, false, true, json!([]), json!([]), 0, 0, "allowed"),
        };
    let governance_decision = json!({
        "contract_version": "shared-governance-decision.v1",
        "consumer_type": "deployment_gate",
        "decision": shared_decision,
        "agent_governance_used": false,
        "break_glass_used": break_glass_used,
        "evidence": {
            "required_evidence": ["deployment_context", "release_evidence_packet"],
            "available_evidence": ["deployment_context", "release_evidence_packet"],
            "missing_evidence": if required > valid { json!(["release_approval", "human_approval"]) } else { json!([]) },
            "evidence_packet_hash": EVIDENCE_HASH,
            "valid_approval_count": valid,
            "required_approval_count": required
        },
        "reason_codes": if required > valid { json!(["missing_release_approval"]) } else { json!([]) },
        "reasons": []
    });
    let details = json!({
        "contract_version": "deployment-gate-authorization.v1",
        "shared_governance_decision": governance_decision
    });
    let evaluation = json!({
        "status": if blocking { "blocked" } else { "approved" },
        "policy_satisfied": !blocking,
        "blocking": blocking,
        "would_block": would_block,
        "valid_approval_count": valid,
        "required_approval_count": required,
        "policy": {
            "mode": if required > 0 { "approval-required" } else { "record-only" },
            "environment": environment,
            "approval_required": required > 0,
            "enforcement": if required > 0 { "blocking" } else { "disabled" },
            "policy_applies": true,
            "quorum_enabled": false,
            "quorum_rules": []
        },
        "approvals": [],
        "issues": if required > valid { json!(["No valid release approval or accepted-risk record was found."]) } else { json!([]) },
        "next_steps": []
    });

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
            break_glass_eligible,
            break_glass_used,
            break_glass_reason,
            break_glass_authorized_by,
            break_glass_expires_at,
            break_glass_approval_id,
            break_glass_approval_hash,
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
            'KAN-121',
            $8,
            '/evidence/packets/tickets/KAN-121',
            $9,
            $10,
            $11,
            $12,
            'Seeded gate for KAN-121 change risk test',
            $13::jsonb,
            $14::jsonb,
            'policy-checksum',
            $15,
            $16,
            $17,
            $18,
            CASE WHEN $19::BIGINT IS NULL THEN NULL ELSE to_timestamp($19::DOUBLE PRECISION / 1000.0) END,
            $20,
            $21,
            $22::jsonb,
            $23::jsonb,
            '{}'::jsonb,
            'integration-test'
        )
        "#,
    )
    .bind(authorization_id)
    .bind(org_id)
    .bind(release_id)
    .bind(REPO_FULL_NAME)
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .bind(environment)
    .bind(EVIDENCE_HASH)
    .bind(decision)
    .bind(approved)
    .bind(blocking)
    .bind(would_block)
    .bind(blocked_by.to_string())
    .bind(warnings.to_string())
    .bind(blocking)
    .bind(break_glass_used)
    .bind(if break_glass_used {
        Some("Emergency production restore approved by incident commander.")
    } else {
        None
    })
    .bind(if break_glass_used { Some("incident@example.com") } else { None })
    .bind(if break_glass_used {
        Some(chrono::Utc::now().timestamp_millis() + 60 * 60 * 1000)
    } else {
        None
    })
    .bind(if break_glass_used { Some("dgbga_seeded") } else { None })
    .bind(if break_glass_used { Some("approval-hash") } else { None })
    .bind(evaluation.to_string())
    .bind(details.to_string())
    .execute(pool)
    .await
    .expect("insert seeded deployment gate");
}

async fn create_agent_key(app: &axum::Router, admin_key: &str) -> String {
    let body = json!({
        "display_name": "kan-121-agent-denied",
        "description": "KAN-121 negative test agent key",
        "environment": "production",
        "allowed_actions": ["deploy"]
    });
    let (status, response) = json_request(
        app,
        "POST",
        "/agent-governance/agent-keys",
        Some(&body.to_string()),
        Some(admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "agent key create: {response}");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("agent key JSON");
    parsed["token"].as_str().expect("agent token").to_string()
}

fn risk_payload(gate_id: Option<&str>, environment: &str) -> serde_json::Value {
    json!({
        "repository_full_name": REPO_FULL_NAME,
        "branch": BRANCH,
        "environment": environment,
        "change_id": "KAN-121-change",
        "deployment_gate_id": gate_id,
        "release_id": "release-kan-121",
        "commit_sha": TARGET_SHA,
        "evidence_refs": ["/evidence/packets/tickets/KAN-121"]
    })
}

#[tokio::test]
async fn change_risk_evaluates_gate_context_without_ai_agents_or_claims() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "risk-org").await;
    insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    let admin_key =
        insert_test_api_key_for_org(&pool, "kan-121-risk-admin", "Admin", &org_id).await;
    let developer_key =
        insert_test_api_key_for_org(&pool, "kan-121-risk-dev", "Developer", &org_id).await;
    let auditor_key =
        insert_test_api_key_for_org(&pool, "kan-122-risk-auditor", "Auditor", &org_id).await;
    insert_gate(
        &pool,
        &org_id,
        "dga_kan121_low",
        "release-kan-121",
        "staging",
        "approved",
        false,
    )
    .await;
    insert_gate(
        &pool,
        &org_id,
        "dga_kan121_high",
        "release-kan-121",
        "production",
        "blocked",
        false,
    )
    .await;
    insert_gate(
        &pool,
        &org_id,
        "dga_kan121_breakglass",
        "release-kan-121",
        "production",
        "break_glass",
        true,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_low"), "staging").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "low risk create: {response}");
    let low: serde_json::Value = serde_json::from_str(&response).expect("low risk JSON");
    assert_eq!(low["risk_level"], "low");
    assert_eq!(low["ruleset_version"], "change_risk_rules.v1");
    assert!(low["trace_hash"]
        .as_str()
        .expect("low trace hash")
        .starts_with("sha256:"));
    assert_eq!(low["advisory_only"], true);
    assert_eq!(low["llm_used"], false);
    assert_eq!(low["agent_governance_used"], false);
    assert_eq!(low["compliance_claim"], false);
    assert_eq!(low["certification"], false);
    assert!(low["risk_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "deployment_gate_allowed"));
    assert_eq!(low["triggered_rules"].as_array().unwrap().len(), 0);
    assert_eq!(
        low["evaluation_trace"]["ruleset_version"],
        "change_risk_rules.v1"
    );
    assert_eq!(low["evaluation_trace"]["advisory_only"], true);
    assert_eq!(low["evaluation_trace"]["llm_used"], false);
    assert_eq!(low["evaluation_trace"]["agent_governance_used"], false);
    let low_id = low["evaluation_id"].as_str().expect("low eval id");
    let low_trace_hash = low["trace_hash"]
        .as_str()
        .expect("low trace hash")
        .to_string();

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_low"), "staging").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "repeat low risk create: {response}"
    );
    let repeat_low: serde_json::Value =
        serde_json::from_str(&response).expect("repeat low risk JSON");
    assert_eq!(repeat_low["trace_hash"], low_trace_hash);

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_high"), "production").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "high risk create: {response}");
    let high: serde_json::Value = serde_json::from_str(&response).expect("high risk JSON");
    assert_eq!(high["risk_level"], "high");
    assert_eq!(high["ruleset_version"], "change_risk_rules.v1");
    assert!(high["risk_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "deployment_gate_blocked"));
    for expected_rule in [
        "missing_release_approval",
        "production_environment",
        "gate_requires_approval",
        "gate_blocked",
        "insufficient_evidence",
    ] {
        assert!(
            high["triggered_rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == expected_rule),
            "expected high risk triggered rule {expected_rule}: {high:?}"
        );
    }
    assert!(high["missing_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "release_approval"));

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_breakglass"), "production").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "break glass risk create: {response}"
    );
    let break_glass: serde_json::Value =
        serde_json::from_str(&response).expect("break glass risk JSON");
    assert_eq!(break_glass["risk_level"], "high");
    assert!(break_glass["risk_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "break_glass_involved"));
    assert!(break_glass["triggered_rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "break_glass_involved"));

    let (status, response) =
        json_request(&app, "GET", "/change-risk/rules", None, Some(&admin_key)).await;
    assert_eq!(status, StatusCode::OK, "get rules: {response}");
    let rules: serde_json::Value = serde_json::from_str(&response).expect("rules JSON");
    assert_eq!(rules["ruleset_version"], "change_risk_rules.v1");
    assert_eq!(rules["advisory_only"], true);
    assert_eq!(rules["llm_used"], false);
    assert_eq!(rules["agent_governance_used"], false);
    assert_eq!(rules["compliance_claim"], false);
    assert_eq!(rules["certification"], false);
    assert!(rules["catalog_hash"]
        .as_str()
        .expect("catalog hash")
        .starts_with("sha256:"));
    assert_eq!(rules["rules"].as_array().unwrap().len(), 12);
    assert!(rules["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| rule["rule_id"] == "missing_ci_evidence"));

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{low_id}"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get low risk: {response}");
    let fetched_low: serde_json::Value = serde_json::from_str(&response).expect("fetched low JSON");
    assert_eq!(fetched_low["trace_hash"], low_trace_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{low_id}/trace"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get low trace: {response}");
    let low_trace: serde_json::Value = serde_json::from_str(&response).expect("trace JSON");
    assert_eq!(low_trace["ruleset_version"], "change_risk_rules.v1");
    assert_eq!(low_trace["trace_hash"], low_trace_hash);
    assert_eq!(low_trace["evaluation_trace"]["risk_level"], "low");
    assert_eq!(low_trace["advisory_only"], true);

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?deployment_gate_id=dga_kan121_high",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list high risk: {response}");
    let listed: serde_json::Value = serde_json::from_str(&response).expect("list JSON");
    assert_eq!(listed["total"], 1);

    for path in [
        "/change-risk/rules".to_string(),
        format!("/change-risk/evaluations/{low_id}"),
        format!("/change-risk/evaluations/{low_id}/trace"),
        "/change-risk/evaluations?deployment_gate_id=dga_kan121_high".to_string(),
    ] {
        let (status, response) = json_request(&app, "GET", &path, None, Some(&auditor_key)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "auditor read-only endpoint failed at {path}: {response}"
        );
    }

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_low"), "staging").to_string()),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not create change risk: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{low_id}/trace"),
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not read change risk trace: {response}"
    );

    let agent_token = create_agent_key(&app, &admin_key).await;
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_low"), "staging").to_string()),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not call change risk: {response}"
    );
    let (status, response) =
        json_request(&app, "GET", "/change-risk/rules", None, Some(&agent_token)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not read change risk rules: {response}"
    );

    let agent_eval_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("agent governance eval count");
    assert_eq!(agent_eval_count, 0);

    let persisted_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_evaluations WHERE org_id = $1::uuid")
            .bind(&org_id)
            .fetch_one(&pool)
            .await
            .expect("change risk persisted count");
    assert_eq!(persisted_count, 4);

    let gate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deployment_gate_authorizations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("deployment gate unchanged count");
    assert_eq!(gate_count, 3);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn change_risk_is_tenant_scoped_and_handles_missing_context_advisory() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "risk-tenant-a").await;
    let org_b = insert_test_org(&pool, "risk-tenant-b").await;
    insert_repo_for_org(&pool, &org_a, REPO_FULL_NAME).await;
    insert_gate(
        &pool,
        &org_a,
        "dga_kan121_tenant_a",
        "release-kan-121",
        "staging",
        "approved",
        false,
    )
    .await;
    let admin_a =
        insert_test_api_key_for_org(&pool, "kan-121-tenant-a-admin", "Admin", &org_a).await;
    let admin_b =
        insert_test_api_key_for_org(&pool, "kan-121-tenant-b-admin", "Admin", &org_b).await;
    let auditor_b =
        insert_test_api_key_for_org(&pool, "kan-122-tenant-b-auditor", "Auditor", &org_b).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_tenant_a"), "staging").to_string()),
        Some(&admin_a),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "tenant A create: {response}");
    let created: serde_json::Value = serde_json::from_str(&response).expect("created JSON");
    let eval_id = created["evaluation_id"].as_str().expect("eval id");

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{eval_id}"),
        None,
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "tenant B must not read tenant A risk: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{eval_id}/trace"),
        None,
        Some(&auditor_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "tenant B auditor must not read tenant A trace: {response}"
    );

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan121_tenant_a"), "staging").to_string()),
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "tenant B must not evaluate tenant A gate: {response}"
    );

    let missing_context = json!({
        "repository_full_name": "risk-tenant-b/repo",
        "branch": "main",
        "environment": "production"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&missing_context.to_string()),
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "missing context risk: {response}"
    );
    let missing: serde_json::Value = serde_json::from_str(&response).expect("missing context JSON");
    assert_eq!(missing["risk_level"], "medium");
    assert_eq!(missing["ruleset_version"], "change_risk_rules.v1");
    for expected_rule in [
        "missing_ci_evidence",
        "missing_code_review",
        "missing_change_link",
        "production_environment",
        "stale_evidence",
        "insufficient_evidence",
    ] {
        assert!(
            missing["triggered_rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == expected_rule),
            "expected missing-context triggered rule {expected_rule}: {missing:?}"
        );
    }
    assert!(missing["missing_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "deployment_gate_authorization"));
    assert!(missing["missing_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "release_evidence_packet"));
    assert_eq!(missing["advisory_only"], true);
    assert_eq!(missing["llm_used"], false);
    assert_eq!(missing["agent_governance_used"], false);
    assert_eq!(missing["compliance_claim"], false);
    assert_eq!(missing["certification"], false);

    let tenant_b_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_evaluations WHERE org_id = $1::uuid")
            .bind(&org_b)
            .fetch_one(&pool)
            .await
            .expect("tenant B risk count");
    assert_eq!(tenant_b_count, 1);

    teardown(&admin_pool, &schema).await;
}
