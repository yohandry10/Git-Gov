use super::common::*;

const ORG_LOGIN: &str = "packet-org";
const REPO_FULL_NAME: &str = "packet-org/repo";
const OTHER_REPO_FULL_NAME: &str = "packet-org/other-repo";
const TICKET_ID: &str = "KAN-702";
const BRANCH: &str = "main";
const TARGET_SHA: &str = "702abc1234567890abcdef1234567890abcdef12";
const OTHER_SHA: &str = "999abc1234567890abcdef1234567890abcdef12";
const GHOST_SHA: &str = "111abc1234567890abcdef1234567890abcdef12";

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
    .expect("insert packet test repo");

    repo_id
}

async fn insert_ticket(pool: &sqlx::PgPool, org_id: &str, ticket_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO project_tickets (
            org_id, ticket_id, project_key, ticket_url, title, status, ingested_at
        ) VALUES (
            $1::uuid, $2, 'KAN', $3, 'Packet reconstruction ticket', 'Open',
            NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(org_id)
    .bind(ticket_id)
    .bind(format!("https://jira.example.test/browse/{ticket_id}"))
    .execute(pool)
    .await
    .expect("insert packet test ticket");
}

async fn insert_commit_event(
    pool: &sqlx::PgPool,
    org_id: &str,
    repo_id: &str,
    commit_sha: &str,
    branch: &str,
) {
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
    .bind(org_id)
    .bind(repo_id)
    .bind(format!("packet-event-{commit_sha}-{branch}"))
    .bind(commit_sha)
    .bind(branch)
    .bind(
        serde_json::json!({ "commit_message": format!("fix({TICKET_ID}): governed change") })
            .to_string(),
    )
    .execute(pool)
    .await
    .expect("insert packet test commit");
}

async fn insert_commit_ticket_correlation(
    pool: &sqlx::PgPool,
    org_id: &str,
    commit_sha: &str,
    ticket_id: &str,
    source: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            org_id, commit_sha, ticket_id, correlation_source, confidence, created_at
        ) VALUES (
            $1::uuid, $2, $3, $4, 1.0, NOW() - INTERVAL '25 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(commit_sha)
    .bind(ticket_id)
    .bind(source)
    .execute(pool)
    .await
    .expect("insert packet test correlation");
}

async fn insert_global_commit_ticket_correlation(
    pool: &sqlx::PgPool,
    commit_sha: &str,
    ticket_id: &str,
    source: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            org_id, commit_sha, ticket_id, correlation_source, confidence, created_at
        ) VALUES (
            NULL, $1, $2, $3, 1.0, NOW() - INTERVAL '25 minutes'
        )
        "#,
    )
    .bind(commit_sha)
    .bind(ticket_id)
    .bind(source)
    .execute(pool)
    .await
    .expect("insert global packet test correlation");
}

struct PrFixture<'a> {
    delivery_id: &'a str,
    pr_number: i32,
    pr_title: &'a str,
    head_sha: &'a str,
    merge_sha: &'a str,
    base_branch: &'a str,
    payload: serde_json::Value,
}

