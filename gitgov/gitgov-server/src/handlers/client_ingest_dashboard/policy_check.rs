fn repo_name_from_policy_check_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("git@")
    {
        if let Some(idx) = trimmed.find(':') {
            // git@github.com:owner/repo.git
            let candidate = &trimmed[idx + 1..];
            return candidate
                .trim_end_matches(".git")
                .trim_matches('/')
                .to_string();
        }
        if let Some(pos) = trimmed.find("github.com/") {
            let candidate = &trimmed[(pos + "github.com/".len())..];
            return candidate
                .trim_end_matches(".git")
                .trim_matches('/')
                .to_string();
        }
    }
    trimmed
        .trim_end_matches(".git")
        .trim_matches('/')
        .to_string()
}

fn branch_matches_policy(policy: &GitGovConfig, branch: &str) -> bool {
    if policy.branches.protected.iter().any(|b| b == branch) {
        return true;
    }

    for pattern in &policy.branches.patterns {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            if glob_pattern.matches(branch) {
                return true;
            }
        } else if pattern == branch {
            return true;
        }
    }

    false
}

fn org_name_from_repo_full_name(repo_full_name: &str) -> Option<&str> {
    repo_full_name
        .split('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn should_block_policy_check_transport(
    state: &AppState,
    repo_full_name: &str,
    branch: &str,
) -> bool {
    if state.policy_check_block_scopes.is_empty() {
        return false;
    }
    let org_name = match org_name_from_repo_full_name(repo_full_name) {
        Some(value) => value,
        None => return false,
    };
    state
        .policy_check_block_scopes
        .iter()
        .any(|scope| scope.matches(org_name, branch))
}

fn policy_check_response_status(
    state: &AppState,
    repo_full_name: &str,
    branch: &str,
    response: &PolicyCheckResponse,
) -> StatusCode {
    if response.allowed {
        return StatusCode::OK;
    }
    if should_block_policy_check_transport(state, repo_full_name, branch) {
        StatusCode::CONFLICT
    } else {
        StatusCode::OK
    }
}

fn ticket_id_regex() -> &'static Regex {
    static TICKET_ID_RE: OnceLock<Regex> = OnceLock::new();
    TICKET_ID_RE.get_or_init(|| {
        Regex::new(r"\b([A-Z][A-Z0-9]{1,15}-[0-9]{1,9})\b").expect("valid ticket id regex")
    })
}

fn extract_ticket_ids(texts: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for text in texts {
        for captures in ticket_id_regex().captures_iter(text) {
            if let Some(ticket) = captures.get(1) {
                let normalized = ticket.as_str().to_ascii_uppercase();
                if seen.insert(normalized.clone()) {
                    found.push(normalized);
                }
            }
        }
    }

    found
}

