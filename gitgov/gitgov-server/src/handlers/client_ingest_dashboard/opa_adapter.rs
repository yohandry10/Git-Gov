struct OpaPolicyCheckContext<'a> {
    config: &'a GitGovConfig,
    source: &'a PolicySourceMetadata,
    repo: &'a Repo,
    repo_name: &'a str,
    branch: &'a str,
    commit_sha: Option<&'a str>,
    user_login: Option<&'a str>,
    auth_user: &'a AuthUser,
    native_response: &'a PolicyCheckResponse,
}

async fn evaluate_opa_policy_check(
    http_client: &reqwest::Client,
    ctx: OpaPolicyCheckContext<'_>,
) -> Option<ExternalPolicyDecision> {
    let opa = &ctx.config.adapters.opa;
    if !opa.enabled {
        return None;
    }

    let started = Instant::now();
    let input = json!({
        "input": {
            "gitgov": {
                "profile": &opa.input_profile,
                "repo": ctx.repo_name,
                "repo_id": &ctx.repo.id,
                "org_id": &ctx.repo.org_id,
                "branch": ctx.branch,
                "commit": ctx.commit_sha,
                "actor": ctx.user_login.unwrap_or(ctx.auth_user.client_id.as_str()),
                "policy_source": ctx.source,
                "native": ctx.native_response
            }
        }
    });
    let input_hash = Some(sha256_json_value(&input));
    let decision_path = opa.decision_path.trim().to_string();
    let enforcement = external_policy_enforcement(ctx.config);

    let base_url = match resolve_opa_base_url(opa) {
        Ok(value) => value,
        Err(error) => {
            return Some(failed_opa_decision(
                opa,
                decision_path,
                enforcement,
                started,
                input_hash,
                error,
            ));
        }
    };
    let url = join_opa_url(&base_url, &decision_path);
    let mut request = http_client.post(url).json(&input);

    if let Some(token_env_var) = opa.token_env_var.as_deref() {
        let token_env_var = match validated_opa_token_env_var(token_env_var) {
            Ok(value) => value,
            Err(error) => {
                return Some(failed_opa_decision(
                    opa,
                    decision_path,
                    enforcement,
                    started,
                    input_hash,
                    error,
                ));
            }
        };
        match std::env::var(&token_env_var) {
            Ok(token) if !token.trim().is_empty() => {
                request = request.bearer_auth(token);
            }
            _ => {
                return Some(failed_opa_decision(
                    opa,
                    decision_path,
                    enforcement,
                    started,
                    input_hash,
                    "OPA token env var is not configured".to_string(),
                ));
            }
        }
    }

    let send_result = timeout(Duration::from_millis(opa.timeout_ms), request.send()).await;
    let response = match send_result {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            return Some(failed_opa_decision(
                opa,
                decision_path,
                enforcement,
                started,
                input_hash,
                format!("OPA request failed: {}", error),
            ));
        }
        Err(_) => {
            return Some(failed_opa_decision(
                opa,
                decision_path,
                enforcement,
                started,
                input_hash,
                format!("OPA request timed out after {}ms", opa.timeout_ms),
            ));
        }
    };

    let status = response.status();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => {
            return Some(failed_opa_decision(
                opa,
                decision_path,
                enforcement,
                started,
                input_hash,
                format!("OPA response read failed: {}", error),
            ));
        }
    };
    let output_hash = Some(sha256_text(&body));

    if !status.is_success() {
        return Some(failed_opa_decision_with_output(
            opa,
            decision_path,
            enforcement,
            started,
            input_hash,
            output_hash,
            format!("OPA returned HTTP {}", status),
        ));
    }

    let body_json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(error) => {
            return Some(failed_opa_decision_with_output(
                opa,
                decision_path,
                enforcement,
                started,
                input_hash,
                output_hash,
                format!("OPA response JSON decode failed: {}", error),
            ));
        }
    };

    let decision = opa_success_decision(
        opa,
        decision_path,
        enforcement,
        started,
        input_hash,
        output_hash,
        &body_json,
    );
    if decision.allowed.is_none() {
        return Some(failed_opa_decision_with_output(
            opa,
            decision.decision_path,
            decision.enforcement,
            started,
            decision.input_hash,
            decision.output_hash,
            "OPA response did not include a mapped boolean decision".to_string(),
        ));
    }

    Some(decision)
}

