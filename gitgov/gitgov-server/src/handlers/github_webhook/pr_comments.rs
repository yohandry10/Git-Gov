fn json_string_at<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_str().map(str::trim).filter(|s| !s.is_empty())
}

fn json_i64_at(value: &serde_json::Value, path: &[&str]) -> Option<i64> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_i64()
}

fn pr_ref(repo_full_name: &str, pr_number: i32) -> Option<String> {
    (pr_number > 0).then(|| format!("{}#{}", repo_full_name, pr_number))
}

fn merged_pr_ticket_targets<'a>(
    head_sha: Option<&'a str>,
    merge_commit_sha: Option<&'a str>,
) -> Vec<(&'static str, &'a str)> {
    let mut targets = Vec::new();

    if let Some(sha) = merge_commit_sha.map(str::trim).filter(|s| !s.is_empty()) {
        targets.push(("pr_title", sha));
    }

    if let Some(sha) = head_sha.map(str::trim).filter(|s| !s.is_empty()) {
        let already_included = targets
            .iter()
            .any(|(_, existing_sha)| existing_sha.eq_ignore_ascii_case(sha));
        if !already_included {
            targets.push(("pr_title", sha));
        }
    }

    targets
}

#[derive(Debug, PartialEq, Eq)]
struct PullRequestReviewCommentEvidence {
    action: String,
    pr_number: i32,
    pr_title: Option<String>,
    base_branch: Option<String>,
    head_sha: Option<String>,
    comment_commit_sha: Option<String>,
    comment_body: Option<String>,
    commit_sha: Option<String>,
}

fn extract_pull_request_review_comment_evidence(
    payload: &serde_json::Value,
) -> PullRequestReviewCommentEvidence {
    let head_sha = json_string_at(payload, &["pull_request", "head", "sha"]).map(str::to_string);
    let comment_commit_sha = json_string_at(payload, &["comment", "commit_id"]).map(str::to_string);
    let commit_sha = comment_commit_sha.clone().or_else(|| head_sha.clone());

    PullRequestReviewCommentEvidence {
        action: json_string_at(payload, &["action"])
            .unwrap_or("unknown")
            .to_string(),
        pr_number: json_i64_at(payload, &["pull_request", "number"]).unwrap_or(0) as i32,
        pr_title: json_string_at(payload, &["pull_request", "title"]).map(str::to_string),
        base_branch: json_string_at(payload, &["pull_request", "base", "ref"]).map(str::to_string),
        head_sha,
        comment_commit_sha,
        comment_body: json_string_at(payload, &["comment", "body"]).map(str::to_string),
        commit_sha,
    }
}

struct TicketEvidenceCorrelation<'a> {
    state: &'a Arc<AppState>,
    org_id: Option<&'a str>,
    repo_full_name: &'a str,
    pr_number: i32,
    commit_sha: Option<&'a str>,
    branch: Option<&'a str>,
    source: &'a str,
    text_sources: &'a [&'a str],
}

async fn correlate_ticket_evidence_to_commit(
    input: TicketEvidenceCorrelation<'_>,
) -> Result<Vec<String>, String> {
    let Some(commit_sha) = input
        .commit_sha
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Ok(vec![]);
    };

    let ticket_ids = extract_ticket_ids(input.text_sources);
    if ticket_ids.is_empty() {
        return Ok(vec![]);
    }

    let pr_ref = pr_ref(input.repo_full_name, input.pr_number);
    let mut correlated = Vec::new();
    for ticket_id in ticket_ids {
        let correlation = CommitTicketCorrelation {
            id: Uuid::new_v4().to_string(),
            org_id: input.org_id.map(str::to_string),
            commit_sha: commit_sha.to_string(),
            ticket_id: ticket_id.clone(),
            correlation_source: input.source.to_string(),
            confidence: 0.9,
            created_at: chrono::Utc::now().timestamp_millis(),
        };

        match input
            .state
            .db
            .insert_commit_ticket_correlation(&correlation)
            .await
        {
            Ok(created) => {
                if created {
                    correlated.push(ticket_id.clone());
                }
                if let Err(e) = input
                    .state
                    .db
                    .append_project_ticket_relations_full(
                        &ticket_id,
                        input.org_id,
                        Some(commit_sha),
                        input.branch,
                        pr_ref.as_deref(),
                    )
                    .await
                {
                    tracing::debug!(
                        ticket_id = %ticket_id,
                        commit_sha = %commit_sha,
                        source = %input.source,
                        error = %e,
                        "Could not append ticket relations after GitHub comment evidence"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    ticket_id = %ticket_id,
                    commit_sha = %commit_sha,
                    source = %input.source,
                    error = %e,
                    "Failed to store ticket correlation from GitHub comment evidence"
                );
            }
        }
    }

    correlated.sort();
    Ok(correlated)
}

