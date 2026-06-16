pub async fn get_first_governed_repo_wizard_state(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FirstGovernedRepoSetupQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let org_id = match resolve_first_setup_org(&state, &auth_user, query.org_name.as_deref()).await {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };
    let setup = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load first governed repo wizard state");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
    first_setup_audit(
        &state,
        &auth_user,
        &org_id,
        "onboarding_state_viewed",
        setup.as_ref(),
        json!({ "surface": "first_governed_repo_wizard" }),
    )
    .await;
    (
        StatusCode::OK,
        Json(FirstGovernedRepoWizardStateResponse {
            org_id: org_id.clone(),
            found: setup.is_some(),
            state: first_setup_wizard_state(&org_id, setup.as_ref(), &stats),
            setup,
        }),
    )
        .into_response()
}

pub async fn create_first_governed_repo_wizard_run(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FirstGovernedRepoWizardActionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let mut upsert_payload = first_setup_action_to_upsert(payload);
    let org_id =
        match resolve_first_setup_org(&state, &auth_user, upsert_payload.org_name.as_deref()).await {
            Ok(org_id) => org_id,
            Err(resp) => return resp,
        };
    if let Ok(Some(existing)) = state.db.get_first_governed_repo_setup(&org_id).await {
        let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
        first_setup_audit(
            &state,
            &auth_user,
            &org_id,
            "onboarding_run_resumed",
            Some(&existing),
            json!({ "surface": "first_governed_repo_wizard" }),
        )
        .await;
        return (
            StatusCode::OK,
            Json(FirstGovernedRepoWizardRunResponse {
                state: first_setup_wizard_state(&org_id, Some(&existing), &stats),
                setup: existing,
            }),
        )
            .into_response();
    }
    upsert_payload.status = Some("draft".to_string());
    let prepared = match first_setup_prepare_payload(upsert_payload) {
        Ok(payload) => payload,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid first governed repo wizard run", "details": errors })),
            )
                .into_response();
        }
    };
    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &prepared, &auth_user.client_id)
        .await
    {
        Ok(setup) => {
            let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
            first_setup_audit(
                &state,
                &auth_user,
                &org_id,
                "onboarding_run_created",
                Some(&setup),
                json!({ "surface": "first_governed_repo_wizard" }),
            )
            .await;
            (
                StatusCode::CREATED,
                Json(FirstGovernedRepoWizardRunResponse {
                    state: first_setup_wizard_state(&org_id, Some(&setup), &stats),
                    setup,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create first governed repo wizard run");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn update_first_governed_repo_wizard_run(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
    Json(payload): Json<FirstGovernedRepoWizardActionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let run_id = match first_setup_validate_run_id(&run_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let upsert_payload = first_setup_action_to_upsert(payload);
    let org_id =
        match resolve_first_setup_org(&state, &auth_user, upsert_payload.org_name.as_deref()).await {
            Ok(org_id) => org_id,
            Err(resp) => return resp,
        };
    let existing = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(Some(value)) if value.run_id == run_id => value,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "First governed repo wizard run not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to load wizard run before update");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let prepared = match first_setup_prepare_payload(upsert_payload) {
        Ok(payload) => payload,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid first governed repo wizard run", "details": errors })),
            )
                .into_response();
        }
    };
    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &prepared, &auth_user.client_id)
        .await
    {
        Ok(setup) => {
            let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
            first_setup_audit(
                &state,
                &auth_user,
                &org_id,
                "onboarding_run_updated",
                Some(&setup),
                json!({ "previous_status": existing.status }),
            )
            .await;
            (
                StatusCode::OK,
                Json(FirstGovernedRepoWizardRunResponse {
                    state: first_setup_wizard_state(&org_id, Some(&setup), &stats),
                    setup,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to update first governed repo wizard run");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn validate_first_governed_repo_wizard_run(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
    Json(payload): Json<FirstGovernedRepoWizardActionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let run_id = match first_setup_validate_run_id(&run_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let org_id = match resolve_first_setup_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };
    let existing = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(Some(value)) if value.run_id == run_id => value,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "First governed repo wizard run not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to load wizard run before validation");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
    let provider_validation = existing
        .selected_providers
        .iter()
        .map(|provider| first_setup_readiness_from_provider(provider, &existing, &stats))
        .collect::<Vec<_>>();
    let mut baseline = first_setup_baseline_map(&existing);
    baseline.insert("current_step".to_string(), json!("validate_evidence_sources"));
    baseline.insert(
        "provider_validation".to_string(),
        json!({
            "validated_at": chrono::Utc::now().timestamp_millis(),
            "checks": provider_validation,
            "reads_secret_values": false,
            "stores_secret_values": false,
            "mutates_provider_state": false,
            "direct_credential_validation": "Use scripts/control-plane/validate_enterprise_provider_connections.ps1 with customer-approved credentials when needed."
        }),
    );
    let prepared = UpsertFirstGovernedRepoSetupRequest {
        org_name: payload.org_name,
        status: Some(existing.status.clone()),
        goal: existing.goal.clone(),
        repository_full_name: existing.repository_full_name.clone(),
        default_branch: existing.default_branch.clone(),
        selected_providers: existing.selected_providers.clone(),
        selected_modules: existing.selected_modules.clone(),
        policy_preset: existing.policy_preset.clone(),
        baseline: serde_json::Value::Object(baseline),
    };
    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &prepared, &auth_user.client_id)
        .await
    {
        Ok(setup) => {
            first_setup_audit(
                &state,
                &auth_user,
                &org_id,
                "onboarding_provider_validated",
                Some(&setup),
                json!({ "provider_count": setup.selected_providers.len() }),
            )
            .await;
            (
                StatusCode::OK,
                Json(FirstGovernedRepoWizardRunResponse {
                    state: first_setup_wizard_state(&org_id, Some(&setup), &stats),
                    setup,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to persist wizard validation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn plan_first_governed_repo_wizard_run(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
    Json(payload): Json<FirstGovernedRepoWizardActionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let run_id = match first_setup_validate_run_id(&run_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let org_id = match resolve_first_setup_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };
    let existing = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(Some(value)) if value.run_id == run_id => value,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "First governed repo wizard run not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to load wizard run before plan");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let mut baseline = first_setup_baseline_map(&existing);
    baseline.insert("current_step".to_string(), json!("baseline_preview"));
    baseline.insert(
        "baseline_plan".to_string(),
        json!({
            "planned_at": chrono::Utc::now().timestamp_millis(),
            "repository_full_name": existing.repository_full_name,
            "default_branch": existing.default_branch,
            "policy_preset": existing.policy_preset,
            "selected_providers": existing.selected_providers,
            "selected_modules": existing.selected_modules,
            "workflow_preview_required": true,
            "provider_mutation": false,
            "repository_mutation": false,
            "release_blocking_default": false,
            "deployment_gate_mode": "advisory",
            "next_manual_steps": [
                "Review generated workflow template pack before installation.",
                "Configure provider secrets outside GitGov.",
                "Run advisory deployment gate simulation manually."
            ]
        }),
    );
    let prepared = UpsertFirstGovernedRepoSetupRequest {
        org_name: payload.org_name,
        status: Some(existing.status.clone()),
        goal: existing.goal.clone(),
        repository_full_name: existing.repository_full_name.clone(),
        default_branch: existing.default_branch.clone(),
        selected_providers: existing.selected_providers.clone(),
        selected_modules: existing.selected_modules.clone(),
        policy_preset: existing.policy_preset.clone(),
        baseline: serde_json::Value::Object(baseline),
    };
    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &prepared, &auth_user.client_id)
        .await
    {
        Ok(setup) => {
            let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
            first_setup_audit(
                &state,
                &auth_user,
                &org_id,
                "onboarding_baseline_planned",
                Some(&setup),
                json!({ "deployment_gate_mode": "advisory" }),
            )
            .await;
            (
                StatusCode::OK,
                Json(FirstGovernedRepoWizardRunResponse {
                    state: first_setup_wizard_state(&org_id, Some(&setup), &stats),
                    setup,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to persist wizard plan");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn complete_first_governed_repo_wizard_run(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
    Json(payload): Json<FirstGovernedRepoWizardActionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let run_id = match first_setup_validate_run_id(&run_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let org_id = match resolve_first_setup_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };
    let existing = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(Some(value)) if value.run_id == run_id => value,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "First governed repo wizard run not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to load wizard run before complete");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if existing
        .baseline
        .get("gate_readiness")
        .and_then(|value| value.as_str())
        != Some("baseline_ready")
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "First governed repo setup must be baseline_ready before completion",
                "code": "first_governed_repo_not_ready"
            })),
        )
            .into_response();
    }
    let mut baseline = first_setup_baseline_map(&existing);
    baseline.insert("current_step".to_string(), json!("first_result"));
    baseline.insert(
        "first_result_refs".to_string(),
        json!({
            "completed_at": chrono::Utc::now().timestamp_millis(),
            "repository_full_name": existing.repository_full_name,
            "default_branch": existing.default_branch,
            "deployment_gate_history_path": "/governance/releases",
            "advisory_authorization_endpoint": "/deployment-gates/authorize",
            "workflow_template_pack_source": "enterprise_adoption_profile",
            "manual_next_steps": [
                "Run provider connection validation with customer-approved credentials if needed.",
                "Install reviewed workflow templates through the existing manual workflow path.",
                "Run advisory deployment gate authorization and keep the evidence packet."
            ],
            "safety": {
                "manual_first": true,
                "mutates_provider_state": false,
                "mutates_customer_repository": false,
                "release_blocking_default": false,
                "agent_governance_required": false,
                "ai_required": false,
                "compliance_claim": false,
                "certification": false
            }
        }),
    );
    let prepared = UpsertFirstGovernedRepoSetupRequest {
        org_name: payload.org_name,
        status: Some("completed".to_string()),
        goal: existing.goal.clone(),
        repository_full_name: existing.repository_full_name.clone(),
        default_branch: existing.default_branch.clone(),
        selected_providers: existing.selected_providers.clone(),
        selected_modules: existing.selected_modules.clone(),
        policy_preset: existing.policy_preset.clone(),
        baseline: serde_json::Value::Object(baseline),
    };
    match state
        .db
        .upsert_first_governed_repo_setup(&org_id, &prepared, &auth_user.client_id)
        .await
    {
        Ok(setup) => {
            let stats = state.db.get_stats(Some(&org_id)).await.unwrap_or_default();
            first_setup_audit(
                &state,
                &auth_user,
                &org_id,
                "onboarding_completed",
                Some(&setup),
                json!({ "first_result": "ready_for_advisory_gate" }),
            )
            .await;
            (
                StatusCode::OK,
                Json(FirstGovernedRepoWizardRunResponse {
                    state: first_setup_wizard_state(&org_id, Some(&setup), &stats),
                    setup,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, run_id = %run_id, "Failed to complete first governed repo wizard run");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
