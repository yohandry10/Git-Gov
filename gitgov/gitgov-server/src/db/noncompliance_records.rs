use super::*;

impl Database {
    pub async fn get_noncompliance_signals(
        &self,
        filter: &NoncomplianceSignalsQuery<'_>,
    ) -> Result<(Vec<NoncomplianceSignal>, i64), DbError> {
        let mut conditions = Vec::new();
        let mut param_count = 1;

        if filter.org_id.is_some() {
            conditions.push(format!("ns.org_id = ${}::uuid", param_count));
            param_count += 1;
        }

        if filter.confidence.is_some() {
            conditions.push(format!("ns.confidence = ${}", param_count));
            param_count += 1;
        }
        if filter.status.is_some() {
            conditions.push(format!(
                "COALESCE((SELECT sd.decision FROM signal_decisions sd WHERE sd.signal_id = ns.id ORDER BY sd.created_at DESC LIMIT 1), ns.status) = ${}",
                param_count
            ));
            param_count += 1;
        }
        if filter.signal_type.is_some() {
            conditions.push(format!("ns.signal_type = ${}", param_count));
            param_count += 1;
        }
        if filter.actor_login.is_some() {
            conditions.push(format!("ns.actor_login = ${}", param_count));
            param_count += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) as total FROM noncompliance_signals ns{}",
            where_clause
        );

        let mut count_sql = sqlx::query(&count_query);

        if let Some(org) = filter.org_id {
            count_sql = count_sql.bind(org);
        }
        if let Some(c) = filter.confidence {
            count_sql = count_sql.bind(c);
        }
        if let Some(s) = filter.status {
            count_sql = count_sql.bind(s);
        }
        if let Some(st) = filter.signal_type {
            count_sql = count_sql.bind(st);
        }
        if let Some(actor) = filter.actor_login {
            count_sql = count_sql.bind(actor);
        }

        let count_row = count_sql
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let total: i64 = count_row.get("total");

        let data_query = format!(
            "SELECT ns.id::text, ns.org_id::text, ns.repo_id::text, ns.github_event_id::text, ns.client_event_id::text, \
             ns.signal_type, ns.confidence, ns.actor_login, ns.branch, ns.commit_sha, ns.evidence, ns.context, \
             COALESCE(sd.decision, ns.status) as status, \
             COALESCE(sd.decided_by, ns.investigated_by) as investigated_by, \
             COALESCE(sd.created_at, ns.investigated_at) as investigated_at, \
             COALESCE(sd.notes, ns.investigation_notes) as investigation_notes, \
             ns.created_at \
             FROM noncompliance_signals ns \
             LEFT JOIN LATERAL ( \
                SELECT decision, decided_by, notes, created_at \
                FROM signal_decisions \
                WHERE signal_id = ns.id \
                ORDER BY created_at DESC \
                LIMIT 1 \
             ) sd ON true{} ORDER BY ns.created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_count, param_count + 1
        );

        let mut data_sql = sqlx::query(&data_query);

        if let Some(org) = filter.org_id {
            data_sql = data_sql.bind(org);
        }
        if let Some(c) = filter.confidence {
            data_sql = data_sql.bind(c);
        }
        if let Some(s) = filter.status {
            data_sql = data_sql.bind(s);
        }
        if let Some(st) = filter.signal_type {
            data_sql = data_sql.bind(st);
        }
        if let Some(actor) = filter.actor_login {
            data_sql = data_sql.bind(actor);
        }
        data_sql = data_sql.bind(filter.limit).bind(filter.offset);

