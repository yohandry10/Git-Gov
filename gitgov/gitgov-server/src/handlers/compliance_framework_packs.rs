// ============================================================================
// CUSTOMER-OWNED COMPLIANCE FRAMEWORK PACKS
// ============================================================================

const CUSTOMER_FRAMEWORK_PACK_SCHEMA_VERSION: &str = "gitgov_customer_framework_pack.v1";
const MAX_CUSTOMER_FRAMEWORK_CONTROLS: usize = 50;
const MAX_CUSTOMER_CONTROL_EVIDENCE_TYPES: usize = 20;

const CUSTOMER_FRAMEWORK_ALLOWED_EVIDENCE_TYPES: &[&str] = &[
    "deployment_gate.decision",
    "policy.checksum",
    "policy.source",
    "release_approval",
    "ci_build_evidence",
    "code_change_evidence",
    "pr_review_evidence",
    "quality_gate_result",
    "deployment_target",
    "missing_evidence",
    "audit_trail",
    "deployment_gate.agent_governance_used",
];

const RESERVED_CUSTOMER_FRAMEWORK_PREFIXES: &[&str] = &[
    "gitgov_",
    "official_",
    "soc2_",
    "iso27001_",
    "nist_",
    "pci_",
    "sbs_",
    "lgpd_",
];

const FRAMEWORK_PACK_REVIEW_NEEDS_REVIEW: &str = "needs_review";
const FRAMEWORK_PACK_REVIEW_REVIEWED: &str = "reviewed";
const FRAMEWORK_PACK_REVIEW_NEEDS_CHANGES: &str = "needs_changes";
const FRAMEWORK_PACK_REVIEW_REJECTED: &str = "rejected";
const FRAMEWORK_PACK_REVIEW_ARCHIVED: &str = "archived";
const MAX_FRAMEWORK_PACK_REVIEW_NOTE_LEN: usize = 1000;

fn customer_framework_pack_hash(pack: &serde_json::Value) -> String {
    let content = serde_json::to_string(pack).unwrap_or_else(|_| "{}".to_string());
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

fn safe_pack_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_separator = false;
    for ch in value.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_separator = false;
        } else if !last_separator {
            slug.push('_');
            last_separator = true;
        }
        if slug.len() >= 32 {
            break;
        }
    }
    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() {
        "framework".to_string()
    } else {
        slug
    }
}

fn normalize_safe_framework_pack_review_text(value: &mut Option<String>) -> Result<(), String> {
    let Some(text) = value.take() else {
        return Ok(());
    };
    let normalized = text.trim().to_string();
    if normalized.is_empty() {
        *value = None;
        return Ok(());
    }
    if normalized.len() > MAX_FRAMEWORK_PACK_REVIEW_NOTE_LEN {
        return Err(format!(
            "review notes must be {MAX_FRAMEWORK_PACK_REVIEW_NOTE_LEN} characters or less"
        ));
    }
    let lowered = normalized.to_ascii_lowercase();
    if lowered.contains("<script")
        || lowered.contains("</")
        || lowered.contains("<iframe")
        || lowered.contains("bearer ")
        || lowered.contains("ghp_")
        || lowered.contains("glpat-")
    {
        return Err("review notes must be plain text and cannot contain secrets".to_string());
    }
    *value = Some(normalized);
    Ok(())
}

