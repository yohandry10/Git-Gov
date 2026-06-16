fn first_setup_action_to_upsert(
    payload: FirstGovernedRepoWizardActionRequest,
) -> UpsertFirstGovernedRepoSetupRequest {
    UpsertFirstGovernedRepoSetupRequest {
        org_name: payload.org_name,
        status: payload.status,
        goal: payload.goal,
        repository_full_name: payload.repository_full_name,
        default_branch: payload.default_branch,
        selected_providers: payload.selected_providers,
        selected_modules: payload.selected_modules,
        policy_preset: payload.policy_preset,
        baseline: payload.baseline,
    }
}

fn first_setup_validate_run_id(run_id: &str) -> Result<String, &'static str> {
    Uuid::parse_str(run_id)
        .map(|uuid| uuid.to_string())
        .map_err(|_| "run_id must be a valid UUID")
}

async fn resolve_first_setup_org(
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
            Json(json!({ "error": first_governed_repo_scope_error_message(err) })),
        )
            .into_response()),
    }
}

fn first_setup_readiness_from_provider(
    provider: &str,
    setup: &FirstGovernedRepoSetupRecord,
    stats: &AuditStats,
) -> serde_json::Value {
    let repo_ready = first_setup_repo_format_valid(&setup.repository_full_name);
    match provider {
        "github" => {
            let status = if !repo_ready {
                "needs-config"
            } else if stats.github_events.total > 0 || stats.client_events.total > 0 {
                "ready"
            } else {
                "needs-evidence"
            };
            json!({
                "provider": "github",
                "status": status,
                "evidence": {
                    "github_events_total": stats.github_events.total,
                    "client_events_total": stats.client_events.total,
                    "repository_full_name": setup.repository_full_name
                },
                "next_step": if status == "ready" {
                    "Keep signed GitHub webhook/workflow evidence current."
                } else if status == "needs-config" {
                    "Select a repository in owner/repo format."
                } else {
                    "Install reviewed GitGov workflow/webhook telemetry for this repository."
                }
            })
        }
        "jira" => {
            let ticket_events = stats
                .client_events
                .by_type
                .get("jira_ticket")
                .copied()
                .unwrap_or(0);
            json!({
                "provider": "jira",
                "status": if ticket_events > 0 { "ready" } else { "needs-evidence" },
                "evidence": { "ticket_events": ticket_events },
                "next_step": "Run Jira ingest/correlation with customer-approved credentials and keep ticket IDs in PRs, branches, or commits."
            })
        }
        "jenkins" => json!({
            "provider": "jenkins",
            "status": if stats.pipeline.total_7d > 0 { "ready" } else { "needs-evidence" },
            "evidence": {
                "pipeline_runs_7d": stats.pipeline.total_7d,
                "pipeline_success_7d": stats.pipeline.success_7d
            },
            "next_step": "Publish Jenkins job telemetry to GitGov and confirm pipeline evidence appears."
        }),
        "sonarqube" => json!({
            "provider": "sonarqube",
            "status": if setup.selected_modules.iter().any(|module| module == "quality-gates") {
                "needs-evidence"
            } else {
                "needs-config"
            },
            "evidence": {
                "quality_gate_module_selected": setup.selected_modules.iter().any(|module| module == "quality-gates"),
                "pipeline_runs_7d": stats.pipeline.total_7d
            },
            "next_step": "Validate SonarQube reachability from the selected runner and publish quality-gate telemetry."
        }),
        "render" | "vercel" => json!({
            "provider": provider,
            "status": if stats.active_repos > 0 { "ready" } else { "needs-evidence" },
            "evidence": { "active_repos": stats.active_repos },
            "next_step": "Record deployment evidence without storing provider tokens in GitGov."
        }),
        _ => json!({
            "provider": provider,
            "status": "needs-config",
            "evidence": {},
            "next_step": "Unsupported provider for the first governed repo setup."
        }),
    }
}

