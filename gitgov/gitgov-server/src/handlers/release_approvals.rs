// ============================================================================
// ENTERPRISE RELEASE APPROVALS
// ============================================================================

const RELEASE_APPROVAL_EVIDENCE_SUMMARY_MAX_BYTES: usize = 16 * 1024;
const RELEASE_APPROVAL_DECISIONS: &[&str] = &["approved", "rejected", "accepted-risk"];
const RELEASE_APPROVAL_RISK_SEVERITIES: &[&str] =
    &["none", "low", "medium", "high", "critical"];
const RELEASE_GOVERNANCE_MODES: &[&str] = &[
    "record-only",
    "advisory",
    "approval-required",
    "quorum-required",
];
const RELEASE_GOVERNANCE_ENFORCEMENT: &[&str] = &["disabled", "advisory", "blocking"];

#[derive(Debug, Clone)]
struct ReleaseGovernancePolicy {
    mode: String,
    environment: String,
    approval_required: bool,
    enforcement: String,
    quorum_rules: Vec<(String, i64)>,
}

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

fn normalize_release_governance_evaluation_query(
    query: &mut EnterpriseReleaseGovernanceEvaluationQuery,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    query.repository_full_name = query.repository_full_name.trim().to_string();
    query.release_id = query.release_id.trim().to_string();
    query.environment = query.environment.trim().to_ascii_lowercase();
    normalize_release_approval_optional_text(&mut query.evidence_packet_hash);

    if !is_valid_release_approval_repo(&query.repository_full_name) {
        errors.push("repository_full_name must look like owner/repo.".to_string());
    }
    if query.release_id.is_empty()
        || query.release_id.len() > 120
        || has_control_chars(&query.release_id)
    {
        errors.push("release_id is required and must be valid.".to_string());
    }
    if query.environment.is_empty()
        || query.environment.len() > 80
        || has_control_chars(&query.environment)
    {
        errors.push("environment is required and must be valid.".to_string());
    }
    if let Some(hash) = query.evidence_packet_hash.as_deref() {
        if !is_valid_release_approval_hex_hash(hash) {
            errors.push("evidence_packet_hash must be a 64-character hex SHA-256 hash.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn release_governance_default_policy(environment: &str) -> ReleaseGovernancePolicy {
    ReleaseGovernancePolicy {
        mode: "record-only".to_string(),
        environment: environment.to_string(),
        approval_required: false,
        enforcement: "disabled".to_string(),
        quorum_rules: Vec::new(),
    }
}

fn release_governance_policy_from_object(
    policy: &serde_json::Map<String, serde_json::Value>,
    environment: &str,
) -> ReleaseGovernancePolicy {
    let mode = policy
        .get("mode")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|mode| RELEASE_GOVERNANCE_MODES.contains(mode))
        .unwrap_or("record-only")
        .to_string();

    let policy_environment = policy
        .get("environment")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(environment)
        .to_ascii_lowercase();

    let expected_approval_required = matches!(mode.as_str(), "approval-required" | "quorum-required");
    let expected_enforcement = match mode.as_str() {
        "advisory" => "advisory",
        "approval-required" | "quorum-required" => "blocking",
        _ => "disabled",
    };

    let approval_required = policy
        .get("approval_required")
        .and_then(|value| value.as_bool())
        .unwrap_or(expected_approval_required);
    let enforcement = policy
        .get("enforcement")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| RELEASE_GOVERNANCE_ENFORCEMENT.contains(value))
        .unwrap_or(expected_enforcement)
        .to_string();

    let quorum_rules = policy
        .get("quorum")
        .and_then(|value| value.as_object())
        .and_then(|quorum| quorum.get("rules"))
        .and_then(|value| value.as_array())
        .map(|rules| {
            rules
                .iter()
                .filter_map(|rule| {
                    let role = rule
                        .get("role")
                        .and_then(|value| value.as_str())
                        .map(str::trim)
                        .filter(|value| !value.is_empty())?
                        .to_ascii_lowercase();
                    let required = rule
                        .get("required")
                        .and_then(|value| value.as_i64())
                        .unwrap_or(1)
                        .clamp(1, 20);
                    Some((role, required))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ReleaseGovernancePolicy {
        mode,
        environment: policy_environment,
        approval_required,
        enforcement,
        quorum_rules,
    }
}

fn release_governance_policy_from_profile(
    profile: Option<&serde_json::Value>,
    environment: &str,
) -> ReleaseGovernancePolicy {
    let Some(policy) = profile
        .and_then(|profile| profile.get("release_governance"))
        .and_then(|value| value.as_object())
    else {
        return release_governance_default_policy(environment);
    };

    let requested_environment = environment.trim().to_ascii_lowercase();
    if let Some(override_policy) = policy
        .get("environment_overrides")
        .and_then(|value| value.as_array())
        .and_then(|overrides| {
            overrides.iter().find_map(|candidate| {
                let object = candidate.as_object()?;
                let candidate_environment = object
                    .get("environment")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())?
                    .to_ascii_lowercase();
                if candidate_environment == requested_environment {
                    Some(object)
                } else {
                    None
                }
            })
        })
    {
        return release_governance_policy_from_object(override_policy, environment);
    }

    release_governance_policy_from_object(policy, environment)
}

fn release_approval_role(record: &EnterpriseReleaseApprovalRecord) -> Option<String> {
    ["approver_role", "approval_role", "role"]
        .iter()
        .find_map(|key| {
            record
                .evidence_summary
                .get(key)
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_lowercase())
        })
}

fn release_approval_counts_toward_policy(
    record: &EnterpriseReleaseApprovalRecord,
    expected_hash: Option<&str>,
    now_ms: i64,
) -> bool {
    let positive_decision = matches!(record.decision.as_str(), "approved" | "accepted-risk");
    if !positive_decision {
        return false;
    }
    if record.expires_at.is_some_and(|expires_at| expires_at <= now_ms) {
        return false;
    }
    if let Some(expected_hash) = expected_hash {
        return record
            .evidence_packet_hash
            .as_deref()
            .is_some_and(|hash| hash.eq_ignore_ascii_case(expected_hash));
    }
    true
}

fn evaluate_release_governance_policy(
    query: &EnterpriseReleaseGovernanceEvaluationQuery,
    policy: ReleaseGovernancePolicy,
    approvals: &[EnterpriseReleaseApprovalRecord],
    now_ms: i64,
) -> EnterpriseReleaseGovernanceEvaluationResponse {
    let expected_hash = query.evidence_packet_hash.as_deref();
    let policy_applies = policy.environment.eq_ignore_ascii_case(&query.environment);
    let mut issues = Vec::new();
    let mut next_steps = Vec::new();
    let mut role_counts: HashMap<String, HashSet<String>> = HashMap::new();
    let mut valid_approval_count = 0_i64;
    let mut rejected_count = 0_i64;
    let mut expired_count = 0_i64;
    let mut hash_mismatch_count = 0_i64;

    let approval_summaries = approvals
        .iter()
        .map(|approval| {
            let role = release_approval_role(approval);
            let expired = approval.expires_at.is_some_and(|expires_at| expires_at <= now_ms);
            if expired {
                expired_count += 1;
            }
            if approval.decision == "rejected" {
                rejected_count += 1;
            }
            if let Some(expected_hash) = expected_hash {
                if !approval
                    .evidence_packet_hash
                    .as_deref()
                    .is_some_and(|hash| hash.eq_ignore_ascii_case(expected_hash))
                {
                    hash_mismatch_count += 1;
                }
            }
            let counts_toward_policy =
                release_approval_counts_toward_policy(approval, expected_hash, now_ms);
            if counts_toward_policy {
                valid_approval_count += 1;
                if let Some(role) = role.as_deref() {
                    role_counts
                        .entry(role.to_string())
                        .or_default()
                        .insert(approval.approver.to_ascii_lowercase());
                }
            }

            EnterpriseReleaseGovernanceApprovalSummary {
                id: approval.id.clone(),
                decision: approval.decision.clone(),
                approver: approval.approver.clone(),
                approver_role: role,
                risk_severity: approval.risk_severity.clone(),
                evidence_packet_hash: approval.evidence_packet_hash.clone(),
                expires_at: approval.expires_at,
                created_at: approval.created_at,
                counts_toward_policy,
            }
        })
        .collect::<Vec<_>>();

    if rejected_count > 0 && valid_approval_count == 0 {
        issues.push("A rejected release approval exists and no valid positive approval was found.".to_string());
    }
    if expired_count > 0 {
        issues.push("One or more matching release approval records are expired.".to_string());
    }
    if hash_mismatch_count > 0 && expected_hash.is_some() {
        issues.push("One or more release approval records do not match the requested evidence packet hash.".to_string());
    }
    if !policy_applies {
        issues.push(format!(
            "Release governance policy is configured for '{}' and does not apply to '{}'.",
            policy.environment, query.environment
        ));
    }

    let mut quorum_rules = policy
        .quorum_rules
        .iter()
        .map(|(role, required)| {
            let observed = role_counts
                .get(role)
                .map(|approvers| approvers.len() as i64)
                .unwrap_or(0);
            EnterpriseReleaseGovernanceQuorumRuleSummary {
                role: role.clone(),
                required: *required,
                observed,
                satisfied: observed >= *required,
            }
        })
        .collect::<Vec<_>>();

    let advisory_approval_expected = policy.mode == "advisory" && policy_applies;
    let required_approval_count = if policy.mode == "quorum-required" && policy_applies {
        policy.quorum_rules.iter().map(|(_, required)| *required).sum()
    } else if policy_applies
        && policy.mode != "record-only"
        && (policy.approval_required || advisory_approval_expected)
    {
        1
    } else {
        0
    };

    let quorum_satisfied = policy.mode != "quorum-required"
        || !policy_applies
        || (!quorum_rules.is_empty() && quorum_rules.iter().all(|rule| rule.satisfied));
    if policy.mode == "quorum-required" && policy_applies && quorum_rules.is_empty() {
        issues.push("Quorum-required release governance has no quorum rules configured.".to_string());
        next_steps.push("Add quorum role rules to the release governance profile.".to_string());
    }
    if policy.mode == "quorum-required" && policy_applies {
        for rule in quorum_rules.iter().filter(|rule| !rule.satisfied) {
            issues.push(format!(
                "Missing quorum for role '{}': observed {}, required {}.",
                rule.role, rule.observed, rule.required
            ));
            next_steps.push(format!(
                "Create a valid release approval with evidence_summary.approver_role='{}'.",
                rule.role
            ));
        }
    }

    let approval_requirement_satisfied = if required_approval_count == 0 {
        true
    } else if policy.mode == "quorum-required" {
        quorum_satisfied
    } else {
        valid_approval_count >= required_approval_count
    };

    if policy_applies
        && matches!(policy.mode.as_str(), "advisory" | "approval-required")
        && valid_approval_count == 0
    {
        issues.push("No valid release approval or accepted-risk record was found.".to_string());
        next_steps.push("Create a release approval bound to the current evidence packet hash.".to_string());
    }
    if policy.mode == "record-only" && valid_approval_count == 0 {
        next_steps.push("Create an optional release approval to strengthen audit evidence.".to_string());
    }
    if !policy_applies {
        next_steps.push("Use record-only behavior for this environment or configure a policy for it explicitly.".to_string());
    }

    let blocking = policy_applies && policy.enforcement == "blocking" && !approval_requirement_satisfied;
    let would_block = policy_applies
        && matches!(policy.mode.as_str(), "advisory" | "approval-required" | "quorum-required")
        && !approval_requirement_satisfied;
    let status = if !policy_applies || policy.mode == "record-only" {
        "recorded"
    } else if approval_requirement_satisfied {
        "approved"
    } else if policy.enforcement == "blocking" {
        "blocked"
    } else if policy.enforcement == "advisory" {
        "advisory-warning"
    } else {
        "would-block"
    }
    .to_string();

    if approval_requirement_satisfied && next_steps.is_empty() {
        next_steps.push("No release governance action required for the current policy.".to_string());
    }

    if policy.mode != "quorum-required" {
        quorum_rules.clear();
    }

    EnterpriseReleaseGovernanceEvaluationResponse {
        status,
        policy_satisfied: approval_requirement_satisfied,
        blocking,
        would_block,
        valid_approval_count,
        required_approval_count,
        policy: EnterpriseReleaseGovernancePolicySummary {
            mode: policy.mode,
            environment: policy.environment,
            approval_required: policy.approval_required,
            enforcement: policy.enforcement,
            policy_applies,
            quorum_enabled: !quorum_rules.is_empty(),
            quorum_rules,
        },
        approvals: approval_summaries,
        issues,
        next_steps,
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

pub async fn evaluate_enterprise_release_governance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<EnterpriseReleaseGovernanceEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = normalize_release_governance_evaluation_query(&mut query) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid release governance evaluation query", "details": errors })),
        )
            .into_response();
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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let profile = match state.db.get_enterprise_adoption_profile(&org_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load release governance profile");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let approval_query = EnterpriseReleaseApprovalQuery {
        org_name: None,
        repository_full_name: Some(query.repository_full_name.clone()),
        release_id: Some(query.release_id.clone()),
        environment: Some(query.environment.clone()),
        decision: None,
        limit: Some(100),
        offset: Some(0),
    };
    let approvals = match state
        .db
        .list_enterprise_release_approvals(&org_id, &approval_query, 100, 0)
        .await
    {
        Ok((items, _)) => items,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load release governance approvals");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let policy = release_governance_policy_from_profile(
        profile.as_ref().map(|record| &record.profile),
        &query.environment,
    );
    let evaluation = evaluate_release_governance_policy(
        &query,
        policy,
        &approvals,
        chrono::Utc::now().timestamp_millis(),
    );

    (StatusCode::OK, Json(evaluation)).into_response()
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

    fn approval_record(
        decision: &str,
        approver: &str,
        role: Option<&str>,
    ) -> EnterpriseReleaseApprovalRecord {
        EnterpriseReleaseApprovalRecord {
            id: Uuid::new_v4().to_string(),
            org_id: Uuid::new_v4().to_string(),
            release_id: "release-2026.05.01".to_string(),
            repository_full_name: "yohandry10/Git-Gov".to_string(),
            branch: Some("main".to_string()),
            target_sha: Some("abcdef1234567890".to_string()),
            environment: "production".to_string(),
            decision: decision.to_string(),
            approver: approver.to_string(),
            ticket_id: Some("KAN-46".to_string()),
            evidence_packet_hash: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
            ),
            evidence_packet_uri: Some("/evidence/packets/tickets/KAN-46".to_string()),
            evidence_summary: role
                .map(|role| json!({ "approver_role": role }))
                .unwrap_or_else(|| json!({})),
            risk_severity: "none".to_string(),
            risk_acceptance_reason: None,
            expires_at: None,
            approval_hash: "approval-hash".to_string(),
            created_by: "admin".to_string(),
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    fn evaluation_query() -> EnterpriseReleaseGovernanceEvaluationQuery {
        EnterpriseReleaseGovernanceEvaluationQuery {
            org_name: Some("yohandry10".to_string()),
            repository_full_name: "yohandry10/Git-Gov".to_string(),
            release_id: "release-2026.05.01".to_string(),
            environment: "production".to_string(),
            evidence_packet_hash: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
            ),
        }
    }

    #[test]
    fn release_governance_record_only_never_blocks_without_approval() {
        let evaluation = evaluate_release_governance_policy(
            &evaluation_query(),
            release_governance_default_policy("production"),
            &[],
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.status, "recorded");
        assert!(evaluation.policy_satisfied);
        assert!(!evaluation.blocking);
        assert_eq!(evaluation.required_approval_count, 0);
    }

    #[test]
    fn release_governance_advisory_warns_without_blocking() {
        let evaluation = evaluate_release_governance_policy(
            &evaluation_query(),
            ReleaseGovernancePolicy {
                mode: "advisory".to_string(),
                environment: "production".to_string(),
                approval_required: false,
                enforcement: "advisory".to_string(),
                quorum_rules: Vec::new(),
            },
            &[],
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.status, "advisory-warning");
        assert!(!evaluation.policy_satisfied);
        assert!(!evaluation.blocking);
        assert!(evaluation.would_block);
        assert_eq!(evaluation.required_approval_count, 1);
    }

    #[test]
    fn release_governance_approval_required_blocks_without_valid_approval() {
        let evaluation = evaluate_release_governance_policy(
            &evaluation_query(),
            ReleaseGovernancePolicy {
                mode: "approval-required".to_string(),
                environment: "production".to_string(),
                approval_required: true,
                enforcement: "blocking".to_string(),
                quorum_rules: Vec::new(),
            },
            &[],
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.status, "blocked");
        assert!(!evaluation.policy_satisfied);
        assert!(evaluation.blocking);
        assert_eq!(evaluation.required_approval_count, 1);
    }

    #[test]
    fn release_governance_profile_uses_matching_environment_override() {
        let profile = json!({
            "release_governance": {
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
            }
        });
        let policy = release_governance_policy_from_profile(Some(&profile), "production");
        let evaluation = evaluate_release_governance_policy(
            &evaluation_query(),
            policy,
            &[],
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.policy.mode, "approval-required");
        assert_eq!(evaluation.policy.environment, "production");
        assert_eq!(evaluation.status, "blocked");
        assert!(evaluation.blocking);
    }

    #[test]
    fn release_governance_profile_falls_back_when_environment_override_does_not_match() {
        let profile = json!({
            "release_governance": {
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
            }
        });
        let mut query = evaluation_query();
        query.environment = "staging".to_string();
        let policy = release_governance_policy_from_profile(Some(&profile), "staging");
        let evaluation = evaluate_release_governance_policy(
            &query,
            policy,
            &[],
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.policy.mode, "record-only");
        assert_eq!(evaluation.policy.environment, "staging");
        assert_eq!(evaluation.status, "recorded");
        assert!(!evaluation.blocking);
    }

    #[test]
    fn release_governance_quorum_required_counts_approver_roles() {
        let approvals = vec![
            approval_record("approved", "eng@example.com", Some("engineering")),
            approval_record("approved", "sec@example.com", Some("security")),
        ];
        let evaluation = evaluate_release_governance_policy(
            &evaluation_query(),
            ReleaseGovernancePolicy {
                mode: "quorum-required".to_string(),
                environment: "production".to_string(),
                approval_required: true,
                enforcement: "blocking".to_string(),
                quorum_rules: vec![
                    ("engineering".to_string(), 1),
                    ("security".to_string(), 1),
                ],
            },
            &approvals,
            chrono::Utc::now().timestamp_millis(),
        );

        assert_eq!(evaluation.status, "approved");
        assert!(evaluation.policy_satisfied);
        assert!(!evaluation.blocking);
        assert_eq!(evaluation.valid_approval_count, 2);
        assert!(evaluation.policy.quorum_rules.iter().all(|rule| rule.satisfied));
    }
}
