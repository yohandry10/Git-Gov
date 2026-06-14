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

pub async fn list_compliance_control_frameworks(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkPackQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
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
    if let Err(resp) = require_admin(&auth_user) {
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
