// ============================================================================
// ENTERPRISE ADOPTION PROFILES
// ============================================================================

const ENTERPRISE_ADOPTION_PROFILE_MAX_BYTES: usize = 32 * 1024;
const ENTERPRISE_ONBOARDING_CHECKLIST_TRACKING_MAX_BYTES: usize = 16 * 1024;
const ADOPTION_POLICY_PRESETS: &[&str] = &["audit-only", "moderate", "strict"];
const ADOPTION_PROVIDER_IDS: &[&str] =
    &["github", "jira", "jenkins", "sonarqube", "render", "vercel"];
const ADOPTION_MODULE_IDS: &[&str] = &[
    "traceability",
    "github-evidence",
    "release-readiness",
    "quality-gates",
    "evidence-packets",
    "vulnerability-review",
    "artifact-monitoring",
    "security-review",
    "trend-enforcement",
    "formal-approval",
];
const ADOPTION_RELEASE_GOVERNANCE_MODES: &[&str] = &[
    "record-only",
    "advisory",
    "approval-required",
    "quorum-required",
];
const ADOPTION_RELEASE_GOVERNANCE_ENFORCEMENT: &[&str] =
    &["disabled", "advisory", "blocking"];
const ONBOARDING_STAGE_IDS: &[&str] = &[
    "profile",
    "providers",
    "workflow-pack",
    "remote-workflows",
    "actions-config",
    "release-governance",
];
const ONBOARDING_TRACKING_STATUSES: &[&str] = &["open", "in-progress", "waiting", "done"];
const TRACKING_FORBIDDEN_SECRET_MARKERS: &[&str] = &[
    "authorization:",
    "bearer ",
    "api_token=",
    "gitgov_api_key=",
    "jira_api_token=",
    "sonar_token=",
    "jenkins_api_token=",
    "vck_",
    "gho_",
    "atatt",
];

fn adoption_profile_string_field<'a>(
    profile: &'a serde_json::Value,
    field: &str,
) -> Option<&'a str> {
    profile
        .get(field)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn adoption_profile_string_array(
    profile: &serde_json::Value,
    field: &str,
    allowed: &[&str],
    errors: &mut Vec<String>,
) -> Vec<String> {
    let Some(value) = profile.get(field) else {
        errors.push(format!("{field} is required."));
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(format!("{field} must be an array."));
        return Vec::new();
    };

    if items.is_empty() {
        errors.push(format!("Select at least one {field} value."));
    }
    if items.len() > allowed.len() {
        errors.push(format!("{field} has too many values."));
    }

    let allowed_set: HashSet<&str> = allowed.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let Some(raw) = item.as_str() else {
            errors.push(format!("{field} values must be strings."));
            continue;
        };
        let value = raw.trim();
        if !allowed_set.contains(value) {
            errors.push(format!("{field} contains unsupported value '{value}'."));
            continue;
        }
        if seen.insert(value.to_string()) {
            result.push(value.to_string());
        }
    }

    result
}

fn onboarding_tracking_text_field(
    item: &serde_json::Value,
    field: &str,
    max_chars: usize,
    errors: &mut Vec<String>,
) -> Option<String> {
    match item.get(field) {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            let Some(text) = value.as_str() else {
                errors.push(format!("{field} must be a string."));
                return None;
            };
            let trimmed = text.trim();
            if trimmed.chars().count() > max_chars {
                errors.push(format!("{field} must be at most {max_chars} characters."));
            }
            let lowered = trimmed.to_ascii_lowercase();
            if TRACKING_FORBIDDEN_SECRET_MARKERS
                .iter()
                .any(|marker| lowered.contains(marker))
            {
                errors.push(format!("{field} must not contain secret-looking values."));
            }
            Some(trimmed.to_string())
        }
    }
}

