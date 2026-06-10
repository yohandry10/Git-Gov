use super::*;

impl Database {
    pub async fn insert_client_event(&self, event: &ClientEvent) -> Result<(), DbError> {
        let files_json = serde_json::to_string(&event.files)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            r#"
            INSERT INTO client_events (
                id, org_id, repo_id, event_uuid, event_type, user_login, user_name,
                branch, commit_sha, files, status, reason, metadata, client_version, created_at
            )
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, $8, $9, $10::jsonb, $11, $12, $13::jsonb, $14, to_timestamp($15::bigint / 1000.0))
            ON CONFLICT (event_uuid) DO NOTHING
            "#,
        )
        .bind(&event.id)
        .bind(&event.org_id)
        .bind(&event.repo_id)
        .bind(&event.event_uuid)
        .bind(event.event_type.as_str())
        .bind(&event.user_login)
        .bind(&event.user_name)
        .bind(&event.branch)
        .bind(&event.commit_sha)
        .bind(&files_json)
        .bind(event.status.as_str())
        .bind(&event.reason)
        .bind(&event.metadata)
        .bind(&event.client_version)
        .bind(event.created_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) if res.rows_affected() == 0 => Err(DbError::Duplicate(format!(
                "event_uuid: {}",
                event.event_uuid
            ))),
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate") => Err(DbError::Duplicate(format!(
                "event_uuid: {}",
                event.event_uuid
            ))),
            Err(e) => Err(DbError::DatabaseError(e.to_string())),
        }
    }

    pub async fn insert_client_events_batch(
        &self,
        events: &[ClientEvent],
    ) -> Result<ClientEventResponse, DbError> {
        let mut in_batch_seen = HashSet::new();
        let mut deduped_events: Vec<&ClientEvent> = Vec::with_capacity(events.len());
        let mut duplicates: Vec<String> = Vec::new();

        for event in events {
            if !in_batch_seen.insert(event.event_uuid.clone()) {
                duplicates.push(event.event_uuid.clone());
                continue;
            }
            deduped_events.push(event);
        }

        match self.insert_client_events_batch_tx(&deduped_events).await {
            Ok(mut response) => {
                response.duplicates.extend(duplicates);
                Ok(response)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    batch_size = deduped_events.len(),
                    "Transactional client event batch insert failed, falling back to per-row inserts"
                );

                let mut accepted = Vec::new();
                let mut errors = Vec::new();
                for event in deduped_events {
                    match self.insert_client_event(event).await {
                        Ok(()) => accepted.push(event.event_uuid.clone()),
                        Err(DbError::Duplicate(_)) => duplicates.push(event.event_uuid.clone()),
                        Err(err) => errors.push(EventError {
                            event_uuid: event.event_uuid.clone(),
                            error: err.to_string(),
                        }),
                    }
                }

                Ok(ClientEventResponse {
                    accepted,
                    duplicates,
                    errors,
                })
            }
        }
    }

    async fn insert_client_events_batch_tx(
        &self,
        events: &[&ClientEvent],
    ) -> Result<ClientEventResponse, DbError> {
        struct PreparedBatchEvent<'a> {
            id: uuid::Uuid,
            org_id: Option<uuid::Uuid>,
            repo_id: Option<uuid::Uuid>,
            files_json: serde_json::Value,
            created_at: chrono::DateTime<chrono::Utc>,
            event: &'a ClientEvent,
        }

        let mut accepted = Vec::new();
        let mut duplicates = Vec::new();
        let mut errors = Vec::new();
        let mut prepared_events: Vec<PreparedBatchEvent<'_>> = Vec::with_capacity(events.len());

        for event in events {
            let id = match uuid::Uuid::parse_str(&event.id) {
                Ok(id) => id,
                Err(e) => {
                    errors.push(EventError {
                        event_uuid: event.event_uuid.clone(),
                        error: DbError::SerializationError(format!(
                            "invalid event id uuid '{}': {}",
                            event.id, e
                        ))
                        .to_string(),
                    });
                    continue;
                }
            };

            let org_id = match event.org_id.as_deref() {
                Some(raw) => match uuid::Uuid::parse_str(raw) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        errors.push(EventError {
                            event_uuid: event.event_uuid.clone(),
                            error: DbError::SerializationError(format!(
                                "invalid org_id uuid '{}': {}",
                                raw, e
                            ))
                            .to_string(),
                        });
                        continue;
                    }
                },
                None => None,
            };

            let repo_id = match event.repo_id.as_deref() {
                Some(raw) => match uuid::Uuid::parse_str(raw) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        errors.push(EventError {
                            event_uuid: event.event_uuid.clone(),
                            error: DbError::SerializationError(format!(
                                "invalid repo_id uuid '{}': {}",
                                raw, e
                            ))
                            .to_string(),
                        });
                        continue;
                    }
                },
                None => None,
            };

            let files_json = match serde_json::to_value(&event.files) {
                Ok(json) => json,
                Err(e) => {
                    errors.push(EventError {
                        event_uuid: event.event_uuid.clone(),
                        error: DbError::SerializationError(e.to_string()).to_string(),
                    });
                    continue;
                }
            };

            let created_at =
                match chrono::DateTime::<chrono::Utc>::from_timestamp_millis(event.created_at) {
                    Some(ts) => ts,
                    None => {
                        errors.push(EventError {
                            event_uuid: event.event_uuid.clone(),
                            error: DbError::SerializationError(format!(
                                "invalid created_at timestamp millis '{}'",
                                event.created_at
                            ))
                            .to_string(),
                        });
                        continue;
                    }
                };

            prepared_events.push(PreparedBatchEvent {
                id,
                org_id,
                repo_id,
                files_json,
                created_at,
                event,
            });
        }

        if !prepared_events.is_empty() {
            let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
                r#"
                INSERT INTO client_events (
                    id, org_id, repo_id, event_uuid, event_type, user_login, user_name,
                    branch, commit_sha, files, status, reason, metadata, client_version, created_at
                )
                "#,
            );

            query_builder.push_values(&prepared_events, |mut builder, row| {
                builder
                    .push_bind(row.id)
                    .push_bind(row.org_id)
                    .push_bind(row.repo_id)
                    .push_bind(&row.event.event_uuid)
                    .push_bind(row.event.event_type.as_str())
                    .push_bind(&row.event.user_login)
                    .push_bind(&row.event.user_name)
                    .push_bind(&row.event.branch)
                    .push_bind(&row.event.commit_sha)
                    .push_bind(&row.files_json)
                    .push_bind(row.event.status.as_str())
                    .push_bind(&row.event.reason)
                    .push_bind(&row.event.metadata)
                    .push_bind(&row.event.client_version)
                    .push_bind(row.created_at);
            });

            query_builder.push(" ON CONFLICT (event_uuid) DO NOTHING RETURNING event_uuid");

            let inserted_event_uuids = match query_builder
                .build_query_scalar::<String>()
                .fetch_all(&self.pool)
                .await
            {
                Ok(rows) => rows,
                Err(e) => return Err(DbError::DatabaseError(e.to_string())),
            };

            let inserted_set: HashSet<&str> =
                inserted_event_uuids.iter().map(String::as_str).collect();
            for row in &prepared_events {
                if inserted_set.contains(row.event.event_uuid.as_str()) {
                    accepted.push(row.event.event_uuid.clone());
                } else {
                    duplicates.push(row.event.event_uuid.clone());
                }
            }
        }

        Ok(ClientEventResponse {
            accepted,
            duplicates,
            errors,
        })
    }

    pub async fn get_client_events(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<ClientEvent>, DbError> {
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
            "SELECT id::text, org_id::text, repo_id::text, event_uuid, event_type, user_login, user_name, branch, commit_sha, files::text, status, reason, metadata::text, client_version, created_at FROM client_events WHERE 1=1"
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
            conditions.push(format!("user_login = ${}", param_count));
            param_count += 1;
        }
        if filter.event_type.is_some() {
            conditions.push(format!("event_type = ${}", param_count));
            param_count += 1;
        }
        if filter.status.is_some() {
            conditions.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if filter.branch.is_some() {
            conditions.push(format!("branch = ${}", param_count));
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
        if let Some(ref status) = filter.status {
            sql_query = sql_query.bind(status);
        }
        if let Some(ref branch) = filter.branch {
            sql_query = sql_query.bind(branch);
        }

        sql_query = sql_query.bind(limit).bind(offset);

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let events: Vec<ClientEvent> = rows
            .iter()
            .map(|row| {
                let files_json: String = row.get("files");
                let metadata_json: String = row.get("metadata");
                let event_type_str: String = row.get("event_type");
                let status_str: String = row.get("status");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                ClientEvent {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    event_uuid: row.get("event_uuid"),
                    event_type: ClientEventType::from_db_str(&event_type_str),
                    user_login: row.get("user_login"),
                    user_name: row.get("user_name"),
                    branch: row.get("branch"),
                    commit_sha: row.get("commit_sha"),
                    files: serde_json::from_str(&files_json).unwrap_or_default(),
                    status: EventStatus::from_db_str(&status_str),
                    reason: row.get("reason"),
                    metadata: serde_json::from_str(&metadata_json)
                        .unwrap_or(serde_json::Value::Null),
                    client_version: row.get("client_version"),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok(events)
    }

    // ========================================================================
    // COMBINED EVENTS (for dashboard)
    // ========================================================================

    pub async fn get_combined_events(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<CombinedEvent>, DbError> {
        let limit = if filter.limit == 0 { 100 } else { filter.limit } as i32;
        let offset = filter.offset as i32;

        let org_id = if let Some(org_name) = filter.org_name.as_deref() {
            self.get_org_by_login(org_name).await?.map(|o| o.id)
        } else {
            // Fallback: handler may have set filter.org_id directly (UUID) to avoid a DB roundtrip.
            filter.org_id.clone()
        };

        let repo_id = if let Some(repo_full_name) = filter.repo_full_name.as_deref() {
            self.get_repo_by_full_name(repo_full_name)
                .await?
                .map(|r| r.id)
        } else {
            None
        };

        // If caller requested a specific org/repo and it doesn't exist, return empty result.
        if filter.org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if filter.repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let start_date = filter
            .start_date
            .and_then(chrono::DateTime::from_timestamp_millis);
        let end_date = filter
            .end_date
            .and_then(chrono::DateTime::from_timestamp_millis);
        let before_created_at = filter
            .before_created_at
            .and_then(chrono::DateTime::from_timestamp_millis);

        // Fast path: skip the expensive UNION ALL with github_events when
        // the caller does not explicitly request source='github'.
        let use_client_only_fast_path = filter.source.as_deref() != Some("github");

        let result = if use_client_only_fast_path {
            sqlx::query(
                r#"
                SELECT
                    c.id::TEXT AS id,
                    'client'::TEXT AS source,
                    c.event_type,
                    c.created_at,
                    COALESCE(ica.canonical_login, c.user_login) AS user_login,
                    r.full_name AS repo_name,
                    c.branch,
                    c.status,
                    jsonb_strip_nulls(
                        jsonb_build_object(
                            'reason', c.reason,
                            'files', c.files,
                            'event_uuid', c.event_uuid,
                            'commit_sha', c.commit_sha,
                            'user_name', c.user_name
                        )
                        || CASE
                            WHEN jsonb_typeof(COALESCE(c.metadata, '{}'::jsonb)) = 'object'
                                THEN COALESCE(c.metadata, '{}'::jsonb)
                            ELSE jsonb_build_object('metadata', COALESCE(c.metadata, 'null'::jsonb))
                        END
                    ) AS details
                FROM client_events c
                LEFT JOIN repos r ON c.repo_id = r.id
                LEFT JOIN identity_aliases ica
                  ON ica.alias_login = c.user_login
                 AND ($1::uuid IS NULL OR ica.org_id = $1::uuid)
                WHERE ($1::uuid IS NULL OR c.org_id = $1::uuid)
                  AND ($2::uuid IS NULL OR c.repo_id = $2::uuid)
                  AND ($4::text IS NULL OR c.event_type = $4)
                  AND ($5::text IS NULL OR c.user_login = $5 OR COALESCE(ica.canonical_login, c.user_login) = $5)
                  AND ($6::text IS NULL OR c.branch = $6)
                  AND ($7::timestamptz IS NULL OR c.created_at >= $7)
                  AND ($8::timestamptz IS NULL OR c.created_at <= $8)
                  AND ($9::text IS NULL OR c.status = $9)
                  AND (
                      $12::timestamptz IS NULL
                      OR c.created_at < $12
                      OR ($13::text IS NOT NULL AND c.created_at = $12 AND c.id::text < $13::text)
                  )
                ORDER BY c.created_at DESC, c.id DESC
                LIMIT $10 OFFSET $11
                "#
            )
            .bind(&org_id)          // $1
            .bind(&repo_id)         // $2
            .bind(&filter.source)   // $3 (unused in fast path but keeps bind order)
            .bind(&filter.event_type) // $4
            .bind(&filter.user_login) // $5
            .bind(&filter.branch)   // $6
            .bind(start_date)       // $7
            .bind(end_date)         // $8
            .bind(&filter.status)   // $9
            .bind(limit)            // $10
            .bind(offset)           // $11
            .bind(before_created_at) // $12
            .bind(&filter.before_id) // $13
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
            r#"
            SELECT id, source, event_type, created_at, user_login, repo_name, branch, status, details
            FROM (
                SELECT
                    g.id::TEXT AS id,
                    'github'::TEXT AS source,
                    g.event_type,
                    g.created_at,
                    COALESCE(iga.canonical_login, g.actor_login) AS user_login,
                    r.full_name AS repo_name,
                    g.ref_name AS branch,
                    NULL::TEXT AS status,
                    jsonb_build_object(
                        'commits_count', g.commits_count,
                        'after_sha', g.after_sha
                    ) AS details
                FROM github_events g
                LEFT JOIN repos r ON g.repo_id = r.id
                LEFT JOIN identity_aliases iga
                  ON iga.alias_login = g.actor_login
                 AND ($1::uuid IS NULL OR iga.org_id = $1::uuid)
                WHERE ($1::uuid IS NULL OR g.org_id = $1::uuid)
                  AND ($2::uuid IS NULL OR g.repo_id = $2::uuid)
                  AND ($3::text IS NULL OR $3 = 'github')
                  AND ($4::text IS NULL OR g.event_type = $4)
                  AND ($5::text IS NULL OR g.actor_login = $5 OR COALESCE(iga.canonical_login, g.actor_login) = $5)
                  AND ($6::text IS NULL OR g.ref_name = $6)
                  AND ($7::timestamptz IS NULL OR g.created_at >= $7)
                  AND ($8::timestamptz IS NULL OR g.created_at <= $8)
                  AND ($9::text IS NULL)

                UNION ALL

                SELECT
                    c.id::TEXT AS id,
                    'client'::TEXT AS source,
                    c.event_type,
                    c.created_at,
                    COALESCE(ica.canonical_login, c.user_login) AS user_login,
                    r.full_name AS repo_name,
                    c.branch,
                    c.status,
                    jsonb_strip_nulls(
                        jsonb_build_object(
                            'reason', c.reason,
                            'files', c.files,
                            'event_uuid', c.event_uuid,
                            'commit_sha', c.commit_sha,
                            'user_name', c.user_name
                        )
                        || CASE
                            WHEN jsonb_typeof(COALESCE(c.metadata, '{}'::jsonb)) = 'object'
                                THEN COALESCE(c.metadata, '{}'::jsonb)
                            ELSE jsonb_build_object('metadata', COALESCE(c.metadata, 'null'::jsonb))
                        END
                    ) AS details
                FROM client_events c
                LEFT JOIN repos r ON c.repo_id = r.id
                LEFT JOIN identity_aliases ica
                  ON ica.alias_login = c.user_login
                 AND ($1::uuid IS NULL OR ica.org_id = $1::uuid)
                WHERE ($1::uuid IS NULL OR c.org_id = $1::uuid)
                  AND ($2::uuid IS NULL OR c.repo_id = $2::uuid)
                  AND ($3::text IS NULL OR $3 = 'client')
                  AND ($4::text IS NULL OR c.event_type = $4)
                  AND ($5::text IS NULL OR c.user_login = $5 OR COALESCE(ica.canonical_login, c.user_login) = $5)
                  AND ($6::text IS NULL OR c.branch = $6)
                  AND ($7::timestamptz IS NULL OR c.created_at >= $7)
                  AND ($8::timestamptz IS NULL OR c.created_at <= $8)
                  AND ($9::text IS NULL OR c.status = $9)
            ) combined
            WHERE (
                $12::timestamptz IS NULL
                OR combined.created_at < $12
                OR ($13::text IS NOT NULL AND combined.created_at = $12 AND combined.id < $13::text)
            )
            ORDER BY created_at DESC, id DESC
            LIMIT $10 OFFSET $11
            "#
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&filter.source)
        .bind(&filter.event_type)
        .bind(&filter.user_login)
        .bind(&filter.branch)
        .bind(start_date)
        .bind(end_date)
        .bind(&filter.status)
        .bind(limit)
        .bind(offset)
        .bind(before_created_at)
        .bind(&filter.before_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?
        };

        let events: Vec<CombinedEvent> = result
            .iter()
            .map(|row| {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let details_json: serde_json::Value = row.get("details");

                CombinedEvent {
                    id: row.get("id"),
                    source: row.get("source"),
                    event_type: row.get("event_type"),
                    created_at: created_at.timestamp_millis(),
                    user_login: row.get("user_login"),
                    repo_name: row.get("repo_name"),
                    branch: row.get("branch"),
                    status: row.get("status"),
                    details: details_json,
                }
            })
            .collect();

        Ok(events)
    }
}
