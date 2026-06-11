use super::common::*;

const ORG_LOGIN: &str = "release-org";
const REPO_FULL_NAME: &str = "release-org/repo";
const TICKET_ID: &str = "KAN-900";
const RELEASE_ID: &str = "release-2026.06.10";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const ENVIRONMENT: &str = "production";

async fn insert_repo_for_org(pool: &sqlx::PgPool, org_id: &str, full_name: &str) -> String {
    let repo_id = uuid::Uuid::new_v4().to_string();
    let repo_name = full_name
        .split('/')
        .next_back()
        .filter(|name| !name.is_empty())
        .unwrap_or(full_name)
        .to_string();

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(org_id)
    .bind(full_name)
    .bind(repo_name)
    .execute(pool)
    .await
    .expect("insert release binding test repo");

    repo_id
}

async fn seed_release_evidence(pool: &sqlx::PgPool) -> (String, String) {
    let org_id = insert_test_org(pool, ORG_LOGIN).await;
    let repo_id = insert_repo_for_org(pool, &org_id, REPO_FULL_NAME).await;
    let decoy_org_id = insert_test_org(pool, "release-decoy").await;
    let decoy_repo_id = insert_repo_for_org(pool, &decoy_org_id, "release-decoy/repo").await;

    sqlx::query(
        r#"
        INSERT INTO project_tickets (
            org_id, ticket_id, project_key, ticket_url, title, status, ingested_at
        ) VALUES (
            $1::uuid, $2, 'KAN', $3, 'Release binding ticket', 'Open', NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(&org_id)
    .bind(TICKET_ID)
    .bind(format!("https://jira.example.test/browse/{TICKET_ID}"))
    .execute(pool)
    .await
    .expect("insert release binding ticket");

    sqlx::query(
        r#"
        INSERT INTO client_events (
            id, org_id, repo_id, event_uuid, event_type, user_login, commit_sha,
            branch, status, metadata, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2::uuid, $3, 'commit', 'dev', $4,
            $5, 'success', $6::jsonb, NOW() - INTERVAL '30 minutes'
        )
        "#,
    )
    .bind(&org_id)
    .bind(&repo_id)
    .bind(format!("event-{TARGET_SHA}"))
    .bind(TARGET_SHA)
    .bind(BRANCH)
    .bind(
        serde_json::json!({ "commit_message": format!("fix({TICKET_ID}): release evidence") })
            .to_string(),
    )
    .execute(pool)
    .await
    .expect("insert release binding commit");

    sqlx::query(
        r#"
        INSERT INTO client_events (
            id, org_id, repo_id, event_uuid, event_type, user_login, commit_sha,
            branch, status, metadata, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2::uuid, $3, 'commit', 'other-dev', $4,
            'develop', 'success', $5::jsonb, NOW() - INTERVAL '10 minutes'
        )
        "#,
    )
    .bind(&decoy_org_id)
    .bind(&decoy_repo_id)
    .bind(format!("decoy-event-{TARGET_SHA}"))
    .bind(TARGET_SHA)
    .bind(
        serde_json::json!({ "commit_message": format!("fix({TICKET_ID}): decoy evidence") })
            .to_string(),
    )
    .execute(pool)
    .await
    .expect("insert release binding cross-tenant decoy commit");

    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            id, org_id, pipeline_id, job_name, status, branch, commit_sha,
            repo_full_name, ingested_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, 'wrong-repo-pipeline', 'Sonar Quality',
            'success', $2, $3, 'release-org/other-repo', NOW() - INTERVAL '5 minutes'
        )
        "#,
    )
    .bind(&org_id)
    .bind(BRANCH)
    .bind(TARGET_SHA)
    .execute(pool)
    .await
    .expect("insert release binding same-org wrong-repo pipeline");

    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            org_id, commit_sha, ticket_id, correlation_source, confidence
        ) VALUES (
            $1::uuid, $2, $3, 'commit_message', 1.0
        )
        "#,
    )
    .bind(&org_id)
    .bind(TARGET_SHA)
    .bind(TICKET_ID)
    .execute(pool)
    .await
    .expect("insert release binding correlation");

    (org_id, repo_id)
}

