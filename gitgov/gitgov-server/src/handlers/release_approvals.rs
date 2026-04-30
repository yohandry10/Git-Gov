// ============================================================================
// ENTERPRISE RELEASE APPROVALS
// ============================================================================

const RELEASE_APPROVAL_EVIDENCE_SUMMARY_MAX_BYTES: usize = 16 * 1024;
const RELEASE_APPROVAL_DECISIONS: &[&str] = &["approved", "rejected", "accepted-risk"];
const RELEASE_APPROVAL_RISK_SEVERITIES: &[&str] =
    &["none", "low", "medium", "high", "critical"];

fn release_approval_scope_error_message(error: OrgScopeError) -> &'static str {
    match error {
        OrgScopeError::BadRequest => "org_name is required for global admin keys",
        OrgScopeError::NotFound => "Organization not found",
        OrgScopeError::Forbidden => "Requested org is outside API key scope",
        OrgScopeError::Internal => "Internal database error",
    }
}

fn normalize_release_approval_optional_text(value: &mut Option<String>) {
    *value = value
        .take()
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty());
}

fn has_control_chars(value: &str) -> bool {
    value.chars().any(|ch| ch.is_control())
}

fn is_valid_release_approval_repo(value: &str) -> bool {
    let parts: Vec<&str> = value.split('/').collect();
    parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && !part.contains(char::is_whitespace))
}