fn normalize_framework_pack_review_request(
    payload: &mut ComplianceFrameworkPackReviewRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.review_status = payload.review_status.trim().to_ascii_lowercase();
    if ![
        FRAMEWORK_PACK_REVIEW_NEEDS_REVIEW,
        FRAMEWORK_PACK_REVIEW_REVIEWED,
        FRAMEWORK_PACK_REVIEW_NEEDS_CHANGES,
        FRAMEWORK_PACK_REVIEW_REJECTED,
        FRAMEWORK_PACK_REVIEW_ARCHIVED,
    ]
    .contains(&payload.review_status.as_str())
    {
        errors.push("review_status must be needs_review, reviewed, needs_changes, rejected, or archived.".to_string());
    }
    if let Err(error) = normalize_safe_framework_pack_review_text(&mut payload.review_notes_safe) {
        errors.push(error);
    }
    if let Err(error) = normalize_safe_framework_pack_review_text(&mut payload.rejected_reason_safe)
    {
        errors.push(error);
    }
    if payload.review_status == FRAMEWORK_PACK_REVIEW_REJECTED
        && payload.rejected_reason_safe.is_none()
    {
        errors.push("rejected_reason_safe is required when review_status is rejected.".to_string());
    }
    if payload.review_status == FRAMEWORK_PACK_REVIEW_NEEDS_CHANGES
        && payload.review_notes_safe.is_none()
    {
        errors.push("review_notes_safe is required when review_status is needs_changes.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_framework_pack_can_be_marked_reviewed(
    pack: &ComplianceFrameworkPackRecord,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if pack.owner_type != "customer" {
        errors.push("only customer-owned framework packs can be reviewed in this flow.".to_string());
    }
    if pack.source != "customer_provided" {
        errors.push("framework pack source must be customer_provided.".to_string());
    }
    if pack.control_count <= 0 {
        errors.push("framework pack must contain at least one control.".to_string());
    }
    if !pack.pack_hash.starts_with("sha256:") || pack.pack_hash.len() != 71 {
        errors.push("framework pack hash must be present before review.".to_string());
    }
    if pack.compliance_claim
        || pack.regulatory_claim
        || pack.gitgov_certifies
        || pack.official_regulatory_mapping
        || !pack.requires_auditor_review
    {
        errors.push("framework pack claim/provenance flags are not safe for customer review.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_framework_pack_id(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("cfp_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err(format!("{field} must be a valid cfp_ identifier"))
    }
}

fn normalize_framework_pack_diff_query(
    query: &mut ComplianceFrameworkPackDiffQuery,
) -> Result<(String, String), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    let base_pack_id = match normalize_framework_pack_id(&query.base_pack_id, "base_pack_id") {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let target_pack_id = match normalize_framework_pack_id(&query.target_pack_id, "target_pack_id") {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    if base_pack_id == target_pack_id && !base_pack_id.is_empty() {
        errors.push("base_pack_id and target_pack_id must be different framework pack versions.".to_string());
    }

    if errors.is_empty() {
        Ok((base_pack_id, target_pack_id))
    } else {
        Err(errors)
    }
}

fn original_customer_framework_id(pack: &serde_json::Value) -> String {
    json_str_field(pack, "/framework/id")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn framework_pack_control_side(
    control: &serde_json::Value,
) -> Option<(String, ComplianceFrameworkPackDiffControlSide)> {
    let control_id =
        normalize_customer_control_id(json_str_field(control, "/control_id").unwrap_or_default());
    if control_id.is_empty() {
        return None;
    }
    let title = json_str_field(control, "/title")
        .unwrap_or_default()
        .trim()
        .to_string();
    let description = json_str_field(control, "/description")
        .unwrap_or_default()
        .trim()
        .to_string();
    let mut required_evidence_types = control
        .pointer("/required_evidence_types")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|value| value.trim().to_string()))
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    required_evidence_types.sort();
    required_evidence_types.dedup();

    Some((
        control_id,
        ComplianceFrameworkPackDiffControlSide {
            title,
            description,
            required_evidence_types,
        },
    ))
}

fn framework_pack_controls_by_id(
    pack: &serde_json::Value,
) -> HashMap<String, ComplianceFrameworkPackDiffControlSide> {
    pack.pointer("/controls")
        .and_then(|value| value.as_array())
        .map(|controls| {
            controls
                .iter()
                .filter_map(framework_pack_control_side)
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

fn framework_pack_changed_fields(
    base: &ComplianceFrameworkPackDiffControlSide,
    target: &ComplianceFrameworkPackDiffControlSide,
) -> Vec<String> {
    let mut fields = Vec::new();
    if base.title != target.title {
        fields.push("title".to_string());
    }
    if base.description != target.description {
        fields.push("description".to_string());
    }
    if base.required_evidence_types != target.required_evidence_types {
        fields.push("required_evidence_types".to_string());
    }
    fields
}

fn framework_pack_claims_are_safe(pack: &ComplianceFrameworkPackRecord) -> bool {
    pack.owner_type == "customer"
        && pack.source == "customer_provided"
        && !pack.compliance_claim
        && !pack.regulatory_claim
        && !pack.gitgov_certifies
        && !pack.official_regulatory_mapping
        && pack.requires_auditor_review
}

fn build_framework_pack_diff_response(
    base_source: crate::db::ComplianceFrameworkPackDiffSource,
    target_source: crate::db::ComplianceFrameworkPackDiffSource,
) -> Result<ComplianceFrameworkPackDiffResponse, (StatusCode, serde_json::Value)> {
    if !framework_pack_claims_are_safe(&base_source.record)
        || !framework_pack_claims_are_safe(&target_source.record)
    {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": "Framework pack diff requires customer-owned no-claim packs",
                "code": "framework_pack_diff_invariant_failed"
            }),
        ));
    }

    let base_original_framework_id = original_customer_framework_id(&base_source.raw_pack_redacted);
    let target_original_framework_id =
        original_customer_framework_id(&target_source.raw_pack_redacted);
    if base_original_framework_id.is_empty()
        || base_original_framework_id != target_original_framework_id
    {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": "Framework pack diff requires two versions of the same customer framework id",
                "code": "framework_pack_framework_mismatch",
                "base_original_framework_id": base_original_framework_id,
                "target_original_framework_id": target_original_framework_id
            }),
        ));
    }

    let base_controls = framework_pack_controls_by_id(&base_source.raw_pack_redacted);
    let target_controls = framework_pack_controls_by_id(&target_source.raw_pack_redacted);
    let mut control_ids = base_controls
        .keys()
        .chain(target_controls.keys())
        .cloned()
        .collect::<Vec<_>>();
    control_ids.sort();
    control_ids.dedup();

    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;
    let mut unchanged = 0usize;
    let mut controls = Vec::with_capacity(control_ids.len());
    for control_id in control_ids {
        let base = base_controls.get(&control_id).cloned();
        let target = target_controls.get(&control_id).cloned();
        let (change_type, changed_fields) = match (base.as_ref(), target.as_ref()) {
            (None, Some(_)) => {
                added += 1;
                ("added".to_string(), Vec::new())
            }
            (Some(_), None) => {
                removed += 1;
                ("removed".to_string(), Vec::new())
            }
            (Some(base), Some(target)) => {
                let changed_fields = framework_pack_changed_fields(base, target);
                if changed_fields.is_empty() {
                    unchanged += 1;
                    ("unchanged".to_string(), changed_fields)
                } else {
                    changed += 1;
                    ("changed".to_string(), changed_fields)
                }
            }
            (None, None) => continue,
        };
        controls.push(ComplianceFrameworkPackDiffControl {
            control_id,
            change_type,
            base,
            target,
            changed_fields,
        });
    }

    Ok(ComplianceFrameworkPackDiffResponse {
        base_pack: base_source.record,
        target_pack: target_source.record,
        original_framework_id: base_original_framework_id,
        same_original_framework: true,
        summary: ComplianceFrameworkPackDiffSummary {
            added,
            removed,
            changed,
            unchanged,
        },
        controls,
        compliance_claim: false,
        regulatory_claim: false,
        gitgov_certifies: false,
        official_regulatory_mapping: false,
        requires_auditor_review: true,
    })
}