fn onboarding_tracking_target_date_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn validate_enterprise_onboarding_checklist_tracking(
    tracking: &serde_json::Value,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let size = serde_json::to_vec(tracking).map(|bytes| bytes.len()).unwrap_or(usize::MAX);
    if size > ENTERPRISE_ONBOARDING_CHECKLIST_TRACKING_MAX_BYTES {
        errors.push("Onboarding checklist tracking is too large.".to_string());
    }

    let Some(object) = tracking.as_object() else {
        errors.push("tracking must be an object.".to_string());
        return Err(errors);
    };

    if let Some(version) = object.get("version") {
        if version.as_i64() != Some(1) {
            errors.push("version must be 1.".to_string());
        }
    }

    let Some(items_value) = object.get("items") else {
        errors.push("items is required.".to_string());
        return Err(errors);
    };
    let Some(items) = items_value.as_array() else {
        errors.push("items must be an array.".to_string());
        return Err(errors);
    };
    if items.len() > ONBOARDING_STAGE_IDS.len() {
        errors.push("items has too many entries.".to_string());
    }

    let allowed_stages: HashSet<&str> = ONBOARDING_STAGE_IDS.iter().copied().collect();
    let allowed_statuses: HashSet<&str> = ONBOARDING_TRACKING_STATUSES.iter().copied().collect();
    let mut seen_stages = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let Some(item_object) = item.as_object() else {
            errors.push(format!("items[{index}] must be an object."));
            continue;
        };

        let stage_id = item_object
            .get("stage_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .unwrap_or_default();
        if !allowed_stages.contains(stage_id) {
            errors.push(format!("items[{index}].stage_id is unsupported."));
        } else if !seen_stages.insert(stage_id.to_string()) {
            errors.push(format!("items contains duplicate stage_id '{stage_id}'."));
        }

        let status = item_object
            .get("status")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .unwrap_or_default();
        if !allowed_statuses.contains(status) {
            errors.push(format!("items[{index}].status is unsupported."));
        }

        onboarding_tracking_text_field(item, "owner", 80, &mut errors);
        onboarding_tracking_text_field(item, "note", 1000, &mut errors);
        onboarding_tracking_text_field(item, "external_ref", 120, &mut errors);

        if let Some(target_date) = onboarding_tracking_text_field(item, "target_date", 10, &mut errors) {
            if !target_date.is_empty() && !onboarding_tracking_target_date_valid(&target_date) {
                errors.push("target_date must use YYYY-MM-DD format.".to_string());
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_release_governance_policy_object(
    policy: &serde_json::Map<String, serde_json::Value>,
    modules: &[String],
    errors: &mut Vec<String>,
    prefix: &str,
    require_environment: bool,
) -> Option<String> {
    let mode = match policy.get("mode") {
        Some(value) => match value.as_str().map(str::trim).filter(|item| !item.is_empty()) {
            Some(mode) if ADOPTION_RELEASE_GOVERNANCE_MODES.contains(&mode) => mode,
            Some(_) | None => {
                errors.push(format!("{prefix}.mode must be one of record-only, advisory, approval-required, quorum-required."));
                return None;
            }
        },
        None => "record-only",
    };

    let mut normalized_environment = None;
    if let Some(value) = policy.get("environment") {
        match value.as_str().map(str::trim) {
            Some(environment) if !environment.is_empty() && environment.len() <= 64 => {
                normalized_environment = Some(environment.to_ascii_lowercase());
            }
            Some(_) => errors.push(format!("{prefix}.environment is required and must be 64 characters or fewer.")),
            None => errors.push(format!("{prefix}.environment must be a string.")),
        }
    } else if require_environment {
        errors.push(format!("{prefix}.environment is required and must be 64 characters or fewer."));
    }

    let expected_enforcement = match mode {
        "advisory" => "advisory",
        "approval-required" | "quorum-required" => "blocking",
        _ => "disabled",
    };
    let enforcement = match policy.get("enforcement") {
        Some(value) => match value.as_str().map(str::trim).filter(|item| !item.is_empty()) {
            Some(enforcement) if ADOPTION_RELEASE_GOVERNANCE_ENFORCEMENT.contains(&enforcement) => enforcement,
            Some(_) | None => {
                errors.push(format!("{prefix}.enforcement must be one of disabled, advisory, blocking."));
                expected_enforcement
            }
        },
        None => expected_enforcement,
    };

    let expected_approval_required = matches!(mode, "approval-required" | "quorum-required");
    let approval_required = match policy.get("approval_required") {
        Some(value) => match value.as_bool() {
            Some(value) => value,
            None => {
                errors.push(format!("{prefix}.approval_required must be boolean."));
                expected_approval_required
            }
        },
        None => expected_approval_required,
    };

    let mut quorum_enabled = false;
    let mut valid_quorum_rule_count = 0usize;
    if let Some(quorum_value) = policy.get("quorum") {
        let Some(quorum) = quorum_value.as_object() else {
            errors.push(format!("{prefix}.quorum must be an object."));
            return normalized_environment;
        };

        if let Some(enabled_value) = quorum.get("enabled") {
            match enabled_value.as_bool() {
                Some(value) => quorum_enabled = value,
                None => errors.push(format!("{prefix}.quorum.enabled must be boolean.")),
            }
        }

        if let Some(rules_value) = quorum.get("rules") {
            let Some(rules) = rules_value.as_array() else {
                errors.push(format!("{prefix}.quorum.rules must be an array."));
                return normalized_environment;
            };
            if rules.len() > 10 {
                errors.push(format!("{prefix}.quorum.rules has too many values."));
            }
            for rule in rules {
                let Some(rule_object) = rule.as_object() else {
                    errors.push(format!("{prefix}.quorum.rules values must be objects."));
                    continue;
                };

                let valid_role = rule_object
                    .get("role")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|role| !role.is_empty() && role.len() <= 64)
                    .is_some();
                if !valid_role {
                    errors.push(format!("{prefix}.quorum.rules.role is required and must be 64 characters or fewer."));
                }

                let valid_required = rule_object
                    .get("required")
                    .and_then(|value| value.as_i64())
                    .map(|required| (1..=20).contains(&required))
                    .unwrap_or(false);
                if !valid_required {
                    errors.push(format!("{prefix}.quorum.rules.required must be an integer from 1 to 20."));
                }

                if valid_role && valid_required {
                    valid_quorum_rule_count += 1;
                }
            }
        }
    }

    if mode != "record-only" && !modules.iter().any(|module| module == "formal-approval") {
        if prefix == "release_governance" {
            errors.push("release_governance mode requires the formal-approval module unless it is record-only.".to_string());
        } else {
            errors.push(format!("{prefix} mode requires the formal-approval module unless it is record-only."));
        }
    }

    match mode {
        "record-only" => {
            if approval_required {
                errors.push(format!("{prefix} record-only release governance cannot require approval."));
            }
            if enforcement != "disabled" {
                if prefix == "release_governance" {
                    errors.push("record-only release governance must use disabled enforcement.".to_string());
                } else {
                    errors.push(format!("{prefix} record-only release governance must use disabled enforcement."));
                }
            }
            if quorum_enabled || valid_quorum_rule_count > 0 {
                errors.push(format!("{prefix} record-only release governance cannot enable quorum."));
            }
        }
        "advisory" => {
            if approval_required {
                errors.push(format!("{prefix} advisory release governance cannot require approval."));
            }
            if enforcement != "advisory" {
                errors.push(format!("{prefix} advisory release governance must use advisory enforcement."));
            }
            if quorum_enabled || valid_quorum_rule_count > 0 {
                errors.push(format!("{prefix} advisory release governance cannot enable quorum."));
            }
        }
        "approval-required" => {
            if !approval_required {
                errors.push(format!("{prefix} approval-required release governance must require approval."));
            }
            if enforcement != "blocking" {
                errors.push(format!("{prefix} approval-required release governance must use blocking enforcement."));
            }
            if quorum_enabled || valid_quorum_rule_count > 0 {
                errors.push(format!("{prefix} approval-required release governance cannot enable quorum; use quorum-required."));
            }
        }
        "quorum-required" => {
            if !approval_required {
                errors.push(format!("{prefix} quorum-required release governance must require approval."));
            }
            if enforcement != "blocking" {
                errors.push(format!("{prefix} quorum-required release governance must use blocking enforcement."));
            }
            if !quorum_enabled || valid_quorum_rule_count == 0 {
                errors.push(format!("{prefix} quorum-required release governance needs at least one quorum rule."));
            }
        }
        _ => {}
    }

    normalized_environment
}

fn validate_release_governance_policy(
    profile: &serde_json::Value,
    modules: &[String],
    errors: &mut Vec<String>,
) {
    let Some(value) = profile.get("release_governance") else {
        return;
    };
    if value.is_null() {
        return;
    }

    let Some(policy) = value.as_object() else {
        errors.push("release_governance must be an object.".to_string());
        return;
    };

    validate_release_governance_policy_object(
        policy,
        modules,
        errors,
        "release_governance",
        false,
    );

    let Some(overrides_value) = policy.get("environment_overrides") else {
        return;
    };
    let Some(overrides) = overrides_value.as_array() else {
        errors.push("release_governance.environment_overrides must be an array.".to_string());
        return;
    };
    if overrides.len() > 10 {
        errors.push("release_governance.environment_overrides has too many values.".to_string());
    }

    let mut seen_environments = HashSet::new();
    for (index, override_value) in overrides.iter().enumerate() {
        let prefix = format!("release_governance.environment_overrides[{index}]");
        let Some(override_policy) = override_value.as_object() else {
            errors.push(format!("{prefix} must be an object."));
            continue;
        };
        if let Some(environment) = validate_release_governance_policy_object(
            override_policy,
            modules,
            errors,
            &prefix,
            true,
        ) {
            if !seen_environments.insert(environment.clone()) {
                errors.push(format!(
                    "release_governance.environment_overrides contains duplicate environment '{environment}'."
                ));
            }
        }
    }
}

fn validate_enterprise_adoption_profile(profile: &serde_json::Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if !profile.is_object() {
        return Err(vec!["profile must be a JSON object.".to_string()]);
    }

    let serialized_len = serde_json::to_vec(profile).map(|bytes| bytes.len()).unwrap_or(0);
    if serialized_len > ENTERPRISE_ADOPTION_PROFILE_MAX_BYTES {
        errors.push("profile is too large.".to_string());
    }

    let customer_name = adoption_profile_string_field(profile, "customer_name");
    if customer_name.is_none() {
        errors.push("Customer name is required.".to_string());
    } else if customer_name.unwrap_or_default().len() > 120 {
        errors.push("Customer name is too long.".to_string());
    }

    let repository_full_name = adoption_profile_string_field(profile, "repository_full_name");
    match repository_full_name {
        None => errors.push("Repository is required.".to_string()),
        Some(repo) => {
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() != 2 || parts.iter().any(|part| part.is_empty() || part.contains(char::is_whitespace)) {
                errors.push("Repository must look like owner/repo.".to_string());
            }
            if repo.len() > 200 {
                errors.push("Repository name is too long.".to_string());
            }
        }
    }

    let default_branch = adoption_profile_string_field(profile, "default_branch");
    if default_branch.is_none() {
        errors.push("Default branch is required.".to_string());
    } else if default_branch.unwrap_or_default().len() > 200 {
        errors.push("Default branch is too long.".to_string());
    }

    let policy_preset = adoption_profile_string_field(profile, "policy_preset");
    match policy_preset {
        Some(value) if ADOPTION_POLICY_PRESETS.contains(&value) => {}
        Some(_) => errors.push("policy_preset must be one of audit-only, moderate, strict.".to_string()),
        None => errors.push("policy_preset is required.".to_string()),
    }

    let providers =
        adoption_profile_string_array(profile, "providers", ADOPTION_PROVIDER_IDS, &mut errors);
    let modules = adoption_profile_string_array(profile, "modules", ADOPTION_MODULE_IDS, &mut errors);
    validate_release_governance_policy(profile, &modules, &mut errors);

    let jira_key = adoption_profile_string_field(profile, "jira_project_key").unwrap_or_default();
    if modules.iter().any(|module| module == "traceability") && jira_key.is_empty() {
        errors.push("Jira project key is required when traceability is selected.".to_string());
    }
    if !jira_key.is_empty() {
        let valid_jira_key = jira_key.len() >= 2
            && jira_key.len() <= 16
            && jira_key
                .chars()
                .enumerate()
                .all(|(index, ch)| {
                    ch.is_ascii_uppercase() || (index > 0 && ch.is_ascii_digit())
                });
        if !valid_jira_key {
            errors.push("Jira project key should be uppercase letters/numbers, like KAN.".to_string());
        }
    }

    if providers.is_empty() {
        errors.push("Select at least one provider.".to_string());
    }
    if modules.is_empty() {
        errors.push("Select at least one module.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn adoption_scope_error_message(error: OrgScopeError) -> &'static str {
    match error {
        OrgScopeError::BadRequest => "org_name is required for global admin keys",
        OrgScopeError::NotFound => "Organization not found",
        OrgScopeError::Forbidden => "Requested org is outside API key scope",
        OrgScopeError::Internal => "Internal database error",
    }
}

pub async fn get_enterprise_adoption_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<EnterpriseAdoptionProfileQuery>,
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
                Json(json!({ "error": adoption_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state.db.get_enterprise_adoption_profile(&org_id).await {
        Ok(Some(profile)) => (
            StatusCode::OK,
            Json(EnterpriseAdoptionProfileResponse {
                found: true,
                profile: Some(profile),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(EnterpriseAdoptionProfileResponse {
                found: false,
                profile: None,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load enterprise adoption profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn upsert_enterprise_adoption_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpsertEnterpriseAdoptionProfileRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = validate_enterprise_adoption_profile(&payload.profile) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid enterprise adoption profile", "details": errors })),
        )
            .into_response();
    }

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
                Json(json!({ "error": adoption_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .upsert_enterprise_adoption_profile(&org_id, &payload.profile, &auth_user.client_id)
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "upsert_enterprise_adoption_profile".to_string(),
                target_type: Some("enterprise_adoption_profile".to_string()),
                target_id: Some(org_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "repository_full_name": adoption_profile_string_field(&record.profile, "repository_full_name"),
                    "policy_preset": adoption_profile_string_field(&record.profile, "policy_preset"),
                    "provider_count": record.profile.get("providers").and_then(|value| value.as_array()).map(|items| items.len()).unwrap_or(0),
                    "module_count": record.profile.get("modules").and_then(|value| value.as_array()).map(|items| items.len()).unwrap_or(0)
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (enterprise adoption profile)");
            }

            (StatusCode::OK, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to save enterprise adoption profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_enterprise_onboarding_checklist_tracking(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<EnterpriseOnboardingChecklistTrackingQuery>,
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
                Json(json!({ "error": adoption_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .get_enterprise_onboarding_checklist_tracking(&org_id)
        .await
    {
        Ok(Some(tracking)) => (
            StatusCode::OK,
            Json(EnterpriseOnboardingChecklistTrackingResponse {
                found: true,
                tracking: Some(tracking),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(EnterpriseOnboardingChecklistTrackingResponse {
                found: false,
                tracking: None,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load enterprise onboarding checklist tracking");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn upsert_enterprise_onboarding_checklist_tracking(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpsertEnterpriseOnboardingChecklistTrackingRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = validate_enterprise_onboarding_checklist_tracking(&payload.tracking) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid enterprise onboarding checklist tracking", "details": errors })),
        )
            .into_response();
    }

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
                Json(json!({ "error": adoption_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .upsert_enterprise_onboarding_checklist_tracking(
            &org_id,
            &payload.tracking,
            &auth_user.client_id,
        )
        .await
    {
        Ok(record) => {
            let items = record
                .tracking
                .get("items")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "upsert_enterprise_onboarding_checklist_tracking".to_string(),
                target_type: Some("enterprise_onboarding_checklist_tracking".to_string()),
                target_id: Some(org_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "item_count": items.len(),
                    "done_count": items.iter().filter(|item| item.get("status").and_then(|value| value.as_str()) == Some("done")).count(),
                    "waiting_count": items.iter().filter(|item| item.get("status").and_then(|value| value.as_str()) == Some("waiting")).count()
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (enterprise onboarding checklist tracking)");
            }

            (StatusCode::OK, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to save enterprise onboarding checklist tracking");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod adoption_profile_tests {
    use super::*;

    fn valid_profile() -> serde_json::Value {
        json!({
            "customer_name": "ExampleCo",
            "repository_full_name": "example-org/example-repo",
            "default_branch": "main",
            "jira_project_key": "KAN",
            "policy_preset": "moderate",
            "providers": ["github", "jira", "sonarqube"],
            "modules": ["traceability", "release-readiness", "vulnerability-review", "artifact-monitoring"],
            "release_governance": {
                "mode": "record-only",
                "environment": "production",
                "approval_required": false,
                "enforcement": "disabled",
                "quorum": {
                    "enabled": false,
                    "rules": []
                }
            }
        })
    }

    #[test]
    fn enterprise_adoption_profile_validation_accepts_valid_profile() {
        assert!(validate_enterprise_adoption_profile(&valid_profile()).is_ok());
    }

    #[test]
    fn enterprise_adoption_profile_validation_rejects_bad_repo_and_jira_key() {
        let mut profile = valid_profile();
        profile["repository_full_name"] = json!("missing-owner");
        profile["jira_project_key"] = json!("kan");

        let errors = validate_enterprise_adoption_profile(&profile).unwrap_err();

        assert!(errors.contains(&"Repository must look like owner/repo.".to_string()));
        assert!(errors
            .contains(&"Jira project key should be uppercase letters/numbers, like KAN.".to_string()));
    }

    #[test]
    fn enterprise_adoption_profile_validation_rejects_unknown_modules() {
        let mut profile = valid_profile();
        profile["modules"] = json!(["traceability", "unknown"]);

        let errors = validate_enterprise_adoption_profile(&profile).unwrap_err();

        assert!(errors.contains(&"modules contains unsupported value 'unknown'.".to_string()));
    }

    #[test]
    fn enterprise_adoption_profile_validation_accepts_quorum_required_when_opted_in() {
        let mut profile = valid_profile();
        profile["modules"] = json!(["traceability", "release-readiness", "formal-approval"]);
        profile["release_governance"] = json!({
            "mode": "quorum-required",
            "environment": "production",
            "approval_required": true,
            "enforcement": "blocking",
            "quorum": {
                "enabled": true,
                "rules": [
                    { "role": "engineering", "required": 1 },
                    { "role": "security", "required": 1 }
                ]
            }
        });

        assert!(validate_enterprise_adoption_profile(&profile).is_ok());
    }

    #[test]
    fn enterprise_adoption_profile_validation_accepts_environment_overrides_when_opted_in() {
        let mut profile = valid_profile();
        profile["modules"] = json!(["traceability", "release-readiness", "formal-approval"]);
        profile["release_governance"] = json!({
            "mode": "record-only",
            "environment": "staging",
            "approval_required": false,
            "enforcement": "disabled",
            "quorum": {
                "enabled": false,
                "rules": []
            },
            "environment_overrides": [
                {
                    "mode": "approval-required",
                    "environment": "production",
                    "approval_required": true,
                    "enforcement": "blocking",
                    "quorum": {
                        "enabled": false,
                        "rules": []
                    }
                }
            ]
        });

        assert!(validate_enterprise_adoption_profile(&profile).is_ok());
    }

    #[test]
    fn enterprise_adoption_profile_validation_rejects_duplicate_environment_overrides() {
        let mut profile = valid_profile();
        profile["modules"] = json!(["traceability", "release-readiness", "formal-approval"]);
        profile["release_governance"] = json!({
            "mode": "record-only",
            "environment": "staging",
            "approval_required": false,
            "enforcement": "disabled",
            "quorum": {
                "enabled": false,
                "rules": []
            },
            "environment_overrides": [
                {
                    "mode": "advisory",
                    "environment": "Production",
                    "approval_required": false,
                    "enforcement": "advisory",
                    "quorum": {
                        "enabled": false,
                        "rules": []
                    }
                },
                {
                    "mode": "approval-required",
                    "environment": "production",
                    "approval_required": true,
                    "enforcement": "blocking",
                    "quorum": {
                        "enabled": false,
                        "rules": []
                    }
                }
            ]
        });

        let errors = validate_enterprise_adoption_profile(&profile).unwrap_err();

        assert!(errors.contains(
            &"release_governance.environment_overrides contains duplicate environment 'production'."
                .to_string()
        ));
    }

    #[test]
    fn enterprise_adoption_profile_validation_rejects_approval_required_without_formal_approval() {
        let mut profile = valid_profile();
        profile["release_governance"] = json!({
            "mode": "approval-required",
            "environment": "production",
            "approval_required": true,
            "enforcement": "blocking",
            "quorum": {
                "enabled": false,
                "rules": []
            }
        });

        let errors = validate_enterprise_adoption_profile(&profile).unwrap_err();

        assert!(errors.contains(
            &"release_governance mode requires the formal-approval module unless it is record-only."
                .to_string()
        ));
    }

    #[test]
    fn enterprise_adoption_profile_validation_rejects_blocking_record_only_policy() {
        let mut profile = valid_profile();
        profile["release_governance"] = json!({
            "mode": "record-only",
            "environment": "production",
            "approval_required": false,
            "enforcement": "blocking",
            "quorum": {
                "enabled": false,
                "rules": []
            }
        });

        let errors = validate_enterprise_adoption_profile(&profile).unwrap_err();

        assert!(errors
            .contains(&"record-only release governance must use disabled enforcement.".to_string()));
    }

    #[test]
    fn enterprise_onboarding_checklist_tracking_accepts_valid_tracking() {
        let tracking = json!({
            "version": 1,
            "items": [
                {
                    "stage_id": "providers",
                    "status": "in-progress",
                    "owner": "Platform owner",
                    "note": "Waiting for provider evidence",
                    "external_ref": "KAN-60",
                    "target_date": "2026-05-08"
                },
                {
                    "stage_id": "actions-config",
                    "status": "waiting",
                    "owner": "Repository admin",
                    "note": "Variable and secret names still need customer setup"
                }
            ]
        });

        assert!(validate_enterprise_onboarding_checklist_tracking(&tracking).is_ok());
    }

    #[test]
    fn enterprise_onboarding_checklist_tracking_rejects_secret_looking_values() {
        let tracking = json!({
            "version": 1,
            "items": [
                {
                    "stage_id": "providers",
                    "status": "in-progress",
                    "note": "Bearer abc123 should not be stored here"
                }
            ]
        });

        let errors = validate_enterprise_onboarding_checklist_tracking(&tracking).unwrap_err();

        assert!(errors.contains(&"note must not contain secret-looking values.".to_string()));
    }

    #[test]
    fn enterprise_onboarding_checklist_tracking_rejects_duplicate_or_unknown_stage() {
        let tracking = json!({
            "version": 1,
            "items": [
                { "stage_id": "providers", "status": "open" },
                { "stage_id": "providers", "status": "done" },
                { "stage_id": "unknown", "status": "open", "target_date": "20260508" }
            ]
        });

        let errors = validate_enterprise_onboarding_checklist_tracking(&tracking).unwrap_err();

        assert!(errors.contains(&"items contains duplicate stage_id 'providers'.".to_string()));
        assert!(errors.contains(&"items[2].stage_id is unsupported.".to_string()));
        assert!(errors.contains(&"target_date must use YYYY-MM-DD format.".to_string()));
    }
}