async fn process_pull_request_review_comment_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let evidence = extract_pull_request_review_comment_evidence(payload);
    let repo_val = match payload.get("repository") {
        Some(r) => r,
        None => {
            tracing::warn!(
                "pull_request_review_comment event missing 'repository' field, delivery_id={}",
                delivery_id
            );
            return Ok(());
        }
    };
    let repo: GitHubRepository = serde_json::from_value(repo_val.clone()).map_err(|e| {
        format!(
            "Failed to parse repository in pull_request_review_comment event: {}",
            e
        )
    })?;
    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
    let (actor_login, actor_id) = extract_sender_actor(payload);

    let review_comment_text_sources = [
        evidence.comment_body.as_deref().unwrap_or_default(),
        evidence.pr_title.as_deref().unwrap_or_default(),
    ];
    let correlated_tickets = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
        state,
        org_id: Some(&org_id),
        repo_full_name: &repo.full_name,
        pr_number: evidence.pr_number,
        commit_sha: evidence.commit_sha.as_deref(),
        branch: evidence.base_branch.as_deref(),
        source: "github_pr_review_comment",
        text_sources: &review_comment_text_sources,
    })
    .await?;

    let mut enriched_payload = payload.clone();
    if let Some(obj) = enriched_payload.as_object_mut() {
        obj.insert(
            "gitgov".to_string(),
            serde_json::json!({
                "action": evidence.action,
                "pr_number": evidence.pr_number,
                "comment_kind": "pull_request_review_comment",
                "ticket_correlations": correlated_tickets
            }),
        );
    }

    let commit_shas = evidence.commit_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        event_type: "pull_request_review_comment".to_string(),
        actor_login,
        actor_id,
        ref_name: evidence
            .base_branch
            .or_else(|| (evidence.pr_number > 0).then_some(format!("pr/{}", evidence.pr_number))),
        ref_type: Some("pull_request".to_string()),
        before_sha: None,
        after_sha: evidence.commit_sha,
        commit_shas: commit_shas.clone(),
        commits_count: commit_shas.len() as i32,
        payload: enriched_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    match state.db.insert_github_event(&event).await {
        Ok(()) => {}
        Err(DbError::Duplicate(_)) => {
            tracing::debug!(
                "Duplicate pull_request_review_comment event ignored: delivery_id={}",
                delivery_id
            );
            return Ok(());
        }
        Err(e) => {
            tracing::error!(
                "Failed to insert pull_request_review_comment github event: {}",
                e
            );
            return Err("Internal database error".to_string());
        }
    }

    tracing::info!(
        repo = %repo.full_name,
        pr_number = evidence.pr_number,
        correlated_tickets = ?correlated_tickets,
        "Processed pull_request_review_comment event"
    );

    Ok(())
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestLookupHead {
    sha: Option<String>,
    #[serde(default)]
    #[serde(rename = "ref")]
    ref_field: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestLookupBase {
    #[serde(default)]
    #[serde(rename = "ref")]
    ref_field: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestLookup {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    head: Option<GitHubPullRequestLookupHead>,
    #[serde(default)]
    base: Option<GitHubPullRequestLookupBase>,
}

async fn fetch_pr_lookup(
    http_client: &reqwest::Client,
    github_token: &str,
    repo_full_name: &str,
    pr_number: i32,
) -> Result<GitHubPullRequestLookup, String> {
    let url = format!(
        "https://api.github.com/repos/{}/pulls/{}",
        repo_full_name, pr_number
    );
    let response = http_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", github_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "gitgov-server")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("GitHub PR lookup request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("GitHub PR lookup API returned {}", status));
    }

    response
        .json()
        .await
        .map_err(|e| format!("GitHub PR lookup decode failed: {}", e))
}

