use super::*;

#[test]
fn user_role_roundtrip() {
    let roles = [
        UserRole::Admin,
        UserRole::Architect,
        UserRole::Developer,
        UserRole::PM,
    ];
    for role in &roles {
        assert_eq!(&UserRole::from_str(role.as_str()), role);
    }
}

#[test]
fn user_role_unknown_defaults_to_developer() {
    assert_eq!(UserRole::from_str("unknown"), UserRole::Developer);
    assert_eq!(UserRole::from_str(""), UserRole::Developer);
}

#[test]
fn org_invitation_accept_login_prefers_invite_login() {
    let invitation = OrgInvitation {
        invite_email: Some("alice@example.com".to_string()),
        invite_login: Some("alice-gh".to_string()),
        ..OrgInvitation::default()
    };

    assert_eq!(
        invitation.resolved_accept_login(),
        Some("alice-gh".to_string())
    );
    assert!(invitation.accepts_requested_login(Some("alice-gh")));
    assert!(!invitation.accepts_requested_login(Some("mallory")));
}

#[test]
fn org_invitation_accept_login_uses_email_local_part_as_legacy_fallback() {
    let invitation = OrgInvitation {
        invite_email: Some("alice@example.com".to_string()),
        ..OrgInvitation::default()
    };

    assert_eq!(
        invitation.resolved_accept_login(),
        Some("alice".to_string())
    );
    assert!(invitation.accepts_requested_login(None));
    assert!(invitation.accepts_requested_login(Some("alice")));
    assert!(!invitation.accepts_requested_login(Some("mallory")));
}

#[test]
fn client_event_type_roundtrip() {
    let types = [
        ClientEventType::AttemptPush,
        ClientEventType::BlockedPush,
        ClientEventType::SuccessfulPush,
        ClientEventType::PushFailed,
        ClientEventType::GovernanceBlockedPush,
        ClientEventType::GovernanceWarnedPush,
        ClientEventType::CliCommand,
        ClientEventType::CliCommandCompleted,
        ClientEventType::Heartbeat,
        ClientEventType::CreateBranch,
        ClientEventType::BlockedBranch,
        ClientEventType::StageFiles,
        ClientEventType::Commit,
        ClientEventType::CheckoutBranch,
        ClientEventType::Login,
        ClientEventType::Logout,
    ];
    for t in &types {
        assert_eq!(&ClientEventType::parse(t.as_str()).unwrap(), t);
    }
    assert_eq!(ClientEventType::parse("unknown_event"), None);
    assert_eq!(
        ClientEventType::from_db_str("unknown_event"),
        ClientEventType::AttemptPush
    );
}

#[test]
fn event_status_roundtrip() {
    assert_eq!(EventStatus::parse("success"), Some(EventStatus::Success));
    assert_eq!(EventStatus::parse("blocked"), Some(EventStatus::Blocked));
    assert_eq!(EventStatus::parse("failed"), Some(EventStatus::Failed));
    assert_eq!(EventStatus::parse("unknown"), None);
    assert_eq!(EventStatus::from_db_str("unknown"), EventStatus::Failed);
}

#[test]
fn pipeline_status_roundtrip() {
    assert_eq!(
        PipelineStatus::from_str("success"),
        Some(PipelineStatus::Success)
    );
    assert_eq!(
        PipelineStatus::from_str("failure"),
        Some(PipelineStatus::Failure)
    );
    assert_eq!(
        PipelineStatus::from_str("aborted"),
        Some(PipelineStatus::Aborted)
    );
    assert_eq!(
        PipelineStatus::from_str("unstable"),
        Some(PipelineStatus::Unstable)
    );
    assert_eq!(PipelineStatus::from_str("invalid"), None);
}

#[test]
fn signal_type_roundtrip() {
    let types = [
        SignalType::UntrustedPath,
        SignalType::MissingTelemetry,
        SignalType::PolicyViolation,
        SignalType::CorrelationMismatch,
        SignalType::CommitNoTicket,
        SignalType::TicketNoCoverage,
        SignalType::PipelineFailureStreak,
        SignalType::StaleInProgress,
        SignalType::DoneNotDeployed,
    ];
    for t in &types {
        assert_eq!(&SignalType::from_str(t.as_str()), t);
    }
}