fn is_valid_release_approval_sha(value: &str) -> bool {
    (7..=64).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_valid_release_approval_hex_hash(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_valid_release_approval_ticket_id(value: &str) -> bool {
    static TICKET_ID_RE: OnceLock<Regex> = OnceLock::new();
    let re = TICKET_ID_RE
        .get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9]+-[1-9][0-9]*$").expect("valid ticket regex"));
    re.is_match(value)
}

fn is_valid_release_approval_evidence_uri(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with('/')
        || lower.starts_with("http://")
        || lower.starts_with("https://")
}

fn normalize_and_validate_release_approval(
    payload: &mut CreateEnterpriseReleaseApprovalRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    payload.org_name = payload
        .org_name
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    payload.release_id = payload.release_id.trim().to_string();
    payload.repository_full_name = payload.repository_full_name.trim().to_string();
    payload.environment = payload.environment.trim().to_ascii_lowercase();
    payload.decision = payload.decision.trim().to_ascii_lowercase();
    payload.approver = payload.approver.trim().to_string();
    normalize_release_approval_optional_text(&mut payload.branch);
    normalize_release_approval_optional_text(&mut payload.target_sha);
    normalize_release_approval_optional_text(&mut payload.ticket_id);
    normalize_release_approval_optional_text(&mut payload.evidence_packet_hash);
    normalize_release_approval_optional_text(&mut payload.evidence_packet_uri);
    normalize_release_approval_optional_text(&mut payload.risk_acceptance_reason);

    payload.risk_severity = payload
        .risk_severity
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("none".to_string()));

    if payload.evidence_summary.is_null() {
        payload.evidence_summary = json!({});
    }

    if payload.release_id.is_empty() {
        errors.push("release_id is required.".to_string());
    } else if payload.release_id.len() > 120 || has_control_chars(&payload.release_id) {
        errors.push("release_id is invalid or too long.".to_string());
    }

    if !is_valid_release_approval_repo(&payload.repository_full_name) {
        errors.push("repository_full_name must look like owner/repo.".to_string());
    } else if payload.repository_full_name.len() > 200 {
        errors.push("repository_full_name is too long.".to_string());
    }

    if let Some(branch) = payload.branch.as_deref() {
        if branch.len() > 200 || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }

    if let Some(target_sha) = payload.target_sha.as_deref() {
        if !is_valid_release_approval_sha(target_sha) {
            errors.push("target_sha must be 7 to 64 hexadecimal characters.".to_string());
        }
    }

    if payload.environment.is_empty() {
        errors.push("environment is required.".to_string());
    } else if payload.environment.len() > 80 || has_control_chars(&payload.environment) {
        errors.push("environment is invalid or too long.".to_string());
    }

    if !RELEASE_APPROVAL_DECISIONS.contains(&payload.decision.as_str()) {
        errors.push("decision must be approved, rejected, or accepted-risk.".to_string());
    }

    if payload.approver.is_empty() {
        errors.push("approver is required.".to_string());
    } else if payload.approver.len() > 160 || has_control_chars(&payload.approver) {
        errors.push("approver is invalid or too long.".to_string());
    }

    if let Some(ticket_id) = payload.ticket_id.as_deref() {
        if ticket_id.len() > 32 || !is_valid_release_approval_ticket_id(ticket_id) {
            errors.push("ticket_id must look like KAN-37.".to_string());
        }
    }

    match payload.evidence_packet_hash.as_deref() {
        Some(hash) if is_valid_release_approval_hex_hash(hash) => {}
        Some(_) => errors.push("evidence_packet_hash must be a 64-character hex SHA-256 hash.".to_string()),
        None => errors.push("evidence_packet_hash is required.".to_string()),
    }

    if let Some(uri) = payload.evidence_packet_uri.as_deref() {
        if uri.len() > 500 || uri.contains(char::is_whitespace) || !is_valid_release_approval_evidence_uri(uri) {
            errors.push("evidence_packet_uri must be a relative API path or http(s) URL.".to_string());
        }
    }

    if !payload.evidence_summary.is_object() {
        errors.push("evidence_summary must be a JSON object.".to_string());
    } else {
        let summary_len = serde_json::to_vec(&payload.evidence_summary)
            .map(|bytes| bytes.len())
            .unwrap_or(0);
        if summary_len > RELEASE_APPROVAL_EVIDENCE_SUMMARY_MAX_BYTES {
            errors.push("evidence_summary is too large.".to_string());
        }
    }

    let risk_severity = payload.risk_severity.as_deref().unwrap_or("none");
    if !RELEASE_APPROVAL_RISK_SEVERITIES.contains(&risk_severity) {
        errors.push("risk_severity must be none, low, medium, high, or critical.".to_string());
    }

    if payload.decision == "approved" && matches!(risk_severity, "high" | "critical") {
        errors.push("high or critical risk requires rejected or accepted-risk decision.".to_string());
    }

    if payload.decision == "accepted-risk" {
        if risk_severity == "none" {
            errors.push("accepted-risk requires a non-none risk_severity.".to_string());
        }
        match payload.risk_acceptance_reason.as_deref() {
            Some(reason) if reason.len() <= 2000 && !has_control_chars(reason) => {}
            Some(_) => errors.push("risk_acceptance_reason is invalid or too long.".to_string()),
            None => errors.push("accepted-risk requires risk_acceptance_reason.".to_string()),
        }
        if payload.expires_at.is_none() {
            errors.push("accepted-risk requires expires_at.".to_string());
        }
    } else if let Some(reason) = payload.risk_acceptance_reason.as_deref() {
        if reason.len() > 2000 || has_control_chars(reason) {
            errors.push("risk_acceptance_reason is invalid or too long.".to_string());
        }
    }

    if let Some(expires_at) = payload.expires_at {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let max_ms = now_ms + 366_i64 * 24 * 60 * 60 * 1000;
        if expires_at <= now_ms {
            errors.push("expires_at must be in the future.".to_string());
        }
        if expires_at > max_ms {
            errors.push("expires_at cannot be more than 366 days in the future.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_release_approval_query(
    query: &mut EnterpriseReleaseApprovalQuery,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.repository_full_name);
    normalize_release_approval_optional_text(&mut query.release_id);
    normalize_release_approval_optional_text(&mut query.environment);
    normalize_release_approval_optional_text(&mut query.decision);

    if let Some(repo) = query.repository_full_name.as_deref() {
        if !is_valid_release_approval_repo(repo) {
            errors.push("repository_full_name must look like owner/repo.".to_string());
        }
    }
    if let Some(environment) = query.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
    }
    if let Some(decision) = query.decision.as_mut() {
        *decision = decision.to_ascii_lowercase();
        if !RELEASE_APPROVAL_DECISIONS.contains(&decision.as_str()) {
            errors.push("decision must be approved, rejected, or accepted-risk.".to_string());
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

fn compute_release_approval_hash(
    approval_id: &str,
    org_id: &str,
    created_by: &str,
    payload: &CreateEnterpriseReleaseApprovalRequest,
) -> String {
    let canonical = json!({
        "approval_id": approval_id,
        "org_id": org_id,
        "release_id": &payload.release_id,
        "repository_full_name": &payload.repository_full_name,
        "branch": &payload.branch,
        "target_sha": &payload.target_sha,
        "environment": &payload.environment,
        "decision": &payload.decision,
        "approver": &payload.approver,
        "ticket_id": &payload.ticket_id,
        "evidence_packet_hash": &payload.evidence_packet_hash,
        "evidence_packet_uri": &payload.evidence_packet_uri,
        "evidence_summary": &payload.evidence_summary,
        "risk_severity": &payload.risk_severity,
        "risk_acceptance_reason": &payload.risk_acceptance_reason,
        "expires_at": payload.expires_at,
        "created_by": created_by
    });
    let digest = Sha256::digest(
        serde_json::to_vec(&canonical)
            .expect("canonical enterprise release approval JSON should serialize"),
    );
    format!("{digest:x}")
}

pub async fn list_enterprise_release_approvals(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<EnterpriseReleaseApprovalQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let (limit, offset) = match normalize_release_approval_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid release approval query", "details": errors })),
            )
                .into_response();
        }
    };

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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .list_enterprise_release_approvals(&org_id, &query, limit, offset)
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(EnterpriseReleaseApprovalListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list enterprise release approvals");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn create_enterprise_release_approval(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<CreateEnterpriseReleaseApprovalRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = normalize_and_validate_release_approval(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid release approval", "details": errors })),
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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let approval_id = Uuid::new_v4().to_string();
    let approval_hash =
        compute_release_approval_hash(&approval_id, &org_id, &auth_user.client_id, &payload);

    match state
        .db
        .create_enterprise_release_approval(
            &org_id,
            &approval_id,
            &payload,
            &approval_hash,
            &auth_user.client_id,
        )
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "create_enterprise_release_approval".to_string(),
                target_type: Some("enterprise_release_approval".to_string()),
                target_id: Some(record.id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "release_id": &record.release_id,
                    "repository_full_name": &record.repository_full_name,
                    "environment": &record.environment,
                    "decision": &record.decision,
                    "risk_severity": &record.risk_severity,
                    "ticket_id": &record.ticket_id,
                    "approval_hash": &record.approval_hash
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (enterprise release approval)");
            }

            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(DbError::Duplicate(_)) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Duplicate release approval" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create enterprise release approval");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod release_approval_tests {
    use super::*;

    fn valid_release_approval() -> CreateEnterpriseReleaseApprovalRequest {
        CreateEnterpriseReleaseApprovalRequest {
            org_name: Some("yohandry10".to_string()),
            release_id: "release-2026.04.30".to_string(),
            repository_full_name: "yohandry10/Git-Gov".to_string(),
            branch: Some("main".to_string()),
            target_sha: Some("abcdef1234567890".to_string()),
            environment: "production".to_string(),
            decision: "approved".to_string(),
            approver: "release.manager@example.com".to_string(),
            ticket_id: Some("KAN-37".to_string()),
            evidence_packet_hash: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
            ),
            evidence_packet_uri: Some("/evidence/packets/tickets/KAN-37".to_string()),
            evidence_summary: json!({
                "checks": "passed",
                "policy": "strict"
            }),
            risk_severity: Some("none".to_string()),
            risk_acceptance_reason: None,
            expires_at: None,
        }
    }

    #[test]
    fn enterprise_release_approval_validation_accepts_valid_approval() {
        let mut payload = valid_release_approval();

        assert!(normalize_and_validate_release_approval(&mut payload).is_ok());
        assert_eq!(payload.environment, "production");
        assert_eq!(payload.risk_severity.as_deref(), Some("none"));
    }

    #[test]
    fn enterprise_release_approval_validation_requires_evidence_hash() {
        let mut payload = valid_release_approval();
        payload.evidence_packet_hash = None;

        let errors = normalize_and_validate_release_approval(&mut payload).unwrap_err();

        assert!(errors.contains(&"evidence_packet_hash is required.".to_string()));
    }

    #[test]
    fn enterprise_release_approval_validation_rejects_high_risk_approval() {
        let mut payload = valid_release_approval();
        payload.risk_severity = Some("high".to_string());

        let errors = normalize_and_validate_release_approval(&mut payload).unwrap_err();

        assert!(errors.contains(
            &"high or critical risk requires rejected or accepted-risk decision.".to_string()
        ));
    }

    #[test]
    fn enterprise_release_approval_validation_accepts_bounded_risk_acceptance() {
        let mut payload = valid_release_approval();
        payload.decision = "accepted-risk".to_string();
        payload.risk_severity = Some("medium".to_string());
        payload.risk_acceptance_reason = Some("Temporary launch exception.".to_string());
        payload.expires_at = Some(chrono::Utc::now().timestamp_millis() + 24 * 60 * 60 * 1000);

        assert!(normalize_and_validate_release_approval(&mut payload).is_ok());
    }

    #[test]
    fn enterprise_release_approval_validation_rejects_file_uri() {
        let mut payload = valid_release_approval();
        payload.evidence_packet_uri = Some("file:///tmp/evidence.json".to_string());

        let errors = normalize_and_validate_release_approval(&mut payload).unwrap_err();

        assert!(errors.contains(
            &"evidence_packet_uri must be a relative API path or http(s) URL.".to_string()
        ));
    }
}