async fn process_issue_comment_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let issue_is_pr = payload
        .get("issue")
        .and_then(|issue| issue.get("pull_request"))
        .is_some();
    if !issue_is_pr {
        tracing::debug!(
            "Ignoring issue_comment not linked to PR, delivery_id={}",
            delivery_id
        );
        return Ok(());
    }

    let action = json_string_at(payload, &["action"]).unwrap_or("unknown").to_string();
    let repo_val = match payload.get("repository") {
        Some(r) => r,
        None => {
            tracing::warn!("issue_comment event missing 'repository' field, delivery_id={}", delivery_id);
            return Ok(());
        }
    };
    let repo: GitHubRepository = serde_json::from_value(repo_val.clone())
        .map_err(|e| format!("Failed to parse repository in issue_comment event: {}", e))?;
    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
    let (actor_login, actor_id) = extract_sender_actor(payload);

    let pr_number = json_i64_at(payload, &["issue", "number"]).unwrap_or(0) as i32;
    let comment_body = json_string_at(payload, &["comment", "body"]);
    let issue_title = json_string_at(payload, &["issue", "title"]);

    let mut lookup_title = None;
    let mut head_sha = None;
    let mut base_branch = None;
    if pr_number > 0 {
        if let Some(token) = state.github_personal_access_token.as_deref() {
            match fetch_pr_lookup(&state.http_client, token, &repo.full_name, pr_number).await {
                Ok(lookup) => {
                    lookup_title = lookup.title;
                    head_sha = lookup.head.as_ref().and_then(|head| head.sha.clone());
                    base_branch = lookup.base.as_ref().and_then(|base| base.ref_field.clone());
                    if base_branch.is_none() {
                        base_branch = lookup.head.as_ref().and_then(|head| head.ref_field.clone());
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        delivery_id = %delivery_id,
                        repo = %repo.full_name,
                        pr_number,
                        error = %e,
                        "Failed to fetch PR metadata for issue_comment evidence"
                    );
                }
            }
        }
    }

    let issue_comment_text_sources = [
        comment_body.unwrap_or_default(),
        issue_title.unwrap_or_default(),
        lookup_title.as_deref().unwrap_or_default(),
    ];
    let correlated_tickets = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
        state,
        org_id: Some(&org_id),
        repo_full_name: &repo.full_name,
        pr_number,
        commit_sha: head_sha.as_deref(),
        branch: base_branch.as_deref(),
        source: "github_pr_issue_comment",
        text_sources: &issue_comment_text_sources,
    })
    .await?;

    let mut enriched_payload = payload.clone();
    if let Some(obj) = enriched_payload.as_object_mut() {
        obj.insert(
            "gitgov".to_string(),
            serde_json::json!({
                "action": action,
                "pr_number": pr_number,
                "comment_kind": "issue_comment_on_pull_request",
                "ticket_correlations": correlated_tickets
            }),
        );
    }

    let commit_shas = head_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
    let event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        event_type: "issue_comment".to_string(),
        actor_login,
        actor_id,
        ref_name: base_branch.or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number))),
        ref_type: Some("pull_request".to_string()),
        before_sha: None,
        after_sha: head_sha,
        commit_shas: commit_shas.clone(),
        commits_count: commit_shas.len() as i32,
        payload: enriched_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    match state.db.insert_github_event(&event).await {
        Ok(()) => {}
        Err(DbError::Duplicate(_)) => {
            tracing::debug!("Duplicate issue_comment event ignored: delivery_id={}", delivery_id);
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to insert issue_comment github event: {}", e);
            return Err("Internal database error".to_string());
        }
    }

    tracing::info!(
        repo = %repo.full_name,
        pr_number,
        correlated_tickets = ?correlated_tickets,
        "Processed issue_comment event linked to PR"
    );

    Ok(())
}
