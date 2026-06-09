fn is_short_circuit_intent(intent: NlpIntent) -> bool {
    matches!(
        intent,
        NlpIntent::TodoAdd
            | NlpIntent::TodoList
            | NlpIntent::TodoComplete
            | NlpIntent::FeedbackPositive
            | NlpIntent::FeedbackNegative
            | NlpIntent::Farewell
    )
}

#[allow(clippy::needless_return, clippy::needless_borrow)]
fn handle_short_circuit_intent(
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
) -> (StatusCode, Json<ChatAskResponse>) {
    match nlp.intent {
        NlpIntent::TodoAdd => {
            let text = nlp
                .entities
                .todo_text
                .clone()
                .unwrap_or_else(|| "Tarea pendiente sin descripción".to_string());
            let task = add_todo(&mut *session, &text, "user_request", "medium");
            let answer = format!(
                "Listo. Registré la tarea #{}: {}. Puedes pedirme \"mis tareas\" para ver pendientes.",
                task.id, task.text
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
                    data_refs: vec!["assistant_runtime".to_string(), "todo_runtime".to_string()],

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        NlpIntent::TodoList => {
            let answer = render_todo_list(&session, &nlp.entities.language);
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
                    data_refs: vec!["assistant_runtime".to_string(), "todo_runtime".to_string()],

                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                },
            );
        }
        NlpIntent::TodoComplete => {
            let response = if let Some(todo_id) = nlp.entities.todo_id {
                if let Some(task) = complete_todo(&mut *session, todo_id) {
                    ChatAskResponse {
                        status: "ok".to_string(),
                        answer: format!("Tarea #{} completada: {}", task.id, task.text),
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec![
                            "assistant_runtime".to_string(),
                            "todo_runtime".to_string(),
                        ],
                        sources: vec![],
                        entities_detected: vec![],
                        time_range_used: None,
                        actions_recommended: vec![],
                        confidence: None,
                        trace_id: None,
                    }
                } else {
                    ChatAskResponse {
                        status: "insufficient_data".to_string(),
                        answer: format!(
                            "No encontré una tarea pendiente con id #{}. Usa \"mis tareas\" para ver IDs válidos.",
                            todo_id
                        ),
                        missing_capability: None,
                        can_report_feature: false,
                        data_refs: vec!["todo_runtime".to_string()],
                        sources: vec![],
                        entities_detected: vec![],
                        time_range_used: None,
                        actions_recommended: vec![],
                        confidence: None,
                        trace_id: None,
                    }
                }
            } else {
                ChatAskResponse {
                    status: "insufficient_data".to_string(),
                    answer: "Indica el id de la tarea a completar. Ejemplo: \"completa tarea 3\"."
                        .to_string(),
                    missing_capability: None,
                    can_report_feature: false,
                    data_refs: vec!["todo_runtime".to_string()],
                    sources: vec![],
                    entities_detected: vec![],
                    time_range_used: None,
                    actions_recommended: vec![],
                    confidence: None,
                    trace_id: None,
                }
            };
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                response,
            );
        }
        NlpIntent::FeedbackPositive => {
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer: if nlp.entities.language == "en" {
                        "Great. I will keep this response style for the next interactions."
                            .to_string()
                    } else {
                        "Perfecto. Mantendré este estilo de respuesta en las próximas interacciones.".to_string()
                    },
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
        NlpIntent::FeedbackNegative => {
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer: if nlp.entities.language == "en" {
                        "Understood. I will answer with more precision and concrete steps from now on.".to_string()
                    } else {
                        "Entendido. Voy a responder con más precisión y pasos concretos desde ahora.".to_string()
                    },
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
        NlpIntent::Farewell => {
            return finalize_chat_response(
                &state,
                &conversation_key,
                &mut *session,
                &nlp,
                StatusCode::OK,
                ChatAskResponse {
                    status: "ok".to_string(),
                    answer: farewell_answer(&nlp.entities.language),
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
        _ => unreachable!("chat intent routed to wrong handler"),
    }
}
