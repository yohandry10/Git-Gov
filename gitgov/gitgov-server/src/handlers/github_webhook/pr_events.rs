use base64::Engine;

#[derive(Debug, Deserialize)]
struct GitHubPrReviewUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPrFile {
    filename: String,
}

#[derive(Debug, Deserialize)]
struct GitHubContentResponse {
    #[serde(default)]
    sha: Option<String>,
    content: String,
    encoding: String,
}

struct MergedPrPolicyActivation<'a> {
    state: &'a Arc<AppState>,
    repo_full_name: &'a str,
    repo_id: &'a str,
    pr_number: i32,
    activation_sha: Option<&'a str>,
    activation_branch: Option<&'a str>,
    actor: Option<&'a str>,
    reviewers: &'a [String],
}

#[derive(Debug, Deserialize)]
struct GitHubPrReview {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    user: Option<GitHubPrReviewUser>,
}

fn extract_final_approvers(reviews: &[GitHubPrReview]) -> Vec<String> {
    // GitHub reviews are evaluated per reviewer by latest review state.
    let mut latest_state_by_user: HashMap<String, String> = HashMap::new();

    for review in reviews {
        let Some(user) = review.user.as_ref() else { continue };
        let state = review
            .state
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_ascii_uppercase();
        if state.is_empty() {
            continue;
        }
        latest_state_by_user.insert(user.login.clone(), state);
    }

    let mut approvers: Vec<String> = latest_state_by_user
        .into_iter()
        .filter_map(|(login, state)| (state == "APPROVED").then_some(login))
        .collect();

    approvers.sort();
    approvers
}

async fn fetch_pr_approvers(
    http_client: &reqwest::Client,
    github_token: &str,
    repo_full_name: &str,
    pr_number: i32,
) -> Result<Vec<String>, String> {
    let mut all_reviews = Vec::new();
    let mut page = 1u8;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/pulls/{}/reviews?per_page=100&page={}",
            repo_full_name, pr_number, page
        );

        let response = http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", github_token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gitgov-server")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| format!("GitHub reviews request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("GitHub reviews API returned {}", status));
        }

        let reviews: Vec<GitHubPrReview> = response
            .json()
            .await
            .map_err(|e| format!("GitHub reviews decode failed: {}", e))?;

        let chunk_len = reviews.len();
        all_reviews.extend(reviews);

        if chunk_len < 100 || page >= 10 {
            break;
        }

        page += 1;
    }

    Ok(extract_final_approvers(&all_reviews))
}

async fn fetch_pr_changed_files(
    http_client: &reqwest::Client,
    github_token: &str,
    repo_full_name: &str,
    pr_number: i32,
) -> Result<Vec<GitHubPrFile>, String> {
    let mut files = Vec::new();
    let mut page = 1u8;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/pulls/{}/files?per_page=100&page={}",
            repo_full_name, pr_number, page
        );
        let response = github_get(http_client, github_token, &url).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("GitHub PR files API returned {}", status));
        }

        let chunk: Vec<GitHubPrFile> = response
            .json()
            .await
            .map_err(|e| format!("GitHub PR files decode failed: {}", e))?;
        let chunk_len = chunk.len();
        files.extend(chunk);

        if chunk_len < 100 || page >= 10 {
            break;
        }
        page += 1;
    }

    Ok(files)
}

async fn fetch_policy_file_blob(
    http_client: &reqwest::Client,
    github_token: &str,
    repo_full_name: &str,
    policy_path: &str,
    git_ref: &str,
) -> Result<(String, Option<String>), String> {
    let url = format!(
        "https://api.github.com/repos/{}/contents/{}?ref={}",
        repo_full_name, policy_path, git_ref
    );
    let response = github_get(http_client, github_token, &url).await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("GitHub contents API returned {}", status));
    }

    let body: GitHubContentResponse = response
        .json()
        .await
        .map_err(|e| format!("GitHub contents decode failed: {}", e))?;
    if body.encoding != "base64" {
        return Err(format!("Unsupported GitHub content encoding: {}", body.encoding));
    }

    let compact_content = body.content.replace(['\n', '\r'], "");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(compact_content.as_bytes())
        .map_err(|e| format!("GitHub content base64 decode failed: {}", e))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| format!("GitHub policy file is not UTF-8: {}", e))?;

    Ok((text, body.sha))
}