        let rows = data_sql
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let signals: Vec<NoncomplianceSignal> = rows
            .iter()
            .map(|row| {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let investigated_at: Option<chrono::DateTime<chrono::Utc>> =
                    row.get("investigated_at");

                NoncomplianceSignal {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    github_event_id: row.get("github_event_id"),
                    client_event_id: row.get("client_event_id"),
                    signal_type: row.get("signal_type"),
                    confidence: row.get("confidence"),
                    actor_login: row.get("actor_login"),
                    branch: row.get("branch"),
                    commit_sha: row.get("commit_sha"),
                    evidence: row.get("evidence"),
                    context: row.get("context"),
                    status: row.get("status"),
                    investigated_by: row.get("investigated_by"),
                    investigated_at: investigated_at.map(|t| t.timestamp_millis()),
                    investigation_notes: row.get("investigation_notes"),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok((signals, total))
    }

    pub async fn insert_quality_gate_policy_violation_signal(
        &self,
        input: &QualityGatePolicyViolationSignalInput<'_>,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO noncompliance_signals (
                org_id,
                repo_id,
                signal_type,
                confidence,
                actor_login,
                branch,
                commit_sha,
                evidence,
                context
            )
            SELECT
                $1::uuid,
                $2::uuid,
                'policy_violation',
                'high',
                $3,
                $4,
                $5,
                jsonb_build_object(
                    'rule', 'quality_gate_green',
                    'repo_name', $6,
                    'job_name', $7,
                    'gate_status', $8,
                    'enforcement', $9
                ),
                jsonb_build_object(
                    'source', 'policy_check',
                    'category', 'quality_gates'
                )
            WHERE NOT EXISTS (
                SELECT 1
                FROM noncompliance_signals ns
                WHERE (ns.org_id IS NOT DISTINCT FROM $1::uuid)
                  AND (ns.repo_id IS NOT DISTINCT FROM $2::uuid)
                  AND ns.signal_type = 'policy_violation'
                  AND ns.commit_sha = $5
                  AND COALESCE(ns.evidence->>'rule', '') = 'quality_gate_green'
                  AND ns.created_at >= NOW() - INTERVAL '24 hours'
            )
            "#,
        )
        .bind(input.org_id)
        .bind(input.repo_id)
        .bind(input.actor_login)
        .bind(input.branch)
        .bind(input.commit_sha)
        .bind(input.repo_full_name)
        .bind(input.job_name)
        .bind(input.gate_status)
        .bind(input.enforcement)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_signal_status(
        &self,
        signal_id: &str,
        status: &str,
        decided_by: &str,
        notes: Option<&str>,
    ) -> Result<(), DbError> {
        // Preferred path (append-only): record a decision in signal_decisions.
        // This works with schemas that forbid UPDATE on noncompliance_signals.
        let decision_insert = sqlx::query(
            r#"
            INSERT INTO signal_decisions (id, signal_id, decision, decided_by, notes, created_at)
            VALUES ($1::uuid, $2::uuid, $3, $4, $5, NOW())
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(signal_id)
        .bind(status)
        .bind(decided_by)
        .bind(notes)
        .execute(&self.pool)
        .await;

        match decision_insert {
            Ok(_) => {
                // Legacy compatibility: if schema still allows mutable fields, mirror latest status.
                // Ignore failures here because append-only schemas intentionally reject UPDATE.
                if let Err(e) = sqlx::query(
                    r#"
                    UPDATE noncompliance_signals
                    SET status = $2,
                        investigated_by = $3,
                        investigation_notes = $4,
                        investigated_at = NOW()
                    WHERE id = $1::uuid
                    "#,
                )
                .bind(signal_id)
                .bind(status)
                .bind(decided_by)
                .bind(notes)
                .execute(&self.pool)
                .await
                {
                    tracing::debug!(
                        signal_id = %signal_id,
                        error = %e,
                        "Legacy noncompliance_signals update skipped after decision insert"
                    );
                }

                Ok(())
            }
            Err(insert_err) => {
                let insert_err_msg = insert_err.to_string();

                // Fallback for older schemas without signal_decisions table.
                if insert_err_msg.contains("signal_decisions")
                    && insert_err_msg.contains("does not exist")
                {
                    sqlx::query(
                        r#"
                        UPDATE noncompliance_signals
                        SET status = $2,
                            investigated_by = $3,
                            investigation_notes = $4,
                            investigated_at = NOW()
                        WHERE id = $1::uuid
                        "#,
                    )
                    .bind(signal_id)
                    .bind(status)
                    .bind(decided_by)
                    .bind(notes)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| DbError::DatabaseError(e.to_string()))?;

                    return Ok(());
                }

                Err(DbError::DatabaseError(insert_err_msg))
            }
        }
    }

    pub async fn get_signal_by_id(
        &self,
        signal_id: &str,
    ) -> Result<Option<NoncomplianceSignal>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT ns.id::text, ns.org_id::text, ns.repo_id::text, ns.github_event_id::text, ns.client_event_id::text,
                   ns.signal_type, ns.confidence, ns.actor_login, ns.branch, ns.commit_sha, ns.evidence, ns.context,
                   COALESCE(sd.decision, ns.status) as status,
                   COALESCE(sd.decided_by, ns.investigated_by) as investigated_by,
                   COALESCE(sd.created_at, ns.investigated_at) as investigated_at,
                   COALESCE(sd.notes, ns.investigation_notes) as investigation_notes,
                   ns.created_at
            FROM noncompliance_signals ns
            LEFT JOIN LATERAL (
                SELECT decision, decided_by, notes, created_at
                FROM signal_decisions
                WHERE signal_id = ns.id
                ORDER BY created_at DESC
                LIMIT 1
            ) sd ON true
            WHERE ns.id = $1::uuid
            "#,
        )
        .bind(signal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let investigated_at: Option<chrono::DateTime<chrono::Utc>> =
                    row.get("investigated_at");

                Ok(Some(NoncomplianceSignal {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    github_event_id: row.get("github_event_id"),
                    client_event_id: row.get("client_event_id"),
                    signal_type: row.get("signal_type"),
                    confidence: row.get("confidence"),
                    actor_login: row.get("actor_login"),
                    branch: row.get("branch"),
                    commit_sha: row.get("commit_sha"),
                    evidence: row.get("evidence"),
                    context: row.get("context"),
                    status: row.get("status"),
                    investigated_by: row.get("investigated_by"),
                    investigated_at: investigated_at.map(|t| t.timestamp_millis()),
                    investigation_notes: row.get("investigation_notes"),
                    created_at: created_at.timestamp_millis(),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn confirm_signal_as_violation(
        &self,
        signal_id: &str,
        confirmed_by: &str,
        severity: &str,
    ) -> Result<String, DbError> {
        let signal = self
            .get_signal_by_id(signal_id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("Signal not found: {}", signal_id)))?;

        let violation_id = uuid::Uuid::new_v4().to_string();

        // Insert into violations - APPEND ONLY
        sqlx::query(
            r#"
            INSERT INTO violations (
                id, org_id, repo_id, github_event_id, client_event_id,
                violation_type, severity, confidence_level, reason,
                user_login, branch, commit_sha, details
            )
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::uuid, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(&violation_id)
        .bind(&signal.org_id)
        .bind(&signal.repo_id)
        .bind(&signal.github_event_id)
        .bind(&signal.client_event_id)
        .bind(&signal.signal_type)
        .bind(severity)
        .bind(&signal.confidence)
        .bind(&signal.investigation_notes)
        .bind(&signal.actor_login)
        .bind(&signal.branch)
        .bind(&signal.commit_sha)
        .bind(&signal.evidence)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        // NOTE: We do NOT update noncompliance_signals - it's append-only.
        // The signal remains as-is, the violation is a new record.
        // To track confirmation workflow, use a separate signal_decisions table
        // or track via the violation's creation with confirmed_by.

        // Insert a signal_decision record for audit trail (if table exists)
        let _ = sqlx::query(
            r#"
            INSERT INTO signal_decisions (
                id, signal_id, decision, decided_by, severity, created_at
            )
            VALUES ($1::uuid, $2::uuid, 'confirmed', $3, $4, NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(signal_id)
        .bind(confirmed_by)
        .bind(severity)
        .execute(&self.pool)
        .await;

        tracing::info!(
            "Signal {} confirmed as violation {} by {}",
            signal_id,
            violation_id,
            confirmed_by
        );

        Ok(violation_id)
    }
}
