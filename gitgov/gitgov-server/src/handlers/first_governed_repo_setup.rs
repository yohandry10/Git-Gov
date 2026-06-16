// ============================================================================
// FIRST GOVERNED REPO SETUP
// ============================================================================

const FIRST_GOVERNED_REPO_BASELINE_MAX_BYTES: usize = 24 * 1024;
const FIRST_GOVERNED_REPO_STATUSES: &[&str] = &["draft", "ready", "blocked", "completed"];
const FIRST_GOVERNED_REPO_GOALS: &[&str] = &[
    "govern_release",
    "generate_audit_evidence",
    "standardize_workflows",
    "assess_governance_gaps",
];
const FIRST_GOVERNED_REPO_POLICY_PRESETS: &[&str] = &["audit-only", "moderate", "strict"];
const FIRST_GOVERNED_REPO_PROVIDER_IDS: &[&str] =
    &["github", "jira", "jenkins", "sonarqube", "render", "vercel"];
const FIRST_GOVERNED_REPO_MODULE_IDS: &[&str] = &[
    "traceability",
    "github-evidence",
    "release-readiness",
    "quality-gates",
    "evidence-packets",
    "formal-approval",
];
const FIRST_GOVERNED_REPO_SECRET_MARKERS: &[&str] = &[
    "authorization:",
    "bearer ",
    "api_token",
    "api_token=",
    "gitgov_api_key",
    "gitgov_api_key=",
    "jira_api_token",
    "jira_api_token=",
    "sonar_token",
    "sonar_token=",
    "jenkins_api_token",
    "jenkins_api_token=",
    "render_api_key",
    "render_api_key=",
    "vercel_token",
    "vercel_token=",
    "vck_",
    "gho_",
    "github_pat_",
    "atatt",
];

fn first_governed_repo_scope_error_message(error: OrgScopeError) -> &'static str {
    match error {
        OrgScopeError::BadRequest => "org_name is required for global admin keys",
        OrgScopeError::NotFound => "org_name was not found",
        OrgScopeError::Forbidden => "API key is not authorized for that org_name",
        OrgScopeError::Internal => "Internal database error",
    }
}

