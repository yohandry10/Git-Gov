use super::*;

impl Database {
    /// Same as get_combined_events but without the 100-record default cap.
    /// Used for compliance exports — returns up to 50,000 records.
    pub async fn get_events_for_export(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<CombinedEvent>, DbError> {
        let limit = if filter.limit == 0 {
            50_000_i32
        } else {
            filter.limit.min(50_000) as i32
        };
        let export_filter = EventFilter {
            limit: limit as usize,
            offset: 0,
            ..filter.clone()
        };
        self.get_combined_events(&export_filter).await
    }

    /// Export helper for policy drift audit events.
    /// Applies the same date/user/repo/org scoping model used in compliance exports.
    pub async fn get_policy_drift_events_for_export(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<crate::models::PolicyDriftEventRecord>, DbError> {
        let limit = if filter.limit == 0 {
            50_000_i64
        } else {
            (filter.limit.min(50_000)) as i64
        };
        let start_date = filter
            .start_date
            .and_then(chrono::DateTime::from_timestamp_millis);
        let end_date = filter
            .end_date
            .and_then(chrono::DateTime::from_timestamp_millis);

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
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
            ORDER BY created_at DESC
            LIMIT $6
            "#,
        )
        .bind(filter.org_id.as_deref())
        .bind(filter.user_login.as_deref())
        .bind(filter.repo_full_name.as_deref())
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let records = rows
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

        Ok(records)
    }

    /// Export helper for policy change requests (request/approve/reject workflow).
    /// Applies date/user/repo/org scoping compatible with compliance exports.
    pub async fn get_policy_change_requests_for_export(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<crate::models::PolicyChangeRequestRecord>, DbError> {
        let limit = if filter.limit == 0 {
            50_000_i64
        } else {
            (filter.limit.min(50_000)) as i64
        };
        let start_date = filter
            .start_date
            .and_then(chrono::DateTime::from_timestamp_millis);
        let end_date = filter
            .end_date
            .and_then(chrono::DateTime::from_timestamp_millis);

        let rows = sqlx::query(
            r#"
            SELECT
                r.id::text AS id,
                r.org_id::text AS org_id,
                r.repo_id::text AS repo_id,
                r.repo_name AS repo_name,
                r.requested_by AS requested_by,
                r.requested_checksum AS requested_checksum,
                COALESCE(r.requested_config, '{}'::jsonb) AS requested_config,
                r.reason AS reason,
                COALESCE(d.decision, 'pending') AS status,
                d.decided_by AS decided_by,
                d.note AS decision_note,
                EXTRACT(EPOCH FROM r.created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM d.created_at)::bigint * 1000 AS decided_at_ms
            FROM policy_change_requests r
            LEFT JOIN policy_change_request_decisions d
              ON d.request_id = r.id
            WHERE ($1::uuid IS NULL OR r.org_id = $1::uuid)
              AND ($2::text IS NULL OR r.requested_by = $2)
              AND ($3::text IS NULL OR r.repo_name = $3)
              AND ($4::timestamptz IS NULL OR r.created_at >= $4)
              AND ($5::timestamptz IS NULL OR r.created_at <= $5)
            ORDER BY r.created_at DESC
            LIMIT $6
            "#,
        )
        .bind(filter.org_id.as_deref())
        .bind(filter.user_login.as_deref())
        .bind(filter.repo_full_name.as_deref())
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let records = rows
            .iter()
            .map(|row| {
                let config: serde_json::Value = row.get("requested_config");
                crate::models::PolicyChangeRequestRecord {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    repo_name: row.get("repo_name"),
                    requested_by: row.get("requested_by"),
                    requested_checksum: row.get("requested_checksum"),
                    requested_config: serde_json::from_value(config).unwrap_or_default(),
                    reason: row.get("reason"),
                    status: row.get("status"),
                    decided_by: row.get("decided_by"),
                    decision_note: row.get("decision_note"),
                    created_at: row.get("created_at_ms"),
                    decided_at: row.get("decided_at_ms"),
                }
            })
            .collect();

        Ok(records)
    }

    pub async fn list_export_logs(&self, org_id: Option<&str>) -> Result<Vec<ExportLog>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                exported_by,
                export_type,
                EXTRACT(EPOCH FROM date_range_start)::bigint * 1000 AS date_range_start_ms,
                EXTRACT(EPOCH FROM date_range_end)::bigint * 1000 AS date_range_end_ms,
                COALESCE(filters, 'null'::jsonb) AS filters,
                record_count,
                content_hash,
                file_path,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM export_logs
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let logs = rows
            .iter()
            .map(|row| ExportLog {
                id: row.get("id"),
                org_id: row.get("org_id"),
                exported_by: row.get("exported_by"),
                export_type: row.get("export_type"),
                date_range_start: row.get("date_range_start_ms"),
                date_range_end: row.get("date_range_end_ms"),
                filters: row.get("filters"),
                record_count: row.get("record_count"),
                content_hash: row.get("content_hash"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at_ms"),
            })
            .collect();

        Ok(logs)
    }

    // ========================================================================
    // PIPELINE EVENTS (V1.2-A Jenkins Integration)
    // ========================================================================

    pub async fn create_export_log(&self, export: &ExportLog) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO export_logs (id, org_id, exported_by, export_type, date_range_start, date_range_end, filters, record_count, content_hash, file_path, created_at)
            VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8, $9, $10, to_timestamp($11/1000.0))
            "#,
        )
        .bind(&export.id)
        .bind(&export.org_id)
        .bind(&export.exported_by)
        .bind(&export.export_type)
        .bind(export.date_range_start.map(chrono::DateTime::from_timestamp_millis))
        .bind(export.date_range_end.map(chrono::DateTime::from_timestamp_millis))
        .bind(&export.filters)
        .bind(export.record_count)
        .bind(&export.content_hash)
        .bind(&export.file_path)
        .bind(export.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // GOVERNANCE EVENTS (Audit Log Streaming)
    // ========================================================================
}
