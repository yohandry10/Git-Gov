use super::*;

impl Database {
    pub async fn insert_governance_event(&self, event: &GovernanceEvent) -> Result<(), DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO governance_events (
                id, org_id, repo_id, delivery_id, event_type, actor_login,
                target, old_value, new_value, payload
            )
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (delivery_id) DO NOTHING
            "#,
        )
        .bind(&event.id)
        .bind(&event.org_id)
        .bind(&event.repo_id)
        .bind(&event.delivery_id)
        .bind(&event.event_type)
        .bind(&event.actor_login)
        .bind(&event.target)
        .bind(&event.old_value)
        .bind(&event.new_value)
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

    pub async fn insert_governance_events_batch(
        &self,
        events: &[GovernanceEvent],
    ) -> Result<(i32, Vec<String>), DbError> {
        let mut accepted = 0;
        let mut errors = Vec::new();

        for event in events {
            match self.insert_governance_event(event).await {
                Ok(()) => accepted += 1,
                Err(DbError::Duplicate(_)) => {}
                Err(e) => errors.push(format!("{}: {}", event.delivery_id, e)),
            }
        }

        Ok((accepted, errors))
    }

    pub async fn get_governance_events(
        &self,
        org_id: Option<&str>,
        event_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GovernanceEvent>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, org_id::text, repo_id::text, delivery_id, event_type, actor_login,
                   target, old_value, new_value, payload, created_at
            FROM governance_events
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
              AND ($2 IS NULL OR event_type = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let events: Vec<GovernanceEvent> = rows
            .iter()
            .map(|row| {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                GovernanceEvent {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    delivery_id: row.get("delivery_id"),
                    event_type: row.get("event_type"),
                    actor_login: row.get("actor_login"),
                    target: row.get("target"),
                    old_value: row.get("old_value"),
                    new_value: row.get("new_value"),
                    payload: row.get("payload"),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok(events)
    }
}
