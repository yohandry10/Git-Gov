use super::*;

impl Database {
    pub async fn create_feature_request(
        &self,
        input: &crate::models::FeatureRequestInput,
        requested_by: &str,
    ) -> Result<String, DbError> {
        let metadata = input
            .metadata
            .as_ref()
            .cloned()
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let row = sqlx::query(
            r#"
            INSERT INTO feature_requests
                (org_id, requested_by, question, missing_capability, metadata)
            VALUES ($1::uuid, $2, $3, $4, $5)
            RETURNING id::text
            "#,
        )
        .bind(input.org_id.as_deref())
        .bind(requested_by)
        .bind(&input.question)
        .bind(input.missing_capability.as_deref())
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("id"))
    }

    pub async fn insert_chat_query_event(
        &self,
        input: &ChatQueryEventInsertInput<'_>,
    ) -> Result<(), DbError> {
        let entities_detected = serde_json::to_value(input.entities_detected)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let sources = serde_json::to_value(input.sources)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let actions_recommended = serde_json::to_value(input.actions_recommended)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO chat_query_events (
                trace_id, conversation_key, client_id, org_scope,
                question, intent, response_status, confidence, language,
                entities_detected, time_range_used, sources, actions_recommended, answer_preview
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10::jsonb, $11, $12::jsonb, $13::jsonb, $14
            )
            "#,
        )
        .bind(input.trace_id)
        .bind(input.conversation_key)
        .bind(input.client_id)
        .bind(input.org_scope)
        .bind(input.question)
        .bind(input.intent)
        .bind(input.response_status)
        .bind(input.confidence)
        .bind(input.language)
        .bind(&entities_detected)
        .bind(input.time_range_used)
        .bind(&sources)
        .bind(&actions_recommended)
        .bind(input.answer_preview)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_chat_query_tool_call(
        &self,
        input: &ChatQueryToolCallInsertInput<'_>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO chat_query_tool_calls (
                trace_id, tool_name, tool_status, duration_ms, input_payload, output_payload, error
            )
            VALUES (
                $1, $2, $3, $4, $5::jsonb, $6::jsonb, $7
            )
            "#,
        )
        .bind(input.trace_id)
        .bind(input.tool_name)
        .bind(input.tool_status)
        .bind(input.duration_ms)
        .bind(input.input_payload)
        .bind(input.output_payload)
        .bind(input.error)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // CLI COMMAND AUDIT
    // ========================================================================

    pub async fn insert_cli_command(
        &self,
        record: &crate::models::CliCommandRecord,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO cli_commands (
                id, org_id, user_login, command, origin, branch,
                repo_name, exit_code, duration_ms, metadata, created_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3, $4, $5, $6,
                $7, $8, $9, $10::jsonb, to_timestamp($11::bigint / 1000.0)
            )
            "#,
        )
        .bind(&record.id)
        .bind(&record.org_id)
        .bind(&record.user_login)
        .bind(&record.command)
        .bind(&record.origin)
        .bind(&record.branch)
        .bind(&record.repo_name)
        .bind(record.exit_code)
        .bind(record.duration_ms)
        .bind(&record.metadata)
        .bind(record.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_cli_commands(
        &self,
        org_id: Option<&str>,
        user_login: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<crate::models::CliCommandRecord>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text, org_id::text, user_login, command, origin, branch,
                repo_name, exit_code, duration_ms,
                COALESCE(metadata, '{}'::jsonb) AS metadata,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM cli_commands
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
              AND ($2::text IS NULL OR user_login = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id)
        .bind(user_login)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM cli_commands
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
              AND ($2::text IS NULL OR user_login = $2)
            "#,
        )
        .bind(org_id)
        .bind(user_login)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let records: Vec<crate::models::CliCommandRecord> = rows
            .iter()
            .map(|row| crate::models::CliCommandRecord {
                id: row.get("id"),
                org_id: row.get("org_id"),
                user_login: row.get("user_login"),
                command: row.get("command"),
                origin: row.get("origin"),
                branch: row.get("branch"),
                repo_name: row.get("repo_name"),
                exit_code: row.get("exit_code"),
                duration_ms: row.get("duration_ms"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at_ms"),
            })
            .collect();

        Ok((records, count))
    }

    // ========================================================================
    // POLICY DRIFT AUDIT
    // ========================================================================

    pub async fn insert_policy_drift_event(
        &self,
        record: &crate::models::PolicyDriftEventRecord,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO policy_drift_events (
                id, org_id, user_login, action, repo_name, result,
                before_checksum, after_checksum, duration_ms, metadata, created_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3, $4, $5, $6,
                $7, $8, $9, $10::jsonb, to_timestamp($11::bigint / 1000.0)
            )
            "#,
        )
        .bind(&record.id)
        .bind(&record.org_id)
        .bind(&record.user_login)
        .bind(&record.action)
        .bind(&record.repo_name)
        .bind(&record.result)
        .bind(&record.before_checksum)
        .bind(&record.after_checksum)
        .bind(record.duration_ms)
        .bind(&record.metadata)
        .bind(record.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_policy_drift_events(
        &self,
        org_id: Option<&str>,
        user_login: Option<&str>,
        repo_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<crate::models::PolicyDriftEventRecord>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                user_login,
                action,
                repo_name,
                result,
                before_checksum,
                after_checksum,
                duration_ms,
                COALESCE(metadata, '{}'::jsonb) AS metadata,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM policy_drift_events
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
              AND ($2::text IS NULL OR user_login = $2)
              AND ($3::text IS NULL OR repo_name = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(user_login)
        .bind(repo_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM policy_drift_events
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
              AND ($2::text IS NULL OR user_login = $2)
              AND ($3::text IS NULL OR repo_name = $3)
            "#,
        )
        .bind(org_id)
        .bind(user_login)
        .bind(repo_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let records: Vec<crate::models::PolicyDriftEventRecord> = rows
            .iter()
            .map(|row| crate::models::PolicyDriftEventRecord {
                id: row.get("id"),
                org_id: row.get("org_id"),
                user_login: row.get("user_login"),
                action: row.get("action"),
                repo_name: row.get("repo_name"),
                result: row.get("result"),
                before_checksum: row.get("before_checksum"),
                after_checksum: row.get("after_checksum"),
                duration_ms: row.get("duration_ms"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at_ms"),
            })
            .collect();

        Ok((records, count))
    }
}