async fn github_get(
    http_client: &reqwest::Client,
    github_token: &str,
    url: &str,
) -> Result<reqwest::Response, String> {
    http_client
        .get(url)
        .header("Authorization", format!("Bearer {}", github_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "gitgov-server")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {}", e))
}

async fn activate_repo_policy_from_merged_pr(
    input: MergedPrPolicyActivation<'_>,
) -> Result<(), String> {
    let Some(github_token) = input.state.github_personal_access_token.as_deref() else {
        tracing::debug!(
            repo = %input.repo_full_name,
            pr_number = input.pr_number,
            "GITHUB_PERSONAL_ACCESS_TOKEN not configured; skipping repo Policy-as-Code activation"
        );
        return Ok(());
    };
    let Some(activation_sha) = input.activation_sha else {
        tracing::warn!(
            repo = %input.repo_full_name,
            pr_number = input.pr_number,
            "Merged PR has no activation SHA; skipping repo Policy-as-Code activation"
        );
        return Ok(());
    };

    let files = fetch_pr_changed_files(
        &input.state.http_client,
        github_token,
        input.repo_full_name,
        input.pr_number,
    )
    .await?;
    let changed_policy_paths = files
        .iter()
        .filter_map(|file| {
            gitgov_policy_core::DEFAULT_POLICY_PATHS
                .iter()
                .find(|(path, _)| *path == file.filename.as_str())
                .map(|(path, format)| ((*path).to_string(), *format))
        })
        .collect::<Vec<_>>();

    if changed_policy_paths.is_empty() {
        return Ok(());
    }
    if changed_policy_paths.len() > 1 {
        tracing::warn!(
            repo = %input.repo_full_name,
            pr_number = input.pr_number,
            paths = ?changed_policy_paths,
            "Multiple policy files changed; refusing automatic policy activation"
        );
        return Ok(());
    }

    let (policy_path, format) = changed_policy_paths[0].clone();
    let (content, blob_sha) = fetch_policy_file_blob(
        &input.state.http_client,
        github_token,
        input.repo_full_name,
        &policy_path,
        activation_sha,
    )
    .await?;
    let config = gitgov_policy_core::parse_policy_str(&content, format, &policy_path)
        .map_err(|e| e.to_string())?;
    let checksum = gitgov_policy_core::policy_checksum(&config).map_err(|e| e.to_string())?;
    let source = PolicySourceMetadata {
        source_mode: PolicySourceMode::RepoPolicyAsCode,
        source_path: Some(policy_path.clone()),
        source_format: Some(format),
        activation_branch: input.activation_branch.map(str::to_string),
        commit_sha: Some(activation_sha.to_string()),
        blob_sha,
        pr_number: Some(input.pr_number as i64),
        actor: input.actor.map(str::to_string),
        reviewers: input.reviewers.to_vec(),
        source_checksum: Some(checksum.clone()),
        active_checksum: Some(checksum.clone()),
        drift_status: PolicyDriftStatus::InSync,
        emergency_override: None,
    };
    let override_actor = input.actor.unwrap_or("github-webhook");

    input
        .state
        .db
        .save_policy_with_source(input.repo_id, &config, &checksum, override_actor, &source)
        .await
        .map_err(|e| format!("Failed to activate repo policy: {}", e))?;

    tracing::info!(
        repo = %input.repo_full_name,
        pr_number = input.pr_number,
        policy_path = %policy_path,
        checksum = %checksum,
        commit_sha = %activation_sha,
        "Activated repo Policy-as-Code snapshot from merged PR"
    );

    Ok(())
}

// Processes pull_request webhook events.
// Stores every pull_request action as first-class evidence in github_events.
// Additionally stores merged PRs (action == "closed" && merged == true) in pr_merges.
async fn process_pull_request_event(
    state: &Arc<AppState>,
    delivery_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let pr = match payload.get("pull_request") {
        Some(pr) => pr,
        None => {
            tracing::debug!("pull_request event missing 'pull_request' field, delivery_id={}", delivery_id);
            return Ok(());
        }
    };

    // Extract repository info for org/repo lookup
    let repo_val = match payload.get("repository") {
        Some(r) => r,
        None => {
            tracing::warn!("pull_request event missing 'repository' field, delivery_id={}", delivery_id);
            return Ok(());
        }
    };
    let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to parse repository in pull_request event: {}, delivery_id={}", e, delivery_id);
            return Ok(());
        }
    };

    let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;

    let merged = pr.get("merged").and_then(|v| v.as_bool()).unwrap_or(false);
    let draft = pr.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
    let pr_number = pr.get("number").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let pr_title = pr.get("title").and_then(|v| v.as_str()).map(String::from);
    let author_login = pr.get("user").and_then(|u| u.get("login")).and_then(|v| v.as_str()).map(String::from);
    let merged_by_login = pr.get("merged_by").and_then(|u| u.get("login")).and_then(|v| v.as_str()).map(String::from);
    let head_sha = pr.get("head").and_then(|h| h.get("sha")).and_then(|v| v.as_str()).map(String::from);
    let merge_commit_sha = pr
        .get("merge_commit_sha")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let base_branch = pr.get("base").and_then(|b| b.get("ref")).and_then(|v| v.as_str()).map(String::from);
    let sender_actor = payload
        .get("sender")
        .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
    let actor_login = sender_actor.as_ref().map(|s| s.login.clone());
    let actor_id = sender_actor.as_ref().map(|s| s.id);
    let requested_reviewers_count = pr
        .get("requested_reviewers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    let mut pr_payload = payload.clone();
    if let Some(obj) = pr_payload.as_object_mut() {
        obj.insert(
            "gitgov".to_string(),
            serde_json::json!({
                "action": action.clone(),
                "merged": merged,
                "draft": draft,
                "pr_number": pr_number,
                "requested_reviewers_count": requested_reviewers_count,
                "merge_commit_sha": merge_commit_sha.clone()
            }),
        );
    }

    let pr_commit_shas = if action == "closed" && merged {
        merged_pr_ticket_targets(head_sha.as_deref(), merge_commit_sha.as_deref())
            .into_iter()
            .map(|(_, sha)| sha.to_string())
            .collect::<Vec<_>>()
    } else {
        head_sha.clone().map(|sha| vec![sha]).unwrap_or_default()
    };

    let pr_event = GitHubEvent {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id.clone()),
        repo_id: Some(repo_id.clone()),
        delivery_id: delivery_id.to_string(),
        event_type: "pull_request".to_string(),
        actor_login: actor_login.clone(),
        actor_id,
        ref_name: base_branch
            .clone()
            .or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number))),
        ref_type: Some("pull_request".to_string()),
        before_sha: None,
        after_sha: merge_commit_sha.clone().or_else(|| head_sha.clone()),
        commits_count: pr_commit_shas.len() as i32,
        commit_shas: pr_commit_shas,
        payload: pr_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    let inserted_github_event = match state.db.insert_github_event(&pr_event).await {
        Ok(()) => true,
        Err(DbError::Duplicate(_)) => {
            tracing::debug!(
                "Duplicate pull_request github event observed: delivery_id={}",
                delivery_id
            );
            false
        }
        Err(e) => {
            tracing::error!("Failed to insert pull_request github event: {}", e);
            return Err("Internal database error".to_string());
        }
    };

    if inserted_github_event {
        if let Some(ref org_id) = pr_event.org_id {
            if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
                tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
            }
        }
    }

    // Only merged PRs are materialized into pr_merges.
    if action != "closed" || !merged {
        tracing::info!(
            "Processed pull_request event: repo={} pr=#{} action={} actor={}",
            repo.full_name,
            pr_number,
            action,
            actor_login.as_deref().unwrap_or("unknown"),
        );
        return Ok(());
    }

    let approvers = match state.github_personal_access_token.as_deref() {
        Some(token) => match fetch_pr_approvers(&state.http_client, token, &repo.full_name, pr_number).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    delivery_id = %delivery_id,
                    repo = %repo.full_name,
                    pr_number,
                    error = %e,
                    "Failed to fetch PR approvers from GitHub API"
                );
                vec![]
            }
        },
        None => {
            tracing::debug!(
                delivery_id = %delivery_id,
                repo = %repo.full_name,
                pr_number,
                "GITHUB_PERSONAL_ACCESS_TOKEN not configured; storing PR merge without approvers"
            );
            vec![]
        }
    };
    let approvals_count = approvers.len() as i32;

    let mut enriched_payload = payload.clone();
    if let Some(obj) = enriched_payload.as_object_mut() {
        obj.insert(
            "gitgov".to_string(),
            serde_json::json!({
                "action": action.clone(),
                "merged": merged,
                "draft": draft,
                "pr_number": pr_number,
                "requested_reviewers_count": requested_reviewers_count,
                "approvers": approvers.clone(),
                "approvals_count": approvals_count
            }),
        );
    }

    let head_sha_clone = head_sha.clone();
    let merge_commit_sha_clone = merge_commit_sha.clone();
    let base_branch_clone = base_branch.clone();
    let record = PrMergeRecord {
        id: Uuid::new_v4().to_string(),
        org_id: Some(org_id),
        repo_id: Some(repo_id),
        delivery_id: delivery_id.to_string(),
        pr_number,
        pr_title: pr_title.clone(),
        author_login: author_login.clone(),
        merged_by_login: merged_by_login.clone(),
        head_sha,
        base_branch,
        payload: enriched_payload,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    let inserted_pr_merge = match state.db.insert_pr_merge(&record).await {
        Ok(()) => true,
        Err(DbError::Duplicate(_)) => {
            tracing::debug!("Duplicate PR merge event observed: delivery_id={}", delivery_id);
            false
        }
        Err(e) => return Err(format!("Failed to insert PR merge: {}", e)),
    };

    if inserted_pr_merge {
        tracing::info!(
            "Processed PR merge: #{} '{}' by {} merged by {} (approvals={}), delivery_id={}",
            pr_number,
            pr_title.as_deref().unwrap_or(""),
            author_login.as_deref().unwrap_or("unknown"),
            merged_by_login.as_deref().unwrap_or("unknown"),
            approvals_count,
            delivery_id,
        );
    }

    // Auto-correlate on both fresh and duplicate PR merge deliveries. This keeps
    // webhook processing idempotent and lets redelivery repair missing coverage.
    let title_sources = [pr_title.as_deref().unwrap_or_default()];
    let mut correlated_ticket_ids = std::collections::BTreeSet::new();
    for (source, commit_sha) in merged_pr_ticket_targets(
        head_sha_clone.as_deref(),
        merge_commit_sha_clone.as_deref(),
    ) {
        let correlated = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
            state,
            org_id: record.org_id.as_deref(),
            repo_full_name: &repo.full_name,
            pr_number,
            commit_sha: Some(commit_sha),
            branch: base_branch_clone.as_deref(),
            source,
            text_sources: &title_sources,
        })
        .await?;
        correlated_ticket_ids.extend(correlated);
    }

    if !correlated_ticket_ids.is_empty() {
        tracing::info!(
            pr_ref = format!("{}#{}", repo.full_name, pr_number),
            tickets = ?correlated_ticket_ids,
            "Auto-correlated merged PR commits with tickets from title"
        );
    }

    let activation_sha = merge_commit_sha_clone
        .as_deref()
        .or(head_sha_clone.as_deref());
    if let Some(repo_id) = record.repo_id.as_deref() {
        activate_repo_policy_from_merged_pr(MergedPrPolicyActivation {
            state,
            repo_full_name: &repo.full_name,
            repo_id,
            pr_number,
            activation_sha,
            activation_branch: base_branch_clone.as_deref(),
            actor: merged_by_login.as_deref().or(actor_login.as_deref()),
            reviewers: &approvers,
        })
        .await?;
    }

    Ok(())
}

async fn get_or_create_org_repo(db: &Database, repo: &GitHubRepository) -> Result<(String, String), String> {
    // Get or create org
    let org_id = if let Some(ref org) = repo.organization {
        db.upsert_org(org.id, &org.login, None, None).await
            .map_err(|e| e.to_string())?
    } else {
        // If no organization, use the owner as org
        db.upsert_org(repo.owner.id, &repo.owner.login, None, None).await
            .map_err(|e| e.to_string())?
    };

    // Get or create repo
    let repo_id = db.upsert_repo(
        Some(&org_id),
        repo.id,
        &repo.full_name,
        &repo.name,
        repo.private,
    ).await.map_err(|e| e.to_string())?;

    Ok((org_id, repo_id))
}
