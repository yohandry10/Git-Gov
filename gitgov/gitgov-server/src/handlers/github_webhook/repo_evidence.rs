struct GenericRepoEvidenceEvent<'a> {
    state: &'a Arc<AppState>,
    delivery_id: &'a str,
    payload: &'a serde_json::Value,
    event_type: &'a str,
    actor_login: Option<String>,
    actor_id: Option<i64>,
    ref_name: Option<String>,
    ref_type: Option<String>,
    after_sha: Option<String>,
    metadata: serde_json::Value,
}

#[derive(Debug, PartialEq, Eq)]
struct CheckRunEvidence {
    action: String,
    status: String,
    conclusion: Option<String>,
    after_sha: Option<String>,
    ref_name: Option<String>,
    details_url: Option<String>,
}

fn extract_check_run_evidence(payload: &serde_json::Value) -> CheckRunEvidence {
    let check_run = payload.get("check_run");
    let ref_name = check_run
        .and_then(|v| v.get("check_suite"))
        .and_then(|v| v.get("head_branch"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            check_run
                .and_then(|v| v.get("head_branch"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });

    CheckRunEvidence {
        action: payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        status: check_run
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        conclusion: check_run
            .and_then(|v| v.get("conclusion"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        after_sha: check_run
            .and_then(|v| v.get("head_sha"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        ref_name,
        details_url: check_run
            .and_then(|v| v.get("details_url"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CheckSuiteEvidence {
    action: String,
    status: String,
    conclusion: Option<String>,
    after_sha: Option<String>,
    ref_name: Option<String>,
}

fn extract_check_suite_evidence(payload: &serde_json::Value) -> CheckSuiteEvidence {
    let check_suite = payload.get("check_suite");

    CheckSuiteEvidence {
        action: payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        status: check_suite
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        conclusion: check_suite
            .and_then(|v| v.get("conclusion"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        after_sha: check_suite
            .and_then(|v| v.get("head_sha"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        ref_name: check_suite
            .and_then(|v| v.get("head_branch"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CommitStatusEvidence {
    state_name: String,
    context: Option<String>,
    description: Option<String>,
    target_url: Option<String>,
    after_sha: Option<String>,
    ref_name: Option<String>,
}

fn extract_commit_status_evidence(payload: &serde_json::Value) -> CommitStatusEvidence {
    CommitStatusEvidence {
        state_name: payload
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        context: payload
            .get("context")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        description: payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        target_url: payload
            .get("target_url")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        after_sha: payload
            .get("sha")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        ref_name: payload
            .get("branches")
            .and_then(|v| v.as_array())
            .and_then(|branches| branches.first())
            .and_then(|entry| entry.get("name"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }
}

async fn store_generic_repo_evidence_event(
    input: GenericRepoEvidenceEvent<'_>,
) -> Result<(), String> {
    let repo_val = match input.payload.get("repository") {
        Some(r) => r,
        None => {
            tracing::warn!(
                "{} event missing 'repository' field, delivery_id={}",
                input.event_type,
                input.delivery_id
            );
            return Ok(());
        }
    };
    let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                "Failed to parse repository in {} event: {}, delivery_id={}",
                input.event_type,
                e,
                input.delivery_id
            );
            return Ok(());
        }
    };
    let (org_id, repo_id) = get_or_create_org_repo(&input.state.db, &repo).await?;

    let commit_shas = input.after_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
    let mut enriched_payload = input.payload.clone();
    if let Some(obj) = enriched_payload.as_object_mut() {
        obj.insert("gitgov".to_string(), input.metadata);
    }

    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: input.delivery_id.to_string(),
        event_type: input.event_type.to_string(),
        actor_login: input.actor_login,
        actor_id: input.actor_id,
        ref_name: input.ref_name,
        ref_type: input.ref_type,
        before_sha: None,
        after_sha: input.after_sha,
        commit_shas: commit_shas.clone(),
        commits_count: commit_shas.len() as i32,
        payload: enriched_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    input.state.db.insert_github_event(&event).await.map_err(|e| {
        tracing::error!("Failed to insert {} github event: {}", input.event_type, e);
        "Internal database error".to_string()
    })?;

    if let Some(ref org_id) = event.org_id {
        if let Err(e) = input.state.db.enqueue_job(org_id, "detect_signals", None).await {
            tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
        }
    }

    tracing::info!(
        "Processed {} event: repo={} ref={} sha={} actor={}",
        input.event_type,
        repo.full_name,
        event.ref_name.as_deref().unwrap_or("n/a"),
        event.after_sha.as_deref().unwrap_or("n/a"),
        event.actor_login.as_deref().unwrap_or("unknown")
    );

    Ok(())
}

async fn process_check_run_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let evidence = extract_check_run_evidence(payload);
    let (actor_login, actor_id) = extract_sender_actor(payload);

    store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
        state,
        delivery_id,
        payload,
        event_type: "check_run",
        actor_login,
        actor_id,
        ref_name: evidence.ref_name,
        ref_type: Some("branch".to_string()),
        after_sha: evidence.after_sha,
        metadata: serde_json::json!({
            "action": evidence.action,
            "status": evidence.status,
            "conclusion": evidence.conclusion,
            "details_url": evidence.details_url
        }),
    })
    .await
}

async fn process_check_suite_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let evidence = extract_check_suite_evidence(payload);
    let (actor_login, actor_id) = extract_sender_actor(payload);

    store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
        state,
        delivery_id,
        payload,
        event_type: "check_suite",
        actor_login,
        actor_id,
        ref_name: evidence.ref_name,
        ref_type: Some("branch".to_string()),
        after_sha: evidence.after_sha,
        metadata: serde_json::json!({
            "action": evidence.action,
            "status": evidence.status,
            "conclusion": evidence.conclusion
        }),
    })
    .await
}

async fn process_status_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let evidence = extract_commit_status_evidence(payload);
    let (actor_login, actor_id) = extract_sender_actor(payload);

    store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
        state,
        delivery_id,
        payload,
        event_type: "status",
        actor_login,
        actor_id,
        ref_name: evidence.ref_name,
        ref_type: Some("branch".to_string()),
        after_sha: evidence.after_sha,
        metadata: serde_json::json!({
            "state": evidence.state_name,
            "context": evidence.context,
            "description": evidence.description,
            "target_url": evidence.target_url
        }),
    })
    .await
}
