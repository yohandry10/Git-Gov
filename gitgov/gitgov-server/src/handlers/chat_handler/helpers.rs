fn query_needs_explicit_org_scope(query: &ChatQuery) -> bool {
    matches!(
        query,
        ChatQuery::ControlPlaneExecutiveSummary
            | ChatQuery::QualityGateTopFailingRepos { .. }
            | ChatQuery::QualityGateTopFailingBranches { .. }
            | ChatQuery::TicketsWithNonGreenQualityGate { .. }
            | ChatQuery::TicketsReleasedWithNonGreenQualityGate { .. }
            | ChatQuery::DevelopersWithNonGreenQualityGate { .. }
            | ChatQuery::QualityGateHealthWindow { .. }
            | ChatQuery::ReleaseReadinessTopFailingRepos { .. }
            | ChatQuery::ReleaseReadinessTopFailingBranches { .. }
            | ChatQuery::ReleaseReadinessHealthWindow { .. }
            | ChatQuery::OnlineDevelopersNow { .. }
            | ChatQuery::CommitsWithoutTicketWindow { .. }
            | ChatQuery::PushesNoTicket
            | ChatQuery::BlockedPushesMonth
            | ChatQuery::UserPushesCount { .. }
            | ChatQuery::UserActivityMonth { .. }
            | ChatQuery::UserPushesNoTicketWeek { .. }
            | ChatQuery::UserBlockedPushesMonth { .. }
            | ChatQuery::SessionCommitsCount { .. }
            | ChatQuery::TotalCommitsCount
            | ChatQuery::UserCommitsCount { .. }
            | ChatQuery::UserLastCommit { .. }
            | ChatQuery::UserCommitsRange { .. }
            | ChatQuery::UserAccessProfile { .. }
    )
}

fn is_founder_scope_exception(auth_user: &AuthUser) -> bool {
    auth_user.role == UserRole::Admin
        && auth_user.org_id.is_none()
        && auth_user.client_id.eq_ignore_ascii_case("bootstrap-admin")
}

fn looks_generic_non_answer(text: &str) -> bool {
    let t = text.to_lowercase();
    let markers = [
        "puedo guiarte paso a paso",
        "opciones frecuentes",
        "información detallada",
        "la información detallada",
        "i can guide you step by step",
        "common options",
        "detailed information is available",
    ];
    markers.iter().any(|m| t.contains(m))
}

fn is_logs_precision_query(question: &str) -> bool {
    let q = question.to_lowercase();
    let has_logs_word = q.split(|c: char| !c.is_alphanumeric()).any(|w| {
        matches!(
            w,
            "log" | "logs" | "evento" | "eventos" | "event" | "events" | "historial"
        )
    });
    has_logs_word
        || q.contains("actividad reciente")
        || q.contains("ultimos eventos")
        || q.contains("ultimos logs")
        || q.contains("recent activity")
        || q.contains("recent logs")
        || q.contains("latest logs")
}

fn extract_logs_limit(question: &str, default_limit: usize, max_limit: usize) -> usize {
    for token in question.split(|c: char| !c.is_ascii_digit()) {
        if token.is_empty() {
            continue;
        }
        if let Ok(value) = token.parse::<usize>() {
            if value > 0 {
                return value.min(max_limit);
            }
        }
    }
    default_limit.min(max_limit).max(1)
}

fn extract_logs_event_type_hint(question: &str) -> Option<String> {
    let q = question.to_lowercase();
    if q.contains("blocked_push") || q.contains("push bloque") {
        return Some("blocked_push".to_string());
    }
    if q.contains("successful_push") || q.contains("push exitos") {
        return Some("successful_push".to_string());
    }
    if q.contains("attempt_push") || q.contains("intento de push") {
        return Some("attempt_push".to_string());
    }
    if q.contains("stage_files") || q.contains("staged") || q.contains("staging") {
        return Some("stage_files".to_string());
    }
    if q.contains("commit") {
        return Some("commit".to_string());
    }
    None
}

fn render_precise_logs_answer(events: &[CombinedEvent], language: &str) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(events.len());
    for event in events {
        let user = event.user_login.as_deref().unwrap_or("n/a");
        let repo = event.repo_name.as_deref().unwrap_or("n/a");
        let branch = event.branch.as_deref().unwrap_or("n/a");
        let status = event.status.as_deref().unwrap_or("n/a");
        let ts_label = if let Some(dt_utc) =
            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(event.created_at)
        {
            let lima_tz = chrono::FixedOffset::west_opt(5 * 3600)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("valid offset"));
            let dt_lima = dt_utc.with_timezone(&lima_tz);
            format!(
                "{} Lima | {} UTC | {}ms",
                dt_lima.format("%Y-%m-%d %H:%M:%S"),
                dt_utc.format("%Y-%m-%d %H:%M:%S"),
                event.created_at
            )
        } else {
            format!("{}ms", event.created_at)
        };

        lines.push(format!(
            "- {} | source={} type={} user={} repo={} branch={} status={} id={}",
            ts_label, event.source, event.event_type, user, repo, branch, status, event.id
        ));
    }

    if language == "en" {
        format!(
            "Exact log sample ({} events, deterministic DB query):\n{}",
            events.len(),
            lines.join("\n")
        )
    } else {
        format!(
            "Muestra exacta de logs ({} eventos, consulta deterministica DB):\n{}",
            events.len(),
            lines.join("\n")
        )
    }
}