fn customer_framework_review_block(
    framework: &ComplianceControlFramework,
) -> Option<(&'static str, &'static str)> {
    if framework.owner_type != "customer" && framework.framework_pack_id.is_none() {
        return None;
    }
    let status = framework
        .framework_pack_review_status
        .as_deref()
        .unwrap_or(FRAMEWORK_PACK_REVIEW_NEEDS_REVIEW);
    match status {
        FRAMEWORK_PACK_REVIEW_REVIEWED => None,
        FRAMEWORK_PACK_REVIEW_NEEDS_CHANGES => Some((
            "framework_pack_needs_changes",
            "Customer framework pack requires changes before evidence mapping",
        )),
        FRAMEWORK_PACK_REVIEW_REJECTED => Some((
            "framework_pack_rejected",
            "Customer framework pack was rejected and cannot be used for evidence mapping",
        )),
        FRAMEWORK_PACK_REVIEW_ARCHIVED => Some((
            "framework_pack_archived",
            "Customer framework pack is archived and cannot be used for evidence mapping",
        )),
        _ => Some((
            "framework_pack_not_reviewed",
            "Customer framework pack must be reviewed before evidence mapping",
        )),
    }
}

fn customer_framework_review_block_response(
    framework: &ComplianceControlFramework,
) -> Option<axum::response::Response> {
    customer_framework_review_block(framework).map(|(code, message)| {
        (
            StatusCode::CONFLICT,
            Json(json!({
                "error": message,
                "code": code,
                "framework_id": framework.framework_id,
                "framework_pack_id": framework.framework_pack_id,
                "review_status": framework.framework_pack_review_status
            })),
        )
            .into_response()
    })
}