#[test]
fn confidence_level_roundtrip() {
    assert_eq!(ConfidenceLevel::from_str("high"), ConfidenceLevel::High);
    assert_eq!(ConfidenceLevel::from_str("medium"), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_str("low"), ConfidenceLevel::Low);
    assert_eq!(ConfidenceLevel::from_str("unknown"), ConfidenceLevel::Low);
}

#[test]
fn signal_status_roundtrip() {
    assert_eq!(SignalStatus::from_str("pending"), SignalStatus::Pending);
    assert_eq!(
        SignalStatus::from_str("investigating"),
        SignalStatus::Investigating
    );
    assert_eq!(SignalStatus::from_str("confirmed"), SignalStatus::Confirmed);
    assert_eq!(SignalStatus::from_str("dismissed"), SignalStatus::Dismissed);
    assert_eq!(SignalStatus::from_str("unknown"), SignalStatus::Pending);
}

#[test]
fn client_event_batch_deserialize() {
    let json = r#"{
            "events": [{
                "event_uuid": "abc-123",
                "event_type": "commit",
                "repo_full_name": "yohandry10/Git-Gov",
                "branch": "main",
                "user_login": "dev1",
                "files": ["src/main.rs"],
                "status": "success"
            }]
        }"#;
    let batch: ClientEventBatch = serde_json::from_str(json).unwrap();
    assert_eq!(batch.events.len(), 1);
    assert_eq!(batch.events[0].event_type, "commit");
    assert!(batch.client_id.is_none());
}

#[test]
fn policy_check_response_default() {
    let resp = PolicyCheckResponse::default();
    assert!(!resp.advisory);
    assert!(!resp.allowed);
    assert!(resp.reasons.is_empty());
    assert!(resp.warnings.is_empty());
}

#[test]
fn compliance_dashboard_deserialize_missing_timeline_defaults() {
    let payload = serde_json::json!({
        "signals": {
            "total": 1,
            "pending": 1,
            "high_confidence": 0,
            "by_type": {"commit_no_ticket": 1}
        },
        "correlation": {
            "github_pushes_24h": 3,
            "client_pushes_24h": 3,
            "correlation_rate": 1.0
        },
        "policy": {
            "repos_with_policy": 1,
            "total_repos": 2,
            "recent_changes": 1
        },
        "exports": {
            "total": 2,
            "last_7_days": 1
        }
    });

    let parsed: ComplianceDashboard =
        serde_json::from_value(payload).expect("deserialize compliance dashboard");
    assert!(parsed.timeline.is_empty());
}

#[test]
fn jenkins_pipeline_input_deserialize_with_defaults() {
    let json = r#"{
            "pipeline_id": "build-123",
            "job_name": "main-build",
            "status": "success"
        }"#;
    let input: JenkinsPipelineEventInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.pipeline_id, "build-123");
    assert!(input.commit_sha.is_none());
    assert!(input.stages.is_empty());
    assert!(input.artifacts.is_empty());
}

// ── Golden Path contract tests ────────────────────────────────────────────
// Validate the exact JSON shape the Desktop sends for each step of the
// Golden Path: stage_files → commit → attempt_push → successful_push.
// Pure deserialisation — no DB or server required; run in CI via `cargo test`.

fn gp_batch(event_type: &str, extra_fields: &str) -> ClientEventBatch {
    let json = format!(
        r#"{{
                "events": [{{
                    "event_uuid": "00000000-0000-0000-0000-000000000001",
                    "event_type": "{event_type}",
                    "user_login": "dev1",
                    "repo_full_name": "yohandry10/Git-Gov",
                    "branch": "feat/golden",
                    "files": ["src/main.rs", "src/lib.rs"],
                    "status": "success"
                    {extra_fields}
                }}],
                "client_version": "1.0.0"
            }}"#
    );
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("failed to parse {event_type} batch: {e}"))
}