fn merge_opa_policy_decision(
    config: &GitGovConfig,
    response: &mut PolicyCheckResponse,
    decision: ExternalPolicyDecision,
) {
    response
        .evaluated_rules
        .push("external_policy_opa".to_string());

    let message = external_decision_message(&decision);
    if config.adapters.opa.effect == ExternalPolicyEffect::Advisory && decision.allowed == Some(false)
    {
        response.warnings.push(message);
        response.external_decisions.push(decision);
        return;
    }

    match config.enforcement.external_policy {
        EnforcementLevel::Block
            if config.adapters.opa.effect == ExternalPolicyEffect::Required
                && decision.allowed == Some(false) =>
        {
            response.allowed = false;
            response.reasons.push(message.clone());
            response.violations.push(RuleViolation {
                rule: "external_policy_opa".to_string(),
                category: "external_policy".to_string(),
                enforcement: "block".to_string(),
                message,
            });
        }
        EnforcementLevel::Warn if decision.allowed == Some(false) => {
            response.warnings.push(message.clone());
            response.violations.push(RuleViolation {
                rule: "external_policy_opa".to_string(),
                category: "external_policy".to_string(),
                enforcement: "warn".to_string(),
                message,
            });
        }
        EnforcementLevel::Off if decision.allowed == Some(false) => {
            response
                .warnings
                .push(format!("OPA external policy denied but external_policy enforcement is off: {}", message));
        }
        _ => {
            response.warnings.extend(decision.warnings.iter().cloned());
        }
    }

    response.external_decisions.push(decision);
}

fn opa_success_decision(
    opa: &OpaAdapterConfig,
    decision_path: String,
    enforcement: String,
    started: Instant,
    input_hash: Option<String>,
    output_hash: Option<String>,
    body: &serde_json::Value,
) -> ExternalPolicyDecision {
    let result = body.get("result").unwrap_or(&serde_json::Value::Null);
    let allowed = extract_opa_allowed(result, &opa.result_mapping);
    let mut reasons = extract_string_list(result, &opa.result_mapping.reasons_key);
    let mut warnings = extract_string_list(result, &opa.result_mapping.warnings_key);
    let deny_reasons = extract_opa_deny_reasons(result);

    if let Some(message) = extract_string_value(result, &opa.result_mapping.message_key) {
        if allowed == Some(false) {
            reasons.push(message);
        } else {
            warnings.push(message);
        }
    }

    if reasons.is_empty() && allowed == Some(false) && !deny_reasons.is_empty() {
        reasons.extend(deny_reasons);
    }

    if reasons.is_empty() && allowed == Some(false) {
        reasons.push("OPA external policy denied the action".to_string());
    }

    ExternalPolicyDecision {
        adapter: "opa".to_string(),
        status: match allowed {
            Some(true) => "allowed".to_string(),
            Some(false) => "denied".to_string(),
            None => "unknown".to_string(),
        },
        allowed,
        enforcement,
        decision_id: extract_string_value(body, "decision_id")
            .or_else(|| extract_string_value(result, &opa.result_mapping.decision_id_key)),
        decision_path,
        latency_ms: started.elapsed().as_millis() as u64,
        input_hash,
        output_hash,
        reasons,
        warnings,
        error: None,
    }
}

fn failed_opa_decision(
    opa: &OpaAdapterConfig,
    decision_path: String,
    enforcement: String,
    started: Instant,
    input_hash: Option<String>,
    error: String,
) -> ExternalPolicyDecision {
    failed_opa_decision_with_output(
        opa,
        decision_path,
        enforcement,
        started,
        input_hash,
        None,
        error,
    )
}

fn failed_opa_decision_with_output(
    opa: &OpaAdapterConfig,
    decision_path: String,
    enforcement: String,
    started: Instant,
    input_hash: Option<String>,
    output_hash: Option<String>,
    error: String,
) -> ExternalPolicyDecision {
    let fail_closed = opa.failure_mode == ExternalPolicyFailureMode::FailClosed;
    ExternalPolicyDecision {
        adapter: "opa".to_string(),
        status: if fail_closed {
            "error-fail-closed".to_string()
        } else {
            "error-fail-open".to_string()
        },
        allowed: Some(!fail_closed),
        enforcement,
        decision_id: None,
        decision_path,
        latency_ms: started.elapsed().as_millis() as u64,
        input_hash,
        output_hash,
        reasons: if fail_closed {
            vec![format!("OPA external policy failed closed: {}", error)]
        } else {
            vec![]
        },
        warnings: if fail_closed {
            vec![]
        } else {
            vec![format!("OPA external policy failed open: {}", error)]
        },
        error: Some(error),
    }
}

