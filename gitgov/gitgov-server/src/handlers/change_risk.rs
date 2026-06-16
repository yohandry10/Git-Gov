// ============================================================================
// CHANGE RISK ADVISORY
// ============================================================================

const CHANGE_RISK_LEVELS: &[&str] = &["low", "medium", "high", "unknown"];
const CHANGE_RISK_TEXT_MAX_CHARS: usize = 200;
const CHANGE_RISK_RULESET_VERSION: &str = "change_risk_rules.v1";

fn change_risk_scope_error_message(error: OrgScopeError) -> &'static str {
    release_approval_scope_error_message(error)
}

fn normalize_and_validate_change_risk_request(
    payload: &mut ChangeRiskEvaluationRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_release_approval_optional_text(&mut payload.change_id);
    normalize_release_approval_optional_text(&mut payload.deployment_gate_id);
    normalize_release_approval_optional_text(&mut payload.release_id);
    normalize_release_approval_optional_text(&mut payload.commit_sha);
    normalize_release_approval_optional_text(&mut payload.evidence_packet_hash);
    payload.repository_full_name = payload.repository_full_name.trim().to_string();
    payload.branch = payload.branch.trim().to_string();
    payload.environment = payload.environment.trim().to_ascii_lowercase();
    payload.evidence_refs = payload
        .evidence_refs
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .take(20)
        .collect();
    payload.evidence_refs.sort();
    payload.evidence_refs.dedup();

    if !is_valid_release_approval_repo(&payload.repository_full_name) {
        errors.push("repository_full_name must look like owner/repo.".to_string());
    }
    if payload.branch.is_empty()
        || payload.branch.len() > CHANGE_RISK_TEXT_MAX_CHARS
        || has_control_chars(&payload.branch)
    {
        errors.push("branch is required and must be valid.".to_string());
    }
    if payload.environment.is_empty()
        || payload.environment.len() > 80
        || has_control_chars(&payload.environment)
    {
        errors.push("environment is required and must be valid.".to_string());
    }
    for (field, value) in [
        ("change_id", payload.change_id.as_deref()),
        ("deployment_gate_id", payload.deployment_gate_id.as_deref()),
        ("release_id", payload.release_id.as_deref()),
    ] {
        if let Some(value) = value {
            if value.len() > CHANGE_RISK_TEXT_MAX_CHARS || has_control_chars(value) {
                errors.push(format!("{field} is invalid or too long."));
            }
        }
    }
    if let Some(commit_sha) = payload.commit_sha.as_mut() {
        if is_valid_release_approval_sha(commit_sha) {
            *commit_sha = commit_sha.to_ascii_lowercase();
        } else {
            errors.push(
                "commit_sha must be a full 40 or 64 character hexadecimal commit SHA."
                    .to_string(),
            );
        }
    }
    if let Some(hash) = payload.evidence_packet_hash.as_mut() {
        if is_valid_release_approval_hex_hash(hash) {
            *hash = hash.to_ascii_lowercase();
        } else {
            errors.push("evidence_packet_hash must be a 64-character hex SHA-256 hash.".to_string());
        }
    }
    for evidence_ref in &payload.evidence_refs {
        if evidence_ref.len() > 500
            || evidence_ref.contains(char::is_whitespace)
            || !is_valid_release_approval_evidence_uri(evidence_ref)
        {
            errors.push("evidence_refs must contain relative API paths or https URLs.".to_string());
            break;
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_change_risk_query(
    query: &mut ChangeRiskEvaluationQuery,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.evaluation_id);
    normalize_release_approval_optional_text(&mut query.repository_full_name);
    normalize_release_approval_optional_text(&mut query.branch);
    normalize_release_approval_optional_text(&mut query.environment);
    normalize_release_approval_optional_text(&mut query.change_id);
    normalize_release_approval_optional_text(&mut query.deployment_gate_id);
    normalize_release_approval_optional_text(&mut query.release_id);
    normalize_release_approval_optional_text(&mut query.commit_sha);

    if let Some(repo) = query.repository_full_name.as_deref() {
        if !is_valid_release_approval_repo(repo) {
            errors.push("repository_full_name must look like owner/repo.".to_string());
        }
    }
    if let Some(environment) = query.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
        if environment.len() > 80 || has_control_chars(environment) {
            errors.push("environment is invalid or too long.".to_string());
        }
    }
    if let Some(branch) = query.branch.as_deref() {
        if branch.len() > CHANGE_RISK_TEXT_MAX_CHARS || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }
    if let Some(commit_sha) = query.commit_sha.as_mut() {
        if is_valid_release_approval_sha(commit_sha) {
            *commit_sha = commit_sha.to_ascii_lowercase();
        } else {
            errors.push(
                "commit_sha must be a full 40 or 64 character hexadecimal commit SHA."
                    .to_string(),
            );
        }
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    if errors.is_empty() {
        Ok((limit, offset))
    } else {
        Err(errors)
    }
}

fn unique_change_risk_values(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

include!("change_risk_rules.rs");


fn change_risk_level(reasons: &[String], missing: &[String], blocking: &[String]) -> String {
    if !blocking.is_empty()
        || reasons.iter().any(|reason| {
            matches!(
                reason.as_str(),
                "deployment_gate_blocked" | "break_glass_involved"
            )
        })
    {
        "high".to_string()
    } else if !missing.is_empty()
        || reasons.iter().any(|reason| {
            matches!(
                reason.as_str(),
                "deployment_gate_requires_approval"
                    | "deployment_gate_advisory"
                    | "production_environment"
                    | "stale_or_insufficient_evidence"
            )
        })
    {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn manual_actions_for_change_risk(
    reasons: &[String],
    missing: &[String],
    blocking: &[String],
) -> Vec<String> {
    let mut actions = Vec::new();
    if missing.iter().any(|item| item == "release_evidence_packet") {
        push_unique(
            &mut actions,
            "Generate or attach a release evidence packet for this change.",
        );
    }
    if missing.iter().any(|item| item == "release_approval") {
        push_unique(
            &mut actions,
            "Record a human release approval or accepted-risk decision before deployment.",
        );
    }
    if missing.iter().any(|item| item == "deployment_gate_authorization") {
        push_unique(
            &mut actions,
            "Run the Deployment Gate authorization flow and review its decision.",
        );
    }
    if reasons.iter().any(|item| item == "production_environment") {
        push_unique(
            &mut actions,
            "Have a human reviewer confirm production impact and rollback readiness.",
        );
    }
    if reasons.iter().any(|item| item == "break_glass_involved") {
        push_unique(
            &mut actions,
            "Review the break-glass approval, expiry, and incident context manually.",
        );
    }
    if !blocking.is_empty() {
        push_unique(&mut actions, "Resolve blocking governance gaps or document risk acceptance.");
    }
    if actions.is_empty() {
        push_unique(
            &mut actions,
            "Proceed with normal human release review; no automatic approval is granted.",
        );
    }
    actions
}

fn evaluate_change_risk_advisory(
    payload: &ChangeRiskEvaluationRequest,
    gate: Option<&DeploymentGateAuthorizationRecord>,
) -> (String, Vec<String>, Vec<String>, Vec<String>, Vec<String>, serde_json::Value) {
    let mut reasons = Vec::new();
    let mut missing = Vec::new();
    let mut blocking = Vec::new();

    if matches!(payload.environment.as_str(), "prod" | "production") {
        push_unique(&mut reasons, "production_environment");
    }

    match gate {
        Some(gate) => {
            if gate.decision == "blocked" {
                push_unique(&mut reasons, "deployment_gate_blocked");
                blocking.extend(gate.blocked_by.iter().cloned());
                if blocking.is_empty() {
                    blocking.push(gate.reason.clone());
                }
            } else if gate.decision == "break_glass" || gate.break_glass_used {
                push_unique(&mut reasons, "break_glass_involved");
            } else if gate.would_block
                || gate.decision == "advisory"
                || !gate.warnings.is_empty()
            {
                push_unique(&mut reasons, "deployment_gate_advisory");
            } else {
                push_unique(&mut reasons, "deployment_gate_allowed");
            }

            if gate.evaluation.required_approval_count > gate.evaluation.valid_approval_count {
                push_unique(&mut reasons, "deployment_gate_requires_approval");
                push_unique(&mut missing, "release_approval");
            }
            for item in gate
                .governance_decision
                .get("evidence")
                .and_then(|evidence| evidence.get("missing_evidence"))
                .and_then(|value| value.as_array())
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str())
            {
                push_unique(&mut missing, item);
            }
            for warning in &gate.warnings {
                let code = governance_reason_code(warning);
                if code == "missing_first_governed_repo_setup" {
                    push_unique(&mut missing, "first_governed_repo_setup");
                }
            }
        }
        None => {
            push_unique(&mut reasons, "deployment_gate_not_provided");
            push_unique(&mut missing, "deployment_gate_authorization");
        }
    }

    if payload.evidence_packet_hash.is_none() && gate.is_none() {
        push_unique(&mut reasons, "stale_or_insufficient_evidence");
        push_unique(&mut missing, "release_evidence_packet");
    }
    if payload.release_id.is_none() && gate.is_none() {
        push_unique(&mut missing, "release_context");
    }
    if payload.commit_sha.is_none() && gate.is_none() {
        push_unique(&mut missing, "commit_sha");
    }

    reasons = unique_change_risk_values(reasons);
    missing = unique_change_risk_values(missing);
    blocking = unique_change_risk_values(blocking);
    let risk_level = change_risk_level(&reasons, &missing, &blocking);
    let recommended_manual_actions = manual_actions_for_change_risk(&reasons, &missing, &blocking);

    let evaluation = json!({
        "schema_version": "change-risk-advisory.v1",
        "deterministic": true,
        "advisory_only": true,
        "llm_used": false,
        "agent_governance_used": false,
        "compliance_claim": false,
        "certification": false,
        "inputs": {
            "repository_full_name": payload.repository_full_name,
            "branch": payload.branch,
            "environment": payload.environment,
            "change_id": payload.change_id,
            "deployment_gate_id": payload.deployment_gate_id,
            "release_id": payload.release_id,
            "commit_sha": payload.commit_sha,
            "evidence_packet_hash": payload.evidence_packet_hash,
            "evidence_refs": payload.evidence_refs
        },
        "deployment_gate": gate.map(|gate| json!({
            "authorization_id": gate.authorization_id,
            "decision": gate.decision,
            "approved": gate.approved,
            "blocking": gate.blocking,
            "would_block": gate.would_block,
            "break_glass_used": gate.break_glass_used,
            "reason": gate.reason,
            "policy_checksum": gate.policy_checksum,
            "governance_decision": gate.governance_decision
        })).unwrap_or_else(|| json!({ "found": false }))
    });

    (
        risk_level,
        reasons,
        missing,
        blocking,
        recommended_manual_actions,
        evaluation,
    )
}

async fn resolve_change_risk_org(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_name: Option<&str>,
) -> Result<String, impl IntoResponse> {
    match resolve_and_check_org_scope(state, auth_user.org_id.as_deref(), org_name, true).await {
        Ok(Some(org_id)) => Ok(org_id),
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "org_name is required for global admin keys" })),
        )),
        Err(err) => Err((
            org_scope_status(err),
            Json(json!({ "error": change_risk_scope_error_message(err) })),
        )),
    }
}