async fn generate_bound_packet(app: &axum::Router, api_key: &str) -> String {
    let uri = format!(
        "/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&release_id={RELEASE_ID}&environment={ENVIRONMENT}&hours=72"
    );
    let (status, body) = json_request(app, "GET", &uri, None, Some(api_key)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "bound packet generation failed: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("packet response JSON");
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["packet"]["repo_full_name"], REPO_FULL_NAME);
    assert_eq!(parsed["packet"]["branch"], BRANCH);
    assert_eq!(parsed["packet"]["target_sha"], TARGET_SHA);
    assert_eq!(parsed["packet"]["completeness"]["pipelines"], 0);
    assert_eq!(parsed["packet"]["completeness"]["quality_gates"], 0);
    parsed["packet"]["content_hash"]
        .as_str()
        .expect("packet content hash")
        .to_string()
}

fn release_approval_payload(evidence_hash: &str) -> serde_json::Value {
    serde_json::json!({
        "release_id": RELEASE_ID,
        "repository_full_name": REPO_FULL_NAME,
        "branch": BRANCH,
        "target_sha": TARGET_SHA,
        "environment": ENVIRONMENT,
        "decision": "approved",
        "approver": "release.manager@example.com",
        "ticket_id": TICKET_ID,
        "evidence_packet_hash": evidence_hash,
        "evidence_summary": {
            "approver_role": "engineering"
        },
        "risk_severity": "none"
    })
}

#[tokio::test]
async fn release_approval_rejects_unregistered_evidence_hash() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let (org_id, _) = seed_release_evidence(&pool).await;
    let api_key = insert_test_api_key_for_org(&pool, "release-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let body = release_approval_payload(&"f".repeat(64)).to_string();
    let (status, response) = json_request(
        &app,
        "POST",
        "/enterprise/release-approvals",
        Some(&body),
        Some(&api_key),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response.contains("not a known release evidence packet"),
        "unexpected response: {response}"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn release_bound_packet_requires_complete_binding_context() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let (org_id, _) = seed_release_evidence(&pool).await;
    let api_key = insert_test_api_key_for_org(&pool, "release-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let partial_uri = format!(
        "/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&hours=72"
    );
    let (status, _) = json_request(&app, "GET", &partial_uri, None, Some(&api_key)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    let invalid_ticket_uri = format!(
        "/evidence/packets/tickets/not-a-ticket?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&release_id={RELEASE_ID}&environment={ENVIRONMENT}&hours=72"
    );
    let (status, _) = json_request(&app, "GET", &invalid_ticket_uri, None, Some(&api_key)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn release_governance_counts_only_exact_bound_evidence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let (org_id, _) = seed_release_evidence(&pool).await;
    let api_key = insert_test_api_key_for_org(&pool, "release-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let evidence_hash = generate_bound_packet(&app, &api_key).await;

    let approval_body = release_approval_payload(&evidence_hash).to_string();
    let (status, body) = json_request(
        &app,
        "POST",
        "/enterprise/release-approvals",
        Some(&approval_body),
        Some(&api_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "approval create failed: {body}"
    );

    let evaluate_uri = format!(
        "/enterprise/release-governance/evaluate?repository_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&release_id={RELEASE_ID}&environment={ENVIRONMENT}&evidence_packet_hash={evidence_hash}"
    );
    let (status, body) = json_request(&app, "GET", &evaluate_uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "governance evaluate failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("evaluation response JSON");
    assert_eq!(parsed["valid_approval_count"], 1);
    assert_eq!(parsed["approvals"][0]["counts_toward_policy"], true);

    let wrong_branch_uri = format!(
        "/enterprise/release-governance/evaluate?repository_full_name={REPO_FULL_NAME}&branch=develop&target_sha={TARGET_SHA}&release_id={RELEASE_ID}&environment={ENVIRONMENT}&evidence_packet_hash={evidence_hash}"
    );
    let (status, body) = json_request(&app, "GET", &wrong_branch_uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body.contains("does not match branch"),
        "wrong branch should be rejected by binding check: {body}"
    );

    let wrong_target_sha = "1234567890abcdef1234567890abcdef12345678";
    let wrong_target_uri = format!(
        "/enterprise/release-governance/evaluate?repository_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={wrong_target_sha}&release_id={RELEASE_ID}&environment={ENVIRONMENT}&evidence_packet_hash={evidence_hash}"
    );
    let (status, body) = json_request(&app, "GET", &wrong_target_uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body.contains("does not match target_sha"),
        "wrong target SHA should be rejected by binding check: {body}"
    );

    teardown(&admin_pool, &schema).await;
}