#[test]
fn golden_path_stage_files_contract() {
    let batch = gp_batch("stage_files", "");
    assert_eq!(batch.events.len(), 1);
    let ev = &batch.events[0];
    assert_eq!(ev.event_type, "stage_files");
    assert_eq!(ev.user_login, "dev1");
    assert!(!ev.files.is_empty(), "stage_files must carry file list");
    assert_eq!(ev.status, "success");
    assert!(!ev.event_uuid.is_empty(), "event_uuid required for dedup");
}

#[test]
fn golden_path_commit_contract() {
    let batch = gp_batch(
        "commit",
        r#", "commit_sha": "abc123def4567890abc123def4567890abc12345""#,
    );
    let ev = &batch.events[0];
    assert_eq!(ev.event_type, "commit");
    assert!(
        ev.commit_sha.is_some(),
        "commit event must carry commit_sha"
    );
    assert_eq!(ev.status, "success");
}

#[test]
fn golden_path_attempt_push_contract() {
    let batch = gp_batch(
        "attempt_push",
        r#", "commit_sha": "abc123def4567890abc123def4567890abc12345""#,
    );
    let ev = &batch.events[0];
    assert_eq!(ev.event_type, "attempt_push");
    assert_eq!(ev.branch.as_deref(), Some("feat/golden"));
    assert!(
        ev.commit_sha.is_some(),
        "attempt_push event must carry the pushed HEAD sha"
    );
    assert_eq!(ev.status, "success");
}

#[test]
fn golden_path_successful_push_contract() {
    let batch = gp_batch(
        "successful_push",
        r#", "commit_sha": "abc123def4567890abc123def4567890abc12345""#,
    );
    let ev = &batch.events[0];
    assert_eq!(ev.event_type, "successful_push");
    assert_eq!(ev.status, "success");
    assert!(
        ev.commit_sha.is_some(),
        "successful_push event must carry the pushed HEAD sha"
    );
    assert!(!ev.event_uuid.is_empty());
}

#[test]
fn golden_path_response_accepted_shape() {
    // Validates /events response — Desktop parses this to know if accepted or duped.
    let json =
        r#"{"accepted":["00000000-0000-0000-0000-000000000001"],"duplicates":[],"errors":[]}"#;
    let resp: ClientEventResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.accepted.len(), 1);
    assert!(resp.duplicates.is_empty());
    assert!(resp.errors.is_empty());
}

#[test]
fn golden_path_duplicate_detected_in_response() {
    // Server returns the same UUID as a duplicate on second send.
    let json =
        r#"{"accepted":[],"duplicates":["00000000-0000-0000-0000-000000000001"],"errors":[]}"#;
    let resp: ClientEventResponse = serde_json::from_str(json).unwrap();
    assert!(resp.accepted.is_empty());
    assert_eq!(resp.duplicates.len(), 1);
}

#[test]
fn contract_server_stats_top_level_shape_is_stable() {
    let value = serde_json::to_value(AuditStats::default()).expect("serialize audit stats");
    let obj = value
        .as_object()
        .expect("AuditStats must serialize as JSON object");

    assert_eq!(obj.len(), 6);
    assert!(obj.contains_key("github_events"));
    assert!(obj.contains_key("client_events"));
    assert!(obj.contains_key("violations"));
    assert!(obj.contains_key("pipeline"));
    assert!(obj.contains_key("active_devs_week"));
    assert!(obj.contains_key("active_repos"));
}

#[test]
fn contract_combined_event_shape_is_stable() {
    let event = CombinedEvent {
        id: "evt-1".to_string(),
        source: "client".to_string(),
        event_type: "commit".to_string(),
        created_at: 0,
        user_login: Some("dev1".to_string()),
        repo_name: Some("yohandry10/Git-Gov".to_string()),
        branch: Some("main".to_string()),
        status: Some("success".to_string()),
        details: serde_json::json!({}),
    };

    let value = serde_json::to_value(event).expect("serialize combined event");
    let obj = value
        .as_object()
        .expect("CombinedEvent must serialize as JSON object");

    assert_eq!(obj.len(), 9);
    assert!(obj.contains_key("id"));
    assert!(obj.contains_key("source"));
    assert!(obj.contains_key("event_type"));
    assert!(obj.contains_key("created_at"));
    assert!(obj.contains_key("user_login"));
    assert!(obj.contains_key("repo_name"));
    assert!(obj.contains_key("branch"));
    assert!(obj.contains_key("status"));
    assert!(obj.contains_key("details"));
}