fn json_str_field<'a>(value: &'a serde_json::Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(|field| field.as_str())
}

fn json_bool_field(value: &serde_json::Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(|field| field.as_bool())
        .unwrap_or(false)
}

fn validate_customer_pack_no_secret_fields(
    value: &serde_json::Value,
    path: &str,
    errors: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let lowered = key.to_ascii_lowercase();
                if lowered.contains("token")
                    || lowered.contains("secret")
                    || lowered.contains("password")
                    || lowered.contains("api_key")
                    || lowered.contains("authorization")
                {
                    errors.push(format!(
                        "pack contains secret-like metadata key at {path}/{key}; remove secrets before import"
                    ));
                }
                validate_customer_pack_no_secret_fields(child, &format!("{path}/{key}"), errors);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                validate_customer_pack_no_secret_fields(child, &format!("{path}/{index}"), errors);
            }
        }
        serde_json::Value::String(text) => {
            let lowered = text.to_ascii_lowercase();
            if lowered.contains("bearer ")
                || lowered.contains("ghp_")
                || lowered.contains("glpat-")
            {
                errors.push(format!(
                    "pack contains secret-like metadata value at {path}; remove secrets before import"
                ));
            }
            if lowered.contains("<script")
                || lowered.contains("</")
                || lowered.contains("<iframe")
            {
                errors.push(format!(
                    "pack contains HTML/script-like content at {path}; use plain text only"
                ));
            }
        }
        _ => {}
    }
}

fn parse_customer_framework_pack(
    payload: &mut ComplianceFrameworkPackImportRequest,
) -> Result<serde_json::Value, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    let format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "json".to_string());
    payload.format = Some(format.clone());

    let pack = match format.as_str() {
        "json" => {
            if let Some(pack) = payload.pack.take() {
                Some(pack)
            } else if let Some(content) = payload.content.as_deref() {
                match serde_json::from_str::<serde_json::Value>(content) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        errors.push(format!("content is not valid JSON: {e}"));
                        None
                    }
                }
            } else {
                errors.push("pack or content is required for json imports.".to_string());
                None
            }
        }
        "yaml" | "yml" => {
            if payload.pack.is_some() {
                errors.push("yaml imports must provide content, not pack.".to_string());
            }
            match payload.content.as_deref() {
                Some(content) => match serde_yaml::from_str::<serde_json::Value>(content) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        errors.push(format!("content is not valid YAML: {e}"));
                        None
                    }
                },
                None => {
                    errors.push("content is required for yaml imports.".to_string());
                    None
                }
            }
        }
        _ => {
            errors.push("format must be json, yaml, or yml.".to_string());
            None
        }
    };

    match (pack, errors.is_empty()) {
        (Some(pack), true) => Ok(pack),
        _ => Err(errors),
    }
}

fn normalize_customer_control_id(control_id: &str) -> String {
    control_id.trim().to_ascii_uppercase()
}