pub async fn policy_check(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PolicyCheckRequest>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(PolicyCheckResponse {
                advisory: true,
                allowed: false,
                reasons: vec!["Admin access required".to_string()],
                ..Default::default()
            }),
        );
    }

    if payload.repo.trim().is_empty() || payload.branch.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PolicyCheckResponse {
                advisory: true,
                allowed: false,
                reasons: vec!["repo and branch are required".to_string()],
                ..Default::default()
            }),
        );
    }

    metrics::counter!("gitgov_policy_checks_total").increment(1);

    let repo_name = repo_name_from_policy_check_input(&payload.repo);
    let branch = payload.branch.trim();
    let commit_sha = payload
        .commit
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let mut response = PolicyCheckResponse {
        advisory: true,
        allowed: true,
        reasons: vec![],
        warnings: vec![],
        evaluated_rules: vec![
            "repo_exists".to_string(),
            "policy_exists".to_string(),
            "branch_matches_policy".to_string(),
        ],
        ..Default::default()
    };

    let repo = match state.db.get_repo_by_full_name(&repo_name).await {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            response.allowed = false;
            response
                .reasons
                .push("Repository not found in GitGov".to_string());
            let status =
                policy_check_response_status(state.as_ref(), &repo_name, branch, &response);
            return (status, Json(response));
        }
        Err(_) => {
            response.allowed = false;
            response.reasons.push("Internal database error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    let policy = match state.db.get_policy(&repo.id).await {
        Ok(Some(policy)) => policy,
        Ok(None) => {
            response.allowed = false;
            response
                .reasons
                .push("No policy configured for repository".to_string());
            let status =
                policy_check_response_status(state.as_ref(), &repo_name, branch, &response);
            return (status, Json(response));
        }
        Err(_) => {
            response.allowed = false;
            response.reasons.push("Internal database error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    let config = &policy.config;
    let enforcement = &config.enforcement;
    let mut quality_gate_violation_context: Option<(String, String, String)> = None;
    let now_ms = chrono::Utc::now().timestamp_millis();
    let active_quality_gate_exception = config
        .quality_gate_exception
        .as_ref()
        .filter(|exception| exception.enabled && exception.expires_at > now_ms);
    let external_policy_enforcement = effective_external_policy_enforcement(config);

    // Determine highest enforcement level applied
    let has_block = [
        &enforcement.pull_requests,
        &enforcement.commits,
        &enforcement.branches,
        &enforcement.traceability,
        &enforcement.quality_gates,
        &external_policy_enforcement,
    ]
    .iter()
    .any(|e| **e == EnforcementLevel::Block);
    let has_warn = [
        &enforcement.pull_requests,
        &enforcement.commits,
        &enforcement.branches,
        &enforcement.traceability,
        &enforcement.quality_gates,
        &external_policy_enforcement,
    ]
    .iter()
    .any(|e| **e == EnforcementLevel::Warn);

    response.advisory = !has_block;
    response.enforcement_applied = if has_block {
        "block".to_string()
    } else if has_warn {
        "warn".to_string()
    } else {
        "off".to_string()
    };

    // --- Branch rules ---
    if enforcement.branches != EnforcementLevel::Off {
        response
            .evaluated_rules
            .push("branch_name_valid".to_string());
        if !branch_matches_policy(config, branch) {
            let v = RuleViolation {
                rule: "branch_name_valid".to_string(),
                category: "branches".to_string(),
                enforcement: format!("{:?}", enforcement.branches).to_lowercase(),
                message: format!("Branch '{}' does not match configured patterns", branch),
            };
            if enforcement.branches == EnforcementLevel::Block {
                response.allowed = false;
                response.reasons.push(v.message.clone());
            } else {
                response.warnings.push(v.message.clone());
            }
            response.violations.push(v);
        }

        response
            .evaluated_rules
            .push("not_protected_branch".to_string());
        if config.branches.protected.iter().any(|p| p == branch) {
            let v = RuleViolation {
                rule: "not_protected_branch".to_string(),
                category: "branches".to_string(),
                enforcement: format!("{:?}", enforcement.branches).to_lowercase(),
                message: format!("Branch '{}' is protected; direct push not allowed", branch),
            };
            if enforcement.branches == EnforcementLevel::Block {
                response.allowed = false;
                response.reasons.push(v.message.clone());
            } else {
                response.warnings.push(v.message.clone());
            }
            response.violations.push(v);
        }

        if config.rules.block_force_push {
            response.evaluated_rules.push("no_force_push".to_string());
        }
    }

    // --- Commit rules ---
    if enforcement.commits != EnforcementLevel::Off {
        if config.rules.require_conventional_commits {
            response
                .evaluated_rules
                .push("conventional_commit".to_string());
        }
        if config.rules.require_signed_commits {
            response.evaluated_rules.push("signed_commit".to_string());
        }
        if let Some(max) = config.rules.max_files_per_commit {
            response
                .evaluated_rules
                .push(format!("max_files_per_commit_{}", max));
        }
        if !config.rules.forbidden_patterns.is_empty() {
            response
                .evaluated_rules
                .push("forbidden_patterns".to_string());
        }
    }

    // --- Pull request rules ---
    if enforcement.pull_requests != EnforcementLevel::Off {
        if config.rules.require_pull_request {
            response
                .evaluated_rules
                .push("require_pull_request".to_string());
        }
        if config.rules.min_approvals > 0 {
            response
                .evaluated_rules
                .push(format!("min_approvals_{}", config.rules.min_approvals));
        }
    }

    // --- Traceability rules ---
    if enforcement.traceability != EnforcementLevel::Off && config.rules.require_linked_ticket {
        response
            .evaluated_rules
            .push("require_linked_ticket".to_string());
    }

    // --- Quality gate rules (Sonar) ---
    if enforcement.quality_gates != EnforcementLevel::Off {
        response
            .evaluated_rules
            .push("quality_gate_green".to_string());
        if let Some(exception) = active_quality_gate_exception {
            response
                .evaluated_rules
                .push("quality_gate_exception_active".to_string());
            response.warnings.push(format!(
                "Quality gate exception active until {} (reason: {}; approved_by: {}).",
                exception.expires_at,
                exception.reason,
                exception
                    .approved_by
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("unknown")
            ));
        } else if let Some(exception) = config.quality_gate_exception.as_ref() {
            if exception.enabled && exception.expires_at <= now_ms {
                response
                    .evaluated_rules
                    .push("quality_gate_exception_expired".to_string());
                response.warnings.push(format!(
                    "Quality gate exception expired at {}; strict policy applies.",
                    exception.expires_at
                ));
            }
        }

        if let Some(commit_sha) = commit_sha {
            match state
                .db
                .get_latest_sonar_run_for_commit(&repo_name, commit_sha)
                .await
            {
                Ok(Some(run)) => {
                    let pipeline_status = run.status.trim().to_ascii_lowercase();
                    if pipeline_status != "success" {
                        let message = format!(
                            "Sonar quality gate not green for commit {} (job '{}', status '{}')",
                            commit_sha, run.job_name, run.status
                        );
                        let v = RuleViolation {
                            rule: "quality_gate_green".to_string(),
                            category: "quality_gates".to_string(),
                            enforcement: if active_quality_gate_exception.is_some() {
                                "override".to_string()
                            } else {
                                format!("{:?}", enforcement.quality_gates).to_lowercase()
                            },
                            message: message.clone(),
                        };
                        if active_quality_gate_exception.is_some() {
                            response.warnings.push(format!(
                                "{} (allowed by active quality gate exception)",
                                message
                            ));
                        } else if enforcement.quality_gates == EnforcementLevel::Block {
                            quality_gate_violation_context = Some((
                                commit_sha.to_string(),
                                run.job_name.clone(),
                                run.status.clone(),
                            ));
                            response.allowed = false;
                            response.reasons.push(message);
                        } else {
                            quality_gate_violation_context = Some((
                                commit_sha.to_string(),
                                run.job_name.clone(),
                                run.status.clone(),
                            ));
                            response.warnings.push(message);
                        }
                        response.violations.push(v);
                    }
                }
                Ok(None) => {
                    response.warnings.push(format!(
                        "No Sonar quality gate evidence found for commit {}; quality check skipped",
                        commit_sha
                    ));
                }
                Err(_) => {
                    response.warnings.push(
                        "Could not load Sonar quality gate evidence; quality check skipped"
                            .to_string(),
                    );
                }
            }
        }
    }

    if commit_sha.is_none() {
        response
            .warnings
            .push("Commit SHA not provided; commit-specific checks skipped".to_string());
    }

    if let Some(decision) = evaluate_opa_policy_check(
        &state.http_client,
        OpaPolicyCheckContext {
            config,
            source: &policy.source,
            repo: &repo,
            repo_name: &repo_name,
            branch,
            commit_sha,
            user_login: payload.user_login.as_deref(),
            auth_user: &auth_user,
            native_response: &response,
        },
    )
    .await
    {
        merge_opa_policy_decision(config, &mut response, decision);
    }

    if let Some((failed_commit_sha, job_name, gate_status)) = quality_gate_violation_context {
        let actor = payload
            .user_login
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(auth_user.client_id.as_str())
            .to_string();
        let enforcement_level = format!("{:?}", enforcement.quality_gates).to_lowercase();

        match state
            .db
            .insert_quality_gate_policy_violation_signal(
                &crate::db::QualityGatePolicyViolationSignalInput {
                    org_id: repo.org_id.as_deref(),
                    repo_id: Some(repo.id.as_str()),
                    actor_login: &actor,
                    branch: Some(branch),
                    commit_sha: &failed_commit_sha,
                    repo_full_name: &repo_name,
                    job_name: &job_name,
                    gate_status: &gate_status,
                    enforcement: &enforcement_level,
                },
            )
            .await
        {
            Ok(inserted) => {
                metrics::counter!("gitgov_quality_gate_policy_failures_total").increment(1);
                if inserted {
                    metrics::counter!("gitgov_quality_gate_signals_created_total").increment(1);
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    repo = %repo_name,
                    commit = %failed_commit_sha,
                    "Failed to persist quality gate policy signal"
                );
            }
        }

        if let Some(webhook_url) = state.alert_webhook_url.clone() {
            let text = notifications::format_quality_gate_policy_alert(
                &actor,
                &repo_name,
                branch,
                &failed_commit_sha,
                &job_name,
                &gate_status,
                &enforcement_level,
            );
            let client = state.http_client.clone();
            tokio::spawn(async move {
                notifications::send_alert(&client, &webhook_url, text).await;
            });
        }
    }

    let status = policy_check_response_status(state.as_ref(), &repo_name, branch, &response);
    if status == StatusCode::CONFLICT {
        metrics::counter!("gitgov_policy_checks_transport_blocked_total").increment(1);
    }
    (status, Json(response))
}
