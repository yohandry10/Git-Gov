#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_operational_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
    question: &str,
    proactive_todos: &Vec<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::OnlineDevelopersNow { minutes }) => {
            match state
                .db
                .chat_query_online_developers_count(scoped_org_id.as_deref(), minutes)
                .await
            {
                Ok(count) => {
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::OK,
                        ChatAskResponse {
                            status: "ok".to_string(),
                            answer: format!(
                                "Developers ON detectados: {} (ventana de actividad: últimos {} minutos).",
                                count, minutes
                            ),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec!["client_sessions".to_string()],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: None,
                            actions_recommended: vec![],
                            confidence: None,
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("chat_query_online_developers_count(single) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando developers online".to_string(),
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
            }
        }
        Some(ChatQuery::CommitsWithoutTicketWindow { hours }) => {
            match state
                .db
                .chat_query_commits_without_ticket_count(scoped_org_id.as_deref(), hours)
                .await
            {
                Ok(count) => {
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::OK,
                        ChatAskResponse {
                            status: "ok".to_string(),
                            answer: format!(
                                "Commits sin ticket detectados: {} en la ventana de {} horas.",
                                count, hours
                            ),
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![
                                "client_events".to_string(),
                                "commit_ticket_correlations".to_string(),
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
                Err(e) => {
                    tracing::error!("chat_query_commits_without_ticket_count error: {}", e);
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
            }
        }
        Some(ChatQuery::NeedUserForCommitHistory) => {
            return finalize_chat_response(&state, &conversation_key, &mut *session, &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "insufficient_data".to_string(),
                    answer: "Para contar commits en todo el historial necesito un usuario. Ejemplo: \"¿Cuántos commits hizo el usuario yohandry10 en todo el historial?\"".to_string(),
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
        Some(ChatQuery::GuidedHelp) => {
            let mut answer = build_guided_help_answer(&question);
            if !proactive_todos.is_empty() {
                answer.push_str("\n\nAcciones proactivas añadidas a TODO:\n");
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
                        "project_docs_kb".to_string(),
                        "web_docs_faq".to_string(),
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
        Some(ChatQuery::PushesNoTicket) => match state
            .db
            .chat_query_pushes_no_ticket(scoped_org_id.as_deref())
            .await
        {
            Ok(rows) => {
                let answer = if rows.is_empty() {
                    "No se encontraron pushes a main sin ticket en los últimos 7 días.".to_string()
                } else {
                    format!(
                        "Se encontraron {} pushes a main sin ticket en los últimos 7 días.",
                        rows.len()
                    )
                };
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
                            "github_events".to_string(),
                            "commit_ticket_correlations".to_string(),
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
            Err(e) => {
                tracing::error!("chat_query_pushes_no_ticket error: {}", e);
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut *session,
                    &nlp,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ChatAskResponse {
                        status: "error".to_string(),
                        answer: "Error consultando la base de datos".to_string(),
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
        },
        Some(ChatQuery::BlockedPushesMonth) => match state
            .db
            .chat_query_blocked_pushes_month(scoped_org_id.as_deref())
            .await
        {
            Ok(count) => {
                let answer = format!(
                    "El equipo tiene {} pushes bloqueados en el mes actual.",
                    count
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
                        data_refs: vec!["client_events".to_string()],
                        sources: vec![],
                        entities_detected: vec![],
                        time_range_used: None,
                        actions_recommended: vec![],
                        confidence: None,
                        trace_id: None,
                    },
                );
            }
            Err(e) => {
                tracing::error!("chat_query_blocked_pushes_month error: {}", e);
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut *session,
                    &nlp,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ChatAskResponse {
                        status: "error".to_string(),
                        answer: "Error consultando la base de datos".to_string(),
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
        },
        _ => unreachable!("chat query routed to wrong handler"),
    }
}