#[test]
fn relevant_audit_actions_contains_expected() {
    assert!(RELEVANT_AUDIT_ACTIONS.contains(&"protected_branch.create"));
    assert!(RELEVANT_AUDIT_ACTIONS.contains(&"repo.access"));
    assert!(!RELEVANT_AUDIT_ACTIONS.contains(&"random_action"));
}

// Pagination defaults — regression tests for "missing field offset/limit"
#[test]
fn event_filter_offset_optional_defaults_to_zero() {
    let f: EventFilter = serde_json::from_str(r#"{"limit": 5}"#).unwrap();
    assert_eq!(f.offset, 0);
    assert_eq!(f.limit, 5);
}

#[test]
fn event_filter_all_pagination_optional() {
    let f: EventFilter = serde_json::from_str(r#"{}"#).unwrap();
    assert_eq!(f.offset, 0);
    assert_eq!(f.limit, 0); // 0 → handler uses its fallback default
}

#[test]
fn event_filter_explicit_offset_respected() {
    let f: EventFilter = serde_json::from_str(r#"{"limit": 10, "offset": 25}"#).unwrap();
    assert_eq!(f.offset, 25);
    assert_eq!(f.limit, 10);
}

#[test]
fn jenkins_correlation_filter_offset_optional() {
    let f: JenkinsCorrelationFilter = serde_json::from_str(r#"{"limit": 10}"#).unwrap();
    assert_eq!(f.offset, 0);
    assert_eq!(f.limit, 10);
}

#[test]
fn jenkins_correlation_filter_all_pagination_optional() {
    let f: JenkinsCorrelationFilter = serde_json::from_str(r#"{}"#).unwrap();
    assert_eq!(f.offset, 0);
    assert_eq!(f.limit, 0); // 0 → handler uses its fallback default (20)
}

// ── Identity alias expansion (get_combined_events with aliases) ───────────

#[test]
fn expand_aliases_canonical_only_when_no_aliases() {
    let result = expand_login_aliases("alice", &[]);
    assert_eq!(result, vec!["alice"]);
}

#[test]
fn expand_aliases_includes_all_matching_aliases() {
    // Filtering by canonical "alice" must also return events for her aliases.
    let aliases = vec![
        IdentityAlias {
            canonical_login: "alice".to_string(),
            alias_login: "alice-personal".to_string(),
            org_id: None,
            created_at: 0,
        },
        IdentityAlias {
            canonical_login: "alice".to_string(),
            alias_login: "alice-work".to_string(),
            org_id: None,
            created_at: 0,
        },
    ];
    let result = expand_login_aliases("alice", &aliases);
    assert_eq!(result, vec!["alice", "alice-personal", "alice-work"]);
}

#[test]
fn expand_aliases_ignores_aliases_for_different_canonical() {
    let aliases = vec![IdentityAlias {
        canonical_login: "bob".to_string(),
        alias_login: "bob-work".to_string(),
        org_id: None,
        created_at: 0,
    }];
    let result = expand_login_aliases("alice", &aliases);
    assert_eq!(result, vec!["alice"]);
}

#[test]
fn expand_aliases_canonical_is_always_first_element() {
    let aliases = vec![IdentityAlias {
        canonical_login: "carol".to_string(),
        alias_login: "carol-oss".to_string(),
        org_id: Some("uuid-org".to_string()),
        created_at: 1_700_000_000_000,
    }];
    let result = expand_login_aliases("carol", &aliases);
    assert_eq!(result[0], "carol");
    assert_eq!(result[1], "carol-oss");
    assert_eq!(result.len(), 2);
}