fn external_policy_enforcement(config: &GitGovConfig) -> String {
    if config.adapters.opa.effect == ExternalPolicyEffect::Advisory {
        return "advisory".to_string();
    }
    format!("{:?}", effective_external_policy_enforcement(config)).to_lowercase()
}

fn effective_external_policy_enforcement(config: &GitGovConfig) -> EnforcementLevel {
    if config.adapters.opa.enabled && config.adapters.opa.effect == ExternalPolicyEffect::Required {
        config.enforcement.external_policy.clone()
    } else {
        EnforcementLevel::Off
    }
}

fn external_decision_message(decision: &ExternalPolicyDecision) -> String {
    decision
        .reasons
        .first()
        .or_else(|| decision.warnings.first())
        .cloned()
        .or_else(|| decision.error.clone())
        .unwrap_or_else(|| "OPA external policy denied the action".to_string())
}

fn resolve_opa_base_url(config: &OpaAdapterConfig) -> Result<String, String> {
    if let Some(base_url) = config
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return validated_opa_base_url(base_url);
    }

    if let Some(connection) = config
        .connection
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let env_key = format!("GITGOV_OPA_{}_URL", env_fragment(connection));
        if let Ok(value) = std::env::var(&env_key) {
            if !value.trim().is_empty() {
                return validated_opa_base_url(&value);
            }
        }
    }

    let value = std::env::var("GITGOV_OPA_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "OPA base URL is not configured; set adapters.opa.base_url or GITGOV_OPA_URL"
                .to_string()
        })?;
    validated_opa_base_url(&value)
}

fn validated_opa_base_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    gitgov_policy_core::validate_opa_base_url(trimmed).map_err(|err| err.to_string())?;
    Ok(trimmed.to_string())
}

fn validated_opa_token_env_var(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    gitgov_policy_core::validate_opa_token_env_var_name(trimmed)
        .map_err(|_| "OPA token env var name is invalid".to_string())?;
    Ok(trimmed.to_string())
}

fn env_fragment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn join_opa_url(base_url: &str, decision_path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        decision_path.trim_start_matches('/')
    )
}

fn extract_opa_allowed(result: &serde_json::Value, mapping: &OpaResultMapping) -> Option<bool> {
    if let Some(value) = result.as_bool() {
        return Some(value);
    }
    result
        .get(&mapping.allowed_key)
        .or_else(|| result.get("allowed"))
        .and_then(|value| value.as_bool())
        .or_else(|| result.get("deny").and_then(opa_deny_value_to_allowed))
}

fn opa_deny_value_to_allowed(value: &serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(denied) => Some(!denied),
        serde_json::Value::Array(items) => Some(items.is_empty()),
        serde_json::Value::Object(items) => Some(items.is_empty()),
        serde_json::Value::String(item) => Some(item.trim().is_empty()),
        _ => None,
    }
}

fn extract_opa_deny_reasons(result: &serde_json::Value) -> Vec<String> {
    match result.get("deny") {
        Some(serde_json::Value::String(item)) => vec![item.trim().to_string()]
            .into_iter()
            .filter(|item| !item.is_empty())
            .collect(),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        Some(serde_json::Value::Object(items)) => items
            .iter()
            .filter_map(|(key, value)| match value {
                serde_json::Value::Bool(true) => Some(key.trim().to_string()),
                serde_json::Value::String(message) => Some(message.trim().to_string()),
                _ => None,
            })
            .filter(|item| !item.is_empty())
            .collect(),
        _ => vec![],
    }
}

fn extract_string_value(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
}

fn extract_string_list(value: &serde_json::Value, key: &str) -> Vec<String> {
    match value.get(key) {
        Some(serde_json::Value::String(item)) => vec![item.trim().to_string()]
            .into_iter()
            .filter(|item| !item.is_empty())
            .collect(),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        _ => vec![],
    }
}