async fn insert_pr_merge(pool: &sqlx::PgPool, org_id: &str, repo_id: &str, fixture: PrFixture<'_>) {
    let mut payload = fixture.payload;
    if payload.pointer("/pull_request/merge_commit_sha").is_none() {
        payload["pull_request"]["merge_commit_sha"] = serde_json::json!(fixture.merge_sha);
    }

    sqlx::query(
        r#"
        INSERT INTO pull_request_merges (
            org_id, repo_id, delivery_id, pr_number, pr_title, author_login,
            merged_by_login, head_sha, base_branch, payload, created_at
        ) VALUES (
            $1::uuid, $2::uuid, $3, $4, $5, 'alice', 'bob', $6, $7,
            $8::jsonb, NOW() - INTERVAL '20 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(repo_id)
    .bind(fixture.delivery_id)
    .bind(fixture.pr_number)
    .bind(fixture.pr_title)
    .bind(fixture.head_sha)
    .bind(fixture.base_branch)
    .bind(payload.to_string())
    .execute(pool)
    .await
    .expect("insert packet test PR merge");
}

async fn insert_pipeline_event(
    pool: &sqlx::PgPool,
    org_id: &str,
    pipeline_id: &str,
    job_name: &str,
    repo_full_name: &str,
    branch: &str,
    commit_sha: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            id, org_id, pipeline_id, job_name, status, commit_sha, branch,
            repo_full_name, triggered_by, ingested_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2, $3, 'success', $4, $5,
            $6, 'jenkins', NOW() - INTERVAL '10 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(pipeline_id)
    .bind(job_name)
    .bind(commit_sha)
    .bind(branch)
    .bind(repo_full_name)
    .execute(pool)
    .await
    .expect("insert packet test pipeline");
}

async fn insert_legacy_pipeline_event_without_repo_branch(
    pool: &sqlx::PgPool,
    org_id: &str,
    pipeline_id: &str,
    job_name: &str,
    commit_sha: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            id, org_id, pipeline_id, job_name, status, commit_sha,
            triggered_by, ingested_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2, $3, 'success', $4,
            'legacy-jenkins', NOW() - INTERVAL '9 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(pipeline_id)
    .bind(job_name)
    .bind(commit_sha)
    .execute(pool)
    .await
    .expect("insert packet test legacy pipeline");
}

#[tokio::test]
async fn evidence_packet_does_not_turn_correlation_only_rows_into_commit_evidence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, ORG_LOGIN).await;
    let _repo_id = insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    insert_ticket(&pool, &org_id, TICKET_ID).await;
    insert_commit_ticket_correlation(&pool, &org_id, GHOST_SHA, TICKET_ID, "manual").await;

    let api_key = insert_test_api_key_for_org(&pool, "packet-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let generic_uri = format!("/evidence/packets/tickets/{TICKET_ID}?hours=72");
    let (status, body) = json_request(&app, "GET", &generic_uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "generic packet failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("packet JSON");
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["packet"]["completeness"]["ticket_found"], true);
    assert_eq!(parsed["packet"]["completeness"]["commits"], 0);
    assert_eq!(
        parsed["packet"]["reconstruction"]["sources"]["commit_correlations"],
        0
    );

    let release_uri = format!(
        "/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={GHOST_SHA}&release_id=release-ghost&environment=production&hours=72"
    );
    let (status, body) = json_request(&app, "GET", &release_uri, None, Some(&api_key)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "release-bound packet must not bind correlation-only SHA: {body}"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn coverage_and_packet_ignore_global_correlations_inside_scoped_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, ORG_LOGIN).await;
    let repo_id = insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    insert_ticket(&pool, &org_id, TICKET_ID).await;
    insert_commit_event(&pool, &org_id, &repo_id, TARGET_SHA, BRANCH).await;
    insert_global_commit_ticket_correlation(&pool, TARGET_SHA, TICKET_ID, "manual").await;

    let db = Database::from_pool(pool.clone());
    let coverage = db
        .get_ticket_coverage(None, Some(&org_id), Some(REPO_FULL_NAME), Some(BRANCH), 72)
        .await
        .expect("coverage with global correlation");
    assert_eq!(coverage.total_commits, 1);
    assert_eq!(
        coverage.commits_with_ticket, 0,
        "global correlation must not verify scoped tenant coverage"
    );
    assert_eq!(coverage.detected_unverified_commits, 0);

    let api_key = insert_test_api_key_for_org(&pool, "packet-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(db));

    let generic_uri =
        format!("/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&hours=72");
    let (status, body) = json_request(&app, "GET", &generic_uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "packet failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("packet JSON");
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["packet"]["completeness"]["commits"], 0);

    let release_uri = format!(
        "/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&release_id=release-global&environment=production&hours=72"
    );
    let (status, body) = json_request(&app, "GET", &release_uri, None, Some(&api_key)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "release-bound packet must not use global correlation: {body}"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn evidence_packet_reconstructs_pr_merge_commit_without_client_event_and_matches_coverage() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, ORG_LOGIN).await;
    let repo_id = insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    insert_ticket(&pool, &org_id, TICKET_ID).await;
    insert_pr_merge(
        &pool,
        &org_id,
        &repo_id,
        PrFixture {
            delivery_id: "packet-pr-only",
            pr_number: 17,
            pr_title: "fix(KAN-702): merge-only evidence",
            head_sha: "702feed1234567890abcdef1234567890abcdef",
            merge_sha: TARGET_SHA,
            base_branch: BRANCH,
            payload: serde_json::json!({ "repository": { "full_name": REPO_FULL_NAME } }),
        },
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_id,
        &repo_id,
        PrFixture {
            delivery_id: "packet-pr-title-boundary",
            pr_number: 18,
            pr_title: "fix(KAN-7020): unrelated ticket prefix",
            head_sha: "702aaaa1234567890abcdef1234567890abcdef",
            merge_sha: "702bbbb1234567890abcdef1234567890abcdef",
            base_branch: BRANCH,
            payload: serde_json::json!({ "repository": { "full_name": REPO_FULL_NAME } }),
        },
    )
    .await;
    insert_commit_ticket_correlation(&pool, &org_id, TARGET_SHA, TICKET_ID, "pr_title").await;

    let db = Database::from_pool(pool.clone());
    let coverage = db
        .get_ticket_coverage(None, Some(&org_id), Some(REPO_FULL_NAME), Some(BRANCH), 72)
        .await
        .expect("packet coverage");
    assert_eq!(coverage.total_commits, 2);
    assert_eq!(coverage.commits_with_ticket, 1);
    assert_eq!(coverage.detected_unverified_commits, 0);
    assert_eq!(coverage.commits_without_ticket.len(), 1);

    let api_key = insert_test_api_key_for_org(&pool, "packet-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(db));
    let uri =
        format!("/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&hours=72");
    let (status, body) = json_request(&app, "GET", &uri, None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "packet request failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("packet JSON");

    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["packet"]["completeness"]["commits"], 1);
    assert_eq!(parsed["packet"]["commits"][0]["commit_sha"], TARGET_SHA);
    assert_eq!(
        parsed["packet"]["commits"][0]["evidence_source"],
        "pull_request_merge"
    );
    assert_eq!(
        parsed["packet"]["pull_requests"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        parsed["packet"]["reconstruction"]["sources"]["pull_request_merge_commits"],
        1
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn release_bound_packet_excludes_wrong_payload_repo_branch_and_non_target_evidence() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, ORG_LOGIN).await;
    let repo_id = insert_repo_for_org(&pool, &org_id, REPO_FULL_NAME).await;
    let other_repo_id = insert_repo_for_org(&pool, &org_id, OTHER_REPO_FULL_NAME).await;
    insert_ticket(&pool, &org_id, TICKET_ID).await;
    insert_commit_event(&pool, &org_id, &repo_id, TARGET_SHA, BRANCH).await;
    insert_commit_event(&pool, &org_id, &repo_id, OTHER_SHA, BRANCH).await;
    insert_commit_ticket_correlation(&pool, &org_id, TARGET_SHA, TICKET_ID, "commit_message").await;
    insert_commit_ticket_correlation(&pool, &org_id, OTHER_SHA, TICKET_ID, "commit_message").await;

    insert_pr_merge(
        &pool,
        &org_id,
        &repo_id,
        PrFixture {
            delivery_id: "packet-valid-pr",
            pr_number: 21,
            pr_title: "fix(KAN-702): release target",
            head_sha: "7021111234567890abcdef1234567890abcdef12",
            merge_sha: TARGET_SHA,
            base_branch: BRANCH,
            payload: serde_json::json!({ "repository": { "full_name": REPO_FULL_NAME } }),
        },
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_id,
        &repo_id,
        PrFixture {
            delivery_id: "packet-title-prefix-false-positive",
            pr_number: 24,
            pr_title: "fix(KAN-7020): unrelated ticket prefix",
            head_sha: "7025551234567890abcdef1234567890abcdef12",
            merge_sha: "7026661234567890abcdef1234567890abcdef12",
            base_branch: BRANCH,
            payload: serde_json::json!({ "repository": { "full_name": REPO_FULL_NAME } }),
        },
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_id,
        &repo_id,
        PrFixture {
            delivery_id: "packet-payload-false-positive",
            pr_number: 22,
            pr_title: "docs: unrelated cleanup",
            head_sha: "7022221234567890abcdef1234567890abcdef12",
            merge_sha: "7023331234567890abcdef1234567890abcdef12",
            base_branch: BRANCH,
            payload: serde_json::json!({ "body": format!("mentions {TICKET_ID} only in payload") }),
        },
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_id,
        &other_repo_id,
        PrFixture {
            delivery_id: "packet-wrong-repo-pr",
            pr_number: 23,
            pr_title: "fix(KAN-702): other repo",
            head_sha: "7024441234567890abcdef1234567890abcdef12",
            merge_sha: TARGET_SHA,
            base_branch: BRANCH,
            payload: serde_json::json!({ "repository": { "full_name": OTHER_REPO_FULL_NAME } }),
        },
    )
    .await;

    insert_pipeline_event(
        &pool,
        &org_id,
        "packet-build-target",
        "Build",
        REPO_FULL_NAME,
        BRANCH,
        TARGET_SHA,
    )
    .await;
    insert_pipeline_event(
        &pool,
        &org_id,
        "packet-sonar-target",
        "Sonar Quality Gate",
        REPO_FULL_NAME,
        BRANCH,
        TARGET_SHA,
    )
    .await;
    insert_legacy_pipeline_event_without_repo_branch(
        &pool,
        &org_id,
        "packet-legacy-unscoped",
        "Sonar Quality Gate",
        TARGET_SHA,
    )
    .await;
    insert_pipeline_event(
        &pool,
        &org_id,
        "packet-other-target",
        "Build",
        REPO_FULL_NAME,
        BRANCH,
        OTHER_SHA,
    )
    .await;
    insert_pipeline_event(
        &pool,
        &org_id,
        "packet-wrong-repo",
        "Sonar Quality Gate",
        OTHER_REPO_FULL_NAME,
        BRANCH,
        TARGET_SHA,
    )
    .await;
    insert_pipeline_event(
        &pool,
        &org_id,
        "packet-wrong-branch",
        "Sonar Quality Gate",
        REPO_FULL_NAME,
        "develop",
        TARGET_SHA,
    )
    .await;

    let api_key = insert_test_api_key_for_org(&pool, "packet-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));
    let uri = format!(
        "/evidence/packets/tickets/{TICKET_ID}?repo_full_name={REPO_FULL_NAME}&branch={BRANCH}&target_sha={TARGET_SHA}&release_id=release-702&environment=production&hours=72"
    );
    let (status, body) = json_request(&app, "GET", &uri, None, Some(&api_key)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "release-bound packet failed: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("packet JSON");
    let packet = &parsed["packet"];

    assert_eq!(packet["completeness"]["commits"], 1);
    assert_eq!(packet["commits"][0]["commit_sha"], TARGET_SHA);
    assert_eq!(packet["pull_requests"].as_array().unwrap().len(), 1);
    assert_eq!(packet["pull_requests"][0]["pr_number"], 21);
    assert_eq!(packet["completeness"]["pipelines"], 2);
    assert_eq!(packet["completeness"]["quality_gates"], 1);

    let pipeline_ids: Vec<&str> = packet["pipelines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["pipeline_id"].as_str())
        .collect();
    assert!(pipeline_ids.contains(&"packet-build-target"));
    assert!(pipeline_ids.contains(&"packet-sonar-target"));
    assert!(!pipeline_ids.contains(&"packet-other-target"));
    assert!(!pipeline_ids.contains(&"packet-wrong-repo"));
    assert!(!pipeline_ids.contains(&"packet-wrong-branch"));
    assert_eq!(
        packet["reconstruction"]["sources"]["legacy_pipeline_scope_fallbacks"],
        0
    );

    teardown(&admin_pool, &schema).await;
}
