#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_quality_gate_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::TicketsReleasedWithNonGreenQualityGate { hours, limit }) => {
            match state
                .db
                .chat_query_tickets_released_with_non_green_quality_gate(
                    scoped_org_id.as_deref(),
                    hours,
                    limit,
                )
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::OK,
                            ChatAskResponse {
                                status: "ok".to_string(),
                                answer: format!(
                                    "No se detectaron tickets desplegados con quality gate no verde en las últimas {}h.",
                                    hours
                                ),
                                missing_capability: None,
                                can_report_feature: false,
                                data_refs: vec![
                                    "pipeline_events".to_string(),
                                    "commit_ticket_correlations".to_string(),
                                ],
                                sources: vec![],
                                entities_detected: vec![],
                                time_range_used: Some(format!("last_{}h", hours)),
                                actions_recommended: vec![],
                                confidence: Some(0.8),
                                trace_id: None,
                            },
                        );
                    }

                    let mut lines = Vec::with_capacity(rows.len());
                    for (idx, item) in rows.iter().enumerate() {
                        let ticket_id = item
                            .get("ticket_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let non_green_runs = item
                            .get("non_green_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let successful_release_runs = item
                            .get("successful_release_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        lines.push(format!(
                            "{}. {} -> releases: {}, non_green_runs: {}",
                            idx + 1,
                            ticket_id,
                            successful_release_runs,
                            non_green_runs
                        ));
                    }

                    let answer = format!(
                        "Top tickets desplegados con quality gate no verde (últimas {hours}h, top {limit}):\n{lines}",
                        hours = hours,
                        limit = limit,
                        lines = lines.join("\n")
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
                                "pipeline_events".to_string(),
                                "commit_ticket_correlations".to_string(),
                            ],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: vec![
                                "Revisar riesgo residual de tickets ya desplegados con gate no verde".to_string(),
                                "Priorizar remediación y seguimiento en próximos sprints".to_string(),
                            ],
                            confidence: Some(0.89),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "chat_query_tickets_released_with_non_green_quality_gate error: {}",
                        e
                    );
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer:
                                "Error consultando tickets desplegados con quality gate no verde"
                                    .to_string(),
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
        Some(ChatQuery::TicketsWithNonGreenQualityGate { hours, limit }) => {
            match state
                .db
                .chat_query_tickets_with_non_green_quality_gate(
                    scoped_org_id.as_deref(),
                    hours,
                    limit,
                )
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::OK,
                            ChatAskResponse {
                                status: "ok".to_string(),
                                answer: format!(
                                    "No se detectaron tickets asociados a quality gate no verde en las últimas {}h.",
                                    hours
                                ),
                                missing_capability: None,
                                can_report_feature: false,
                                data_refs: vec![
                                    "pipeline_events".to_string(),
                                    "commit_ticket_correlations".to_string(),
                                ],
                                sources: vec![],
                                entities_detected: vec![],
                                time_range_used: Some(format!("last_{}h", hours)),
                                actions_recommended: vec![],
                                confidence: Some(0.8),
                                trace_id: None,
                            },
                        );
                    }

                    let mut lines = Vec::with_capacity(rows.len());
                    for (idx, item) in rows.iter().enumerate() {
                        let ticket_id = item
                            .get("ticket_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let non_green_runs = item
                            .get("non_green_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let repos_affected = item
                            .get("repos_affected")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let commits_affected = item
                            .get("commits_affected")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        lines.push(format!(
                            "{}. {} -> non_green_runs: {}, repos: {}, commits: {}",
                            idx + 1,
                            ticket_id,
                            non_green_runs,
                            repos_affected,
                            commits_affected
                        ));
                    }

                    let answer = format!(
                        "Top tickets con quality gate no verde (últimas {hours}h, top {limit}):\n{lines}",
                        hours = hours,
                        limit = limit,
                        lines = lines.join("\n")
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
                                "pipeline_events".to_string(),
                                "commit_ticket_correlations".to_string(),
                            ],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: vec![
                                "Priorizar tickets con mayor volumen de quality gate no verde"
                                    .to_string(),
                                "Validar excepciones activas antes de release/merge".to_string(),
                            ],
                            confidence: Some(0.9),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "chat_query_tickets_with_non_green_quality_gate error: {}",
                        e
                    );
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer:
                                "Error consultando ranking de tickets con quality gate no verde"
                                    .to_string(),
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
        Some(ChatQuery::DevelopersWithNonGreenQualityGate { hours, limit }) => {
            match state
                .db
                .chat_query_developers_with_non_green_quality_gate(
                    scoped_org_id.as_deref(),
                    hours,
                    limit,
                )
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::OK,
                            ChatAskResponse {
                                status: "ok".to_string(),
                                answer: format!(
                                    "No se detectaron developers/equipos con quality gate no verde en las últimas {}h.",
                                    hours
                                ),
                                missing_capability: None,
                                can_report_feature: false,
                                data_refs: vec!["pipeline_events".to_string()],
                                sources: vec![],
                                entities_detected: vec![],
                                time_range_used: Some(format!("last_{}h", hours)),
                                actions_recommended: vec![],
                                confidence: Some(0.8),
                                trace_id: None,
                            },
                        );
                    }

                    let mut lines = Vec::with_capacity(rows.len());
                    for (idx, item) in rows.iter().enumerate() {
                        let actor_login = item
                            .get("actor_login")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let non_green_runs = item
                            .get("non_green_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let repos_affected = item
                            .get("repos_affected")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let commits_affected = item
                            .get("commits_affected")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        lines.push(format!(
                            "{}. {} -> non_green_runs: {}, repos: {}, commits: {}",
                            idx + 1,
                            actor_login,
                            non_green_runs,
                            repos_affected,
                            commits_affected
                        ));
                    }

                    let answer = format!(
                        "Top developers/equipos con quality gate no verde (últimas {hours}h, top {limit}):\n{lines}",
                        hours = hours,
                        limit = limit,
                        lines = lines.join("\n")
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
                            data_refs: vec!["pipeline_events".to_string()],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: vec![
                                "Revisar coaching técnico en los equipos con mayor non_green"
                                    .to_string(),
                                "Endurecer quality gate en repos críticos con alta recurrencia"
                                    .to_string(),
                            ],
                            confidence: Some(0.88),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "chat_query_developers_with_non_green_quality_gate error: {}",
                        e
                    );
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer:
                                "Error consultando ranking de developers con quality gate no verde"
                                    .to_string(),
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
        Some(ChatQuery::QualityGateTopFailingRepos { hours, limit }) => {
            match state
                .db
                .chat_query_quality_gate_top_failing_repos(scoped_org_id.as_deref(), hours, limit)
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::OK,
                            ChatAskResponse {
                                status: "ok".to_string(),
                                answer: format!(
                                    "No se detectaron repos con quality gate no verde en las últimas {}h.",
                                    hours
                                ),
                                missing_capability: None,
                                can_report_feature: false,
                                data_refs: vec!["pipeline_events".to_string()],
                                sources: vec![],
                                entities_detected: vec![],
                                time_range_used: Some(format!("last_{}h", hours)),
                                actions_recommended: vec![],
                                confidence: Some(0.8),
                                trace_id: None,
                            },
                        );
                    }

                    let mut lines = Vec::with_capacity(rows.len());
                    for (idx, item) in rows.iter().enumerate() {
                        let repo = item
                            .get("repo_full_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let non_green_runs = item
                            .get("non_green_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let total_runs =
                            item.get("total_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let non_green_pct = item
                            .get("non_green_pct")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        lines.push(format!(
                            "{}. {} -> non_green: {}/{} ({:.1}%)",
                            idx + 1,
                            repo,
                            non_green_runs,
                            total_runs,
                            non_green_pct
                        ));
                    }

                    let answer = format!(
                        "Top repos con quality gate no verde (últimas {hours}h, top {limit}):\n{lines}",
                        hours = hours,
                        limit = limit,
                        lines = lines.join("\n")
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
                            data_refs: vec!["pipeline_events".to_string()],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: vec![
                                "Priorizar remediación en los repos con mayor volumen non_green".to_string(),
                                "Revisar causas raíz en quality gate (coverage, bugs, vulnerabilidades)".to_string(),
                            ],
                            confidence: Some(0.9),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("chat_query_quality_gate_top_failing_repos error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando ranking de repos con quality gate no verde"
                                .to_string(),
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
        Some(ChatQuery::QualityGateTopFailingBranches { hours, limit }) => {
            match state
                .db
                .chat_query_quality_gate_top_failing_branches(
                    scoped_org_id.as_deref(),
                    hours,
                    limit,
                )
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        return finalize_chat_response(
                            &state,
                            &conversation_key,
                            &mut *session,
                            &nlp,
                            StatusCode::OK,
                            ChatAskResponse {
                                status: "ok".to_string(),
                                answer: format!(
                                    "No se detectaron ramas con quality gate no verde en las últimas {}h.",
                                    hours
                                ),
                                missing_capability: None,
                                can_report_feature: false,
                                data_refs: vec!["pipeline_events".to_string()],
                                sources: vec![],
                                entities_detected: vec![],
                                time_range_used: Some(format!("last_{}h", hours)),
                                actions_recommended: vec![],
                                confidence: Some(0.8),
                                trace_id: None,
                            },
                        );
                    }

                    let mut lines = Vec::with_capacity(rows.len());
                    for (idx, item) in rows.iter().enumerate() {
                        let branch_name = item
                            .get("branch_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let non_green_runs = item
                            .get("non_green_runs")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let total_runs =
                            item.get("total_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let non_green_pct = item
                            .get("non_green_pct")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        lines.push(format!(
                            "{}. {} -> non_green: {}/{} ({:.1}%)",
                            idx + 1,
                            branch_name,
                            non_green_runs,
                            total_runs,
                            non_green_pct
                        ));
                    }

                    let answer = format!(
                        "Top ramas con quality gate no verde (últimas {hours}h, top {limit}):\n{lines}",
                        hours = hours,
                        limit = limit,
                        lines = lines.join("\n")
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
                            data_refs: vec!["pipeline_events".to_string()],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: vec![
                                "Priorizar estabilización en ramas con mayor volumen non_green"
                                    .to_string(),
                                "Revisar calidad de cambios por rama antes de promover merge"
                                    .to_string(),
                            ],
                            confidence: Some(0.89),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("chat_query_quality_gate_top_failing_branches error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando ranking de ramas con quality gate no verde"
                                .to_string(),
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
        Some(ChatQuery::QualityGateHealthWindow { hours }) => {
            match state
                .db
                .chat_query_quality_gate_window_summary(scoped_org_id.as_deref(), hours)
                .await
            {
                Ok(summary) => {
                    let total_runs = summary
                        .get("total_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let green_runs = summary
                        .get("green_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let non_green_runs = summary
                        .get("non_green_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let repos_affected = summary
                        .get("repos_affected")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let commits_affected = summary
                        .get("commits_affected")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let policy_violation_signals = summary
                        .get("policy_violation_signals")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    let green_rate = if total_runs > 0 {
                        (green_runs as f64 / total_runs as f64) * 100.0
                    } else {
                        0.0
                    };
                    let non_green_rate = if total_runs > 0 {
                        (non_green_runs as f64 / total_runs as f64) * 100.0
                    } else {
                        0.0
                    };

                    let answer = format!(
                        "Resumen quality gate (últimas {hours}h)\n\
Runs con señal quality_gate: {total_runs}\n\
Gate verde: {green_runs} ({green_rate:.1}%)\n\
Gate no verde: {non_green_runs} ({non_green_rate:.1}%)\n\
Repos afectados: {repos_affected}\n\
Commits afectados: {commits_affected}\n\
Signals policy_violation (quality_gate_green, no resueltas): {policy_violation_signals}",
                        hours = hours,
                        total_runs = total_runs,
                        green_runs = green_runs,
                        green_rate = green_rate,
                        non_green_runs = non_green_runs,
                        non_green_rate = non_green_rate,
                        repos_affected = repos_affected,
                        commits_affected = commits_affected,
                        policy_violation_signals = policy_violation_signals
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
                                "pipeline_events".to_string(),
                                "noncompliance_signals".to_string(),
                            ],
                            sources: vec![],
                            entities_detected: vec![],
                            time_range_used: Some(format!("last_{}h", hours)),
                            actions_recommended: if non_green_runs > 0 {
                                vec![
                                    "Revisar pipelines con stage quality_gate no verde".to_string(),
                                    "Validar excepciones activas y expiración de overrides"
                                        .to_string(),
                                ]
                            } else {
                                vec![]
                            },
                            confidence: Some(if total_runs > 0 { 0.92 } else { 0.75 }),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("chat_query_quality_gate_window_summary error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando health de quality gate".to_string(),
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
