#[allow(
    clippy::needless_return,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::ptr_arg
)]
async fn handle_release_readiness_query(
    query: Option<ChatQuery>,
    state: &Arc<AppState>,
    conversation_key: &str,
    session: &mut ConversationState,
    nlp: &NlpAnalysis,
    scoped_org_id: &Option<String>,
) -> (StatusCode, Json<ChatAskResponse>) {
    match query {
        Some(ChatQuery::ReleaseReadinessTopFailingRepos { hours, limit }) => {
            match state
                .db
                .chat_query_release_readiness_top_failing_repos(
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
                                    "No se detectaron repos con release readiness FAIL en las últimas {}h.",
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
                        let fail_runs = item.get("fail_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let total_runs =
                            item.get("total_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let fail_pct = item.get("fail_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        lines.push(format!(
                            "{}. {} -> fail: {}/{} ({:.1}%)",
                            idx + 1,
                            repo,
                            fail_runs,
                            total_runs,
                            fail_pct
                        ));
                    }

                    let answer = format!(
                        "Top repos con release readiness FAIL (últimas {hours}h, top {limit}):\n{lines}",
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
                                "Priorizar repos con mayor tasa de FAIL en release readiness"
                                    .to_string(),
                                "Revisar reasons/warnings del stage release_readiness en Jenkins"
                                    .to_string(),
                            ],
                            confidence: Some(0.89),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "chat_query_release_readiness_top_failing_repos error: {}",
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
                            answer: "Error consultando ranking de repos con release readiness FAIL"
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
        Some(ChatQuery::ReleaseReadinessTopFailingBranches { hours, limit }) => {
            match state
                .db
                .chat_query_release_readiness_top_failing_branches(
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
                                    "No se detectaron ramas con release readiness FAIL en las últimas {}h.",
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
                        let fail_runs = item.get("fail_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let total_runs =
                            item.get("total_runs").and_then(|v| v.as_i64()).unwrap_or(0);
                        let fail_pct = item.get("fail_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        lines.push(format!(
                            "{}. {} -> fail: {}/{} ({:.1}%)",
                            idx + 1,
                            branch_name,
                            fail_runs,
                            total_runs,
                            fail_pct
                        ));
                    }

                    let answer = format!(
                        "Top ramas con release readiness FAIL (últimas {hours}h, top {limit}):\n{lines}",
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
                                "Priorizar estabilización en ramas con mayor ratio de FAIL"
                                    .to_string(),
                                "Revisar reglas por tier antes de promover merges desde esas ramas"
                                    .to_string(),
                            ],
                            confidence: Some(0.88),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "chat_query_release_readiness_top_failing_branches error: {}",
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
                            answer: "Error consultando ranking de ramas con release readiness FAIL"
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
        Some(ChatQuery::ReleaseReadinessHealthWindow { hours }) => {
            match state
                .db
                .chat_query_release_readiness_window_summary(scoped_org_id.as_deref(), hours)
                .await
            {
                Ok(summary) => {
                    let total_runs = summary
                        .get("total_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let pass_runs = summary
                        .get("pass_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let warn_runs = summary
                        .get("warn_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let fail_runs = summary
                        .get("fail_runs")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let other_runs = summary
                        .get("other_runs")
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

                    let pass_rate = if total_runs > 0 {
                        (pass_runs as f64 / total_runs as f64) * 100.0
                    } else {
                        0.0
                    };
                    let fail_rate = if total_runs > 0 {
                        (fail_runs as f64 / total_runs as f64) * 100.0
                    } else {
                        0.0
                    };

                    let answer = format!(
                        "Resumen release readiness gate (últimas {hours}h)\n\
Runs con stage release_readiness: {total_runs}\n\
PASS: {pass_runs} ({pass_rate:.1}%)\n\
WARN: {warn_runs}\n\
FAIL: {fail_runs} ({fail_rate:.1}%)\n\
Otros estados: {other_runs}\n\
Repos afectados: {repos_affected}\n\
Commits afectados: {commits_affected}",
                        hours = hours,
                        total_runs = total_runs,
                        pass_runs = pass_runs,
                        pass_rate = pass_rate,
                        warn_runs = warn_runs,
                        fail_runs = fail_runs,
                        fail_rate = fail_rate,
                        other_runs = other_runs,
                        repos_affected = repos_affected,
                        commits_affected = commits_affected
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
                            actions_recommended: if fail_runs > 0 {
                                vec![
                                    "Revisar ramas con release readiness FAIL y razones en stage payload".to_string(),
                                    "Ajustar thresholds/tier si el ruido operativo supera el baseline".to_string(),
                                ]
                            } else {
                                vec![]
                            },
                            confidence: Some(if total_runs > 0 { 0.9 } else { 0.72 }),
                            trace_id: None,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("chat_query_release_readiness_window_summary error: {}", e);
                    return finalize_chat_response(
                        &state,
                        &conversation_key,
                        &mut *session,
                        &nlp,
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ChatAskResponse {
                            status: "error".to_string(),
                            answer: "Error consultando health de release readiness gate"
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
        _ => unreachable!("chat query routed to wrong handler"),
    }
}
