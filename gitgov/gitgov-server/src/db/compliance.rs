use super::*;

impl Database {
    async fn get_compliance_timeline_monthly(
        &self,
        org_id: &str,
        months: i64,
    ) -> Result<Vec<ComplianceTimelinePoint>, DbError> {
        let safe_months = months.clamp(1, 24) as i32;
        let rows = sqlx::query(
            r#"
            WITH bounds AS (
              SELECT
                date_trunc('month', (NOW() AT TIME ZONE 'UTC'))::date AS end_month,
                (
                  date_trunc('month', (NOW() AT TIME ZONE 'UTC'))::date
                  - (($2::int - 1) * INTERVAL '1 month')
                )::date AS start_month
            ),
            months AS (
              SELECT generate_series(
                (SELECT start_month FROM bounds),
                (SELECT end_month FROM bounds),
                INTERVAL '1 month'
              )::date AS month_start
            ),
            signals AS (
              SELECT
                date_trunc('month', ns.created_at AT TIME ZONE 'UTC')::date AS month_start,
                COUNT(*)::bigint AS signals_detected
              FROM noncompliance_signals ns
              WHERE ns.org_id = $1::uuid
                AND ns.created_at >= (SELECT start_month::timestamp FROM bounds)
              GROUP BY 1
            ),
            violations AS (
              SELECT
                date_trunc('month', v.created_at AT TIME ZONE 'UTC')::date AS month_start,
                COUNT(*)::bigint AS violations_confirmed
              FROM violations v
              WHERE v.org_id = $1::uuid
                AND v.created_at >= (SELECT start_month::timestamp FROM bounds)
              GROUP BY 1
            ),
            commit_coverage AS (
              SELECT
                date_trunc('month', ce.created_at AT TIME ZONE 'UTC')::date AS month_start,
                COUNT(DISTINCT ce.commit_sha)::bigint AS commits_total,
                COUNT(DISTINCT CASE WHEN ctc.commit_sha IS NOT NULL THEN ce.commit_sha END)::bigint AS commits_with_ticket
              FROM client_events ce
              LEFT JOIN (
                SELECT DISTINCT org_id, commit_sha
                FROM commit_ticket_correlations
                WHERE org_id = $1::uuid
              ) ctc
                ON ctc.org_id = ce.org_id
               AND ctc.commit_sha = ce.commit_sha
              WHERE ce.org_id = $1::uuid
                AND ce.event_type = 'commit'
                AND ce.commit_sha IS NOT NULL
                AND ce.created_at >= (SELECT start_month::timestamp FROM bounds)
              GROUP BY 1
            ),
            pipeline AS (
              SELECT
                date_trunc('month', pe.ingested_at AT TIME ZONE 'UTC')::date AS month_start,
                COUNT(*)::bigint AS pipeline_runs_total,
                COUNT(*) FILTER (WHERE pe.status = 'success')::bigint AS pipeline_runs_success
              FROM pipeline_events pe
              WHERE pe.org_id = $1::uuid
                AND pe.ingested_at >= (SELECT start_month::timestamp FROM bounds)
              GROUP BY 1
            )
            SELECT
              to_char(m.month_start, 'YYYY-MM') AS month,
              COALESCE(s.signals_detected, 0)::bigint AS signals_detected,
              COALESCE(v.violations_confirmed, 0)::bigint AS violations_confirmed,
              COALESCE(c.commits_total, 0)::bigint AS commits_total,
              COALESCE(c.commits_with_ticket, 0)::bigint AS commits_with_ticket,
              CASE
                WHEN COALESCE(c.commits_total, 0) > 0 THEN
                  ROUND((COALESCE(c.commits_with_ticket, 0)::numeric * 100.0) / NULLIF(c.commits_total, 0), 1)::double precision
                ELSE 100.0
              END AS ticket_coverage_pct,
              COALESCE(p.pipeline_runs_total, 0)::bigint AS pipeline_runs_total,
              CASE
                WHEN COALESCE(p.pipeline_runs_total, 0) > 0 THEN
                  ROUND((COALESCE(p.pipeline_runs_success, 0)::numeric * 100.0) / NULLIF(p.pipeline_runs_total, 0), 1)::double precision
                ELSE 100.0
              END AS pipeline_success_pct
            FROM months m
            LEFT JOIN signals s ON s.month_start = m.month_start
            LEFT JOIN violations v ON v.month_start = m.month_start
            LEFT JOIN commit_coverage c ON c.month_start = m.month_start
            LEFT JOIN pipeline p ON p.month_start = m.month_start
            ORDER BY m.month_start ASC
            "#,
        )
        .bind(org_id)
        .bind(safe_months)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| ComplianceTimelinePoint {
                month: row.get("month"),
                signals_detected: row.get("signals_detected"),
                violations_confirmed: row.get("violations_confirmed"),
                commits_total: row.get("commits_total"),
                commits_with_ticket: row.get("commits_with_ticket"),
                ticket_coverage_pct: row.get("ticket_coverage_pct"),
                pipeline_runs_total: row.get("pipeline_runs_total"),
                pipeline_success_pct: row.get("pipeline_success_pct"),
            })
            .collect())
    }

    pub async fn get_compliance_dashboard(
        &self,
        org_id: &str,
    ) -> Result<ComplianceDashboard, DbError> {
        let row = sqlx::query("SELECT get_compliance_dashboard($1::uuid) as dashboard")
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let mut dashboard_value: serde_json::Value = row
            .try_get::<sqlx::types::Json<serde_json::Value>, _>("dashboard")
            .map(|json| json.0)
            .or_else(|_| row.try_get::<serde_json::Value, _>("dashboard"))
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if let Some(obj) = dashboard_value.as_object_mut() {
            for key in ["signals", "correlation", "policy", "exports"] {
                let is_null = obj.get(key).map(|v| v.is_null()).unwrap_or(false);
                if is_null {
                    obj.remove(key);
                }
            }
            let timeline_is_null = obj.get("timeline").map(|v| v.is_null()).unwrap_or(false);
            if timeline_is_null {
                obj.insert("timeline".to_string(), serde_json::json!([]));
            }
            if let Some(signals_obj) = obj.get_mut("signals").and_then(|v| v.as_object_mut()) {
                let by_type_is_null = signals_obj
                    .get("by_type")
                    .map(|v| v.is_null())
                    .unwrap_or(false);
                if by_type_is_null {
                    signals_obj.insert("by_type".to_string(), serde_json::json!({}));
                }
            }
        }

        let mut resolved = match serde_json::from_value::<ComplianceDashboard>(dashboard_value) {
            Ok(value) => value,
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "Failed to deserialize compliance dashboard payload; using defaults"
                );
                ComplianceDashboard::default()
            }
        };
        match self.get_compliance_timeline_monthly(org_id, 6).await {
            Ok(timeline) => {
                resolved.timeline = timeline;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "Monthly compliance timeline skipped due to database error"
                );
            }
        }

        Ok(resolved)
    }

    // ========================================================================
    // POLICY HISTORY
    // ========================================================================
}