fn first_setup_trimmed_or_default(value: &str, default: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn first_setup_valid_repo_part(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= 100
        && !part.starts_with('.')
        && !part.ends_with('.')
        && part
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn first_setup_repo_format_valid(repository_full_name: &str) -> bool {
    let trimmed = repository_full_name.trim();
    if trimmed.len() > 200 || trimmed.contains(char::is_whitespace) {
        return false;
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    parts.len() == 2
        && first_setup_valid_repo_part(parts[0])
        && first_setup_valid_repo_part(parts[1])
}

fn first_setup_normalize_choice(
    raw: &str,
    default: &str,
    allowed: &[&str],
    field: &str,
    errors: &mut Vec<String>,
) -> String {
    let value = first_setup_trimmed_or_default(raw, default);
    if !allowed.contains(&value.as_str()) {
        errors.push(format!("{field} must be one of {}.", allowed.join(", ")));
    }
    value
}

fn first_setup_normalize_status(
    raw: Option<&str>,
    default: &str,
    errors: &mut Vec<String>,
) -> String {
    let value = raw
        .map(|item| first_setup_trimmed_or_default(item, default))
        .unwrap_or_else(|| default.to_string());
    if !FIRST_GOVERNED_REPO_STATUSES.contains(&value.as_str()) {
        errors.push(format!(
            "status must be one of {}.",
            FIRST_GOVERNED_REPO_STATUSES.join(", ")
        ));
    }
    value
}

fn first_setup_normalize_string_array(
    values: &[String],
    field: &str,
    allowed: &[&str],
    default_values: &[&str],
    require_github: bool,
    errors: &mut Vec<String>,
) -> Vec<String> {
    let source: Vec<String> = if values.is_empty() {
        default_values.iter().map(|value| value.to_string()).collect()
    } else {
        values.to_vec()
    };
    if source.len() > allowed.len() {
        errors.push(format!("{field} has too many values."));
    }

    let allowed_set: HashSet<&str> = allowed.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for value in source {
        let trimmed = value.trim();
        if !allowed_set.contains(trimmed) {
            errors.push(format!("{field} contains unsupported value '{trimmed}'."));
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    if normalized.is_empty() {
        errors.push(format!("Select at least one {field} value."));
    }
    if require_github && !normalized.iter().any(|value| value == "github") {
        errors.push("selected_providers must include github.".to_string());
    }

    normalized
}

fn first_setup_contains_secret(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(text) => {
            let lowered = text.to_ascii_lowercase();
            FIRST_GOVERNED_REPO_SECRET_MARKERS
                .iter()
                .any(|marker| lowered.contains(marker))
        }
        serde_json::Value::Array(items) => items.iter().any(first_setup_contains_secret),
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            let lowered_key = key.to_ascii_lowercase();
            FIRST_GOVERNED_REPO_SECRET_MARKERS
                .iter()
                .any(|marker| lowered_key.contains(marker))
                || first_setup_contains_secret(value)
        }),
        _ => false,
    }
}

fn first_setup_bool_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> bool {
    object
        .get(field)
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn first_setup_prepare_payload(
    mut payload: UpsertFirstGovernedRepoSetupRequest,
) -> Result<UpsertFirstGovernedRepoSetupRequest, Vec<String>> {
    let mut errors = Vec::new();

    payload.repository_full_name = payload.repository_full_name.trim().to_string();
    if !first_setup_repo_format_valid(&payload.repository_full_name) {
        errors.push("repository_full_name must use owner/repo format.".to_string());
    }

    payload.default_branch = first_setup_trimmed_or_default(&payload.default_branch, "main");
    if payload.default_branch.len() > 120 || payload.default_branch.contains(char::is_whitespace) {
        errors.push("default_branch must be a branch name without whitespace.".to_string());
    }

    payload.goal = first_setup_normalize_choice(
        &payload.goal,
        "govern_release",
        FIRST_GOVERNED_REPO_GOALS,
        "goal",
        &mut errors,
    );
    payload.policy_preset = first_setup_normalize_choice(
        &payload.policy_preset,
        "moderate",
        FIRST_GOVERNED_REPO_POLICY_PRESETS,
        "policy_preset",
        &mut errors,
    );
    payload.selected_providers = first_setup_normalize_string_array(
        &payload.selected_providers,
        "selected_providers",
        FIRST_GOVERNED_REPO_PROVIDER_IDS,
        &["github"],
        true,
        &mut errors,
    );
    payload.selected_modules = first_setup_normalize_string_array(
        &payload.selected_modules,
        "selected_modules",
        FIRST_GOVERNED_REPO_MODULE_IDS,
        &["traceability", "release-readiness", "evidence-packets"],
        false,
        &mut errors,
    );

    let size = serde_json::to_vec(&payload.baseline)
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX);
    if size > FIRST_GOVERNED_REPO_BASELINE_MAX_BYTES {
        errors.push("baseline is too large.".to_string());
    }
    if first_setup_contains_secret(&payload.baseline) {
        errors.push("baseline must not contain secret-looking values.".to_string());
    }

    let mut baseline = match payload.baseline {
        serde_json::Value::Null => serde_json::Map::new(),
        serde_json::Value::Object(map) => map,
        _ => {
            errors.push("baseline must be an object.".to_string());
            serde_json::Map::new()
        }
    };

    let preview_acknowledged =
        first_setup_bool_field(&baseline, "policy_workflow_preview_acknowledged");
    let repo_ready = first_setup_repo_format_valid(&payload.repository_full_name);
    let github_ready = payload.selected_providers.iter().any(|value| value == "github");
    let gate_readiness = if repo_ready && github_ready && preview_acknowledged {
        "baseline_ready"
    } else if repo_ready && github_ready {
        "needs_preview"
    } else {
        "needs_repo"
    };

    let mut action_center_gaps = Vec::new();
    if !repo_ready {
        action_center_gaps.push("repository_full_name");
    }
    if !preview_acknowledged {
        action_center_gaps.push("policy_workflow_preview");
    }
    if !payload
        .selected_modules
        .iter()
        .any(|value| value == "quality-gates")
    {
        action_center_gaps.push("quality_gate_evidence");
    }
    if !payload
        .selected_modules
        .iter()
        .any(|value| value == "formal-approval")
    {
        action_center_gaps.push("formal_approval_policy");
    }

    let default_status = if gate_readiness == "baseline_ready" {
        "ready"
    } else {
        "draft"
    };
    let normalized_status =
        first_setup_normalize_status(payload.status.as_deref(), default_status, &mut errors);
    if normalized_status == "completed" && gate_readiness != "baseline_ready" {
        errors.push("completed status requires baseline_ready gate_readiness.".to_string());
    }
    payload.status = Some(normalized_status);

    baseline.insert("version".to_string(), json!(1));
    baseline.insert("gate_readiness".to_string(), json!(gate_readiness));
    baseline.insert(
        "setup_summary".to_string(),
        json!({
            "repository_full_name": payload.repository_full_name.clone(),
            "default_branch": payload.default_branch.clone(),
            "goal": payload.goal.clone(),
            "policy_preset": payload.policy_preset.clone(),
            "provider_count": payload.selected_providers.len(),
            "module_count": payload.selected_modules.len(),
            "github_selected": github_ready,
            "policy_workflow_preview_acknowledged": preview_acknowledged
        }),
    );
    baseline.insert(
        "action_center_gaps".to_string(),
        serde_json::Value::Array(
            action_center_gaps
                .iter()
                .map(|value| json!(value))
                .collect::<Vec<_>>(),
        ),
    );
    baseline.insert(
        "first_result".to_string(),
        json!({
            "status": if gate_readiness == "baseline_ready" { "ready_for_advisory_gate" } else { "needs_setup" },
            "deployment_gate_mode": "advisory",
            "cta": "simulate_deployment_gate",
            "evidence_contract": {
                "repo": payload.repository_full_name.clone(),
                "branch": payload.default_branch.clone(),
                "providers": payload.selected_providers.clone(),
                "modules": payload.selected_modules.clone()
            }
        }),
    );
    payload.baseline = serde_json::Value::Object(baseline);

    if errors.is_empty() {
        Ok(payload)
    } else {
        Err(errors)
    }
}


pub async fn get_first_governed_repo_setup(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FirstGovernedRepoSetupQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": first_governed_repo_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(Some(setup)) => (
            StatusCode::OK,
            Json(FirstGovernedRepoSetupResponse {
                found: true,
                setup: Some(setup),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(FirstGovernedRepoSetupResponse {
                found: false,
                setup: None,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load first governed repo setup");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn upsert_first_governed_repo_setup(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpsertFirstGovernedRepoSetupRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let payload = match first_setup_prepare_payload(payload) {
        Ok(payload) => payload,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid first governed repo setup", "details": errors })),
            )
                .into_response();
        }
    };

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": first_governed_repo_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &payload, &auth_user.client_id)
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "upsert_first_governed_repo_setup".to_string(),
                target_type: Some("enterprise_first_governed_repo_setup".to_string()),
                target_id: Some(org_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "run_id": record.run_id.clone(),
                    "status": record.status.clone(),
                    "goal": record.goal.clone(),
                    "repository_full_name": record.repository_full_name.clone(),
                    "default_branch": record.default_branch.clone(),
                    "policy_preset": record.policy_preset.clone(),
                    "provider_count": record.selected_providers.len(),
                    "module_count": record.selected_modules.len(),
                    "gate_readiness": record.baseline.get("gate_readiness").and_then(|value| value.as_str())
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (first governed repo setup)");
            }

            (StatusCode::OK, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to save first governed repo setup");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}


#[cfg(test)]
mod first_governed_repo_setup_tests {
    use super::*;

    #[test]
    fn first_setup_prepare_payload_normalizes_defaults_and_gaps() {
        let payload = UpsertFirstGovernedRepoSetupRequest {
            repository_full_name: " example/repo ".to_string(),
            baseline: json!({ "policy_workflow_preview_acknowledged": true }),
            ..Default::default()
        };

        let prepared = first_setup_prepare_payload(payload).expect("valid payload");

        assert_eq!(prepared.goal, "govern_release");
        assert_eq!(prepared.default_branch, "main");
        assert_eq!(prepared.selected_providers, vec!["github"]);
        assert_eq!(
            prepared.selected_modules,
            vec!["traceability", "release-readiness", "evidence-packets"]
        );
        assert_eq!(
            prepared.baseline.get("gate_readiness").and_then(|value| value.as_str()),
            Some("baseline_ready")
        );
        assert_eq!(prepared.status.as_deref(), Some("ready"));
    }

    #[test]
    fn first_setup_prepare_payload_rejects_secret_like_baseline() {
        let payload = UpsertFirstGovernedRepoSetupRequest {
            repository_full_name: "example/repo".to_string(),
            baseline: json!({
                "notes": {
                    "jira_api_token": "redacted",
                    "token": "Bearer abc123"
                }
            }),
            ..Default::default()
        };

        let errors = first_setup_prepare_payload(payload).expect_err("secret-looking baseline");

        assert!(errors
            .iter()
            .any(|error| error.contains("secret-looking")));
    }
}