fn normalize_llm_response(
    mut response: ChatAskResponse,
    question: &str,
    language: &str,
) -> ChatAskResponse {
    let normalized_status = match response.status.as_str() {
        "ok" | "insufficient_data" | "feature_not_available" | "error" => response.status.clone(),
        _ => "error".to_string(),
    };
    response.status = normalized_status;

    response.answer = response.answer.trim().to_string();
    if response.answer.is_empty() {
        response.status = "error".to_string();
        response.answer = if language == "en" {
            "The model returned an empty response. Please try again.".to_string()
        } else {
            "El modelo devolvió una respuesta vacía. Intenta de nuevo.".to_string()
        };
    }

    if response.status == "error" {
        response.status = "insufficient_data".to_string();
        response.answer = if language == "en" {
            "I could not provide a verified answer for that exact request. I can still help with GitGov analytics, integrations, onboarding, and troubleshooting.".to_string()
        } else {
            "No pude dar una respuesta verificable para esa consulta exacta. Sí puedo ayudarte con analítica de GitGov, integraciones, onboarding y troubleshooting.".to_string()
        };
        response.missing_capability = None;
        response.can_report_feature = false;
    }

    if is_secret_exfiltration_request(question) {
        response.status = "ok".to_string();
        response.answer = "No puedo revelar API keys, tokens ni secretos de ningún usuario. Por seguridad solo puedo ayudarte con estado de acceso (rol, miembro activo/inactivo, y si existe key activa) sin mostrar valores sensibles.".to_string();
        response.missing_capability = None;
        response.can_report_feature = false;
        response.data_refs = vec!["security_policy".to_string()];
        response.sources = vec!["security_policy".to_string()];
        response.entities_detected = vec!["security_request".to_string()];
        response.time_range_used = None;
        response.actions_recommended = vec![];
        response.confidence = Some(1.0);
        return response;
    }

    if response.status == "ok" && looks_generic_non_answer(&response.answer) {
        response.status = "insufficient_data".to_string();
        response.answer = if language == "en" {
            "I need a more specific question or verifiable data to answer precisely. If your question is about metrics, include user/org/time window.".to_string()
        } else {
            "Necesito una pregunta más específica o datos verificables para responder con precisión. Si es una métrica, incluye usuario/org/ventana de tiempo.".to_string()
        };
    }

    if response.status == "insufficient_data" {
        let lower = response.answer.to_lowercase();
        let has_reason = [
            "falt",
            "insuf",
            "scope",
            "org",
            "dato",
            "context",
            "missing",
            "not enough",
        ]
        .iter()
        .any(|m| lower.contains(m));
        if !has_reason {
            response.answer = if language == "en" {
                "I can't answer that with the current scope/data. Please provide user/org/time window or ask a question covered by available project data.".to_string()
            } else {
                "No puedo responder eso con el scope/datos actuales. Indica usuario/org/ventana de tiempo o formula una pregunta cubierta por los datos disponibles.".to_string()
            };
        }
    }

    if response.status == "feature_not_available" {
        response.can_report_feature = true;
        if response.missing_capability.is_none() {
            response.missing_capability = Some("capability_not_available".to_string());
        }
    } else {
        response.can_report_feature = false;
        response.missing_capability = None;
    }

    response.data_refs = response
        .data_refs
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty() && r.len() <= 80)
        .take(12)
        .collect();
    response.data_refs.sort();
    response.data_refs.dedup();

    response.sources = response
        .sources
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty() && r.len() <= 120)
        .take(20)
        .collect();
    response.sources.sort();
    response.sources.dedup();

    response.entities_detected = response
        .entities_detected
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty() && r.len() <= 160)
        .take(24)
        .collect();
    response.entities_detected.sort();
    response.entities_detected.dedup();

    response.actions_recommended = response
        .actions_recommended
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty() && r.len() <= 240)
        .take(12)
        .collect();
    response.actions_recommended.sort();
    response.actions_recommended.dedup();

    response.time_range_used = response
        .time_range_used
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 120);

    response.confidence = response
        .confidence
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0);

    response
}

fn llm_degraded_answer(question: &str, language: &str) -> String {
    if let Some(answer) = build_knowledge_fallback_answer(question, language) {
        return answer;
    }

    let mut answer = build_guided_help_answer(question);
    if language == "en" {
        answer.push_str(
            "\n\nI could not use the language model for this turn, so I answered with local project context.",
        );
    } else {
        answer.push_str(
            "\n\nNo pude usar el modelo en este turno, así que respondí con contexto local del proyecto.",
        );
    }
    answer
}

async fn user_exists_in_scope(
    state: &Arc<AppState>,
    user: &str,
    scoped_org_id: Option<&str>,
) -> Result<bool, DbError> {
    Ok(state
        .db
        .chat_query_user_access_profile(user, scoped_org_id)
        .await?
        .is_some())
}
