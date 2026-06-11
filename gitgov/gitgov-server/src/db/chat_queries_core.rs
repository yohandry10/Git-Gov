use super::*;

impl Database {
    pub async fn chat_query_pushes_no_ticket(
        &self,
        org_id: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                ge.actor_login,
                ge.ref_name   AS branch,
                ge.commit_shas::text AS commit_shas,
                EXTRACT(EPOCH FROM ge.created_at)::bigint * 1000 AS event_ts
            FROM github_events ge
            WHERE ge.event_type = 'push'
              AND (ge.ref_name = 'refs/heads/main' OR ge.ref_name = 'main')
              AND ge.created_at >= NOW() - INTERVAL '7 days'
              AND ($1::uuid IS NULL OR ge.org_id = $1::uuid)
              AND NOT EXISTS (
                  SELECT 1
                  FROM commit_ticket_correlations ctc
                  WHERE ctc.commit_sha = ANY(
                      SELECT jsonb_array_elements_text(ge.commit_shas::jsonb)
                  )
              )
            ORDER BY ge.created_at DESC
            LIMIT 50
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "actor": r.get::<Option<String>, _>("actor_login").unwrap_or_default(),
                    "branch": r.get::<Option<String>, _>("branch").unwrap_or_default(),
                    "timestamp": r.get::<i64, _>("event_ts"),
                })
            })
            .collect())
    }

    /// Q1a: Count pushes to main in the last 7 days with no Jira correlation.
    pub async fn chat_query_pushes_no_ticket_count(
        &self,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM github_events ge
            WHERE ge.event_type = 'push'
              AND (ge.ref_name = 'refs/heads/main' OR ge.ref_name = 'main')
              AND ge.created_at >= NOW() - INTERVAL '7 days'
              AND ($1::uuid IS NULL OR ge.org_id = $1::uuid)
              AND NOT EXISTS (
                  SELECT 1
                  FROM commit_ticket_correlations ctc
                  WHERE ctc.commit_sha = ANY(
                      SELECT jsonb_array_elements_text(ge.commit_shas::jsonb)
                  )
              )
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q1b: Count commits without Jira correlation in a recent time window.
    pub async fn chat_query_commits_without_ticket_count(
        &self,
        org_id: Option<&str>,
        hours: i64,
    ) -> Result<i64, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT c.commit_sha)::bigint AS cnt
            FROM client_events c
            LEFT JOIN commit_ticket_correlations ct ON ct.commit_sha = c.commit_sha
            WHERE c.event_type = 'commit'
              AND c.commit_sha IS NOT NULL
              AND c.created_at >= NOW() - make_interval(hours => $1::int)
              AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
              AND ct.id IS NULL
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q1c: Count developers considered online by recent client heartbeat/activity.
    pub async fn chat_query_online_developers_count(
        &self,
        org_id: Option<&str>,
        minutes: i64,
    ) -> Result<i64, DbError> {
        let safe_minutes = minutes.clamp(1, 24 * 60) as i32;
        let row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT client_id)::bigint AS cnt
            FROM client_sessions
            WHERE last_seen_at >= NOW() - make_interval(mins => $1::int)
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            "#,
        )
        .bind(safe_minutes)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q2: Count blocked pushes this calendar month.
    pub async fn chat_query_blocked_pushes_month(
        &self,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM client_events
            WHERE event_type IN ('blocked_push', 'governance_blocked_push', 'push_failed', 'attempt_push')
              AND status IN ('blocked', 'failed')
              AND created_at >= date_trunc('month', NOW())
              AND ($1::uuid IS NULL OR org_id = $1::uuid)
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q2b: Count blocked pushes this calendar month for a specific user (alias-aware).
    pub async fn chat_query_user_blocked_pushes_month(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM client_events c
            LEFT JOIN identity_aliases ica
              ON ica.alias_login = c.user_login
             AND ($2::uuid IS NULL OR ica.org_id = $2::uuid)
            WHERE c.event_type IN ('blocked_push', 'governance_blocked_push', 'push_failed', 'attempt_push')
              AND c.status IN ('blocked', 'failed')
              AND c.created_at >= date_trunc('month', NOW())
              AND (c.user_login ILIKE $1 OR COALESCE(ica.canonical_login, c.user_login) ILIKE $1)
              AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q2c: Count successful pushes for a specific user, optionally scoped to a time window.
    pub async fn chat_query_user_pushes_count(
        &self,
        user_login: &str,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM client_events c
            LEFT JOIN identity_aliases ica
              ON ica.alias_login = c.user_login
             AND ($4::uuid IS NULL OR ica.org_id = $4::uuid)
            WHERE c.event_type = 'successful_push'
              AND (c.user_login ILIKE $1 OR COALESCE(ica.canonical_login, c.user_login) ILIKE $1)
              AND ($2::bigint IS NULL OR c.created_at >= to_timestamp($2::bigint / 1000.0))
              AND ($3::bigint IS NULL OR c.created_at <= to_timestamp($3::bigint / 1000.0))
              AND ($4::uuid IS NULL OR c.org_id = $4::uuid)
            "#,
        )
        .bind(user_login)
        .bind(start_ms)
        .bind(end_ms)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q1b: Count pushes to main this week with no Jira ticket for a specific user (alias-aware).
    pub async fn chat_query_user_pushes_no_ticket_week(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM github_events ge
            LEFT JOIN identity_aliases iga
              ON iga.alias_login = ge.actor_login
             AND ($2::uuid IS NULL OR iga.org_id = $2::uuid)
            WHERE ge.event_type = 'push'
              AND (ge.ref_name = 'refs/heads/main' OR ge.ref_name = 'main')
              AND ge.created_at >= NOW() - INTERVAL '7 days'
              AND (ge.actor_login ILIKE $1 OR COALESCE(iga.canonical_login, ge.actor_login) ILIKE $1)
              AND ($2::uuid IS NULL OR ge.org_id = $2::uuid)
              AND NOT EXISTS (
                  SELECT 1
                  FROM commit_ticket_correlations ctc
                  WHERE ctc.commit_sha = ANY(
                      SELECT jsonb_array_elements_text(ge.commit_shas::jsonb)
                  )
              )
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q3: Commits by a specific user in a time range [start_ms, end_ms] (epoch millis).
    pub async fn chat_query_user_commits_range(
        &self,
        user_login: &str,
        start_ms: i64,
        end_ms: i64,
        org_id: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                COALESCE(ica.canonical_login, c.user_login) AS canonical_user_login,
                c.branch,
                c.commit_sha,
                (EXTRACT(EPOCH FROM c.created_at) * 1000)::bigint AS event_ts
            FROM client_events c
            LEFT JOIN identity_aliases ica
              ON ica.alias_login = c.user_login
             AND ($4::uuid IS NULL OR ica.org_id = $4::uuid)
            WHERE c.event_type = 'commit'
              AND (c.user_login ILIKE $1 OR COALESCE(ica.canonical_login, c.user_login) ILIKE $1)
              AND c.created_at >= to_timestamp($2::bigint / 1000.0)
              AND c.created_at <= to_timestamp($3::bigint / 1000.0)
              AND ($4::uuid IS NULL OR c.org_id = $4::uuid)
            ORDER BY c.created_at DESC
            LIMIT 100
            "#,
        )
        .bind(user_login)
        .bind(start_ms)
        .bind(end_ms)
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "user_login": r.get::<String, _>("canonical_user_login"),
                    "branch": r.get::<Option<String>, _>("branch").unwrap_or_default(),
                    "commit_sha": r.get::<Option<String>, _>("commit_sha").unwrap_or_default(),
                    "timestamp": r.get::<i64, _>("event_ts"),
                })
            })
            .collect())
    }

    /// Q4: Count commits by a specific user, optionally scoped to a time window.
    pub async fn chat_query_user_commits_count(
        &self,
        user_login: &str,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM client_events c
            LEFT JOIN identity_aliases ica
              ON ica.alias_login = c.user_login
             AND ($4::uuid IS NULL OR ica.org_id = $4::uuid)
            WHERE c.event_type = 'commit'
              AND (c.user_login ILIKE $1 OR COALESCE(ica.canonical_login, c.user_login) ILIKE $1)
              AND ($2::bigint IS NULL OR c.created_at >= to_timestamp($2::bigint / 1000.0))
              AND ($3::bigint IS NULL OR c.created_at <= to_timestamp($3::bigint / 1000.0))
              AND ($4::uuid IS NULL OR c.org_id = $4::uuid)
            "#,
        )
        .bind(user_login)
        .bind(start_ms)
        .bind(end_ms)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }

    /// Q4b: Latest commit metadata by user in current scope (alias-aware).
    pub async fn chat_query_user_last_commit(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(ica.canonical_login, c.user_login) AS canonical_user_login,
                c.user_name,
                c.event_uuid,
                c.branch,
                c.commit_sha,
                r.full_name AS repo_full_name,
                COALESCE(c.metadata->>'commit_message', c.metadata->>'message') AS commit_message,
                (EXTRACT(EPOCH FROM c.created_at) * 1000)::bigint AS event_ts
            FROM client_events c
            LEFT JOIN repos r ON c.repo_id = r.id
            LEFT JOIN identity_aliases ica
              ON ica.alias_login = c.user_login
             AND ($2::uuid IS NULL OR ica.org_id = $2::uuid)
            WHERE c.event_type = 'commit'
              AND (c.user_login ILIKE $1 OR COALESCE(ica.canonical_login, c.user_login) ILIKE $1)
              AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
            ORDER BY c.created_at DESC, c.id DESC
            LIMIT 1
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            serde_json::json!({
                "user_login": r.get::<String, _>("canonical_user_login"),
                "user_name": r.get::<Option<String>, _>("user_name"),
                "event_uuid": r.get::<String, _>("event_uuid"),
                "branch": r.get::<Option<String>, _>("branch"),
                "commit_sha": r.get::<Option<String>, _>("commit_sha"),
                "repo_full_name": r.get::<Option<String>, _>("repo_full_name"),
                "commit_message": r.get::<Option<String>, _>("commit_message"),
                "timestamp": r.get::<i64, _>("event_ts"),
            })
        }))
    }

    /// Access profile by user login in org scope (never returns plaintext API key values).
    pub async fn chat_query_user_access_profile(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                ou.login,
                ou.role,
                ou.status,
                EXISTS (
                    SELECT 1
                    FROM api_keys ak
                    WHERE ak.client_id = ou.login
                      AND ak.is_active = TRUE
                      AND ($2::uuid IS NULL OR ak.org_id = $2::uuid)
                ) AS has_active_api_key
            FROM org_users ou
            WHERE ou.login ILIKE $1
              AND ($2::uuid IS NULL OR ou.org_id = $2::uuid)
            LIMIT 1
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            serde_json::json!({
                "login": r.get::<String, _>("login"),
                "role": r.get::<String, _>("role"),
                "status": r.get::<String, _>("status"),
                "has_active_api_key": r.get::<bool, _>("has_active_api_key"),
            })
        }))
    }

    /// Q5: Count commits in an optional time window, optionally scoped by org.
    pub async fn chat_query_commits_count(
        &self,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        org_id: Option<&str>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::bigint AS cnt
            FROM client_events
            WHERE event_type = 'commit'
              AND ($1::bigint IS NULL OR created_at >= to_timestamp($1::bigint / 1000.0))
              AND ($2::bigint IS NULL OR created_at <= to_timestamp($2::bigint / 1000.0))
              AND ($3::uuid IS NULL OR org_id = $3::uuid)
            "#,
        )
        .bind(start_ms)
        .bind(end_ms)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("cnt"))
    }
}