fn validate_customer_framework_pack(
    org_id: &str,
    pack: &serde_json::Value,
) -> Result<CreateComplianceFrameworkPackInput, Vec<String>> {
    let mut errors = Vec::new();
    validate_customer_pack_no_secret_fields(pack, "", &mut errors);

    let schema_version = json_str_field(pack, "/schema_version")
        .unwrap_or(CUSTOMER_FRAMEWORK_PACK_SCHEMA_VERSION)
        .trim()
        .to_string();
    if schema_version != CUSTOMER_FRAMEWORK_PACK_SCHEMA_VERSION {
        errors.push(format!(
            "schema_version must be {CUSTOMER_FRAMEWORK_PACK_SCHEMA_VERSION}."
        ));
    }

    for pointer in [
        "/compliance_claim",
        "/regulatory_claim",
        "/gitgov_certifies",
        "/official_regulatory_mapping",
        "/framework/compliance_claim",
        "/framework/regulatory_claim",
        "/framework/gitgov_certifies",
        "/framework/official_regulatory_mapping",
    ] {
        if json_bool_field(pack, pointer) {
            errors.push(format!(
                "{pointer} cannot be true for customer-owned framework packs"
            ));
        }
    }

    let original_framework_id = json_str_field(pack, "/framework/id")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if original_framework_id.is_empty() || original_framework_id.len() > 64 {
        errors.push("framework.id is required and must be 64 characters or less.".to_string());
    }
    if RESERVED_CUSTOMER_FRAMEWORK_PREFIXES
        .iter()
        .any(|prefix| original_framework_id.starts_with(prefix))
    {
        errors.push("framework.id uses a reserved official/GitGov prefix.".to_string());
    }

    let framework_name = json_str_field(pack, "/framework/name")
        .unwrap_or_default()
        .trim()
        .to_string();
    if framework_name.is_empty() || framework_name.len() > 256 {
        errors.push("framework.name is required and must be 256 characters or less.".to_string());
    }

    let framework_version = json_str_field(pack, "/framework/version")
        .unwrap_or_default()
        .trim()
        .to_string();
    if framework_version.is_empty() || framework_version.len() > 64 {
        errors.push("framework.version is required and must be 64 characters or less.".to_string());
    }

    let description = json_str_field(pack, "/framework/description")
        .unwrap_or("Customer-provided control framework pack for evidence review.")
        .trim()
        .to_string();
    if description.len() > 2000 {
        errors.push("framework.description must be 2000 characters or less.".to_string());
    }

    let owner_name = json_str_field(pack, "/framework/owner_name")
        .or_else(|| json_str_field(pack, "/owner_name"))
        .unwrap_or("Customer")
        .trim()
        .to_string();
    if owner_name.is_empty() || owner_name.len() > 128 {
        errors.push("framework.owner_name must be 128 characters or less.".to_string());
    }

    let controls_value = pack.pointer("/controls").and_then(|value| value.as_array());
    let Some(controls_value) = controls_value else {
        errors.push("controls must be a non-empty array.".to_string());
        return Err(errors);
    };
    if controls_value.is_empty() {
        errors.push("controls must be a non-empty array.".to_string());
    }
    if controls_value.len() > MAX_CUSTOMER_FRAMEWORK_CONTROLS {
        errors.push(format!(
            "controls cannot exceed {MAX_CUSTOMER_FRAMEWORK_CONTROLS} items."
        ));
    }

    let mut seen_control_ids = HashSet::new();
    let mut controls = Vec::new();
    for (index, control) in controls_value.iter().enumerate() {
        let control_id =
            normalize_customer_control_id(json_str_field(control, "/control_id").unwrap_or_default());
        if control_id.is_empty()
            || control_id.len() > 64
            || !control_id.chars().all(|ch| {
                ch.is_ascii_uppercase()
                    || ch.is_ascii_digit()
                    || matches!(ch, '-' | '_' | '.' | ':')
            })
        {
            errors.push(format!(
                "controls[{index}].control_id must be 1-64 safe uppercase characters."
            ));
        } else if !seen_control_ids.insert(control_id.clone()) {
            errors.push(format!("duplicate control_id: {control_id}"));
        }

        let title = json_str_field(control, "/title")
            .unwrap_or_default()
            .trim()
            .to_string();
        if title.is_empty() || title.len() > 256 {
            errors.push(format!(
                "controls[{index}].title is required and must be 256 characters or less."
            ));
        }

        let description = json_str_field(control, "/description")
            .unwrap_or_default()
            .trim()
            .to_string();
        if description.is_empty() || description.len() > 2000 {
            errors.push(format!(
                "controls[{index}].description is required and must be 2000 characters or less."
            ));
        }

        let evidence_types = control
            .pointer("/required_evidence_types")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.trim().to_string()))
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if evidence_types.is_empty() {
            errors.push(format!(
                "controls[{index}].required_evidence_types must be non-empty."
            ));
        }
        if evidence_types.len() > MAX_CUSTOMER_CONTROL_EVIDENCE_TYPES {
            errors.push(format!(
                "controls[{index}].required_evidence_types cannot exceed {MAX_CUSTOMER_CONTROL_EVIDENCE_TYPES} items."
            ));
        }
        for evidence_type in &evidence_types {
            if !CUSTOMER_FRAMEWORK_ALLOWED_EVIDENCE_TYPES.contains(&evidence_type.as_str()) {
                errors.push(format!(
                    "controls[{index}] uses unsupported evidence type: {evidence_type}"
                ));
            }
        }

        controls.push(CreateComplianceFrameworkPackControlInput {
            control_row_id: format!("cctl_{}", Uuid::new_v4().simple()),
            control_id,
            title,
            description,
            required_evidence_types: evidence_types,
            sort_order: ((index + 1) * 10) as i32,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let pack_hash = customer_framework_pack_hash(pack);
    let slug = safe_pack_slug(&original_framework_id);
    let digest = format!(
        "{:x}",
        Sha256::digest(
            format!("{org_id}:{original_framework_id}:{framework_version}:{pack_hash}").as_bytes(),
        )
    );
    let framework_id = format!("customer_{slug}_{}", &digest[..12]);
    let framework_pack_id = format!("cfp_{}", &digest[..32]);

    Ok(CreateComplianceFrameworkPackInput {
        framework_pack_id,
        org_id: org_id.to_string(),
        framework_id,
        framework_name,
        framework_version,
        description,
        owner_name,
        schema_version,
        pack_hash,
        raw_pack_redacted: pack.clone(),
        created_by_user_id: String::new(),
        controls,
    })
}

async fn resolve_required_compliance_framework_org(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_name: Option<&str>,
) -> Result<String, axum::response::Response> {
    match resolve_and_check_org_scope(state, auth_user.org_id.as_deref(), org_name, true).await {
        Ok(Some(org_id)) => Ok(org_id),
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "org_name is required for global admin keys" })),
        )
            .into_response()),
        Err(err) => Err((
            org_scope_status(err),
            Json(json!({ "error": agent_governance_scope_error_message(err) })),
        )
            .into_response()),
    }
}