async fn load_change_risk_gate(
    state: &Arc<AppState>,
    org_id: &str,
    payload: &ChangeRiskEvaluationRequest,
) -> Result<Option<DeploymentGateAuthorizationRecord>, impl IntoResponse> {
    let Some(deployment_gate_id) = payload.deployment_gate_id.as_deref() else {
        return Ok(None);
    };

    let query = DeploymentGateAuthorizationQuery {
        authorization_id: Some(deployment_gate_id.to_string()),
        repository_full_name: Some(payload.repository_full_name.clone()),
        branch: Some(payload.branch.clone()),
        environment: Some(payload.environment.clone()),
        release_id: payload.release_id.clone(),
        target_sha: payload.commit_sha.clone(),
        org_name: None,
        decision: None,
        deployer: None,
        limit: Some(1),
        offset: Some(0),
    };

    match state
        .db
        .list_deployment_gate_authorizations(org_id, &query, 1, 0)
        .await
    {
        Ok((items, _)) => match items.into_iter().next() {
            Some(gate) => Ok(Some(gate)),
            None => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid change risk evaluation",
                    "details": ["deployment_gate_id was not found for the requested organization and change scope."]
                })),
            )),
        },
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load change risk deployment gate");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            ))
        }
    }
}

fn change_risk_trace_response_from_record(
    record: ChangeRiskEvaluationRecord,
) -> ChangeRiskEvaluationTraceResponse {
    ChangeRiskEvaluationTraceResponse {
        evaluation_id: record.evaluation_id,
        org_id: record.org_id,
        ruleset_version: record.ruleset_version,
        triggered_rules: record.triggered_rules,
        non_triggered_rules: record.non_triggered_rules,
        evaluation_trace: record.evaluation_trace,
        trace_hash: record.trace_hash,
        advisory_only: record.advisory_only,
        llm_used: record.llm_used,
        agent_governance_used: record.agent_governance_used,
        compliance_claim: record.compliance_claim,
        certification: record.certification,
        created_at: record.created_at,
    }
}

