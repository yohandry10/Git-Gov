use super::*;

impl Database {
    pub async fn insert_github_event(&self, event: &GitHubEvent) -> Result<(), DbError> {
        let commit_shas_json = serde_json::to_string(&event.commit_shas)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            r#"
            INSERT INTO github_events (
                id, org_id, repo_id, delivery_id, event_type, actor_login, actor_id,
                ref_name, ref_type, before_sha, after_sha, commit_shas, commits_count, payload
            )
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13, $14::jsonb)
            ON CONFLICT (delivery_id) DO NOTHING
            "#,
        )
        .bind(&event.id)
        .bind(&event.org_id)
        .bind(&event.repo_id)
        .bind(&event.delivery_id)
        .bind(&event.event_type)
        .bind(&event.actor_login)
        .bind(event.actor_id)
        .bind(&event.ref_name)
        .bind(&event.ref_type)
        .bind(&event.before_sha)
        .bind(&event.after_sha)
        .bind(&commit_shas_json)
        .bind(event.commits_count)
        .bind(&event.payload)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) if res.rows_affected() == 0 => Err(DbError::Duplicate(format!(
                "delivery_id: {}",
                event.delivery_id
            ))),
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate") => Err(DbError::Duplicate(format!(
                "delivery_id: {}",
                event.delivery_id
            ))),
            Err(e) => Err(DbError::DatabaseError(e.to_string())),
        }
    }

    pub async fn get_github_events(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<GitHubEvent>, DbError> {
        let limit = if filter.limit == 0 { 100 } else { filter.limit } as i64;
        let offset = filter.offset as i64;

        let org_id = if let Some(org_name) = filter.org_name.as_deref() {
            self.get_org_by_login(org_name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(repo_full_name) = filter.repo_full_name.as_deref() {
            self.get_repo_by_full_name(repo_full_name)
                .await?
                .map(|r| r.id)
        } else {
            None
        };

        if filter.org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if filter.repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let mut query = String::from(
            "SELECT id::text, org_id::text, repo_id::text, delivery_id, event_type, actor_login, actor_id, ref_name, ref_type, before_sha, after_sha, commit_shas::text, commits_count, payload::text, created_at FROM github_events WHERE 1=1"
        );
        let mut param_count = 1;

        let mut conditions = Vec::new();

        if org_id.is_some() {
            conditions.push(format!("org_id = ${}", param_count));
            param_count += 1;
        }
        if repo_id.is_some() {
            conditions.push(format!("repo_id = ${}", param_count));
            param_count += 1;
        }
        if filter.start_date.is_some() {
            conditions.push(format!(
                "created_at >= to_timestamp(${0}/1000.0)",
                param_count
            ));
            param_count += 1;
        }
        if filter.end_date.is_some() {
            conditions.push(format!(
                "created_at <= to_timestamp(${0}/1000.0)",
                param_count
            ));
            param_count += 1;
        }
        if filter.user_login.is_some() {
            conditions.push(format!("actor_login = ${}", param_count));
            param_count += 1;
        }
        if filter.event_type.is_some() {
            conditions.push(format!("event_type = ${}", param_count));
            param_count += 1;
        }
        if filter.branch.is_some() {
            conditions.push(format!("ref_name = ${}", param_count));
            param_count += 1;
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_count,
            param_count + 1
        ));

        let mut sql_query = sqlx::query(&query);

        if let Some(ref org_id) = org_id {
            sql_query = sql_query.bind(org_id);
        }
        if let Some(ref repo_id) = repo_id {
            sql_query = sql_query.bind(repo_id);
        }
        if let Some(start) = filter.start_date {
            sql_query = sql_query.bind(start);
        }
        if let Some(end) = filter.end_date {
            sql_query = sql_query.bind(end);
        }
        if let Some(ref login) = filter.user_login {
            sql_query = sql_query.bind(login);
        }
        if let Some(ref event_type) = filter.event_type {
            sql_query = sql_query.bind(event_type);
        }
        if let Some(ref branch) = filter.branch {
            sql_query = sql_query.bind(branch);
        }

        sql_query = sql_query.bind(limit).bind(offset);

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let events: Vec<GitHubEvent> = rows
            .iter()
            .map(|row| {
                let commit_shas_json: String = row.get("commit_shas");
                let payload_json: String = row.get("payload");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                GitHubEvent {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    delivery_id: row.get("delivery_id"),
                    event_type: row.get("event_type"),
                    actor_login: row.get("actor_login"),
                    actor_id: row.get("actor_id"),
                    ref_name: row.get("ref_name"),
                    ref_type: row.get("ref_type"),
                    before_sha: row.get("before_sha"),
                    after_sha: row.get("after_sha"),
                    commit_shas: serde_json::from_str(&commit_shas_json).unwrap_or_default(),
                    commits_count: row.get("commits_count"),
                    payload: serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok(events)
    }

    // ========================================================================
    // CLIENT EVENTS (Telemetry)
    // ========================================================================
}