async fn resolve_optional_compliance_framework_org(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_name: Option<&str>,
) -> Result<Option<String>, axum::response::Response> {
    match resolve_and_check_org_scope(state, auth_user.org_id.as_deref(), org_name, true).await {
        Ok(org_id) => Ok(org_id),
        Err(err) => Err((
            org_scope_status(err),
            Json(json!({ "error": agent_governance_scope_error_message(err) })),
        )
            .into_response()),
    }
}

pub async fn import_compliance_framework_pack(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ComplianceFrameworkPackImportRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let pack = match parse_customer_framework_pack(&mut payload) {
        Ok(pack) => pack,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid customer framework pack import request", "details": errors })),
            )
                .into_response();
        }
    };

    let org_id = match resolve_required_compliance_framework_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    let mut input = match validate_customer_framework_pack(&org_id, &pack) {
        Ok(input) => input,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid customer framework pack", "details": errors })),
            )
                .into_response();
        }
    };
    input.created_by_user_id = auth_user.client_id.clone();

    match state.db.create_compliance_framework_pack(input).await {
        Ok(response) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_pack.imported".to_string(),
                target_type: Some("compliance_framework_pack".to_string()),
                target_id: Some(response.framework_pack.framework_pack_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "framework_pack_id": response.framework_pack.framework_pack_id,
                    "framework_id": response.framework.framework_id,
                    "framework_version": response.framework.version,
                    "owner_type": "customer",
                    "source": "customer_provided",
                    "control_count": response.framework.controls.len(),
                    "pack_hash": response.framework_pack.pack_hash,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "gitgov_certifies": false,
                    "official_regulatory_mapping": false,
                    "requires_auditor_review": true
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework pack import audit log: {}", e);
            }
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to import customer framework pack");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_framework_packs(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_required_compliance_framework_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state.db.list_compliance_framework_packs(&org_id).await {
        Ok(framework_packs) => (
            StatusCode::OK,
            Json(ComplianceFrameworkPackListResponse { framework_packs }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list compliance framework packs");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn diff_compliance_framework_packs(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkPackDiffQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let (base_pack_id, target_pack_id) = match normalize_framework_pack_diff_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid framework pack diff query",
                    "details": errors
                })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_required_compliance_framework_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    let base_source = match state
        .db
        .get_compliance_framework_pack_diff_source(&org_id, &base_pack_id)
        .await
    {
        Ok(Some(source)) => source,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Base compliance framework pack not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_pack_id = %base_pack_id, "Failed to load base framework pack for diff");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let target_source = match state
        .db
        .get_compliance_framework_pack_diff_source(&org_id, &target_pack_id)
        .await
    {
        Ok(Some(source)) => source,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Target compliance framework pack not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_pack_id = %target_pack_id, "Failed to load target framework pack for diff");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    match build_framework_pack_diff_response(base_source, target_source) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err((status, body)) => (status, Json(body)).into_response(),
    }
}

