#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_user_identity_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::UserAccessProfile { ref user }) => {
            match state
                .db
                .chat_query_user_access_profile(user, scoped_org_id.as_deref())
                .await
            {
                Ok(Some(profile)) => {
                    let login = profile
                        .get("login")
                        .and_then(|v| v.as_str())
                        .unwrap_or(user.as_str());
                    let role = profile
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("desconocido");
                    let status = profile
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("desconocido");
                    let has_active_api_key = profile
                        .get("has_active_api_key")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let answer = format!(
                        "Perfil de acceso de {}: rol={}, estado={}, API key activa={}. Nota: por seguridad no expongo valores de API key ni hashes.",
                        login,
                        role,
                        status,
                        if has_active_api_key { "sí" } else { "no" }
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
                            data_refs: vec!["org_users".to_string(), "api_keys".to_string()],

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
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::OK,
                        ChatAskResponse {
                            status: "insufficient_data".to_string(),
                            answer: format!(
                                "No encontré al usuario {} en el scope activo. Verifica el login exacto y la organización seleccionada.",
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
                Err(e) => {
                    tracing::error!("chat_query_user_access_profile error: {}", e);
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
        Some(ChatQuery::UserScopeClarification { ref user }) => {
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "insufficient_data".to_string(),
                    answer: format!(
                        "¿Qué métrica quieres para {}? Opciones directas: 1) commits en rango, 2) pushes bloqueados del mes, 3) pushes a main sin ticket (7d), 4) rol/estado de acceso (sin exponer API key).",
                        user
                    ),
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
        Some(ChatQuery::SessionCommitsCount { ref user }) => {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let start_ms = session.session_started_ms.max(0);
            let selected_user = user
                .clone()
                .or_else(|| session.slots.last_user_login.clone());
            if let Some(ref selected_user) = selected_user {
                match state
                    .db
                    .chat_query_user_commits_count(
                        selected_user,
                        Some(start_ms),
                        Some(now_ms),
                        scoped_org_id.as_deref(),
                    )
                    .await
                {
                    Ok(count) => {
                        let answer = format!(
                            "En esta sesión (desde {} UTC), el usuario {} ha realizado {} commits.",
                            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(start_ms)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "inicio no disponible".to_string()),
                            selected_user,
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
                    Err(e) => {
                        tracing::error!("chat_query_user_commits_count(session) error: {}", e);
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

            match state
                .db
                .chat_query_commits_count(Some(start_ms), Some(now_ms), scoped_org_id.as_deref())
                .await
            {
                Ok(count) => {
                    let answer = format!(
                        "En esta sesión (desde {} UTC) hay {} commits registrados en el Control Plane.",
                        chrono::DateTime::<chrono::Utc>::from_timestamp_millis(start_ms)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "inicio no disponible".to_string()),
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
                Err(e) => {
                    tracing::error!("chat_query_commits_count(session) error: {}", e);
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
