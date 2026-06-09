#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_commit_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::TotalCommitsCount) => {
            match state
                .db
                .chat_query_commits_count(None, None, scoped_org_id.as_deref())
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
                                "El Control Plane registra {} commits en el historial disponible para tu scope.",
                                count
                            ),
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
                    tracing::error!("chat_query_commits_count(total) error: {}", e);
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
        Some(ChatQuery::UserCommitsCount {
            ref user,
            start_ms,
            end_ms,
        }) => {
            match state
                .db
                .chat_query_user_commits_count(user, start_ms, end_ms, scoped_org_id.as_deref())
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
                                    "user_exists_in_scope(user_commits_count) error: {}",
                                    e
                                );
                                false
                            }
                        };
                        if exists_in_scope {
                            if start_ms.is_some() && end_ms.is_some() {
                                format!(
                                    "El usuario {} no tiene commits en el rango solicitado.",
                                    user
                                )
                            } else {
                                format!(
                                    "El usuario {} no tiene commits en el historial disponible para el scope activo.",
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
                            "El usuario {} ha realizado {} commits en el rango solicitado.",
                            user, count
                        )
                    } else {
                        format!(
                            "El usuario {} ha realizado {} commits en todo el historial.",
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
                    tracing::error!("chat_query_user_commits_count error: {}", e);
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
        Some(ChatQuery::UserLastCommit { ref user }) => {
            match state
                .db
                .chat_query_user_last_commit(user, scoped_org_id.as_deref())
                .await
            {
                Ok(Some(last_commit)) => {
                    let login = last_commit
                        .get("user_login")
                        .and_then(|v| v.as_str())
                        .unwrap_or(user.as_str());
                    let user_name = last_commit
                        .get("user_name")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| {
                            !v.is_empty()
                                && !v.eq_ignore_ascii_case("unknown")
                                && !v.eq_ignore_ascii_case("desconocido")
                        });
                    let branch = last_commit
                        .get("branch")
                        .and_then(|v| v.as_str())
                        .unwrap_or("desconocida");
                    let sha = last_commit
                        .get("commit_sha")
                        .and_then(|v| v.as_str())
                        .unwrap_or("desconocido");
                    let repo = last_commit
                        .get("repo_full_name")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| !v.is_empty());
                    let commit_message = last_commit
                        .get("commit_message")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| !v.is_empty());
                    let display_user = if let Some(name) = user_name {
                        if !name.eq_ignore_ascii_case(login) {
                            format!("{} ({})", login, name)
                        } else {
                            login.to_string()
                        }
                    } else {
                        login.to_string()
                    };
                    let repo_fragment = repo.map(|r| format!(" | Repo: {}", r)).unwrap_or_default();
                    let message_fragment = commit_message
                        .map(|m| format!(" | Mensaje: {}", m))
                        .unwrap_or_default();
                    let timestamp_ms = last_commit.get("timestamp").and_then(|v| v.as_i64());
                    let answer = if let Some(ts_ms) = timestamp_ms {
                        if let Some(dt_utc) =
                            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts_ms)
                        {
                            let lima_tz =
                                chrono::FixedOffset::west_opt(5 * 3600).unwrap_or_else(|| {
                                    chrono::FixedOffset::east_opt(0).expect("valid offset")
                                });
                            let dt_lima = dt_utc.with_timezone(&lima_tz);
                            format!(
                                "Último commit registrado para {}: {} en la rama `{}`{}{}. Fecha del evento: {} (America/Lima) | {} UTC.",
                                display_user,
                                sha,
                                branch,
                                repo_fragment,
                                message_fragment,
                                dt_lima.format("%Y-%m-%d %H:%M:%S"),
                                dt_utc.format("%Y-%m-%d %H:%M:%S")
                            )
                        } else {
                            format!(
                                "Último commit registrado para {}: {} en la rama `{}`{}{}. No pude convertir su timestamp a fecha legible.",
                                display_user, sha, branch, repo_fragment, message_fragment
                            )
                        }
                    } else {
                        format!(
                            "Último commit registrado para {}: {} en la rama `{}`{}{}. No hay timestamp disponible en el evento.",
                            display_user, sha, branch, repo_fragment, message_fragment
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
                                "client_events".to_string(),
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
                Ok(None) => {
                    let exists_in_scope =
                        match user_exists_in_scope(&state, user, scoped_org_id.as_deref()).await {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "user_exists_in_scope(user_last_commit) error: {}",
                                    e
                                );
                                false
                            }
                        };
                    let answer = if exists_in_scope {
                        format!(
                            "No encontré commits para {} en el historial disponible del scope activo.",
                            user
                        )
                    } else {
                        format!(
                            "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                            user
                        )
                    };
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::OK,
                        ChatAskResponse {
                            status: "insufficient_data".to_string(),
                            answer,
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec!["client_events".to_string(), "org_users".to_string()],

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
                    tracing::error!("chat_query_user_last_commit error: {}", e);
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
        Some(ChatQuery::UserCommitsRange {
            ref user,
            start_ms,
            end_ms,
        }) => {
            match state
                .db
                .chat_query_user_commits_range(user, start_ms, end_ms, scoped_org_id.as_deref())
                .await
            {
                Ok(rows) => {
                    let answer = if rows.is_empty() {
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
                                    "user_exists_in_scope(user_commits_range) error: {}",
                                    e
                                );
                                false
                            }
                        };
                        if exists_in_scope {
                            format!("No encontré commits de {} en el rango solicitado.", user)
                        } else {
                            format!(
                                "No encontré al usuario {} dentro del scope activo. Verifica login exacto y organización seleccionada.",
                                user
                            )
                        }
                    } else {
                        format!(
                            "Encontré {} commits de {} en el rango solicitado.",
                            rows.len(),
                            user
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
                    tracing::error!("chat_query_user_commits_range error: {}", e);
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