pub async fn get_compliance_framework_pack(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(framework_pack_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_required_compliance_framework_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_framework_pack(&org_id, framework_pack_id.trim())
        .await
    {
        Ok(Some(framework_pack)) => (StatusCode::OK, Json(framework_pack)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework pack not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_pack_id = %framework_pack_id, "Failed to load compliance framework pack");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn review_compliance_framework_pack(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(framework_pack_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
    Json(mut payload): Json<ComplianceFrameworkPackReviewRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    if payload.org_name.is_none() {
        payload.org_name = query.org_name;
    }
    if let Err(errors) = normalize_framework_pack_review_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid framework pack review request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_required_compliance_framework_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    let framework_pack_id = framework_pack_id.trim().to_string();
    let current_pack = match state
        .db
        .get_compliance_framework_pack(&org_id, &framework_pack_id)
        .await
    {
        Ok(Some(pack)) => pack,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance framework pack not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_pack_id = %framework_pack_id, "Failed to load compliance framework pack for review");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if payload.review_status == FRAMEWORK_PACK_REVIEW_REVIEWED {
        if let Err(errors) = validate_framework_pack_can_be_marked_reviewed(&current_pack) {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Framework pack cannot be marked reviewed",
                    "code": "framework_pack_review_invariant_failed",
                    "details": errors
                })),
            )
                .into_response();
        }
    }

    match state
        .db
        .review_compliance_framework_pack(ReviewComplianceFrameworkPackInput {
            org_id: org_id.clone(),
            framework_pack_id: framework_pack_id.clone(),
            review_status: payload.review_status.clone(),
            reviewed_by_user_id: auth_user.client_id.clone(),
            review_notes_safe: payload.review_notes_safe.clone(),
            rejected_reason_safe: payload.rejected_reason_safe.clone(),
        })
        .await
    {
        Ok(Some(framework_pack)) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_pack.review_updated".to_string(),
                target_type: Some("compliance_framework_pack".to_string()),
                target_id: Some(framework_pack.framework_pack_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "framework_pack_id": framework_pack.framework_pack_id,
                    "framework_id": framework_pack.framework_id,
                    "review_status": framework_pack.review_status,
                    "reviewed_by_user_id": framework_pack.reviewed_by_user_id,
                    "reviewed_at": framework_pack.reviewed_at,
                    "review_updated_at": framework_pack.review_updated_at,
                    "archived_at": framework_pack.archived_at,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "gitgov_certifies": false,
                    "official_regulatory_mapping": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework pack review audit log: {}", e);
            }
            (StatusCode::OK, Json(framework_pack)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework pack not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_pack_id = %framework_pack_id, "Failed to update compliance framework pack review");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_control_frameworks(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_optional_compliance_framework_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .list_compliance_control_frameworks(org_id.as_deref())
        .await
    {
        Ok(frameworks) => {
            (StatusCode::OK, Json(json!({ "frameworks": frameworks }))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to list compliance control frameworks");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_control_framework(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(framework_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let framework_id = framework_id.trim().to_ascii_lowercase();
    let org_id = match resolve_optional_compliance_framework_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_control_framework(org_id.as_deref(), &framework_id)
        .await
    {
        Ok(Some(framework)) => (StatusCode::OK, Json(framework)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance control framework not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, framework_id = %framework_id, "Failed to load compliance control framework");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
