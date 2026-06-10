pub async fn chat_ask(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatAskRequest>,
) -> impl IntoResponse {
    let chat_allowed = matches!(
        auth_user.role,
        UserRole::Admin | UserRole::Architect | UserRole::PM
    );
    if !chat_allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(ChatAskResponse {
                status: "error".to_string(),
                answer: "Admin, Architect, or PM access required".to_string(),
                missing_capability: None,
                can_report_feature: false,
                data_refs: vec![],
                sources: vec![],
                entities_detected: vec![],
                time_range_used: None,
                actions_recommended: vec![],
                confidence: None,
                trace_id: None,
            }),
        );
    }

    let question = payload.question.trim().to_string();
    if question.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatAskResponse {
                status: "error".to_string(),
                answer: "La pregunta no puede estar vacía".to_string(),
                missing_capability: None,
                can_report_feature: false,
                data_refs: vec![],
                sources: vec![],
                entities_detected: vec![],
                time_range_used: None,
                actions_recommended: vec![],
                confidence: None,
                trace_id: None,
            }),
        );
    }

    let org_name = payload.org_name.as_deref();
    let scoped_org_id =
        match resolve_and_check_org_scope(&state, auth_user.org_id.as_deref(), org_name, true)
            .await
        {
            Ok(org_id) => org_id,
            Err(err) => {
                let error = match err {
                    OrgScopeError::BadRequest => "org_name is required",
                    OrgScopeError::NotFound => "Organization not found",
                    OrgScopeError::Forbidden => "Requested org is outside API key scope",
                    OrgScopeError::Internal => "Internal database error",
                };
                return (
                    org_scope_status(err),
                    Json(ChatAskResponse {
                        status: "error".to_string(),
                        answer: error.to_string(),
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec![],
                        sources: vec![],
                        entities_detected: vec![],
                        time_range_used: None,
                        actions_recommended: vec![],
                        confidence: None,
                        trace_id: None,
                    }),
                );
            }
        };

    let conversation_key = build_conversation_key(&auth_user, scoped_org_id.as_deref());
    let mut session = load_conversation_state(&state, &conversation_key);
    ensure_session_initialized(&mut session);
    let nlp = analyze_nlp(&question, &session);
    let safe_question_for_state = sanitize_chat_answer_text(&question);
    push_turn(
        &mut session,
        "user",
        &safe_question_for_state,
        nlp.intent.as_str(),
    );
    update_slots_from_nlp(&mut session, &nlp, org_name);

    let mut snapshot_refs =
        refresh_project_snapshot_if_stale(&state, &mut session, scoped_org_id.as_deref()).await;
    let proactive_todos = apply_proactive_todos_from_snapshot(&mut session);

    if is_short_circuit_intent(nlp.intent) {
        return handle_short_circuit_intent(&state, &conversation_key, &mut session, &nlp);
    }

    if is_secret_exfiltration_request(&question) {
        return finalize_chat_response(
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            StatusCode::OK,
            ChatAskResponse {
                status: "ok".to_string(),
                answer: "No puedo revelar API keys, tokens ni secretos de ningún usuario. Por seguridad solo puedo ayudarte con estado de acceso (rol, miembro activo/inactivo, y si existe key activa) sin mostrar valores sensibles.".to_string(),
                missing_capability: None,
                can_report_feature: false,
                data_refs: vec!["security_policy".to_string()],
                sources: vec![],
                entities_detected: vec![],
                time_range_used: None,
                actions_recommended: vec![],
                confidence: None,
                trace_id: None,
            },
        );
    }

    let mut query = detect_query(&question);
    if let Some(ChatQuery::NeedUserForCommitHistory) = query {
        if let Some(ref remembered_user) = session.slots.last_user_login {
            query = Some(ChatQuery::UserCommitsCount {
                user: remembered_user.clone(),
                start_ms: None,
                end_ms: None,
            });
        }
    }
    if query.is_none() {
        let q = question.to_lowercase();
        if (q.contains("todo el historial") || q.contains("all history"))
            && !q.contains("commit")
            && !q.contains("commits")
        {
            if let Some(ref remembered_user) = session.slots.last_user_login {
                query = Some(ChatQuery::UserCommitsCount {
                    user: remembered_user.clone(),
                    start_ms: None,
                    end_ms: None,
                });
            }
        }
        if let Some(ref remembered_user) = session.slots.last_user_login {
            if (q.contains("rol")
                || q.contains("role")
                || q.contains("api key")
                || q.contains("apikey"))
                && !q.contains("usuario")
                && !q.contains("user ")
            {
                query = Some(ChatQuery::UserAccessProfile {
                    user: remembered_user.clone(),
                });
            } else if (q.contains("bloqueado") || q.contains("blocked"))
                && (q.contains("push") || q.contains("pushes"))
                && !q.contains("usuario")
                && !q.contains("user ")
            {
                query = Some(ChatQuery::UserBlockedPushesMonth {
                    user: remembered_user.clone(),
                });
            } else if (q.contains("push") || q.contains("pushes"))
                && (q.contains("ticket")
                    || q.contains("jira")
                    || q.contains("sin ticket")
                    || q.contains("without ticket"))
                && !q.contains("usuario")
                && !q.contains("user ")
            {
                query = Some(ChatQuery::UserPushesNoTicketWeek {
                    user: remembered_user.clone(),
                });
            } else if (q.contains("push") || q.contains("pushes"))
                && (q.contains("cuanto")
                    || q.contains("cuánto")
                    || q.contains("how many")
                    || q.contains("total"))
                && !q.contains("bloqueado")
                && !q.contains("blocked")
                && !q.contains("ticket")
                && !q.contains("jira")
                && !q.contains("usuario")
                && !q.contains("user ")
            {
                query = Some(ChatQuery::UserPushesCount {
                    user: remembered_user.clone(),
                    start_ms: None,
                    end_ms: None,
                });
            }
        }
    }

    let founder_scope_exception = is_founder_scope_exception(&auth_user);
    if auth_user.org_id.is_none() && scoped_org_id.is_none() && !founder_scope_exception {
        if let Some(ref q) = query {
            if query_needs_explicit_org_scope(q) {
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut session,
                    &nlp,
                    StatusCode::OK,
                    ChatAskResponse {
                        status: "insufficient_data".to_string(),
                        answer: if nlp.entities.language == "en" {
                            "This query needs an organization scope. Select or provide `org_name` first to avoid cross-org ambiguity.".to_string()
                        } else {
                            "Esta consulta requiere un scope de organización. Selecciona o envía `org_name` primero para evitar ambigüedad entre organizaciones.".to_string()
                        },
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec!["org_scope".to_string()],
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

    if matches!(
        &query,
        Some(ChatQuery::Greeting)
            | Some(ChatQuery::DateMismatchClarification)
            | Some(ChatQuery::CurrentDateTime)
            | Some(ChatQuery::CapabilityOverview)
            | Some(ChatQuery::ControlPlaneExecutiveSummary)
    ) {
        return handle_intro_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
            &auth_user,
            &proactive_todos,
            &snapshot_refs,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::TicketsReleasedWithNonGreenQualityGate { .. })
            | Some(ChatQuery::TicketsWithNonGreenQualityGate { .. })
            | Some(ChatQuery::DevelopersWithNonGreenQualityGate { .. })
            | Some(ChatQuery::QualityGateTopFailingRepos { .. })
            | Some(ChatQuery::QualityGateTopFailingBranches { .. })
            | Some(ChatQuery::QualityGateHealthWindow { .. })
    ) {
        return handle_quality_gate_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::ReleaseReadinessTopFailingRepos { .. })
            | Some(ChatQuery::ReleaseReadinessTopFailingBranches { .. })
            | Some(ChatQuery::ReleaseReadinessHealthWindow { .. })
    ) {
        return handle_release_readiness_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::OnlineDevelopersNow { .. })
            | Some(ChatQuery::CommitsWithoutTicketWindow { .. })
            | Some(ChatQuery::NeedUserForCommitHistory)
            | Some(ChatQuery::GuidedHelp)
            | Some(ChatQuery::PushesNoTicket)
            | Some(ChatQuery::BlockedPushesMonth)
    ) {
        return handle_operational_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
            &question,
            &proactive_todos,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::UserPushesCount { .. })
            | Some(ChatQuery::UserActivityMonth { .. })
            | Some(ChatQuery::UserPushesNoTicketWeek { .. })
            | Some(ChatQuery::UserBlockedPushesMonth { .. })
    ) {
        return handle_user_push_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::UserAccessProfile { .. })
            | Some(ChatQuery::UserScopeClarification { .. })
            | Some(ChatQuery::SessionCommitsCount { .. })
    ) {
        return handle_user_identity_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
        )
        .await;
    }
    if matches!(
        &query,
        Some(ChatQuery::TotalCommitsCount)
            | Some(ChatQuery::UserCommitsCount { .. })
            | Some(ChatQuery::UserLastCommit { .. })
            | Some(ChatQuery::UserCommitsRange { .. })
    ) {
        return handle_commit_query(
            query,
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            &scoped_org_id,
        )
        .await;
    }

    if is_logs_precision_query(&question) {
        let mut filter = EventFilter {
            limit: extract_logs_limit(&question, 5, 20),
            ..EventFilter::default()
        };
        filter.org_id = scoped_org_id.clone();
        filter.org_name = None;
        filter.user_login = nlp.entities.user_login.clone();
        filter.event_type = extract_logs_event_type_hint(&question);

        match state.db.get_combined_events(&filter).await {
            Ok(events) => {
                if events.is_empty() {
                    let answer = if nlp.entities.language == "en" {
                        "I did not find log events for the requested scope/filters. Provide org/user/event_type or a narrower time window.".to_string()
                    } else {
                        "No encontre eventos de log para el scope/filtros solicitados. Indica org/usuario/tipo de evento o una ventana de tiempo mas acotada.".to_string()
                    };
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut session,
                        &nlp,
                        StatusCode::OK,
                        ChatAskResponse {
                            status: "insufficient_data".to_string(),
                            answer,
                            missing_capability: None,
                            can_report_feature: false,
                            data_refs: vec![
                                "logs_endpoint".to_string(),
                                "deterministic_sql_results".to_string(),
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

                let answer = render_precise_logs_answer(&events, &nlp.entities.language);
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut session,
                    &nlp,
                    StatusCode::OK,
                    ChatAskResponse {
                        status: "ok".to_string(),
                        answer,
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec![
                            "logs_endpoint".to_string(),
                            "deterministic_sql_results".to_string(),
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
                tracing::error!("deterministic logs answer error: {}", e);
                return finalize_chat_response(
                    &state,
                    &conversation_key,
                    &mut session,
                    &nlp,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ChatAskResponse {
                        status: "error".to_string(),
                        answer: "Error consultando logs exactos en la base de datos".to_string(),
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec!["logs_endpoint".to_string()],
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

    if let Some(answer) = build_grounded_knowledge_answer(&question, &nlp.entities.language) {
        return finalize_chat_response(
            &state,
            &conversation_key,
            &mut session,
            &nlp,
            StatusCode::OK,
            ChatAskResponse {
                status: "ok".to_string(),
                answer,
                missing_capability: None,
                can_report_feature: false,
                data_refs: vec!["project_docs_kb".to_string(), "web_docs_faq".to_string()],

                sources: vec![],
                entities_detected: vec![],
                time_range_used: None,
                actions_recommended: vec![],
                confidence: None,
                trace_id: None,
            },
        );
    }

    let Some(api_key) = state.llm_api_key.as_deref() else {
        tracing::warn!("GEMINI_API_KEY not configured; returning feature_not_available");
        return finalize_chat_response(&state, &conversation_key, &mut session, &nlp,
            StatusCode::OK,
            ChatAskResponse {
                status: "feature_not_available".to_string(),
                answer: "El asistente conversacional no está configurado en este servidor. Configura GEMINI_API_KEY para activarlo.".to_string(),
                missing_capability: Some("llm_integration".to_string()),
                can_report_feature: true,
                data_refs: vec![],
                sources: vec![],
                entities_detected: vec![],
                time_range_used: None,
                actions_recommended: vec![],
                confidence: None,
                trace_id: None,
            },
        );
    };

    let mut data_refs = vec![
        "project_docs_kb".to_string(),
        "web_docs_faq".to_string(),
        "conversation_context".to_string(),
        "todo_runtime".to_string(),
    ];
    data_refs.append(&mut snapshot_refs);

    let queue_timeout = Duration::from_millis(state.chat_llm_queue_timeout_ms);
    let llm_timeout = Duration::from_millis(state.chat_llm_timeout_ms);
    let permit = match timeout(
        queue_timeout,
        state.chat_llm_semaphore.clone().acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => permit,
        Ok(Err(e)) => {
            tracing::error!("chat llm semaphore acquire failed: {}", e);
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut session,
                &nlp,
                StatusCode::SERVICE_UNAVAILABLE,
                ChatAskResponse {
                    status: "error".to_string(),
                    answer: if nlp.entities.language == "en" {
                        "Chat is temporarily unavailable due to internal capacity controls. Try again in a few seconds.".to_string()
                    } else {
                        "El chat está temporalmente no disponible por control interno de capacidad. Intenta de nuevo en unos segundos.".to_string()
                    },
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs,
                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        Err(_) => {
            tracing::warn!(
                queue_timeout_ms = state.chat_llm_queue_timeout_ms,
                "chat request rejected due to LLM queue timeout"
            );
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut session,
                &nlp,
                StatusCode::TOO_MANY_REQUESTS,
                ChatAskResponse {
                    status: "error".to_string(),
                    answer: if nlp.entities.language == "en" {
                        "Chat is busy right now. Try again in a few seconds.".to_string()
                    } else {
                        "El chat está ocupado en este momento. Intenta de nuevo en unos segundos."
                            .to_string()
                    },
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs,
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

    let llm_question = sanitize_chat_answer_text(&question);
    let data = build_advanced_conversation_payload(&llm_question, &nlp, &session);
    let llm_result = timeout(
        llm_timeout,
        call_llm(
            &state.http_client,
            api_key,
            &state.llm_model,
            &llm_question,
            &data,
        ),
    )
    .await;
    drop(permit);

    match llm_result {
        Ok(Ok(mut resp)) => {
            resp = normalize_llm_response(resp, &question, &nlp.entities.language);
            // Track whether the answer text is the model's own output. When the
            // KB override replaces it, the answer is grounded in project docs,
            // not LLM-generated.
            let mut answer_is_llm_generated = true;
            if should_override_llm_answer_with_kb(&resp, &question) {
                if let Some(answer) =
                    build_grounded_knowledge_answer(&question, &nlp.entities.language)
                {
                    resp.status = "ok".to_string();
                    resp.answer = answer;
                    resp.missing_capability = None;
                    resp.can_report_feature = false;
                    resp.data_refs.push("project_docs_kb".to_string());
                    resp.data_refs.push("web_docs_faq".to_string());
                    answer_is_llm_generated = false;
                }
            }
            // The LLM must never claim deterministic/server-only provenance.
            let llm_refs = sanitize_llm_data_refs(
                std::mem::take(&mut resp.data_refs),
                answer_is_llm_generated,
            );
            let mut refs = data_refs.clone();
            refs.extend(llm_refs);
            refs.sort();
            refs.dedup();
            resp.data_refs = refs;
            finalize_chat_response(
                &state,
                &conversation_key,
                &mut session,
                &nlp,
                StatusCode::OK,
                resp,
            )
        }
        Ok(Err(e)) => {
            tracing::error!("LLM call failed: {}", e);
            let answer = llm_degraded_answer(&question, &nlp.entities.language);
            finalize_chat_response(
                &state,
                &conversation_key,
                &mut session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs,

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            )
        }
        Err(_) => {
            tracing::warn!(
                llm_timeout_ms = state.chat_llm_timeout_ms,
                "chat request exceeded LLM timeout"
            );
            let answer = llm_degraded_answer(&question, &nlp.entities.language);
            finalize_chat_response(
                &state,
                &conversation_key,
                &mut session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer,
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs,

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            )
        }
    }
}
