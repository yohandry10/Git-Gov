use super::common::*;

#[tokio::test]
async fn ticket_coverage_counts_pr_merge_commit_without_client_event() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Database::from_pool(pool.clone());

    let org_id = uuid::Uuid::new_v4().to_string();
    let repo_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind("acme")
        .bind("Acme Inc")
        .execute(&pool)
        .await
        .expect("insert org for ticket coverage test");

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(&org_id)
    .bind("acme/repo")
    .bind("repo")
    .execute(&pool)
    .await
    .expect("insert repo for ticket coverage test");

    sqlx::query(
        r#"
        INSERT INTO pull_request_merges (
            org_id, repo_id, delivery_id, pr_number, pr_title,
            author_login, merged_by_login, head_sha, base_branch, payload, created_at
        ) VALUES (
            $1::uuid, $2::uuid, 'delivery-pr-1', 34, 'docs(KAN-4): validate traceability',
            'alice', 'bob', 'head123', 'main',
            '{"pull_request":{"merge_commit_sha":"merge123"}}'::jsonb,
            NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(&org_id)
    .bind(&repo_id)
    .execute(&pool)
    .await
    .expect("insert PR merge evidence");

    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            org_id, commit_sha, ticket_id, correlation_source, created_at
        ) VALUES (
            $1::uuid, 'merge123', 'KAN-4', 'pr_title', NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert PR-title correlation");

    // The ticket id is verified only because a real Jira ticket was ingested.
    sqlx::query(
        r#"
        INSERT INTO project_tickets (org_id, ticket_id, project_key, status)
        VALUES ($1::uuid, 'KAN-4', 'KAN', 'Done')
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert ingested Jira ticket KAN-4");

    let coverage = db
        .get_ticket_coverage(Some("acme"), None, Some("acme/repo"), Some("main"), 72)
        .await
        .expect("ticket coverage should include PR merge commits");

    assert_eq!(coverage.total_commits, 1);
    assert_eq!(coverage.commits_with_ticket, 1);
    assert_eq!(coverage.coverage_percentage, 100.0);
    assert_eq!(coverage.detected_unverified_commits, 0);
    assert!(coverage.commits_without_ticket.is_empty());

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn ticket_coverage_excludes_pattern_only_unverified_tickets() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Database::from_pool(pool.clone());

    let org_id = uuid::Uuid::new_v4().to_string();
    let repo_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind("acme")
        .bind("Acme Inc")
        .execute(&pool)
        .await
        .expect("insert org for unverified coverage test");

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(&org_id)
    .bind("acme/repo")
    .bind("repo")
    .execute(&pool)
    .await
    .expect("insert repo for unverified coverage test");

    // A real commit event whose message embedded a ticket id...
    sqlx::query(
        r#"
        INSERT INTO client_events (
            id, org_id, repo_id, event_uuid, event_type, user_login, commit_sha, branch, status, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2::uuid, $3, 'commit', 'mallory', 'forged1', 'main', 'success',
            NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(&org_id)
    .bind(&repo_id)
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(&pool)
    .await
    .expect("insert commit event");

    // ...and a pattern-detected correlation for a ticket that was NEVER ingested
    // from Jira (no matching project_tickets row).
    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            org_id, commit_sha, ticket_id, correlation_source, created_at
        ) VALUES (
            $1::uuid, 'forged1', 'KAN-9999', 'commit_message', NOW() - INTERVAL '1 hour'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert pattern-only correlation");

    let coverage = db
        .get_ticket_coverage(Some("acme"), None, Some("acme/repo"), Some("main"), 72)
        .await
        .expect("ticket coverage should compute");

    // The unverified ticket must NOT raise verified coverage.
    assert_eq!(coverage.total_commits, 1);
    assert_eq!(coverage.commits_with_ticket, 0);
    assert_eq!(coverage.coverage_percentage, 0.0);
    // It is surfaced as an unverified detection instead.
    assert_eq!(coverage.detected_unverified_commits, 1);
    assert_eq!(coverage.unverified_coverage_percentage, 100.0);
    // The commit appears as "without verified ticket" but flagged as having an unverified link.
    assert_eq!(coverage.commits_without_ticket.len(), 1);
    assert_eq!(
        coverage.commits_without_ticket[0]
            .get("unverified_link")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn ticket_coverage_is_isolated_per_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Database::from_pool(pool.clone());

    // Two tenants, each with one verified commit on main.
    let mut org_ids = Vec::new();
    for (login, sha) in [("org-a", "shaA"), ("org-b", "shaB")] {
        let org_id = uuid::Uuid::new_v4().to_string();
        let repo_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $2)")
            .bind(&org_id)
            .bind(login)
            .execute(&pool)
            .await
            .expect("insert org");
        sqlx::query(
            "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $3)",
        )
        .bind(&repo_id)
        .bind(&org_id)
        .bind(format!("{login}/repo"))
        .execute(&pool)
        .await
        .expect("insert repo");
        sqlx::query(
            r#"
            INSERT INTO client_events (
                id, org_id, repo_id, event_uuid, event_type, user_login, commit_sha, branch, status, created_at
            ) VALUES (
                gen_random_uuid(), $1::uuid, $2::uuid, $3, 'commit', 'dev', $4, 'main', 'success',
                NOW() - INTERVAL '1 hour'
            )
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(sha)
        .execute(&pool)
        .await
        .expect("insert commit");
        sqlx::query(
            "INSERT INTO commit_ticket_correlations (org_id, commit_sha, ticket_id, correlation_source, created_at) VALUES ($1::uuid, $2, 'KAN-100', 'pr_title', NOW() - INTERVAL '1 hour')",
        )
        .bind(&org_id)
        .bind(sha)
        .execute(&pool)
        .await
        .expect("insert correlation");
        sqlx::query(
            "INSERT INTO project_tickets (org_id, ticket_id, project_key, status) VALUES ($1::uuid, 'KAN-100', 'KAN', 'Done')",
        )
        .bind(&org_id)
        .execute(&pool)
        .await
        .expect("insert ticket");
        org_ids.push(org_id);
    }

    // Each org's coverage must only count its own commit, never the other org's.
    let cov_a = db
        .get_ticket_coverage(None, Some(&org_ids[0]), None, Some("main"), 72)
        .await
        .expect("coverage A");
    assert_eq!(cov_a.total_commits, 1, "org A must only see its own commit");
    assert_eq!(cov_a.commits_with_ticket, 1);

    let cov_b = db
        .get_ticket_coverage(None, Some(&org_ids[1]), None, Some("main"), 72)
        .await
        .expect("coverage B");
    assert_eq!(cov_b.total_commits, 1, "org B must only see its own commit");
    assert_eq!(cov_b.commits_with_ticket, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn append_ticket_relations_does_not_contaminate_other_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Database::from_pool(pool.clone());

    // Both tenants own a ticket with the same id (KAN-4 is reused per org).
    let mut org_ids = Vec::new();
    for login in ["org-a", "org-b"] {
        let org_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $2)")
            .bind(&org_id)
            .bind(login)
            .execute(&pool)
            .await
            .expect("insert org");
        sqlx::query(
            "INSERT INTO project_tickets (org_id, ticket_id, project_key, status) VALUES ($1::uuid, 'KAN-4', 'KAN', 'Open')",
        )
        .bind(&org_id)
        .execute(&pool)
        .await
        .expect("insert ticket");
        org_ids.push(org_id);
    }

    // Append a commit relation scoped to org A only.
    let updated = db
        .append_project_ticket_relations_full(
            "KAN-4",
            Some(&org_ids[0]),
            Some("commitA"),
            Some("main"),
            None,
        )
        .await
        .expect("append relations");
    assert!(updated, "org A's ticket should be updated");

    let a_commits: Vec<String> = sqlx::query_scalar(
        "SELECT UNNEST(related_commits) FROM project_tickets WHERE org_id = $1::uuid AND ticket_id = 'KAN-4'",
    )
    .bind(&org_ids[0])
    .fetch_all(&pool)
    .await
    .expect("read A commits");
    assert_eq!(
        a_commits,
        vec!["commitA".to_string()],
        "org A must record the commit"
    );

    let b_count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(array_length(related_commits, 1), 0)::bigint FROM project_tickets WHERE org_id = $1::uuid AND ticket_id = 'KAN-4'",
    )
    .bind(&org_ids[1])
    .fetch_one(&pool)
    .await
    .expect("read B commits");
    assert_eq!(
        b_count, 0,
        "org B's identically-named ticket must NOT be contaminated"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn trigger_detection_falls_back_when_legacy_sql_detector_errors() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let org_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind("acme")
        .bind("Acme Inc")
        .execute(&pool)
        .await
        .expect("insert org for detect test");

    // Force legacy detector to fail and ensure HTTP path remains resilient.
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION detect_noncompliance_signals(
            p_org_id UUID,
            p_window_minutes INTEGER DEFAULT 15,
            p_tolerance_minutes INTEGER DEFAULT 30
        ) RETURNS INTEGER AS $$
        BEGIN
            RAISE EXCEPTION 'forced legacy detector failure';
        END;
        $$ LANGUAGE plpgsql;
        "#,
    )
    .execute(&pool)
    .await
    .expect("override legacy detector function");

    let (status, body) =
        json_request(&app, "POST", "/signals/detect/acme", None, Some(&api_key)).await;

    assert_eq!(status, StatusCode::OK, "detect endpoint failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(parsed["signals_created"].is_number());

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn compliance_dashboard_includes_monthly_timeline_points() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let org_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind("acme")
        .bind("Acme Inc")
        .execute(&pool)
        .await
        .expect("insert org");

    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION get_compliance_dashboard(p_org_id UUID)
        RETURNS JSON AS $$
        BEGIN
            RETURN json_build_object(
                'signals', json_build_object(
                    'total', 0,
                    'pending', 0,
                    'high_confidence', 0,
                    'by_type', '{}'::json
                ),
                'correlation', json_build_object(
                    'github_pushes_24h', 0,
                    'client_pushes_24h', 0,
                    'correlation_rate', 1.0
                ),
                'policy', json_build_object(
                    'repos_with_policy', 0,
                    'total_repos', 0,
                    'recent_changes', 0
                ),
                'exports', json_build_object(
                    'total', 0,
                    'last_7_days', 0
                )
            );
        END;
        $$ LANGUAGE plpgsql;
        "#,
    )
    .execute(&pool)
    .await
    .expect("create test compliance dashboard function");

    let repo_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(&org_id)
    .bind("acme/repo")
    .bind("repo")
    .execute(&pool)
    .await
    .expect("insert repo");

    sqlx::query(
        r#"
        INSERT INTO client_events (
            id, org_id, repo_id, event_uuid, event_type, user_login, commit_sha, status, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2::uuid, $3, 'commit', 'test-admin', 'deadbeef', 'success',
            NOW() - INTERVAL '5 days'
        )
        "#,
    )
    .bind(&org_id)
    .bind(&repo_id)
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(&pool)
    .await
    .expect("insert commit event");

    sqlx::query(
        r#"
        INSERT INTO commit_ticket_correlations (
            id, org_id, commit_sha, ticket_id, correlation_source, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, 'deadbeef', 'ACME-1', 'test', NOW() - INTERVAL '5 days'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert commit-ticket correlation");

    sqlx::query(
        r#"
        INSERT INTO pipeline_events (
            id, org_id, pipeline_id, job_name, status, commit_sha, branch, repo_full_name,
            ingested_at, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, 'p-1', 'job/main', 'success', 'deadbeef', 'main',
            'acme/repo',
            NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert pipeline event");

    sqlx::query(
        r#"
        INSERT INTO noncompliance_signals (
            id, org_id, signal_type, status, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, 'commit_no_ticket', 'pending',
            NOW() - INTERVAL '5 days'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert signal");

    sqlx::query(
        r#"
        INSERT INTO violations (
            id, org_id, repo_id, violation_type, created_at
        ) VALUES (
            gen_random_uuid(), $1::uuid, $2::uuid, 'policy_violation',
            NOW() - INTERVAL '5 days'
        )
        "#,
    )
    .bind(&org_id)
    .bind(&repo_id)
    .execute(&pool)
    .await
    .expect("insert violation");

    let (status, body) = json_request(&app, "GET", "/compliance/acme", None, Some(&api_key)).await;

    assert_eq!(status, StatusCode::OK, "compliance failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let timeline = parsed["timeline"].as_array().expect("timeline array");
    assert_eq!(timeline.len(), 6, "expected 6 monthly points");
    assert!(timeline
        .iter()
        .any(|item| item["commits_total"].as_i64().unwrap_or(0) >= 1));
    assert!(timeline
        .iter()
        .any(|item| item["pipeline_runs_total"].as_i64().unwrap_or(0) >= 1));

    teardown(&admin_pool, &schema).await;
}