pub async fn get_change_risk_rules(
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }

    (
        StatusCode::OK,
        Json(ChangeRiskRuleCatalogResponse {
            ruleset_version: CHANGE_RISK_RULESET_VERSION.to_string(),
            catalog_hash: change_risk_catalog_hash(),
            rules: change_risk_rule_catalog(),
            advisory_only: true,
            llm_used: false,
            agent_governance_used: false,
            compliance_claim: false,
            certification: false,
        }),
    )
        .into_response()
}

pub async fn create_change_risk_evaluation(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ChangeRiskEvaluationRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = normalize_and_validate_change_risk_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid change risk evaluation", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    let gate = match load_change_risk_gate(&state, &org_id, &payload).await {
        Ok(gate) => gate,
        Err(response) => return response.into_response(),
    };

    if payload.evidence_packet_hash.is_none() {
        payload.evidence_packet_hash = gate.as_ref().map(|gate| gate.evidence_packet_hash.clone());
    }
    if payload.release_id.is_none() {
        payload.release_id = gate.as_ref().map(|gate| gate.release_id.clone());
    }
    if payload.commit_sha.is_none() {
        payload.commit_sha = gate.as_ref().map(|gate| gate.target_sha.clone());
    }

    let (risk_level, risk_reasons, missing_evidence, blocking_gaps, recommended_manual_actions, mut evaluation) =
        evaluate_change_risk_advisory(&payload, gate.as_ref());
    debug_assert!(CHANGE_RISK_LEVELS.contains(&risk_level.as_str()));
    let (triggered_rules, non_triggered_rules, evaluation_trace, trace_hash) =
        build_change_risk_rule_trace(
            &payload,
            gate.as_ref(),
            &risk_level,
            &risk_reasons,
            &missing_evidence,
            &blocking_gaps,
        );
    if let Some(evaluation_object) = evaluation.as_object_mut() {
        evaluation_object.insert(
            "ruleset_version".to_string(),
            json!(CHANGE_RISK_RULESET_VERSION),
        );
        evaluation_object.insert("triggered_rules".to_string(), json!(&triggered_rules));
        evaluation_object.insert("trace_hash".to_string(), json!(&trace_hash));
    }
    let request_payload = match serde_json::to_value(&payload) {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize change risk request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let evaluation_id = format!("cra_{}", Uuid::new_v4().simple());
    let create_input = CreateChangeRiskEvaluationInput {
        evaluation_id,
        payload,
        risk_level,
        ruleset_version: CHANGE_RISK_RULESET_VERSION.to_string(),
        risk_reasons,
        missing_evidence,
        blocking_gaps,
        recommended_manual_actions,
        triggered_rules,
        non_triggered_rules,
        evaluation_trace,
        trace_hash,
        evaluation,
        request_payload,
        created_by: auth_user.client_id.clone(),
    };

    match state
        .db
        .create_change_risk_evaluation(&org_id, &create_input)
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "create_change_risk_evaluation".to_string(),
                target_type: Some("change_risk_evaluation".to_string()),
                target_id: Some(record.evaluation_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "evaluation_id": &record.evaluation_id,
                    "repository_full_name": &record.repository_full_name,
                    "branch": &record.branch,
                    "environment": &record.environment,
                    "change_id": &record.change_id,
                    "deployment_gate_id": &record.deployment_gate_id,
                    "release_id": &record.release_id,
                    "risk_level": &record.risk_level,
                    "ruleset_version": &record.ruleset_version,
                    "triggered_rules": &record.triggered_rules,
                    "trace_hash": &record.trace_hash,
                    "advisory_only": record.advisory_only,
                    "llm_used": record.llm_used,
                    "agent_governance_used": record.agent_governance_used,
                    "compliance_claim": record.compliance_claim,
                    "certification": record.certification
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (change risk evaluation)");
            }
            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to persist change risk evaluation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_change_risk_evaluation(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(evaluation_id): Path<String>,
    Query(mut query): Query<ChangeRiskEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    match state
        .db
        .get_change_risk_evaluation(&org_id, evaluation_id.trim())
        .await
    {
        Ok(Some(record)) => (StatusCode::OK, Json(record)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change risk evaluation not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to get change risk evaluation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_change_risk_evaluation_trace(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(evaluation_id): Path<String>,
    Query(mut query): Query<ChangeRiskEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    match state
        .db
        .get_change_risk_evaluation(&org_id, evaluation_id.trim())
        .await
    {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(change_risk_trace_response_from_record(record)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change risk evaluation not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to get change risk evaluation trace");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_change_risk_evaluations(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ChangeRiskEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }

    let (limit, offset) = match normalize_change_risk_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid change risk evaluation query", "details": errors })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    match state
        .db
        .list_change_risk_evaluations(&org_id, &query, limit, offset)
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(ChangeRiskEvaluationListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list change risk evaluations");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