fn first_setup_wizard_state(
    org_id: &str,
    setup: Option<&FirstGovernedRepoSetupRecord>,
    stats: &AuditStats,
) -> serde_json::Value {
    let Some(setup) = setup else {
        return json!({
            "schema_version": "gitgov_first_governed_repo_wizard_state.v1",
            "org_id": org_id,
            "status": "not_started",
            "current_step": "current_state",
            "selected_repo": null,
            "provider_health": [],
            "evidence_availability": {
                "github_events_total": stats.github_events.total,
                "client_events_total": stats.client_events.total,
                "pipeline_runs_7d": stats.pipeline.total_7d,
                "active_repos": stats.active_repos
            },
            "gaps": ["repository_full_name", "policy_workflow_preview", "provider_evidence"],
            "next_manual_steps": [
                "Select the first repository in owner/repo format.",
                "Review the policy/workflow baseline preview.",
                "Run provider validation with customer-approved credentials outside GitGov when needed."
            ],
            "safety": {
                "manual_first": true,
                "reads_secret_values": false,
                "stores_secret_values": false,
                "mutates_provider_state": false,
                "mutates_customer_repository": false,
                "agent_governance_required": false,
                "ai_required": false,
                "compliance_claim": false,
                "certification": false,
                "legal_attestation": false,
                "official_regulatory_mapping": false,
                "public_link": false,
                "email_delivery": false,
                "scheduler": false
            }
        });
    };

    let provider_health = setup
        .selected_providers
        .iter()
        .map(|provider| first_setup_readiness_from_provider(provider, setup, stats))
        .collect::<Vec<_>>();
    let gaps = setup
        .baseline
        .get("action_center_gaps")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let provider_gap = provider_health
        .iter()
        .any(|check| check.get("status").and_then(|value| value.as_str()) != Some("ready"));
    let mut gap_values = gaps;
    if provider_gap && !gap_values.iter().any(|value| value == "provider_evidence") {
        gap_values.push(json!("provider_evidence"));
    }
    let gate_readiness = setup
        .baseline
        .get("gate_readiness")
        .and_then(|value| value.as_str())
        .unwrap_or("needs_repo");
    let current_step = setup
        .baseline
        .get("current_step")
        .and_then(|value| value.as_str())
        .unwrap_or(if setup.status == "completed" {
            "first_result"
        } else if gate_readiness == "baseline_ready" {
            "baseline_preview"
        } else if first_setup_repo_format_valid(&setup.repository_full_name) {
            "validate_evidence_sources"
        } else {
            "select_repo"
        });

    json!({
        "schema_version": "gitgov_first_governed_repo_wizard_state.v1",
        "org_id": org_id,
        "status": setup.status,
        "run_id": setup.run_id,
        "current_step": current_step,
        "selected_repo": {
            "repository_full_name": setup.repository_full_name,
            "default_branch": setup.default_branch
        },
        "selected_providers": setup.selected_providers,
        "selected_modules": setup.selected_modules,
        "policy_preset": setup.policy_preset,
        "gate_readiness": gate_readiness,
        "provider_health": provider_health,
        "evidence_availability": {
            "github_events_total": stats.github_events.total,
            "client_events_total": stats.client_events.total,
            "pipeline_runs_7d": stats.pipeline.total_7d,
            "pipeline_success_7d": stats.pipeline.success_7d,
            "active_repos": stats.active_repos
        },
        "gaps": gap_values,
        "baseline_plan": setup.baseline.get("baseline_plan"),
        "first_result": setup.baseline.get("first_result"),
        "first_result_refs": setup.baseline.get("first_result_refs"),
        "provider_validation": setup.baseline.get("provider_validation"),
        "next_manual_steps": if setup.status == "completed" {
            json!(["Use Governance > Releases for advisory gate simulation.", "Keep evidence fresh before production enforcement."])
        } else if gate_readiness == "baseline_ready" {
            json!(["Complete setup to persist the first result refs.", "Run advisory deployment gate simulation manually."])
        } else {
            json!(["Review policy/workflow preview.", "Run provider validation with customer-approved credentials outside GitGov when needed.", "Save the baseline before completion."])
        },
        "safety": {
            "manual_first": true,
            "reads_secret_values": false,
            "stores_secret_values": false,
            "mutates_provider_state": false,
            "mutates_customer_repository": false,
            "agent_governance_required": false,
            "ai_required": false,
            "compliance_claim": false,
            "certification": false,
            "legal_attestation": false,
            "official_regulatory_mapping": false,
            "public_link": false,
            "email_delivery": false,
            "scheduler": false
        }
    })
}

fn first_setup_baseline_map(setup: &FirstGovernedRepoSetupRecord) -> serde_json::Map<String, serde_json::Value> {
    setup
        .baseline
        .as_object()
        .cloned()
        .unwrap_or_default()
}

async fn first_setup_audit(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    action: &str,
    setup: Option<&FirstGovernedRepoSetupRecord>,
    metadata: serde_json::Value,
) {
    let entry = AdminAuditLogEntry {
        id: Uuid::new_v4().to_string(),
        actor_client_id: auth_user.client_id.clone(),
        action: action.to_string(),
        target_type: Some("enterprise_first_governed_repo_setup".to_string()),
        target_id: setup
            .map(|record| record.run_id.clone())
            .or_else(|| Some(org_id.to_string())),
        metadata: json!({
            "org_id": org_id,
            "run_id": setup.map(|record| record.run_id.clone()),
            "status": setup.map(|record| record.status.clone()),
            "repository_full_name": setup.map(|record| record.repository_full_name.clone()),
            "manual_first": true,
            "stores_secret_values": false,
            "mutates_provider_state": false,
            "mutates_customer_repository": false,
            "agent_governance_required": false,
            "compliance_claim": false,
            "metadata": metadata
        }),
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = state.db.insert_admin_audit_log(&entry).await {
        tracing::warn!(error = %e, action = %action, "Failed to write first governed repo wizard audit log");
    }
}
