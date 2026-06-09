#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_user_push_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::UserPushesCount {
            ref user,
            start_ms,
            end_ms,
        }) => {
            match state
                .db
                .chat_query_user_pushes_count(user, start_ms, end_ms, scoped_org_id.as_deref())
                .await
            {
                Ok(count) => {
                    let answer = if count == 0 {
                        let exists_in_scope = match user_exists_in_scope(
                            &state,
                            user,
                            scoped_org_id.as_deref(),
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "user_exists_in_scope(user_pushes_count) error: {}",
                                    e
                                );
                                false
                            }
                        };
                        if exists_in_scope {
                            if start_ms.is_some() && end_ms.is_some() {
                                format!(
                                    "El usuario {} no tiene pushes exitosos en el rango solicitado.",
                                    user
                                )
                            } else {
                                format!(
                                    "El usuario {} no tiene pushes exitosos en el historial disponible para el scope activo.",
                                    user
                                )
                            }
                        } else {
                            format!(
                                "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                                user
                            )
                        }
                    } else if start_ms.is_some() && end_ms.is_some() {
                        format!(
                            "El usuario {} tiene {} pushes exitosos en el rango solicitado.",
                            user, count
                        )
                    } else {
                        format!(
                            "El usuario {} tiene {} pushes exitosos en el historial disponible para tu scope.",
                            user, count
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
                    tracing::error!("chat_query_user_pushes_count error: {}", e);
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
            }
        }
        Some(ChatQuery::UserActivityMonth { ref user }) => {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let month_start_ms = {
                let dt = chrono::Utc::now();
                let date = match chrono::NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1) {
                    Some(d) => d,
                    None => {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ChatAskResponse {
                                status: "error".to_string(),
                                answer: "No pude calcular el inicio del mes actual".to_string(),
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
                date.and_hms_opt(0, 0, 0)
                    .map(|x| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(x, chrono::Utc)
                            .timestamp_millis()
                    })
                    .unwrap_or(0)
            };

            let exists_in_scope =
                match user_exists_in_scope(&state, user, scoped_org_id.as_deref()).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("user_exists_in_scope(user_activity_month) error: {}", e);
                        false
                    }
                };
            if !exists_in_scope {
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut *session,
                    &nlp,
                    StatusCode::OK,
                    ChatAskResponse {
                        status: "insufficient_data".to_string(),
                        answer: format!(
                            "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                            user
                        ),
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec!["org_users".to_string()],
                        sources: vec![],
                        entities_detected: vec![],
                        time_range_used: None,
                        actions_recommended: vec![],
                        confidence: None,
                        trace_id: None,
                    },
                );
            }

            let commits = match state
                .db
                .chat_query_user_commits_count(
                    user,
                    Some(month_start_ms),
                    Some(now_ms),
                    scoped_org_id.as_deref(),
                )
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_user_commits_count(activity_month) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando commits del usuario".to_string(),
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

            let pushes = match state
                .db
                .chat_query_user_pushes_count(
                    user,
                    Some(month_start_ms),
                    Some(now_ms),
                    scoped_org_id.as_deref(),
                )
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("chat_query_user_pushes_count(activity_month) error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando pushes del usuario".to_string(),
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

            let blocked_pushes = match state
                .db
                .chat_query_user_blocked_pushes_month(user, scoped_org_id.as_deref())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        "chat_query_user_blocked_pushes_month(activity_month) error: {}",
                        e
                    );
                    0
                }
            };

            let answer = format!(
                "Actividad de {} en el mes actual (acumulado hasta ahora): commits={}, pushes exitosos={}, pushes bloqueados={}.",
                user, commits, pushes, blocked_pushes
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
        Some(ChatQuery::UserPushesNoTicketWeek { ref user }) => {
            match state
                .db
                .chat_query_user_pushes_no_ticket_week(user, scoped_org_id.as_deref())
                .await
            {
                Ok(count) => {
                    let answer = if count == 0 {
                        let exists_in_scope = match user_exists_in_scope(
                            &state,
                            user,
                            scoped_org_id.as_deref(),
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "user_exists_in_scope(pushes_no_ticket) error: {}",
                                    e
                                );
                                false
                            }
                        };
                        if exists_in_scope {
                            format!(
                                "No encontré pushes a main sin ticket para {} en los últimos 7 días.",
                                user
                            )
                        } else {
                            format!(
                                "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                                user
                            )
                        }
                    } else {
                        format!(
                            "Encontré {} pushes a main sin ticket para {} en los últimos 7 días.",
                            count, user
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
                    tracing::error!("chat_query_user_pushes_no_ticket_week error: {}", e);
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
            }
        }
        Some(ChatQuery::UserBlockedPushesMonth { ref user }) => {
            match state
                .db
                .chat_query_user_blocked_pushes_month(user, scoped_org_id.as_deref())
                .await
            {
                Ok(count) => {
                    let answer = if count == 0 {
                        let exists_in_scope = match user_exists_in_scope(
                            &state,
                            user,
                            scoped_org_id.as_deref(),
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "user_exists_in_scope(blocked_pushes) error: {}",
                                    e
                                );
                                false
                            }
                        };
                        if exists_in_scope {
                            format!("{} no tiene pushes bloqueados en el mes actual.", user)
                        } else {
                            format!(
                                "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                                user
                            )
                        }
                    } else {
                        format!(
                            "{} tiene {} pushes bloqueados en el mes actual.",
                            user, count
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
                    tracing::error!("chat_query_user_blocked_pushes_month error: {}", e);
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
            }
        }
        _ => unreachable!("chat query routed to wrong handler"),
    }
}
