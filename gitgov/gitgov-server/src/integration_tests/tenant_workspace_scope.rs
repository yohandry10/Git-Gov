use super::common::*;

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
    .expect("insert tenant test repo");

    repo_id
}

async fn insert_ticket(pool: &sqlx::PgPool, org_id: &str, ticket_id: &str, title: &str) {
    sqlx::query(
        r#"
        INSERT INTO project_tickets (
            org_id, ticket_id, project_key, ticket_url, title, status, ingested_at
        ) VALUES (
            $1::uuid, $2, 'KAN', $3, $4, 'Open', NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(org_id)
    .bind(ticket_id)
    .bind(format!("https://jira.example.test/browse/{ticket_id}"))
    .bind(title)
    .execute(pool)
    .await
    .expect("insert tenant test ticket");
}

async fn insert_commit_event(
    pool: &sqlx::PgPool,
    org_id: &str,
    repo_id: &str,
    event_uuid: &str,
    commit_sha: &str,
    branch: &str,
    commit_message: &str,
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
    .bind(event_uuid)
    .bind(commit_sha)
    .bind(branch)
    .bind(serde_json::json!({ "commit_message": commit_message }).to_string())
    .execute(pool)
    .await
    .expect("insert tenant test commit event");
}

async fn insert_pipeline_event(
    pool: &sqlx::PgPool,
    org_id: &str,
    repo_full_name: &str,
    pipeline_id: &str,
    job_name: &str,
    commit_sha: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            id, org_id, pipeline_id, job_name, status, commit_sha, branch,
            repo_full_name, triggered_by, ingested_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2, $3, 'success', $4, 'main',
            $5, 'jenkins', NOW() - INTERVAL '20 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(pipeline_id)
    .bind(job_name)
    .bind(commit_sha)
    .bind(repo_full_name)
    .execute(pool)
    .await
    .expect("insert tenant test pipeline event");
}

async fn insert_pr_merge(
    pool: &sqlx::PgPool,
    org_id: &str,
    repo_id: &str,
    delivery_id: &str,
    pr_number: i32,
    pr_title: &str,
    head_sha: &str,
    repo_full_name: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO pull_request_merges (
            org_id, repo_id, delivery_id, pr_number, pr_title, author_login,
            merged_by_login, head_sha, base_branch, payload, created_at
        ) VALUES (
            $1::uuid, $2::uuid, $3, $4, $5, 'alice', 'bob', $6, 'main',
            $7::jsonb, NOW() - INTERVAL '15 minutes'
        )
        "#,
    )
    .bind(org_id)
    .bind(repo_id)
    .bind(delivery_id)
    .bind(pr_number)
    .bind(pr_title)
    .bind(head_sha)
    .bind(serde_json::json!({ "repository": { "full_name": repo_full_name } }).to_string())
    .execute(pool)
    .await
    .expect("insert tenant test PR merge");
}

#[tokio::test]
async fn jira_ticket_and_evidence_routes_are_bound_to_effective_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let scoped_a_key = insert_test_api_key_for_org(&pool, "tenant-a-admin", "Admin", &org_a).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    insert_ticket(&pool, &org_a, "KAN-42", "Tenant A ticket").await;
    insert_ticket(&pool, &org_b, "KAN-42", "Tenant B ticket").await;

    let (status, _) = json_request(
        &app,
        "GET",
        "/integrations/jira/tickets/KAN-42",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = json_request(
        &app,
        "GET",
        "/integrations/jira/tickets/KAN-42?org_name=tenant-a",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ticket detail failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["ticket"]["org_id"], org_a);
    assert_eq!(parsed["ticket"]["title"], "Tenant A ticket");

    let (status, _) = json_request(
        &app,
        "GET",
        "/integrations/jira/tickets/KAN-42?org_name=tenant-b",
        None,
        Some(&scoped_a_key),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = json_request(
        &app,
        "GET",
        "/evidence/packets/tickets/KAN-42?org_name=tenant-a&hours=72",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "evidence packet failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["packet"]["org_name"], "tenant-a");
    assert_eq!(parsed["packet"]["ticket"]["org_id"], org_a);
    assert_eq!(parsed["packet"]["ticket"]["title"], "Tenant A ticket");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn coverage_and_jenkins_routes_require_workspace_and_filter_by_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let repo_a = insert_repo_for_org(&pool, &org_a, "tenant-a/repo").await;
    let repo_b = insert_repo_for_org(&pool, &org_b, "tenant-b/repo").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    for (org_id, repo_id, full_name, sha, ticket) in [
        (&org_a, &repo_a, "tenant-a/repo", "sha-a", "KAN-101"),
        (&org_b, &repo_b, "tenant-b/repo", "sha-b", "KAN-202"),
    ] {
        insert_ticket(&pool, org_id, ticket, &format!("{ticket} ticket")).await;
        insert_commit_event(
            &pool,
            org_id,
            repo_id,
            &format!("event-{sha}"),
            sha,
            "main",
            &format!("fix({ticket}): scoped commit"),
        )
        .await;
        sqlx::query(
            r#"
            INSERT INTO commit_ticket_correlations (
                org_id, commit_sha, ticket_id, correlation_source, confidence, created_at
            ) VALUES (
                $1::uuid, $2, $3, 'commit_message', 1.0, NOW() - INTERVAL '25 minutes'
            )
            "#,
        )
        .bind(org_id)
        .bind(sha)
        .bind(ticket)
        .execute(&pool)
        .await
        .expect("insert tenant test correlation");
        insert_pipeline_event(
            &pool,
            org_id,
            full_name,
            &format!("pipe-{sha}"),
            full_name,
            sha,
        )
        .await;
    }
    insert_pipeline_event(
        &pool,
        &org_a,
        "tenant-a/other-repo",
        "pipe-sha-a-wrong-repo",
        "tenant-a/other-repo",
        "sha-a",
    )
    .await;

    let (status, _) = json_request(
        &app,
        "GET",
        "/integrations/jira/ticket-coverage?branch=main&hours=72",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = json_request(
        &app,
        "GET",
        "/integrations/jira/ticket-coverage?org_name=tenant-a&branch=main&hours=72",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "coverage failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["total_commits"], 1);
    assert_eq!(parsed["commits_with_ticket"], 1);

    let (status, _) = json_request(
        &app,
        "GET",
        "/integrations/jenkins/correlations?limit=20",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = json_request(
        &app,
        "GET",
        "/integrations/jenkins/correlations?org_name=tenant-a&limit=20",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "jenkins correlations failed: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let correlations = parsed["correlations"].as_array().unwrap();
    assert_eq!(correlations.len(), 1);
    assert_eq!(correlations[0]["commit_sha"], "sha-a");
    assert_eq!(correlations[0]["pipeline"]["pipeline_id"], "pipe-sha-a");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn integration_status_routes_require_workspace_and_filter_by_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    insert_ticket(&pool, &org_a, "KAN-301", "Tenant A ticket").await;
    insert_ticket(&pool, &org_b, "KAN-302", "Tenant B ticket").await;
    insert_pipeline_event(
        &pool,
        &org_a,
        "tenant-a/repo",
        "status-a",
        "tenant-a/job",
        "sha-a",
    )
    .await;
    insert_pipeline_event(
        &pool,
        &org_b,
        "tenant-b/repo",
        "status-b",
        "tenant-b/job",
        "sha-b",
    )
    .await;

    for uri in ["/integrations/jira/status", "/integrations/jenkins/status"] {
        let (status, _) = json_request(&app, "GET", uri, None, Some(&global_admin_key)).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{uri} should require org_name for global admin keys"
        );
    }

    let (status, body) = json_request(
        &app,
        "GET",
        "/integrations/jira/status?org_name=tenant-a",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "jira status failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["recent_tickets_24h"], 1);

    let (status, body) = json_request(
        &app,
        "GET",
        "/integrations/jenkins/status?org_name=tenant-a",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "jenkins status failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["recent_events_24h"], 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn pr_merge_evidence_route_requires_workspace_and_filters_by_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let repo_a = insert_repo_for_org(&pool, &org_a, "tenant-a/repo").await;
    let repo_b = insert_repo_for_org(&pool, &org_b, "tenant-b/repo").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    insert_pr_merge(
        &pool,
        &org_a,
        &repo_a,
        "tenant-a-pr-evidence",
        11,
        "fix(KAN-401): tenant A PR",
        "sha-a-401",
        "tenant-a/repo",
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_b,
        &repo_b,
        "tenant-b-pr-evidence",
        22,
        "fix(KAN-402): tenant B PR",
        "sha-b-402",
        "tenant-b/repo",
    )
    .await;

    let (status, _) = json_request(
        &app,
        "GET",
        "/pr-merges?limit=20",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = json_request(
        &app,
        "GET",
        "/pr-merges?org_name=tenant-a&limit=20",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "PR merge evidence failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = parsed["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["org_id"], org_a);
    assert_eq!(entries[0]["repo_full_name"], "tenant-a/repo");
    assert_eq!(entries[0]["pr_number"], 11);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn jira_correlate_pr_phase2_does_not_attach_prs_from_other_tenant() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let repo_a = insert_repo_for_org(&pool, &org_a, "tenant-a/repo").await;
    let repo_b = insert_repo_for_org(&pool, &org_b, "tenant-b/repo").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    insert_ticket(&pool, &org_a, "KAN-900", "Tenant A release ticket").await;
    insert_ticket(&pool, &org_b, "KAN-900", "Tenant B release ticket").await;
    insert_commit_event(
        &pool,
        &org_a,
        &repo_a,
        "tenant-a-correlate-commit",
        "sha-a-900",
        "main",
        "fix(KAN-900): tenant A governed change",
    )
    .await;
    insert_pr_merge(
        &pool,
        &org_b,
        &repo_b,
        "tenant-b-pr-delivery",
        77,
        "fix(KAN-900): tenant B unrelated PR",
        "sha-b-900",
        "tenant-b/repo",
    )
    .await;

    let payload = serde_json::json!({
        "org_name": "tenant-a",
        "repo_full_name": "tenant-a/repo",
        "hours": 72,
        "limit": 100
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/integrations/jira/correlate",
        Some(&payload.to_string()),
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "jira correlate failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["scanned_commits"], 1);
    assert_eq!(parsed["correlations_created"], 1);

    let related_prs: Vec<String> = sqlx::query_scalar(
        "SELECT UNNEST(related_prs) FROM project_tickets WHERE org_id = $1::uuid AND ticket_id = 'KAN-900'",
    )
    .bind(&org_a)
    .fetch_all(&pool)
    .await
    .expect("read related PRs for tenant A ticket");

    assert!(
        related_prs.is_empty(),
        "tenant A ticket must not receive tenant B PR refs: {related_prs:?}"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn client_events_ingest_requires_and_enforces_effective_org_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "tenant-a").await;
    let org_b = insert_test_org(&pool, "tenant-b").await;
    let _repo_a = insert_repo_for_org(&pool, &org_a, "tenant-a/repo").await;
    let _repo_b = insert_repo_for_org(&pool, &org_b, "tenant-b/repo").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let scoped_a_key = insert_test_api_key_for_org(&pool, "tenant-a-admin", "Admin", &org_a).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let unscoped_payload = serde_json::json!({
        "events": [{
            "event_uuid": "unscoped-event",
            "event_type": "commit",
            "user_login": "dev",
            "files": [],
            "status": "success",
            "commit_sha": "unscoped-sha"
        }],
        "client_version": "integration-test"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&unscoped_payload.to_string()),
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unscoped ingest response failed: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 1);
    assert_eq!(
        parsed["errors"][0]["error"],
        "org_name or resolvable repo_full_name is required for global admin keys"
    );

    let scoped_payload = serde_json::json!({
        "events": [{
            "event_uuid": "scoped-event",
            "event_type": "commit",
            "org_name": "tenant-a",
            "repo_full_name": "tenant-a/repo",
            "user_login": "dev",
            "files": [],
            "status": "success",
            "commit_sha": "scoped-sha"
        }],
        "client_version": "integration-test"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&scoped_payload.to_string()),
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "scoped ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 1);

    let stored_org: String = sqlx::query_scalar(
        "SELECT org_id::text FROM client_events WHERE event_uuid = 'scoped-event'",
    )
    .fetch_one(&pool)
    .await
    .expect("read scoped event org");
    assert_eq!(stored_org, org_a);

    let cross_payload = serde_json::json!({
        "events": [{
            "event_uuid": "cross-event",
            "event_type": "commit",
            "org_name": "tenant-b",
            "repo_full_name": "tenant-b/repo",
            "user_login": "dev",
            "files": [],
            "status": "success",
            "commit_sha": "cross-sha"
        }],
        "client_version": "integration-test"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&cross_payload.to_string()),
        Some(&scoped_a_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "cross ingest response failed: {body}"
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(
        parsed["errors"][0]["error"],
        "Event org_name is outside API key scope"
    );

    teardown(&admin_pool, &schema).await;
}
