use super::common::*;
use crate::db::Database;
use axum::http::StatusCode;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const REPO_FULL_NAME: &str = "risk-org/repo";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const EVIDENCE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn canonical_json_hash(value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).expect("canonical JSON serialization");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

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
    insert_gate_for_repo(
        pool,
        org_id,
        REPO_FULL_NAME,
        BRANCH,
        TARGET_SHA,
        EVIDENCE_HASH,
        authorization_id,
        release_id,
        environment,
        decision,
        break_glass_used,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn insert_gate_for_repo(
    pool: &sqlx::PgPool,
    org_id: &str,
    repository_full_name: &str,
    branch: &str,
    target_sha: &str,
    evidence_hash: &str,
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
            "evidence_packet_hash": evidence_hash,
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
    .bind(repository_full_name)
    .bind(branch)
    .bind(target_sha)
    .bind(environment)
    .bind(evidence_hash)
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

fn risk_payload_for_repo(
    repository_full_name: &str,
    branch: &str,
    target_sha: &str,
    gate_id: Option<&str>,
    change_id: &str,
    release_id: &str,
    environment: &str,
) -> serde_json::Value {
    json!({
        "repository_full_name": repository_full_name,
        "branch": branch,
        "environment": environment,
        "change_id": change_id,
        "deployment_gate_id": gate_id,
        "release_id": release_id,
        "commit_sha": target_sha,
        "evidence_refs": [format!("/evidence/packets/tickets/{change_id}")]
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
    let high_id = high["evaluation_id"].as_str().expect("high eval id");
    let high_trace_hash = high["trace_hash"]
        .as_str()
        .expect("high trace hash")
        .to_string();
    assert_eq!(high["risk_level"], "high");
    assert_eq!(high["review_status"], "needs_review");
    assert_eq!(high["reviewed_by_user_id"], serde_json::Value::Null);
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
        &format!("/change-risk/evaluations/{high_id}/review"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get high review: {response}");
    let initial_review: serde_json::Value =
        serde_json::from_str(&response).expect("initial review JSON");
    assert_eq!(initial_review["review_status"], "needs_review");
    assert_eq!(initial_review["risk_level"], "high");
    assert_eq!(initial_review["trace_hash"], high_trace_hash);
    assert_eq!(initial_review["advisory_only"], true);
    assert_eq!(initial_review["llm_used"], false);
    assert_eq!(initial_review["agent_governance_used"], false);
    assert_eq!(initial_review["compliance_claim"], false);
    assert_eq!(initial_review["certification"], false);

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?review_status=needs_review&limit=20",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "list needs_review queue before review: {response}"
    );
    let needs_review_before: serde_json::Value =
        serde_json::from_str(&response).expect("needs_review queue before JSON");
    assert!(needs_review_before["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["evaluation_id"] == high_id));

    let review_payload = json!({
        "review_status": "reviewed",
        "review_notes": "CAB reviewed deterministic risk trace and rollback owner.",
        "mitigation_notes": "Rollback plan confirmed for production release.",
        "decision_reason": "Manual review completed without changing risk evidence."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&review_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "review high risk: {response}");
    let reviewed: serde_json::Value = serde_json::from_str(&response).expect("reviewed JSON");
    assert_eq!(reviewed["review_status"], "reviewed");
    assert_eq!(reviewed["reviewed_by_user_id"], "kan-121-risk-admin");
    assert_eq!(
        reviewed["review_notes_safe"],
        "CAB reviewed deterministic risk trace and rollback owner."
    );
    assert_eq!(reviewed["risk_level"], "high");
    assert_eq!(reviewed["ruleset_version"], "change_risk_rules.v1");
    assert_eq!(reviewed["trace_hash"], high_trace_hash);
    assert_eq!(reviewed["advisory_only"], true);
    assert_eq!(reviewed["agent_governance_used"], false);

    let accepted_payload = json!({
        "review_status": "accepted_risk",
        "review_notes": "Risk accepted by release owner after manual review.",
        "mitigation_notes": "Monitor deployment and keep rollback owner online.",
        "decision_reason": "Business exception accepted for KAN-123 validation."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&accepted_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "accept high risk: {response}");
    let accepted: serde_json::Value = serde_json::from_str(&response).expect("accepted JSON");
    assert_eq!(accepted["review_status"], "accepted_risk");
    assert_eq!(
        accepted["mitigation_notes_safe"],
        "Monitor deployment and keep rollback owner online."
    );
    assert_eq!(
        accepted["decision_reason_safe"],
        "Business exception accepted for KAN-123 validation."
    );
    assert_eq!(accepted["trace_hash"], high_trace_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?review_status=needs_review&limit=20",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "list needs_review queue after accepted risk: {response}"
    );
    let needs_review_after: serde_json::Value =
        serde_json::from_str(&response).expect("needs_review queue after JSON");
    assert!(!needs_review_after["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["evaluation_id"] == high_id));

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?review_status=accepted_risk&limit=20",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "list accepted_risk queue: {response}"
    );
    let accepted_queue: serde_json::Value =
        serde_json::from_str(&response).expect("accepted_risk queue JSON");
    assert!(accepted_queue["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["evaluation_id"] == high_id));

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?review_status=approved",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "invalid review queue status must be rejected: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{high_id}"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get reviewed high risk: {response}");
    let fetched_high: serde_json::Value =
        serde_json::from_str(&response).expect("fetched high JSON");
    assert_eq!(fetched_high["risk_level"], "high");
    assert_eq!(fetched_high["ruleset_version"], "change_risk_rules.v1");
    assert_eq!(fetched_high["trace_hash"], high_trace_hash);
    assert_eq!(fetched_high["review_status"], "accepted_risk");

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/evaluations/{high_id}/trace"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "trace after review must load: {response}"
    );
    let high_trace_after_review: serde_json::Value =
        serde_json::from_str(&response).expect("high trace after review JSON");
    assert_eq!(high_trace_after_review["trace_hash"], high_trace_hash);

    let unsafe_review_payload = json!({
        "review_status": "reviewed",
        "review_notes": "Authorization: Bearer should-not-store",
        "mitigation_notes": "plain",
        "decision_reason": "plain"
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&unsafe_review_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "secret-like review notes must be rejected: {response}"
    );

    let review_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_review_updated' AND target_id = $1",
    )
    .bind(high_id)
    .fetch_one(&pool)
    .await
    .expect("change risk review audit count");
    assert_eq!(review_audit_count, 2);

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
        format!("/change-risk/evaluations/{high_id}/review"),
        "/change-risk/evaluations?deployment_gate_id=dga_kan121_high".to_string(),
        "/change-risk/evaluations?review_status=accepted_risk".to_string(),
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
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&review_payload.to_string()),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not update change risk review: {response}"
    );
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&review_payload.to_string()),
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "auditor remains read-only for change risk review in KAN-123: {response}"
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
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&review_payload.to_string()),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not update change risk review: {response}"
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
        "GET",
        &format!("/change-risk/evaluations/{eval_id}/review"),
        None,
        Some(&auditor_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "tenant B auditor must not read tenant A review: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/evaluations?review_status=needs_review&limit=20",
        None,
        Some(&auditor_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "tenant B auditor can list its own empty review queue: {response}"
    );
    let tenant_b_queue: serde_json::Value =
        serde_json::from_str(&response).expect("tenant B review queue JSON");
    assert!(!tenant_b_queue["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["evaluation_id"] == eval_id));

    let tenant_b_review_payload = json!({
        "review_status": "reviewed",
        "review_notes": "Wrong tenant should not update this review."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{eval_id}/review"),
        Some(&tenant_b_review_payload.to_string()),
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "tenant B admin must not update tenant A review: {response}"
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

    let ci_review_context = json!({
        "repository_full_name": "risk-tenant-b/repo",
        "branch": "main",
        "environment": "production",
        "change_id": "KAN-122-ci-ref",
        "release_id": "KAN-122-ci-ref",
        "commit_sha": TARGET_SHA,
        "evidence_refs": [
            "https://github.com/yohandry10/Git-Gov/actions/runs/27591470098",
            "https://github.com/yohandry10/Git-Gov/pull/426"
        ]
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&ci_review_context.to_string()),
        Some(&admin_b),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "ci/review referenced risk: {response}"
    );
    let ci_review: serde_json::Value =
        serde_json::from_str(&response).expect("ci/review context JSON");
    for absent_rule in [
        "missing_ci_evidence",
        "missing_code_review",
        "missing_change_link",
    ] {
        assert!(
            !ci_review["triggered_rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == absent_rule),
            "real CI/PR evidence should avoid {absent_rule}: {ci_review:?}"
        );
    }
    assert!(ci_review["triggered_rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "production_environment"));

    let tenant_b_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_evaluations WHERE org_id = $1::uuid")
            .bind(&org_b)
            .fetch_one(&pool)
            .await
            .expect("tenant B risk count");
    assert_eq!(tenant_b_count, 2);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn multi_repo_executive_governance_view_is_read_only_and_tenant_scoped() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "executive-org").await;
    let other_org_id = insert_test_org(&pool, "executive-other").await;
    let repo_a = "executive-org/payments";
    let repo_b = "executive-org/portal";
    let other_repo = "executive-other/repo";
    insert_repo_for_org(&pool, &org_id, repo_a).await;
    insert_repo_for_org(&pool, &org_id, repo_b).await;
    insert_repo_for_org(&pool, &other_org_id, other_repo).await;

    let admin_key =
        insert_test_api_key_for_org(&pool, "kan-129-exec-admin", "Admin", &org_id).await;
    let auditor_key =
        insert_test_api_key_for_org(&pool, "kan-129-exec-auditor", "Auditor", &org_id).await;
    let developer_key =
        insert_test_api_key_for_org(&pool, "kan-129-exec-dev", "Developer", &org_id).await;
    let other_admin_key =
        insert_test_api_key_for_org(&pool, "kan-129-other-admin", "Admin", &other_org_id).await;

    insert_gate_for_repo(
        &pool,
        &org_id,
        repo_a,
        "main",
        "1111111111111111111111111111111111111111",
        "1111111111111111111111111111111111111111111111111111111111111111",
        "dga_kan129_payments",
        "release-kan-129-payments",
        "production",
        "blocked",
        false,
    )
    .await;
    insert_gate_for_repo(
        &pool,
        &org_id,
        repo_b,
        "main",
        "2222222222222222222222222222222222222222",
        "2222222222222222222222222222222222222222222222222222222222222222",
        "dga_kan129_portal",
        "release-kan-129-portal",
        "staging",
        "approved",
        false,
    )
    .await;
    insert_gate_for_repo(
        &pool,
        &other_org_id,
        other_repo,
        "main",
        "3333333333333333333333333333333333333333",
        "3333333333333333333333333333333333333333333333333333333333333333",
        "dga_kan129_other",
        "release-kan-129-other",
        "production",
        "approved",
        false,
    )
    .await;

    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(
            &risk_payload_for_repo(
                repo_a,
                "main",
                "1111111111111111111111111111111111111111",
                Some("dga_kan129_payments"),
                "KAN-129-payments",
                "release-kan-129-payments",
                "production",
            )
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "payments risk create: {response}"
    );
    let payments_risk: serde_json::Value =
        serde_json::from_str(&response).expect("payments risk JSON");
    let payments_eval_id = payments_risk["evaluation_id"]
        .as_str()
        .expect("payments eval id")
        .to_string();
    assert_eq!(payments_risk["risk_level"], "high");

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{payments_eval_id}/review"),
        Some(
            &json!({
                "review_status": "accepted_risk",
                "review_notes": "Executive review accepts the production risk manually.",
                "mitigation_notes": "Release manager keeps rollback owner online.",
                "decision_reason": "Business owner accepted residual risk."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "payments risk review: {response}");

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(
            &risk_payload_for_repo(
                repo_b,
                "main",
                "2222222222222222222222222222222222222222",
                Some("dga_kan129_portal"),
                "KAN-129-portal",
                "release-kan-129-portal",
                "staging",
            )
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "portal risk create: {response}"
    );
    let portal_risk: serde_json::Value = serde_json::from_str(&response).expect("portal risk JSON");
    assert_eq!(portal_risk["risk_level"], "low");

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(
            &json!({
                "name": "KAN-129 executive payments CAB",
                "evaluation_ids": [payments_eval_id]
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "executive CAB packet: {response}"
    );
    let packet: serde_json::Value = serde_json::from_str(&response).expect("packet JSON");
    let packet_id = packet["packet"]["packet_id"]
        .as_str()
        .expect("packet id")
        .to_string();

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{packet_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes": "CAB reviewed the payments risk packet.",
                "mitigation_notes": "No additional mitigation requested.",
                "decision_reason": "Ready for executive visibility."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "executive CAB review: {response}");

    let (status, response) = json_request(
        &app,
        "POST",
        &format!("/change-risk/cab-packets/{packet_id}/decision-manifests"),
        Some(r#"{"org_name":"executive-org"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "executive CAB manifest: {response}"
    );
    let manifest: serde_json::Value = serde_json::from_str(&response).expect("manifest JSON");
    let manifest_hash = manifest["manifest"]["manifest_hash"]
        .as_str()
        .expect("manifest hash")
        .to_string();

    let before_gate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deployment_gate_authorizations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("before executive gate count");
    let before_risk_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_evaluations WHERE org_id = $1::uuid")
            .bind(&org_id)
            .fetch_one(&pool)
            .await
            .expect("before executive risk count");
    let before_packet_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_cab_packets WHERE org_id = $1::uuid")
            .bind(&org_id)
            .fetch_one(&pool)
            .await
            .expect("before executive packet count");
    let before_manifest_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM change_risk_cab_decision_manifests WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("before executive manifest count");
    let before_agent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("before executive agent count");

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor executive repo view: {response}"
    );
    let executive: serde_json::Value =
        serde_json::from_str(&response).expect("executive repo JSON");
    assert_eq!(executive["advisory_only"], true);
    assert_eq!(executive["enforcement_used"], false);
    assert_eq!(executive["deployment_execution"], false);
    assert_eq!(executive["provider_mutation"], false);
    assert_eq!(executive["repository_mutation"], false);
    assert_eq!(executive["llm_used"], false);
    assert_eq!(executive["agent_governance_used"], false);
    assert_eq!(executive["compliance_claim"], false);
    assert_eq!(executive["certification"], false);
    assert_eq!(executive["totals"]["repositories"], 2);
    assert_eq!(executive["totals"]["gate_count"], 2);
    assert_eq!(executive["totals"]["change_risk_count"], 2);
    assert_eq!(executive["totals"]["cab_packet_count"], 1);
    assert_eq!(executive["totals"]["cab_manifest_count"], 1);

    let repos = executive["repositories"].as_array().expect("repo array");
    let payments = repos
        .iter()
        .find(|repo| repo["repository_full_name"] == repo_a)
        .expect("payments repo summary");
    assert_eq!(payments["posture"], "attention");
    assert_eq!(payments["gate_count"], 1);
    assert_eq!(payments["blocked_gate_count"], 1);
    assert_eq!(payments["change_risk_count"], 1);
    assert_eq!(payments["high_risk_count"], 1);
    assert_eq!(payments["cab_packet_count"], 1);
    assert_eq!(payments["cab_manifest_count"], 1);
    assert_eq!(payments["active_manifest_count"], 1);
    assert_eq!(payments["latest_gate_id"], "dga_kan129_payments");
    assert_eq!(payments["latest_risk_level"], "high");
    assert_eq!(payments["latest_review_status"], "accepted_risk");
    assert_eq!(payments["latest_manifest_hash"], manifest_hash);
    let portal = repos
        .iter()
        .find(|repo| repo["repository_full_name"] == repo_b)
        .expect("portal repo summary");
    assert_eq!(portal["posture"], "review");
    assert_eq!(portal["gate_count"], 1);
    assert_eq!(portal["blocked_gate_count"], 0);
    assert_eq!(portal["advisory_gate_count"], 0);
    assert_eq!(portal["high_risk_count"], 0);
    assert_eq!(portal["needs_review_count"], 1);
    assert_eq!(portal["latest_review_status"], "needs_review");
    assert_eq!(portal["cab_packet_count"], 0);

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&posture=attention&environment=production&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "attention production filter: {response}"
    );
    let filtered: serde_json::Value =
        serde_json::from_str(&response).expect("attention filter JSON");
    assert_eq!(filtered["totals"]["repositories"], 1);
    assert_eq!(
        filtered["repositories"][0]["repository_full_name"],
        "executive-org/payments"
    );
    assert_eq!(filtered["repositories"][0]["posture"], "attention");
    assert_eq!(
        filtered["repositories"][0]["latest_gate_decision"],
        "blocked"
    );
    assert_eq!(filtered["repositories"][0]["latest_risk_level"], "high");

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&environment=staging&review_status=needs_review&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "staging needs-review filter: {response}"
    );
    let staging: serde_json::Value = serde_json::from_str(&response).expect("staging filter JSON");
    assert_eq!(staging["totals"]["repositories"], 1);
    assert_eq!(
        staging["repositories"][0]["repository_full_name"],
        "executive-org/portal"
    );
    assert_eq!(staging["repositories"][0]["latest_risk_level"], "low");
    assert_eq!(
        staging["repositories"][0]["latest_review_status"],
        "needs_review"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&gate_decision=blocked&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "blocked gate filter: {response}");
    let blocked: serde_json::Value = serde_json::from_str(&response).expect("blocked filter JSON");
    assert_eq!(blocked["totals"]["repositories"], 1);
    assert_eq!(
        blocked["repositories"][0]["repository_full_name"],
        "executive-org/payments"
    );
    assert_eq!(blocked["repositories"][0]["gate_count"], 1);

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&repository=portal&risk_level=low&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "repository/risk filter: {response}");
    let low_risk: serde_json::Value =
        serde_json::from_str(&response).expect("low risk filter JSON");
    assert_eq!(low_risk["totals"]["repositories"], 1);
    assert_eq!(
        low_risk["repositories"][0]["repository_full_name"],
        "executive-org/portal"
    );
    assert_eq!(low_risk["repositories"][0]["change_risk_count"], 1);

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&gate_decision=blocked&risk_level=low&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "conflicting filters: {response}");
    let empty_filter: serde_json::Value =
        serde_json::from_str(&response).expect("empty filter JSON");
    assert_eq!(empty_filter["totals"]["repositories"], 0);
    assert!(empty_filter["repositories"].as_array().unwrap().is_empty());

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org&posture=critical",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "invalid posture must fail closed: {response}"
    );

    let (status, response) = json_request(
        &app,
        "POST",
        "/executive/snapshots",
        Some(
            &json!({
                "org_name": "executive-org",
                "name": "KAN-131 unfiltered executive snapshot",
                "filters": {
                    "limit": 10,
                    "offset": 0
                },
                "include_repository_rows": true,
                "include_summary": true
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "unfiltered executive snapshot create: {response}"
    );
    let unfiltered_snapshot: serde_json::Value =
        serde_json::from_str(&response).expect("unfiltered snapshot JSON");
    assert_eq!(unfiltered_snapshot["snapshot"]["repository_count"], 2);
    assert_eq!(
        unfiltered_snapshot["artifact"]["schema_version"],
        "gitgov_executive_governance_snapshot.v1"
    );
    assert_eq!(unfiltered_snapshot["artifact"]["flags"]["read_only"], true);
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["manual_first"],
        true
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["advisory_only"],
        true
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["enforcement_used"],
        false
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["deployment_execution"],
        false
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["provider_mutation"],
        false
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["repository_mutation"],
        false
    );
    assert_eq!(unfiltered_snapshot["artifact"]["flags"]["llm_used"], false);
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["agent_governance_used"],
        false
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["compliance_claim"],
        false
    );
    assert_eq!(
        unfiltered_snapshot["artifact"]["flags"]["certification"],
        false
    );

    let (status, response) = json_request(
        &app,
        "POST",
        "/executive/snapshots",
        Some(
            &json!({
                "org_name": "executive-org",
                "name": "KAN-131 production attention snapshot",
                "filters": {
                    "environment": "production",
                    "posture": "attention",
                    "gate_decision": "blocked",
                    "risk_level": "high",
                    "review_status": "accepted_risk",
                    "limit": 10,
                    "offset": 0
                },
                "include_repository_rows": true,
                "include_summary": true
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "filtered executive snapshot create: {response}"
    );
    let filtered_snapshot: serde_json::Value =
        serde_json::from_str(&response).expect("filtered snapshot JSON");
    let snapshot_id = filtered_snapshot["snapshot"]["snapshot_id"]
        .as_str()
        .expect("snapshot id")
        .to_string();
    let snapshot_hash = filtered_snapshot["snapshot"]["artifact_hash"]
        .as_str()
        .expect("snapshot hash")
        .to_string();
    assert!(snapshot_id.starts_with("egs_"));
    assert_eq!(filtered_snapshot["snapshot"]["repository_count"], 1);
    assert_eq!(
        filtered_snapshot["artifact"]["repositories"][0]["repository_full_name"],
        "executive-org/payments"
    );
    assert_eq!(
        filtered_snapshot["artifact"]["filters"]["environment"],
        "production"
    );
    assert_eq!(
        filtered_snapshot["artifact"]["filters"]["posture"],
        "attention"
    );
    assert_eq!(filtered_snapshot["artifact"]["summary"]["repositories"], 1);

    let mut snapshot_preimage = filtered_snapshot["artifact"].clone();
    snapshot_preimage["artifact_hash"] = serde_json::Value::Null;
    assert_eq!(canonical_json_hash(&snapshot_preimage), snapshot_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/snapshots?org_name=executive-org&status=active&limit=10",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor list snapshots: {response}");
    let snapshot_list: serde_json::Value =
        serde_json::from_str(&response).expect("snapshot list JSON");
    assert_eq!(snapshot_list["total"], 2);
    assert!(snapshot_list["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["snapshot_id"] == snapshot_id));

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/executive/snapshots/{snapshot_id}?org_name=executive-org"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor get snapshot: {response}");
    let fetched_snapshot: serde_json::Value =
        serde_json::from_str(&response).expect("fetched snapshot JSON");
    assert_eq!(fetched_snapshot["snapshot"]["artifact_hash"], snapshot_hash);
    assert_eq!(
        fetched_snapshot["artifact"]["repositories"][0]["repository_full_name"],
        "executive-org/payments"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/executive/snapshots/{snapshot_id}/download?org_name=executive-org"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor download snapshot: {response}"
    );
    let downloaded_snapshot: serde_json::Value =
        serde_json::from_str(&response).expect("downloaded snapshot JSON");
    assert_eq!(downloaded_snapshot["artifact_hash"], snapshot_hash);
    let mut downloaded_preimage = downloaded_snapshot.clone();
    downloaded_preimage["artifact_hash"] = serde_json::Value::Null;
    assert_eq!(canonical_json_hash(&downloaded_preimage), snapshot_hash);
    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM executive_governance_snapshots WHERE snapshot_id = $1",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("snapshot download count");
    assert_eq!(download_count, 1);

    let (status, response) = json_request(
        &app,
        "POST",
        "/executive/snapshots",
        Some(
            &json!({
                "org_name": "executive-org",
                "name": "Developer should not create executive snapshot",
                "filters": {"limit": 10}
            })
            .to_string(),
        ),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not create snapshot: {response}"
    );

    let agent_token = create_agent_key(&app, &admin_key).await;
    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/snapshots?org_name=executive-org",
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not list executive snapshots: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/snapshots?org_name=executive-org",
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "other tenant must not list executive snapshots: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/executive/snapshots/{snapshot_id}/archive"),
        Some(r#"{"org_name":"executive-org","name":"archive"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "archive snapshot: {response}");
    let archived_snapshot: serde_json::Value =
        serde_json::from_str(&response).expect("archived snapshot JSON");
    assert_eq!(archived_snapshot["snapshot"]["status"], "archived");
    assert_eq!(
        archived_snapshot["snapshot"]["artifact_hash"],
        snapshot_hash
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/executive/snapshots/{snapshot_id}/download?org_name=executive-org"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "archived snapshot download must be blocked: {response}"
    );

    let after_gate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deployment_gate_authorizations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("after executive gate count");
    let after_risk_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_evaluations WHERE org_id = $1::uuid")
            .bind(&org_id)
            .fetch_one(&pool)
            .await
            .expect("after executive risk count");
    let after_packet_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM change_risk_cab_packets WHERE org_id = $1::uuid")
            .bind(&org_id)
            .fetch_one(&pool)
            .await
            .expect("after executive packet count");
    let after_manifest_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM change_risk_cab_decision_manifests WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("after executive manifest count");
    let after_agent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("after executive agent count");
    assert_eq!(after_gate_count, before_gate_count);
    assert_eq!(after_risk_count, before_risk_count);
    assert_eq!(after_packet_count, before_packet_count);
    assert_eq!(after_manifest_count, before_manifest_count);
    assert_eq!(after_agent_count, before_agent_count);

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-org",
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not read executive governance view: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        "/executive/repositories?org_name=executive-other",
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "other tenant executive view: {response}"
    );
    let other_view: serde_json::Value =
        serde_json::from_str(&response).expect("other executive JSON");
    assert_eq!(other_view["totals"]["repositories"], 1);
    assert!(other_view["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .all(|repo| repo["repository_full_name"] != repo_a));

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "risk-cab-org").await;
    let other_org_id = insert_test_org(&pool, "risk-cab-other").await;
    insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    insert_repo_for_org(&pool, &other_org_id, "risk-cab-other/repo").await;
    let admin_key = insert_test_api_key_for_org(&pool, "kan-125-cab-admin", "Admin", &org_id).await;
    let auditor_key =
        insert_test_api_key_for_org(&pool, "kan-125-cab-auditor", "Auditor", &org_id).await;
    let developer_key =
        insert_test_api_key_for_org(&pool, "kan-125-cab-dev", "Developer", &org_id).await;
    let other_admin_key =
        insert_test_api_key_for_org(&pool, "kan-125-other-admin", "Admin", &other_org_id).await;
    insert_gate(
        &pool,
        &org_id,
        "dga_kan125_low",
        "release-kan-121",
        "staging",
        "approved",
        false,
    )
    .await;
    insert_gate(
        &pool,
        &org_id,
        "dga_kan125_high",
        "release-kan-121",
        "production",
        "blocked",
        false,
    )
    .await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan125_low"), "staging").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "low eval create: {response}");
    let low: serde_json::Value = serde_json::from_str(&response).expect("low eval JSON");
    let low_id = low["evaluation_id"]
        .as_str()
        .expect("low eval id")
        .to_string();
    assert_eq!(low["risk_level"], "low");

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&risk_payload(Some("dga_kan125_high"), "production").to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "high eval create: {response}");
    let high: serde_json::Value = serde_json::from_str(&response).expect("high eval JSON");
    let high_id = high["evaluation_id"]
        .as_str()
        .expect("high eval id")
        .to_string();
    let high_trace_hash = high["trace_hash"]
        .as_str()
        .expect("high trace hash")
        .to_string();
    assert_eq!(high["risk_level"], "high");

    let medium_payload = json!({
        "repository_full_name": REPO_FULL_NAME,
        "branch": BRANCH,
        "environment": "production",
        "change_id": "KAN-125-medium",
        "release_id": "release-kan-125-medium",
        "commit_sha": TARGET_SHA
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&medium_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "medium eval create: {response}"
    );
    let medium: serde_json::Value = serde_json::from_str(&response).expect("medium eval JSON");
    let medium_id = medium["evaluation_id"]
        .as_str()
        .expect("medium eval id")
        .to_string();
    assert_eq!(medium["risk_level"], "medium");

    let other_payload = json!({
        "org_name": "risk-cab-other",
        "repository_full_name": "risk-cab-other/repo",
        "branch": BRANCH,
        "environment": "production",
        "change_id": "KAN-125-other"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/evaluations",
        Some(&other_payload.to_string()),
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "other tenant eval create: {response}"
    );
    let other_eval: serde_json::Value =
        serde_json::from_str(&response).expect("other tenant eval JSON");
    let other_eval_id = other_eval["evaluation_id"]
        .as_str()
        .expect("other eval id")
        .to_string();

    let review_low = json!({
        "review_status": "reviewed",
        "review_notes": "Low risk checked for CAB packet coverage.",
        "mitigation_notes": "No additional mitigation required.",
        "decision_reason": "Manual review completed."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{low_id}/review"),
        Some(&review_low.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "low review update: {response}");

    let review_medium = json!({
        "review_status": "needs_mitigation",
        "review_notes": "Medium risk needs evidence cleanup before CAB.",
        "mitigation_notes": "Attach missing deployment evidence.",
        "decision_reason": "Manual mitigation requested."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{medium_id}/review"),
        Some(&review_medium.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "medium review update: {response}");

    let review_high = json!({
        "review_status": "accepted_risk",
        "review_notes": "CAB accepts production risk after trace review.",
        "mitigation_notes": "Rollback owner remains online through release.",
        "decision_reason": "Business exception accepted manually."
    });
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/evaluations/{high_id}/review"),
        Some(&review_high.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "high review update: {response}");

    let before_gate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deployment_gate_authorizations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("before gate count");
    let before_agent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("before agent count");

    let filter_packet_payload = json!({
        "name": "KAN-125 accepted risk CAB",
        "repository_full_name": REPO_FULL_NAME,
        "review_status": "accepted_risk"
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(&filter_packet_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "CAB packet by filter create: {response}"
    );
    let filter_packet: serde_json::Value =
        serde_json::from_str(&response).expect("filter CAB packet JSON");
    let filter_packet_id = filter_packet["packet"]["packet_id"]
        .as_str()
        .expect("filter packet id")
        .to_string();
    let filter_hash = filter_packet["packet"]["artifact_hash"]
        .as_str()
        .expect("filter packet hash");
    assert!(filter_hash.starts_with("sha256:"));
    assert_eq!(
        filter_packet["artifact"]["schema_version"],
        "gitgov_change_risk_cab_packet.v1"
    );
    assert_eq!(filter_packet["artifact"]["summary"]["total_evaluations"], 1);
    assert_eq!(
        filter_packet["artifact"]["evaluations"][0]["evaluation_id"],
        high_id
    );
    assert_eq!(
        filter_packet["artifact"]["evaluations"][0]["trace_hash"],
        high_trace_hash
    );
    assert_eq!(
        filter_packet["artifact"]["verification"]["packet_hash"],
        filter_hash
    );
    assert_eq!(filter_packet["artifact"]["claims"]["advisory_only"], true);
    assert_eq!(
        filter_packet["artifact"]["claims"]["manual_review_packet"],
        true
    );
    assert_eq!(
        filter_packet["artifact"]["claims"]["compliance_claim"],
        false
    );
    assert_eq!(filter_packet["artifact"]["claims"]["certification"], false);
    assert_eq!(
        filter_packet["artifact"]["audit_metadata"]["source_evaluations_mutated"],
        false
    );
    assert_eq!(
        filter_packet["artifact"]["audit_metadata"]["agent_governance_used"],
        false
    );
    assert_eq!(
        filter_packet["artifact"]["audit_metadata"]["deployment_execution"],
        false
    );

    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/cab-packets?status=active&limit=20",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor CAB list: {response}");
    let listed: serde_json::Value = serde_json::from_str(&response).expect("CAB list JSON");
    assert!(listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["packet_id"] == filter_packet_id));

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor CAB get: {response}");
    let fetched_filter: serde_json::Value =
        serde_json::from_str(&response).expect("fetched filter packet JSON");
    assert_eq!(fetched_filter["packet"]["artifact_hash"], filter_hash);
    assert_eq!(fetched_filter["packet"]["review_status"], "pending_review");

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor CAB review read: {response}"
    );
    let initial_cab_review: serde_json::Value =
        serde_json::from_str(&response).expect("initial CAB review JSON");
    assert_eq!(initial_cab_review["review_status"], "pending_review");
    assert_eq!(initial_cab_review["artifact_hash"], filter_hash);
    assert_eq!(initial_cab_review["manual_cab_disposition_only"], true);
    assert_eq!(initial_cab_review["advisory_only"], true);
    assert_eq!(initial_cab_review["llm_used"], false);
    assert_eq!(initial_cab_review["agent_governance_used"], false);
    assert_eq!(initial_cab_review["release_blocking"], false);
    assert_eq!(initial_cab_review["deployment_execution"], false);
    assert_eq!(initial_cab_review["compliance_claim"], false);
    assert_eq!(initial_cab_review["certification"], false);

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes": "CAB coordinator reviewed the packet contents manually.",
                "mitigation_notes": "No mitigation requested for this packet.",
                "decision_reason": "Packet is ready for CAB discussion."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "CAB reviewed update: {response}");
    let cab_reviewed: serde_json::Value =
        serde_json::from_str(&response).expect("CAB reviewed JSON");
    assert_eq!(cab_reviewed["review_status"], "reviewed");
    assert_eq!(cab_reviewed["reviewed_by_user_id"], "kan-125-cab-admin");
    assert_eq!(cab_reviewed["artifact_hash"], filter_hash);
    assert_eq!(
        cab_reviewed["review_notes_safe"],
        "CAB coordinator reviewed the packet contents manually."
    );
    assert_eq!(cab_reviewed["manual_cab_disposition_only"], true);

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "accepted_risk",
                "review_notes": "CAB accepted risk only as a manual disposition.",
                "mitigation_notes": "Rollback owner remains available.",
                "decision_reason": "Business owner accepted the residual risk."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "CAB accepted risk update: {response}"
    );
    let cab_accepted: serde_json::Value =
        serde_json::from_str(&response).expect("CAB accepted JSON");
    assert_eq!(cab_accepted["review_status"], "accepted_risk");
    assert_eq!(
        cab_accepted["decision_reason_safe"],
        "Business owner accepted the residual risk."
    );
    assert_eq!(cab_accepted["release_blocking"], false);
    assert_eq!(cab_accepted["deployment_execution"], false);

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "needs_mitigation",
                "review_notes": "CAB requires one follow-up before release sign-off elsewhere.",
                "mitigation_notes": "Attach rollback rehearsal evidence to the ticket.",
                "decision_reason": "Missing rollback rehearsal evidence.",
                "follow_up_required": true,
                "follow_up_owner": "release-owner"
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "CAB needs mitigation update: {response}"
    );
    let cab_needs_mitigation: serde_json::Value =
        serde_json::from_str(&response).expect("CAB mitigation JSON");
    assert_eq!(cab_needs_mitigation["review_status"], "needs_mitigation");
    assert_eq!(cab_needs_mitigation["follow_up_required"], true);
    assert_eq!(
        cab_needs_mitigation["follow_up_owner_safe"],
        "release-owner"
    );
    assert_eq!(cab_needs_mitigation["artifact_hash"], filter_hash);

    let (status, response) = json_request(
        &app,
        "POST",
        &format!("/change-risk/cab-packets/{filter_packet_id}/decision-manifests"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "CAB decision manifest create: {response}"
    );
    let created_manifest: serde_json::Value =
        serde_json::from_str(&response).expect("created CAB decision manifest JSON");
    let manifest_id = created_manifest["manifest"]["manifest_id"]
        .as_str()
        .expect("manifest id")
        .to_string();
    let manifest_hash = created_manifest["manifest"]["manifest_hash"]
        .as_str()
        .expect("manifest hash")
        .to_string();
    assert!(manifest_id.starts_with("crcabdm_"));
    assert!(manifest_hash.starts_with("sha256:"));
    assert_eq!(
        created_manifest["manifest"]["cab_packet_id"],
        filter_packet_id
    );
    assert_eq!(created_manifest["manifest"]["cab_packet_hash"], filter_hash);
    assert_eq!(
        created_manifest["manifest"]["review_status_snapshot"],
        "needs_mitigation"
    );
    assert_eq!(
        created_manifest["artifact"]["schema_version"],
        "gitgov_change_risk_cab_decision_manifest.v1"
    );
    assert_eq!(
        created_manifest["artifact"]["cab_packet"]["cab_packet_hash"],
        filter_hash
    );
    assert_eq!(
        created_manifest["artifact"]["review"]["review_status"],
        "needs_mitigation"
    );
    assert_eq!(
        created_manifest["artifact"]["included_evaluations"]["count"],
        1
    );
    assert_eq!(
        created_manifest["artifact"]["included_evaluations"]["trace_hashes"][0],
        high_trace_hash
    );
    assert_eq!(
        created_manifest["artifact"]["claims"]["advisory_only"],
        true
    );
    assert_eq!(created_manifest["artifact"]["claims"]["llm_used"], false);
    assert_eq!(
        created_manifest["artifact"]["claims"]["agent_governance_used"],
        false
    );
    assert_eq!(
        created_manifest["artifact"]["claims"]["compliance_claim"],
        false
    );
    assert_eq!(
        created_manifest["artifact"]["claims"]["certification"],
        false
    );
    assert_eq!(
        created_manifest["artifact"]["audit_metadata"]["deployment_execution"],
        false
    );
    assert_eq!(
        created_manifest["artifact"]["audit_metadata"]["source_cab_packet_mutated"],
        false
    );
    assert_eq!(
        created_manifest["artifact"]["audit_metadata"]["source_evaluations_mutated"],
        false
    );

    let mut manifest_preimage = created_manifest["artifact"].clone();
    manifest_preimage["hash_chain"]["manifest_hash"] = serde_json::Value::Null;
    assert_eq!(canonical_json_hash(&manifest_preimage), manifest_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        &format!(
            "/change-risk/cab-packets/{filter_packet_id}/decision-manifests?org_name=risk-cab-org"
        ),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor manifest list: {response}");
    let listed_manifests: serde_json::Value =
        serde_json::from_str(&response).expect("manifest list JSON");
    assert_eq!(listed_manifests["total"], 1);
    assert_eq!(listed_manifests["items"][0]["manifest_id"], manifest_id);

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-decision-manifests/{manifest_id}?org_name=risk-cab-org"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor manifest get: {response}");
    let fetched_manifest: serde_json::Value =
        serde_json::from_str(&response).expect("fetched manifest JSON");
    assert_eq!(fetched_manifest["manifest"]["manifest_hash"], manifest_hash);
    assert_eq!(
        fetched_manifest["artifact"]["included_evaluations"]["trace_hashes"][0],
        high_trace_hash
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-decision-manifests/{manifest_id}/detail?org_name=risk-cab-org"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor manifest detail: {response}"
    );
    let fetched_manifest_detail: serde_json::Value =
        serde_json::from_str(&response).expect("fetched manifest detail JSON");
    assert_eq!(
        fetched_manifest_detail["manifest"]["manifest_hash"],
        manifest_hash
    );
    assert_eq!(
        fetched_manifest_detail["artifact"]["hash_chain"]["manifest_hash"],
        manifest_hash
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!(
            "/change-risk/cab-decision-manifests/{manifest_id}/download?org_name=risk-cab-org"
        ),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor manifest download: {response}"
    );
    let downloaded_manifest: serde_json::Value =
        serde_json::from_str(&response).expect("downloaded manifest JSON");
    assert_eq!(
        downloaded_manifest["hash_chain"]["manifest_hash"],
        manifest_hash
    );
    assert_eq!(
        downloaded_manifest["hash_chain"]["cab_packet_hash"],
        filter_hash
    );
    let manifest_download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM change_risk_cab_decision_manifests WHERE manifest_id = $1",
    )
    .bind(&manifest_id)
    .fetch_one(&pool)
    .await
    .expect("manifest download count");
    assert_eq!(manifest_download_count, 1);

    let (status, response) = json_request(
        &app,
        "GET",
        "/deployment-gates/dga_kan125_high/risk-context?org_name=risk-cab-org",
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "auditor Deployment Gate risk context: {response}"
    );
    let risk_context: serde_json::Value =
        serde_json::from_str(&response).expect("Deployment Gate risk context JSON");
    assert_eq!(risk_context["deployment_gate_id"], "dga_kan125_high");
    assert_eq!(
        risk_context["authorization"]["authorization_id"],
        "dga_kan125_high"
    );
    assert_eq!(risk_context["latest_risk_level"], "high");
    assert_eq!(risk_context["latest_review_status"], "accepted_risk");
    assert_eq!(risk_context["advisory_only"], true);
    assert_eq!(risk_context["enforcement_used"], false);
    assert_eq!(risk_context["llm_used"], false);
    assert_eq!(risk_context["agent_governance_used"], false);
    assert_eq!(risk_context["compliance_claim"], false);
    assert_eq!(risk_context["certification"], false);
    assert!(risk_context["triggered_rules_count"].as_u64().unwrap() >= 1);
    assert_eq!(
        risk_context["change_risk_evaluations"][0]["evaluation_id"],
        high_id
    );
    assert_eq!(
        risk_context["change_risk_evaluations"][0]["trace_hash"],
        high_trace_hash
    );
    assert!(risk_context["cab_packets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|packet| packet["packet_id"] == filter_packet_id
            && packet["artifact_hash"] == filter_hash));
    assert!(risk_context["cab_decision_manifests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|manifest| manifest["manifest_id"] == manifest_id
            && manifest["manifest_hash"] == manifest_hash
            && manifest["status"] == "active"));

    let gate_after_context: (String, bool) = sqlx::query_as(
        "SELECT decision, blocking FROM deployment_gate_authorizations WHERE authorization_id = $1",
    )
    .bind("dga_kan125_high")
    .fetch_one(&pool)
    .await
    .expect("gate after risk context");
    assert_eq!(gate_after_context.0, "blocked");
    assert!(gate_after_context.1);

    let (status, response) = json_request(
        &app,
        "POST",
        &format!("/change-risk/cab-packets/{filter_packet_id}/decision-manifests"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not create CAB decision manifest: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/deployment-gates/dga_kan125_high/risk-context?org_name=risk-cab-org",
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not read Deployment Gate risk context: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/decision-manifests?org_name=risk-cab-other"),
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "other tenant must not list foreign CAB decision manifests: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/deployment-gates/dga_kan125_high/risk-context?org_name=risk-cab-other",
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "other tenant must not read foreign Deployment Gate risk context: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "accepted_risk",
                "review_notes": "Authorization: Bearer sk-test-secret",
                "decision_reason": "This must be rejected before persistence."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "secret-looking CAB review notes must be rejected: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "accepted_risk",
                "review_notes": "Accepted without a reason should fail."
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "accepted risk CAB disposition requires reason: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "needs_mitigation",
                "mitigation_notes": "Missing follow-up flag should fail.",
                "follow_up_required": false
            })
            .to_string(),
        ),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "needs mitigation requires follow-up flag: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes": "Auditor is read-only for CAB packet disposition."
            })
            .to_string(),
        ),
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "auditor must not update CAB packet disposition: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not read CAB packet disposition: {response}"
    );

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review?org_name=risk-cab-other"),
        None,
        Some(&other_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "other tenant admin must not read CAB packet disposition: {response}"
    );

    let post_review_packet_hash: String = sqlx::query_scalar(
        "SELECT artifact_hash FROM change_risk_cab_packets WHERE packet_id = $1",
    )
    .bind(&filter_packet_id)
    .fetch_one(&pool)
    .await
    .expect("post review CAB hash");
    assert_eq!(post_review_packet_hash, filter_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/download"),
        None,
        Some(&auditor_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "auditor CAB download: {response}");
    let downloaded: serde_json::Value =
        serde_json::from_str(&response).expect("downloaded CAB artifact JSON");
    assert_eq!(downloaded["packet_id"], filter_packet_id);
    assert_eq!(downloaded["verification"]["packet_hash"], filter_hash);
    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM change_risk_cab_packets WHERE packet_id = $1",
    )
    .bind(&filter_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB download count");
    assert_eq!(download_count, 1);

    let selection_packet_payload = json!({
        "name": "KAN-125 mixed CAB selection",
        "evaluation_ids": [low_id.clone(), medium_id.clone(), high_id.clone()]
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(&selection_packet_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "CAB packet by explicit selection: {response}"
    );
    let selection_packet: serde_json::Value =
        serde_json::from_str(&response).expect("selection packet JSON");
    let selection_packet_id = selection_packet["packet"]["packet_id"]
        .as_str()
        .expect("selection packet id")
        .to_string();
    assert_eq!(
        selection_packet["artifact"]["summary"]["total_evaluations"],
        3
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["risk_level_counts"]["low"],
        1
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["risk_level_counts"]["medium"],
        1
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["risk_level_counts"]["high"],
        1
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["review_status_counts"]["reviewed"],
        1
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["review_status_counts"]["needs_mitigation"],
        1
    );
    assert_eq!(
        selection_packet["artifact"]["summary"]["review_status_counts"]["accepted_risk"],
        1
    );

    let (status, response) = json_request(
        &app,
        "POST",
        &format!("/change-risk/cab-packets/{selection_packet_id}/decision-manifests"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "pending review packet cannot create manifest: {response}"
    );

    let cross_tenant_payload = json!({
        "name": "KAN-125 cross tenant should fail",
        "evaluation_ids": [other_eval_id]
    });
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(&cross_tenant_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "cross tenant selection must not package foreign evidence: {response}"
    );

    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(&selection_packet_payload.to_string()),
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not create CAB packets: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/cab-packets",
        None,
        Some(&developer_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "developer must not list CAB packets: {response}"
    );
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{selection_packet_id}/archive"),
        Some("{}"),
        Some(&auditor_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "auditor must not archive CAB packets: {response}"
    );

    let agent_token = create_agent_key(&app, &admin_key).await;
    let (status, response) = json_request(
        &app,
        "POST",
        "/change-risk/cab-packets",
        Some(&selection_packet_payload.to_string()),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not create CAB packets: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/change-risk/cab-packets",
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not list CAB packets: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not read CAB packet disposition: {response}"
    );
    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{filter_packet_id}/review"),
        Some(
            &json!({
                "review_status": "reviewed",
                "review_notes": "Agent keys cannot update CAB packet disposition."
            })
            .to_string(),
        ),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not update CAB packet disposition: {response}"
    );

    let (status, response) = json_request(
        &app,
        "POST",
        &format!("/change-risk/cab-packets/{filter_packet_id}/decision-manifests"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not create CAB decision manifests: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        &format!(
            "/change-risk/cab-decision-manifests/{manifest_id}/download?org_name=risk-cab-org"
        ),
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not download CAB decision manifests: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/deployment-gates/dga_kan125_high/risk-context?org_name=risk-cab-org",
        None,
        Some(&agent_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent key must not read Deployment Gate risk context: {response}"
    );

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-decision-manifests/{manifest_id}/revoke"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin revoke CAB decision manifest: {response}"
    );
    let revoked_manifest: serde_json::Value =
        serde_json::from_str(&response).expect("revoked CAB decision manifest JSON");
    assert_eq!(revoked_manifest["manifest"]["status"], "revoked");
    assert_eq!(revoked_manifest["manifest"]["manifest_hash"], manifest_hash);

    let (status, response) = json_request(
        &app,
        "GET",
        &format!(
            "/change-risk/cab-decision-manifests/{manifest_id}/download?org_name=risk-cab-org"
        ),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "revoked CAB decision manifest download must be blocked: {response}"
    );
    let (status, response) = json_request(
        &app,
        "GET",
        "/deployment-gates/dga_kan125_high/risk-context?org_name=risk-cab-org",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin Deployment Gate risk context after revoke: {response}"
    );
    let revoked_context: serde_json::Value =
        serde_json::from_str(&response).expect("revoked risk context JSON");
    assert!(revoked_context["cab_decision_manifests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|manifest| manifest["manifest_id"] == manifest_id
            && manifest["manifest_hash"] == manifest_hash
            && manifest["status"] == "revoked"));

    let (status, response) = json_request(
        &app,
        "PATCH",
        &format!("/change-risk/cab-packets/{selection_packet_id}/archive"),
        Some(r#"{"org_name":"risk-cab-org"}"#),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "admin archive CAB: {response}");
    let archived: serde_json::Value =
        serde_json::from_str(&response).expect("archived CAB packet JSON");
    assert_eq!(archived["packet"]["status"], "archived");

    let (status, response) = json_request(
        &app,
        "GET",
        &format!("/change-risk/cab-packets/{selection_packet_id}/download"),
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "archived CAB packet download must be blocked: {response}"
    );

    let high_after: (String, String) = sqlx::query_as(
        "SELECT review_status, trace_hash FROM change_risk_evaluations WHERE evaluation_id = $1",
    )
    .bind(&high_id)
    .fetch_one(&pool)
    .await
    .expect("high evaluation after CAB packet");
    assert_eq!(high_after.0, "accepted_risk");
    assert_eq!(high_after.1, high_trace_hash);

    let after_gate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deployment_gate_authorizations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("after gate count");
    let after_agent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_governance_evaluations WHERE org_id = $1::uuid",
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("after agent count");
    assert_eq!(after_gate_count, before_gate_count);
    assert_eq!(after_agent_count, before_agent_count);

    let created_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_cab_packet_created' AND target_id IN ($1, $2)",
    )
    .bind(&filter_packet_id)
    .bind(&selection_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB created audit count");
    assert_eq!(created_audit_count, 2);
    let downloaded_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_cab_packet_downloaded' AND target_id = $1",
    )
    .bind(&filter_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB downloaded audit count");
    assert_eq!(downloaded_audit_count, 1);
    let review_viewed_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_cab_packet_review_viewed' AND target_id = $1",
    )
    .bind(&filter_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB review viewed audit count");
    assert_eq!(review_viewed_audit_count, 1);
    let review_updated_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_cab_packet_review_updated' AND target_id = $1",
    )
    .bind(&filter_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB review updated audit count");
    assert_eq!(review_updated_audit_count, 3);
    let archived_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'change_risk_cab_packet_archived' AND target_id = $1",
    )
    .bind(&selection_packet_id)
    .fetch_one(&pool)
    .await
    .expect("CAB archived audit count");
    assert_eq!(archived_audit_count, 1);
    let manifest_created_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'cab_decision_manifest_created' AND target_id = $1",
    )
    .bind(&manifest_id)
    .fetch_one(&pool)
    .await
    .expect("CAB decision manifest created audit count");
    assert_eq!(manifest_created_audit_count, 1);
    let manifest_downloaded_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'cab_decision_manifest_downloaded' AND target_id = $1",
    )
    .bind(&manifest_id)
    .fetch_one(&pool)
    .await
    .expect("CAB decision manifest downloaded audit count");
    assert_eq!(manifest_downloaded_audit_count, 1);
    let manifest_revoked_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_audit_log WHERE action = 'cab_decision_manifest_revoked' AND target_id = $1",
    )
    .bind(&manifest_id)
    .fetch_one(&pool)
    .await
    .expect("CAB decision manifest revoked audit count");
    assert_eq!(manifest_revoked_audit_count, 1);

    teardown(&admin_pool, &schema).await;
}
