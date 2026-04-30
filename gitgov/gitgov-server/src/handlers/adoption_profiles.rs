// ============================================================================
// ENTERPRISE ADOPTION PROFILES
// ============================================================================

const ENTERPRISE_ADOPTION_PROFILE_MAX_BYTES: usize = 32 * 1024;
const ADOPTION_POLICY_PRESETS: &[&str] = &["audit-only", "moderate", "strict"];
const ADOPTION_PROVIDER_IDS: &[&str] =
    &["github", "jira", "jenkins", "sonarqube", "render", "vercel"];
const ADOPTION_MODULE_IDS: &[&str] = &[
    "traceability",
    "github-evidence",
    "release-readiness",
    "quality-gates",
    "evidence-packets",
    "security-review",
    "trend-enforcement",
    "formal-approval",
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
            "modules": ["traceability", "release-readiness", "security-review"]
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
}