fn sha256_json_value(value: &serde_json::Value) -> String {
    sha256_text(&serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()))
}

fn sha256_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[cfg(test)]
mod opa_adapter_tests {
    use super::*;
    use axum::{routing::post, Json, Router};
    use tokio::net::TcpListener;

    async fn start_mock_opa(result: serde_json::Value) -> String {
        let app = Router::new().route(
            "/v1/data/gitgov/allow",
            post(move |Json(payload): Json<serde_json::Value>| {
                let result = result.clone();
                async move {
                    assert_eq!(payload["input"]["gitgov"]["profile"], "policy-check-v1");
                    assert_eq!(payload["input"]["gitgov"]["repo"], "acme/repo");
                    assert_eq!(payload["input"]["gitgov"]["branch"], "main");
                    assert_eq!(payload["input"]["gitgov"]["commit"], "abc123");
                    assert_eq!(payload["input"]["gitgov"]["actor"], "octo");
                    Json(result)
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{}", addr)
    }

    #[test]
    fn opa_success_decision_maps_object_result() {
        let mut opa = OpaAdapterConfig::default();
        opa.enabled = true;
        let body = serde_json::json!({
            "decision_id": "decision-123",
            "result": {
                "allow": false,
                "reasons": ["ticket approval missing"],
                "warnings": ["evaluated by OPA"]
            }
        });

        let decision = opa_success_decision(
            &opa,
            "/v1/data/gitgov/allow".to_string(),
            "block".to_string(),
            Instant::now(),
            Some("input-hash".to_string()),
            Some("output-hash".to_string()),
            &body,
        );

        assert_eq!(decision.status, "denied");
        assert_eq!(decision.allowed, Some(false));
        assert_eq!(decision.decision_id.as_deref(), Some("decision-123"));
        assert_eq!(decision.reasons, vec!["ticket approval missing"]);
        assert_eq!(decision.warnings, vec!["evaluated by OPA"]);
    }

    #[test]
    fn opa_success_decision_maps_common_rego_deny_collections() {
        let mut opa = OpaAdapterConfig::default();
        opa.enabled = true;
        let body = serde_json::json!({
            "result": {
                "deny": ["ticket missing", "freeze active"]
            }
        });

        let decision = opa_success_decision(
            &opa,
            "/v1/data/gitgov/allow".to_string(),
            "block".to_string(),
            Instant::now(),
            Some("input-hash".to_string()),
            Some("output-hash".to_string()),
            &body,
        );

        assert_eq!(decision.status, "denied");
        assert_eq!(decision.allowed, Some(false));
        assert_eq!(decision.reasons, vec!["ticket missing", "freeze active"]);

        let body = serde_json::json!({
            "result": {
                "deny": []
            }
        });
        let decision = opa_success_decision(
            &opa,
            "/v1/data/gitgov/allow".to_string(),
            "block".to_string(),
            Instant::now(),
            Some("input-hash".to_string()),
            Some("output-hash".to_string()),
            &body,
        );
        assert_eq!(decision.status, "allowed");
        assert_eq!(decision.allowed, Some(true));
    }

    #[test]
    fn required_opa_denial_blocks_when_external_policy_blocks() {
        let mut config = GitGovConfig::default();
        config.adapters.opa.enabled = true;
        config.adapters.opa.effect = ExternalPolicyEffect::Required;
        config.enforcement.external_policy = EnforcementLevel::Block;
        let mut response = PolicyCheckResponse {
            advisory: false,
            allowed: true,
            ..Default::default()
        };
        let decision = ExternalPolicyDecision {
            adapter: "opa".to_string(),
            status: "denied".to_string(),
            allowed: Some(false),
            enforcement: "block".to_string(),
            decision_path: "/v1/data/gitgov/allow".to_string(),
            reasons: vec!["release freeze active".to_string()],
            ..Default::default()
        };

        merge_opa_policy_decision(&config, &mut response, decision);

        assert!(!response.allowed);
        assert_eq!(response.external_decisions.len(), 1);
        assert_eq!(response.violations[0].category, "external_policy");
        assert!(response.reasons[0].contains("release freeze active"));
    }

    #[test]
    fn advisory_opa_denial_does_not_block_native_response() {
        let mut config = GitGovConfig::default();
        config.adapters.opa.enabled = true;
        config.adapters.opa.effect = ExternalPolicyEffect::Advisory;
        config.enforcement.external_policy = EnforcementLevel::Block;
        let mut response = PolicyCheckResponse {
            advisory: false,
            allowed: true,
            ..Default::default()
        };
        let decision = ExternalPolicyDecision {
            adapter: "opa".to_string(),
            status: "denied".to_string(),
            allowed: Some(false),
            enforcement: "advisory".to_string(),
            decision_path: "/v1/data/gitgov/allow".to_string(),
            reasons: vec!["advisory finding".to_string()],
            ..Default::default()
        };

        merge_opa_policy_decision(&config, &mut response, decision);

        assert!(response.allowed);
        assert_eq!(response.external_decisions.len(), 1);
        assert!(response
            .warnings
            .iter()
            .any(|item| item.contains("advisory finding")));
    }

    #[test]
    fn effective_external_enforcement_requires_active_required_opa() {
        let mut config = GitGovConfig::default();
        config.enforcement.external_policy = EnforcementLevel::Block;

        assert_eq!(
            effective_external_policy_enforcement(&config),
            EnforcementLevel::Off
        );

        config.adapters.opa.enabled = true;
        config.adapters.opa.effect = ExternalPolicyEffect::Advisory;
        assert_eq!(
            effective_external_policy_enforcement(&config),
            EnforcementLevel::Off
        );

        config.adapters.opa.effect = ExternalPolicyEffect::Required;
        assert_eq!(
            effective_external_policy_enforcement(&config),
            EnforcementLevel::Block
        );
    }

    #[test]
    fn opa_runtime_env_url_uses_same_safe_url_validation() {
        let mut opa = OpaAdapterConfig {
            connection: Some("unsafe_http_host_test".to_string()),
            ..Default::default()
        };
        std::env::set_var(
            "GITGOV_OPA_UNSAFE_HTTP_HOST_TEST_URL",
            "http://localhost.example.com:8181",
        );

        let error = resolve_opa_base_url(&opa).unwrap_err();

        std::env::remove_var("GITGOV_OPA_UNSAFE_HTTP_HOST_TEST_URL");
        assert!(error.contains("localhost/loopback"), "unexpected error: {}", error);

        opa.base_url = Some("http://127.0.0.1:8181".to_string());
        assert_eq!(
            resolve_opa_base_url(&opa).unwrap(),
            "http://127.0.0.1:8181"
        );
    }

    #[test]
    fn opa_undefined_or_unmapped_result_uses_failure_mode() {
        let mut opa = OpaAdapterConfig {
            enabled: true,
            effect: ExternalPolicyEffect::Required,
            failure_mode: ExternalPolicyFailureMode::FailClosed,
            ..Default::default()
        };
        let body = serde_json::json!({});
        let success = opa_success_decision(
            &opa,
            "/v1/data/gitgov/missing".to_string(),
            "block".to_string(),
            Instant::now(),
            Some("input-hash".to_string()),
            Some("output-hash".to_string()),
            &body,
        );
        assert_eq!(success.status, "unknown");
        assert_eq!(success.allowed, None);

        let decision = failed_opa_decision_with_output(
            &opa,
            success.decision_path,
            success.enforcement,
            Instant::now(),
            success.input_hash,
            success.output_hash,
            "OPA response did not include a mapped boolean decision".to_string(),
        );

        assert_eq!(decision.status, "error-fail-closed");
        assert_eq!(decision.allowed, Some(false));
        assert!(decision.reasons[0].contains("failed closed"));

        opa.failure_mode = ExternalPolicyFailureMode::FailOpen;
        let decision = failed_opa_decision_with_output(
            &opa,
            "/v1/data/gitgov/missing".to_string(),
            "block".to_string(),
            Instant::now(),
            Some("input-hash".to_string()),
            Some("output-hash".to_string()),
            "OPA response did not include a mapped boolean decision".to_string(),
        );
        assert_eq!(decision.status, "error-fail-open");
        assert_eq!(decision.allowed, Some(true));
        assert!(decision.warnings[0].contains("failed open"));
    }

    #[test]
    fn opa_invalid_token_env_var_is_rejected_without_echoing_value() {
        let error = validated_opa_token_env_var("ghp_secret_value_should_not_echo").unwrap_err();

        assert_eq!(error, "OPA token env var name is invalid");
        assert!(!error.contains("ghp_secret"));
    }

    #[tokio::test]
    async fn opa_http_adapter_evaluates_data_api_response() {
        let base_url = start_mock_opa(serde_json::json!({
            "decision_id": "opa-test-decision",
            "result": {
                "allow": false,
                "reasons": ["change window is closed"],
                "warnings": ["evaluated with customer Rego"]
            }
        }))
        .await;
        let mut config = GitGovConfig::default();
        config.enforcement.external_policy = EnforcementLevel::Block;
        config.adapters.opa.enabled = true;
        config.adapters.opa.base_url = Some(base_url);
        config.adapters.opa.effect = ExternalPolicyEffect::Required;

        let repo = Repo {
            id: "repo-id".to_string(),
            org_id: Some("org-id".to_string()),
            github_id: Some(42),
            full_name: "acme/repo".to_string(),
            name: "repo".to_string(),
            private: true,
            created_at: 1_700_000_000,
        };
        let auth_user = AuthUser {
            client_id: "fallback-user".to_string(),
            role: UserRole::Admin,
            org_id: Some("org-id".to_string()),
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        };
        let source = PolicySourceMetadata::control_plane_managed("policy-admin", "checksum");
        let native_response = PolicyCheckResponse {
            allowed: true,
            enforcement_applied: "none".to_string(),
            ..Default::default()
        };

        let decision = evaluate_opa_policy_check(
            &reqwest::Client::new(),
            OpaPolicyCheckContext {
                config: &config,
                source: &source,
                repo: &repo,
                repo_name: "acme/repo",
                branch: "main",
                commit_sha: Some("abc123"),
                user_login: Some("octo"),
                auth_user: &auth_user,
                native_response: &native_response,
            },
        )
        .await
        .expect("OPA adapter should return a decision");

        assert_eq!(decision.adapter, "opa");
        assert_eq!(decision.status, "denied");
        assert_eq!(decision.allowed, Some(false));
        assert_eq!(decision.enforcement, "block");
        assert_eq!(decision.decision_id.as_deref(), Some("opa-test-decision"));
        assert_eq!(decision.reasons, vec!["change window is closed"]);
        assert_eq!(decision.warnings, vec!["evaluated with customer Rego"]);
        assert!(decision.input_hash.is_some());
        assert!(decision.output_hash.is_some());
    }

    #[tokio::test]
    async fn opa_http_adapter_fails_closed_when_data_api_result_is_undefined() {
        let base_url = start_mock_opa(serde_json::json!({})).await;
        let mut config = GitGovConfig::default();
        config.enforcement.external_policy = EnforcementLevel::Block;
        config.adapters.opa.enabled = true;
        config.adapters.opa.base_url = Some(base_url);
        config.adapters.opa.effect = ExternalPolicyEffect::Required;
        config.adapters.opa.failure_mode = ExternalPolicyFailureMode::FailClosed;

        let repo = Repo {
            id: "repo-id".to_string(),
            org_id: Some("org-id".to_string()),
            github_id: Some(42),
            full_name: "acme/repo".to_string(),
            name: "repo".to_string(),
            private: true,
            created_at: 1_700_000_000,
        };
        let auth_user = AuthUser {
            client_id: "fallback-user".to_string(),
            role: UserRole::Admin,
            org_id: Some("org-id".to_string()),
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        };
        let source = PolicySourceMetadata::control_plane_managed("policy-admin", "checksum");
        let native_response = PolicyCheckResponse {
            allowed: true,
            enforcement_applied: "none".to_string(),
            ..Default::default()
        };

        let decision = evaluate_opa_policy_check(
            &reqwest::Client::new(),
            OpaPolicyCheckContext {
                config: &config,
                source: &source,
                repo: &repo,
                repo_name: "acme/repo",
                branch: "main",
                commit_sha: Some("abc123"),
                user_login: Some("octo"),
                auth_user: &auth_user,
                native_response: &native_response,
            },
        )
        .await
        .expect("OPA adapter should return a decision");

        assert_eq!(decision.status, "error-fail-closed");
        assert_eq!(decision.allowed, Some(false));
        assert_eq!(decision.enforcement, "block");
        assert!(decision
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("mapped boolean decision"));
        assert!(decision.output_hash.is_some());
    }
}
