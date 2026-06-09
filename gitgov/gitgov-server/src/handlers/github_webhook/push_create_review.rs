async fn process_push_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let push: PushEvent = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Failed to parse push event: {}", e))?;

    // Extract org/repo info
    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &push.repository).await?;

    // Extract commit SHAs
    let commit_shas: Vec<String> = push.commits.iter().map(|c| c.id.clone()).collect();
    let commits_count = commit_shas.len() as i32;

    // Determine ref type
    let ref_type = if push.r#ref.starts_with("refs/tags/") {
        "tag"
    } else {
        "branch"
    };

    let ref_name = push.r#ref
        .strip_prefix("refs/heads/")
        .or_else(|| push.r#ref.strip_prefix("refs/tags/"))
        .unwrap_or(&push.r#ref)
        .to_string();

    let actor_login = push.sender.login.clone();
    // Keep canonical type as "push" for compatibility with existing stats/signals SQL.
    let event_type = "push";

    if push.forced {
        tracing::warn!(
            actor = %actor_login,
            ref_name = %ref_name,
            repo = %push.repository.full_name,
            "Force push detected — history rewrite on branch"
        );
    }

    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        event_type: event_type.to_string(),
        actor_login: Some(push.sender.login),
        actor_id: Some(push.sender.id),
        ref_name: Some(ref_name.clone()),
        ref_type: Some(ref_type.to_string()),
        before_sha: Some(push.before),
        after_sha: Some(push.after),
        commit_shas,
        commits_count,
        payload: payload.clone(),
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    state.db.insert_github_event(&event).await
        .map_err(|e| {
            tracing::error!("Failed to insert github event: {}", e);
            "Internal database error".to_string()
        })?;

    tracing::info!(
        "Processed {} event: {} commits to {} by {}",
        event_type,
        event.commits_count,
        ref_name,
        actor_login
    );

    // Enqueue detection job instead of spawning directly (backpressure control)
    if let Some(ref org_id) = event.org_id {
        if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
            tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
        }
    }

    Ok(())
}

async fn process_create_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let create: CreateEvent = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Failed to parse create event: {}", e))?;

    // Extract org/repo info
    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &create.repository).await?;

    let ref_name = create.r#ref.clone();
    let ref_type = create.ref_type.clone();
    let actor_login = create.sender.login.clone();

    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        event_type: "create".to_string(),
        actor_login: Some(create.sender.login),
        actor_id: Some(create.sender.id),
        ref_name: Some(create.r#ref),
        ref_type: Some(create.ref_type),
        before_sha: None,
        after_sha: None,
        commit_shas: vec![],
        commits_count: 0,
        payload: payload.clone(),
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    state.db.insert_github_event(&event).await
        .map_err(|e| format!("Failed to insert github event: {}", e))?;

    tracing::info!(
        "Processed create event: {} {} by {}",
        ref_type,
        ref_name,
        actor_login
    );

    Ok(())
}

async fn process_pull_request_review_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let repo_val = match payload.get("repository") {
        Some(r) => r,
        None => {
            tracing::warn!(
                "pull_request_review event missing 'repository' field, delivery_id={}",
                delivery_id
            );
            return Ok(());
        }
    };
    let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                "Failed to parse repository in pull_request_review event: {}, delivery_id={}",
                e,
                delivery_id
            );
            return Ok(());
        }
    };
    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;

    let sender = payload
        .get("sender")
        .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
    let actor_login = sender.as_ref().map(|s| s.login.clone());
    let actor_id = sender.as_ref().map(|s| s.id);

    let pr = payload.get("pull_request");
    let pr_number = pr
        .and_then(|p| p.get("number"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let base_branch = pr
        .and_then(|p| p.get("base"))
        .and_then(|b| b.get("ref"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let head_sha = pr
        .and_then(|p| p.get("head"))
        .and_then(|b| b.get("sha"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let review = payload.get("review");
    let review_state = review
        .and_then(|r| r.get("state"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let review_commit_sha = review
        .and_then(|r| r.get("commit_id"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let after_sha = review_commit_sha.or(head_sha);
    let commit_shas = after_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
    let ref_name = base_branch
        .clone()
        .or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number)));

    let mut enriched_payload = payload.clone();
    if let Some(obj) = enriched_payload.as_object_mut() {
        obj.insert(
            "gitgov".to_string(),
            serde_json::json!({
                "review_action": action,
                "review_state": review_state,
                "pr_number": pr_number
            }),
        );
    }

    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        event_type: "pull_request_review".to_string(),
        actor_login,
        actor_id,
        ref_name,
        ref_type: Some("pull_request".to_string()),
        before_sha: None,
        after_sha,
        commit_shas: commit_shas.clone(),
        commits_count: commit_shas.len() as i32,
        payload: enriched_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    state.db.insert_github_event(&event).await.map_err(|e| {
        tracing::error!("Failed to insert pull_request_review github event: {}", e);
        "Internal database error".to_string()
    })?;

    if let Some(ref org_id) = event.org_id {
        if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
            tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
        }
    }

    tracing::info!(
        "Processed pull_request_review event: repo={} pr=#{} action={} state={} actor={}",
        repo.full_name,
        pr_number,
        action,
        review_state,
        event.actor_login.as_deref().unwrap_or("unknown")
    );

    Ok(())
}

fn extract_sender_actor(payload: &serde_json::Value) -> (Option<String>, Option<i64>) {
    let sender = payload
        .get("sender")
        .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
    (
        sender.as_ref().map(|s| s.login.clone()),
        sender.as_ref().map(|s| s.id),
    )
}
