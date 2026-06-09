#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_intro_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
    auth_user: &AuthUser,
    proactive_todos: &Vec<String>,
    snapshot_refs: &Vec<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::Greeting) => {
            let mut answer = greeting_answer(&nlp.entities.language);
            if !proactive_todos.is_empty() {
                answer.push_str("\n\nSugerencias proactivas registradas en TODO:\n");
                answer.push_str(&proactive_todos.join("\n"));
            }
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: vec![
                        "assistant_runtime".to_string(),
                        "project_docs_kb".to_string(),
                        "todo_runtime".to_string(),
                    ],

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        Some(ChatQuery::DateMismatchClarification) => {
            let now_utc = chrono::Utc::now();
            let lima_tz = chrono::FixedOffset::west_opt(5 * 3600)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("valid offset"));
            let now_lima = now_utc.with_timezone(&lima_tz);
            let answer = format!(
                "Buena alerta. Si una fecha parece \"adelantada\" (por ejemplo 04 vs 03), normalmente es por zona horaria (UTC vs America/Lima) o por una respuesta no determinística del LLM. Hora actual: {} (America/Lima) | {} UTC. Si quieres, te doy el dato exacto consultando el evento en base de datos con UTC y hora local.",
                now_lima.format("%Y-%m-%d %H:%M:%S"),
                now_utc.format("%Y-%m-%d %H:%M:%S")
            );
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: vec!["assistant_runtime".to_string(), "client_events".to_string()],

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        Some(ChatQuery::CurrentDateTime) => {
            let now_utc = chrono::Utc::now();
            let lima_tz = chrono::FixedOffset::west_opt(5 * 3600)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("valid offset"));
            let now_lima = now_utc.with_timezone(&lima_tz);
            let answer = format!(
                "Fecha y hora actuales: {} (America/Lima, {}) | UTC: {}.",
                now_lima.format("%Y-%m-%d %H:%M:%S"),
                weekday_es(now_lima.weekday()),
                now_utc.format("%Y-%m-%d %H:%M:%S")
            );
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: vec!["assistant_runtime".to_string()],
                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        Some(ChatQuery::CapabilityOverview) => {
            let mut answer = "Sí. Puedo consultar datos reales del Control Plane con el scope de tu API key. Hoy tengo consultas en tiempo real para: resumen ejecutivo del control plane, devs online recientes, commits sin ticket (ventana), pushes por usuario (exitosos), pushes sin ticket (global y por usuario), pushes bloqueados del mes (global y por usuario), commits por usuario en rango y conteos de commits. También puedo consultar perfil de acceso de usuario (rol/estado y si tiene key activa), sin exponer secretos.".to_string();
            if !proactive_todos.is_empty() {
                answer.push_str("\n\nTambién detecté acciones sugeridas y las añadí a TODO.");
            }
            let mut refs = vec![
                "client_events".to_string(),
                "github_events".to_string(),
                "project_docs_kb".to_string(),
            ];
            refs.extend(snapshot_refs.clone());
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: refs,
                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        Some(ChatQuery::ControlPlaneExecutiveSummary) => {
            let stats = match state.db.get_stats(scoped_org_id.as_deref()).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("get_stats(executive_summary) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando estadísticas del Control Plane".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let now_ms = chrono::Utc::now().timestamp_millis();
            let start_7d = now_ms - 7 * 24 * 60 * 60 * 1000;
            let commits_7d = match state
                .db
                .chat_query_commits_count(Some(start_7d), Some(now_ms), scoped_org_id.as_deref())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_commits_count(7d) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando commits del Control Plane".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let online_devs = match state
                .db
                .chat_query_online_developers_count(scoped_org_id.as_deref(), 15)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_online_developers_count error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando sesiones activas de developers".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let pushes_no_ticket_7d = match state
                .db
                .chat_query_pushes_no_ticket_count(scoped_org_id.as_deref())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_pushes_no_ticket_count error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando pushes sin ticket".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let commits_no_ticket_7d = match state
                .db
                .chat_query_commits_without_ticket_count(scoped_org_id.as_deref(), 24 * 7)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_commits_without_ticket_count(7d) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando commits sin ticket".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let blocked_month = match state
                .db
                .chat_query_blocked_pushes_month(scoped_org_id.as_deref())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_blocked_pushes_month(executive) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando pushes bloqueados".to_string(),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
            };
            let commits_with_ticket_7d = (commits_7d - commits_no_ticket_7d).max(0);
            let coverage_7d = if commits_7d > 0 {
                (commits_with_ticket_7d as f64 / commits_7d as f64) * 100.0
            } else {
                0.0
            };
            let scope_hint = if scoped_org_id.is_some() {
                "scope org activo"
            } else if auth_user.client_id.eq_ignore_ascii_case("bootstrap-admin") {
                "scope founder/global"
            } else {
                "scope global"
            };
            let now_utc = chrono::Utc::now();
            let lima_tz = chrono::FixedOffset::west_opt(5 * 3600)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("valid offset"));
            let now_lima = now_utc.with_timezone(&lima_tz);
            let answer = format!(
                "Resumen ejecutivo Control Plane ({scope_hint})\n\
Devs ON (últimos 15 min): {online_devs}\n\
Devs activos 7d: {active_devs_week}\n\
Repos activos: {active_repos}\n\
Commits 7d: {commits_7d}\n\
Commits sin ticket 7d: {commits_no_ticket_7d}\n\
Cobertura ticket commits 7d: {coverage_7d:.1}%\n\
Pushes a main sin ticket 7d: {pushes_no_ticket_7d}\n\
Pushes bloqueados (mes actual): {blocked_month}\n\
Violaciones sin resolver: {violations_unresolved}\n\
Corte temporal: {lima} (America/Lima) | {utc} UTC.",
                scope_hint = scope_hint,
                online_devs = online_devs,
                active_devs_week = stats.active_devs_week,
                active_repos = stats.active_repos,
                commits_7d = commits_7d,
                commits_no_ticket_7d = commits_no_ticket_7d,
                coverage_7d = coverage_7d,
                pushes_no_ticket_7d = pushes_no_ticket_7d,
                blocked_month = blocked_month,
                violations_unresolved = stats.violations.unresolved,
                lima = now_lima.format("%Y-%m-%d %H:%M:%S"),
                utc = now_utc.format("%Y-%m-%d %H:%M:%S")
            );
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: vec![
                        "stats".to_string(),
                        "client_sessions".to_string(),
                        "client_events".to_string(),
                        "github_events".to_string(),
                        "commit_ticket_correlations".to_string(),
                        "assistant_runtime".to_string(),
                    ],

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        _ => unreachable!("chat query routed to wrong handler"),
    }
}
