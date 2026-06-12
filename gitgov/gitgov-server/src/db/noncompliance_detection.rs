use super::*;

impl Database {
    async fn detect_v2_commit_no_ticket_signals(
        &self,
        org_id: &str,
        hours: i64,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH latest_commits AS (
                SELECT DISTINCT ON (c.commit_sha)
                    c.id,
                    c.org_id,
                    c.repo_id,
                    c.user_login,
                    c.branch,
                    c.commit_sha,
                    c.created_at,
                    COALESCE(r.full_name, c.metadata->>'repo_name') AS repo_name
                FROM client_events c
                LEFT JOIN repos r ON r.id = c.repo_id
                WHERE c.org_id = $1::uuid
                  AND c.event_type = 'commit'
                  AND c.commit_sha IS NOT NULL
                  AND c.commit_sha <> ''
                  AND c.created_at >= NOW() - make_interval(hours => $2::int)
                ORDER BY c.commit_sha, c.created_at DESC
            ),
            candidates AS (
                SELECT lc.*
                FROM latest_commits lc
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM commit_ticket_correlations ct
                    WHERE ct.commit_sha = lc.commit_sha
                      AND ((lc.org_id IS NULL AND ct.org_id IS NULL) OR ct.org_id = lc.org_id)
                )
                  AND (
                    lc.branch IN ('main', 'master')
                    OR EXISTS (
                        SELECT 1
                        FROM policies p
                        WHERE p.repo_id = lc.repo_id
                          AND jsonb_typeof(p.config->'branches'->'protected') = 'array'
                          AND (p.config->'branches'->'protected') ? lc.branch
                    )
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM noncompliance_signals ns
                    WHERE ns.org_id = lc.org_id
                      AND ns.signal_type = 'commit_no_ticket'
                      AND ns.commit_sha = lc.commit_sha
                  )
            )
            INSERT INTO noncompliance_signals (
                org_id,
                repo_id,
                client_event_id,
                signal_type,
                confidence,
                actor_login,
                branch,
                commit_sha,
                evidence,
                context
            )
            SELECT
                c.org_id,
                c.repo_id,
                c.id,
                'commit_no_ticket',
                'medium',
                c.user_login,
                c.branch,
                c.commit_sha,
                jsonb_build_object(
                    'reason', 'Commit on protected branch without linked ticket',
                    'repo_name', c.repo_name,
                    'commit_created_at', EXTRACT(EPOCH FROM c.created_at)::bigint * 1000
                ),
                jsonb_build_object(
                    'detection_window_hours', $2::int,
                    'source', 'v2_minimal'
                )
            FROM candidates c
            "#,
        )
        .bind(org_id)
        .bind(hours as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    async fn detect_v2_ticket_no_coverage_signals(
        &self,
        org_id: &str,
        hours: i64,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH candidates AS (
                SELECT
                    pt.org_id,
                    pt.ticket_id,
                    pt.status,
                    pt.title,
                    pt.priority,
                    pt.ticket_type,
                    pt.assignee,
                    pt.reporter,
                    COALESCE(pt.updated_at, pt.ingested_at) AS ticket_updated_at
                FROM project_tickets pt
                WHERE pt.org_id = $1::uuid
                  AND COALESCE(pt.updated_at, pt.ingested_at)
                      >= NOW() - make_interval(hours => $2::int)
                  AND (
                    lower(COALESCE(pt.status, '')) IN ('done', 'closed', 'resolved')
                    OR lower(COALESCE(pt.status, '')) LIKE '%done%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%closed%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%resolved%'
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM commit_ticket_correlations ct
                    WHERE ct.ticket_id = pt.ticket_id
                      AND ct.org_id = pt.org_id
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM noncompliance_signals ns
                    WHERE ns.org_id = pt.org_id
                      AND ns.signal_type = 'ticket_no_coverage'
                      AND ns.evidence->>'ticket_id' = pt.ticket_id
                  )
            )
            INSERT INTO noncompliance_signals (
                org_id,
                signal_type,
                confidence,
                actor_login,
                evidence,
                context
            )
            SELECT
                c.org_id,
                'ticket_no_coverage',
                'high',
                COALESCE(NULLIF(c.assignee, ''), NULLIF(c.reporter, ''), 'system'),
                jsonb_build_object(
                    'ticket_id', c.ticket_id,
                    'ticket_status', c.status,
                    'ticket_updated_at', CASE
                        WHEN c.ticket_updated_at IS NULL THEN NULL
                        ELSE EXTRACT(EPOCH FROM c.ticket_updated_at)::bigint * 1000
                    END,
                    'reason', 'Done ticket without correlated commits'
                ),
                jsonb_build_object(
                    'title', c.title,
                    'priority', c.priority,
                    'ticket_type', c.ticket_type,
                    'detection_window_hours', $2::int,
                    'source', 'v2_minimal'
                )
            FROM candidates c
            "#,
        )
        .bind(org_id)
        .bind(hours as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    async fn detect_v2_pipeline_failure_streak_signals(
        &self,
        org_id: &str,
        hours: i64,
        streak_size: i32,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH ranked AS (
                SELECT
                    pe.org_id,
                    COALESCE(NULLIF(pe.repo_full_name, ''), '__unknown_repo__') AS repo_name_key,
                    COALESCE(NULLIF(pe.branch, ''), '__unknown_branch__') AS branch_key,
                    pe.status,
                    pe.job_name,
                    pe.triggered_by,
                    pe.id::text AS pipeline_event_id,
                    pe.ingested_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY
                            COALESCE(NULLIF(pe.repo_full_name, ''), '__unknown_repo__'),
                            COALESCE(NULLIF(pe.branch, ''), '__unknown_branch__')
                        ORDER BY pe.ingested_at DESC, pe.id DESC
                    ) AS rn
                FROM pipeline_events pe
                WHERE pe.org_id = $1::uuid
                  AND pe.ingested_at >= NOW() - make_interval(hours => $2::int)
            ),
            streaks AS (
                SELECT
                    r.org_id,
                    r.repo_name_key,
                    r.branch_key,
                    MAX(r.ingested_at) AS latest_ingested_at,
                    (array_agg(r.pipeline_event_id ORDER BY r.ingested_at DESC, r.pipeline_event_id DESC))[1] AS latest_pipeline_event_id,
                    (array_agg(r.job_name ORDER BY r.ingested_at DESC, r.pipeline_event_id DESC))[1] AS latest_job_name,
                    COALESCE(
                        (array_agg(NULLIF(r.triggered_by, '') ORDER BY r.ingested_at DESC, r.pipeline_event_id DESC))[1],
                        'system'
                    ) AS actor_login,
                    array_agg(r.status ORDER BY r.ingested_at DESC, r.pipeline_event_id DESC) AS recent_statuses,
                    COUNT(*)::int AS sample_size
                FROM ranked r
                WHERE r.rn <= $3::int
                GROUP BY r.org_id, r.repo_name_key, r.branch_key
                HAVING COUNT(*) = $3::int
                   AND BOOL_AND(lower(COALESCE(r.status, '')) IN ('failure', 'aborted', 'unstable'))
            ),
            candidates AS (
                SELECT s.*
                FROM streaks s
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM noncompliance_signals ns
                    WHERE ns.org_id = s.org_id
                      AND ns.signal_type = 'pipeline_failure_streak'
                      AND COALESCE(ns.evidence->>'repo_name', '__unknown_repo__') = s.repo_name_key
                      AND COALESCE(ns.evidence->>'branch', '__unknown_branch__') = s.branch_key
                      AND ns.evidence->>'latest_pipeline_event_id' = s.latest_pipeline_event_id
                )
            )
            INSERT INTO noncompliance_signals (
                org_id,
                signal_type,
                confidence,
                actor_login,
                branch,
                evidence,
                context
            )
            SELECT
                c.org_id,
                'pipeline_failure_streak',
                'high',
                c.actor_login,
                NULLIF(c.branch_key, '__unknown_branch__'),
                jsonb_build_object(
                    'repo_name', NULLIF(c.repo_name_key, '__unknown_repo__'),
                    'branch', NULLIF(c.branch_key, '__unknown_branch__'),
                    'latest_pipeline_event_id', c.latest_pipeline_event_id,
                    'latest_job_name', c.latest_job_name,
                    'recent_statuses', c.recent_statuses,
                    'sample_size', c.sample_size,
                    'latest_ingested_at', EXTRACT(EPOCH FROM c.latest_ingested_at)::bigint * 1000,
                    'reason', 'Three or more consecutive failing pipelines on the same branch'
                ),
                jsonb_build_object(
                    'detection_window_hours', $2::int,
                    'streak_size', $3::int,
                    'source', 'v2_advanced'
                )
            FROM candidates c
            "#,
        )
        .bind(org_id)
        .bind(hours as i32)
        .bind(streak_size)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    async fn detect_v2_stale_in_progress_signals(
        &self,
        org_id: &str,
        hours: i64,
        stale_days: i32,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH ticket_activity AS (
                SELECT
                    pt.org_id,
                    pt.ticket_id,
                    pt.status,
                    pt.title,
                    pt.priority,
                    pt.ticket_type,
                    pt.assignee,
                    pt.reporter,
                    COALESCE(pt.updated_at, pt.ingested_at) AS ticket_updated_at,
                    corr.last_commit_at,
                    COALESCE(corr.commit_links, 0)::bigint AS commit_links,
                    CASE
                        WHEN corr.last_commit_at IS NULL THEN ''
                        ELSE (EXTRACT(EPOCH FROM corr.last_commit_at)::bigint * 1000)::text
                    END AS last_commit_at_ms_text
                FROM project_tickets pt
                LEFT JOIN LATERAL (
                    SELECT
                        MAX(c.created_at) AS last_commit_at,
                        COUNT(*) AS commit_links
                    FROM commit_ticket_correlations ct
                    LEFT JOIN client_events c
                      ON c.commit_sha = ct.commit_sha
                     AND c.event_type = 'commit'
                     AND ((pt.org_id IS NULL AND c.org_id IS NULL) OR c.org_id = pt.org_id)
                    WHERE ct.ticket_id = pt.ticket_id
                      AND ((pt.org_id IS NULL AND ct.org_id IS NULL) OR ct.org_id = pt.org_id)
                ) corr ON TRUE
                WHERE pt.org_id = $1::uuid
                  AND COALESCE(pt.updated_at, pt.ingested_at) >= NOW() - make_interval(hours => $2::int)
                  AND (
                    lower(COALESCE(pt.status, '')) IN (
                        'in progress', 'in_progress', 'doing', 'open', 'todo', 'to do', 'in review'
                    )
                    OR lower(COALESCE(pt.status, '')) LIKE '%progress%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%doing%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%review%'
                  )
            ),
            candidates AS (
                SELECT ta.*
                FROM ticket_activity ta
                WHERE (
                        ta.last_commit_at IS NULL
                        OR ta.last_commit_at < NOW() - make_interval(days => $3::int)
                      )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM noncompliance_signals ns
                    WHERE ns.org_id = ta.org_id
                      AND ns.signal_type = 'stale_in_progress'
                      AND ns.evidence->>'ticket_id' = ta.ticket_id
                      AND COALESCE(ns.evidence->>'last_commit_at', '') = ta.last_commit_at_ms_text
                  )
            )
            INSERT INTO noncompliance_signals (
                org_id,
                signal_type,
                confidence,
                actor_login,
                evidence,
                context
            )
            SELECT
                c.org_id,
                'stale_in_progress',
                'medium',
                COALESCE(NULLIF(c.assignee, ''), NULLIF(c.reporter, ''), 'system'),
                jsonb_build_object(
                    'ticket_id', c.ticket_id,
                    'ticket_status', c.status,
                    'ticket_updated_at', CASE
                        WHEN c.ticket_updated_at IS NULL THEN NULL
                        ELSE EXTRACT(EPOCH FROM c.ticket_updated_at)::bigint * 1000
                    END,
                    'last_commit_at', CASE
                        WHEN c.last_commit_at IS NULL THEN NULL
                        ELSE EXTRACT(EPOCH FROM c.last_commit_at)::bigint * 1000
                    END,
                    'correlated_commit_count', c.commit_links,
                    'reason', 'Ticket in progress without recent commit activity'
                ),
                jsonb_build_object(
                    'detection_window_hours', $2::int,
                    'stale_days', $3::int,
                    'source', 'v2_advanced'
                )
            FROM candidates c
            "#,
        )
        .bind(org_id)
        .bind(hours as i32)
        .bind(stale_days)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    async fn detect_v2_done_not_deployed_signals(
        &self,
        org_id: &str,
        done_window_hours: i64,
        pipeline_lookback_hours: i64,
    ) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH done_tickets AS (
                SELECT
                    pt.org_id,
                    pt.ticket_id,
                    pt.status,
                    pt.title,
                    pt.priority,
                    pt.ticket_type,
                    pt.assignee,
                    pt.reporter,
                    COALESCE(pt.updated_at, pt.ingested_at) AS ticket_updated_at
                FROM project_tickets pt
                WHERE pt.org_id = $1::uuid
                  AND COALESCE(pt.updated_at, pt.ingested_at) >= NOW() - make_interval(hours => $2::int)
                  AND (
                    lower(COALESCE(pt.status, '')) IN ('done', 'closed', 'resolved')
                    OR lower(COALESCE(pt.status, '')) LIKE '%done%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%closed%'
                    OR lower(COALESCE(pt.status, '')) LIKE '%resolved%'
                  )
            ),
            ticket_commits AS (
                SELECT
                    dt.*,
                    ct.commit_sha
                FROM done_tickets dt
                JOIN commit_ticket_correlations ct
                  ON ct.ticket_id = dt.ticket_id
                 AND ((dt.org_id IS NULL AND ct.org_id IS NULL) OR ct.org_id = dt.org_id)
            ),
            ticket_pipeline_eval AS (
                SELECT
                    tc.org_id,
                    tc.ticket_id,
                    tc.status,
                    tc.title,
                    tc.priority,
                    tc.ticket_type,
                    tc.assignee,
                    tc.reporter,
                    tc.ticket_updated_at,
                    COUNT(DISTINCT tc.commit_sha)::bigint AS correlated_commit_count,
                    COALESCE(BOOL_OR(
                        lower(COALESCE(pe.status, '')) = 'success'
                        AND (
                            lower(COALESCE(pe.job_name, '')) LIKE '%deploy%'
                            OR lower(COALESCE(pe.job_name, '')) LIKE '%release%'
                            OR lower(COALESCE(pe.job_name, '')) LIKE '%prod%'
                            OR lower(COALESCE(pe.payload::text, '')) LIKE '%\"environment\":\"production\"%'
                            OR lower(COALESCE(pe.payload::text, '')) LIKE '%deploy%'
                        )
                    ), FALSE) AS has_successful_deploy,
                    MAX(pe.ingested_at) FILTER (WHERE lower(COALESCE(pe.status, '')) = 'success') AS last_success_pipeline_at,
                    MAX(pe.id::text) FILTER (WHERE lower(COALESCE(pe.status, '')) = 'success') AS last_success_pipeline_id
                FROM ticket_commits tc
                LEFT JOIN pipeline_events pe
                  ON pe.org_id = tc.org_id
                 AND pe.commit_sha IS NOT NULL
                 AND (
                    pe.commit_sha = tc.commit_sha
                    OR pe.commit_sha LIKE tc.commit_sha || '%'
                    OR tc.commit_sha LIKE pe.commit_sha || '%'
                 )
                 AND pe.ingested_at >= NOW() - make_interval(hours => $3::int)
                GROUP BY
                    tc.org_id,
                    tc.ticket_id,
                    tc.status,
                    tc.title,
                    tc.priority,
                    tc.ticket_type,
                    tc.assignee,
                    tc.reporter,
                    tc.ticket_updated_at
            ),
            candidates AS (
                SELECT tpe.*
                FROM ticket_pipeline_eval tpe
                WHERE tpe.correlated_commit_count > 0
                  AND tpe.has_successful_deploy = FALSE
                  AND NOT EXISTS (
                    SELECT 1
                    FROM noncompliance_signals ns
                    WHERE ns.org_id = tpe.org_id
                      AND ns.signal_type = 'done_not_deployed'
                      AND ns.evidence->>'ticket_id' = tpe.ticket_id
                  )
            )
            INSERT INTO noncompliance_signals (
                org_id,
                signal_type,
                confidence,
                actor_login,
                evidence,
                context
            )
            SELECT
                c.org_id,
                'done_not_deployed',
                'high',
                COALESCE(NULLIF(c.assignee, ''), NULLIF(c.reporter, ''), 'system'),
                jsonb_build_object(
                    'ticket_id', c.ticket_id,
                    'ticket_status', c.status,
                    'ticket_updated_at', CASE
                        WHEN c.ticket_updated_at IS NULL THEN NULL
                        ELSE EXTRACT(EPOCH FROM c.ticket_updated_at)::bigint * 1000
                    END,
                    'correlated_commit_count', c.correlated_commit_count,
                    'last_success_pipeline_at', CASE
                        WHEN c.last_success_pipeline_at IS NULL THEN NULL
                        ELSE EXTRACT(EPOCH FROM c.last_success_pipeline_at)::bigint * 1000
                    END,
                    'last_success_pipeline_id', c.last_success_pipeline_id,
                    'reason', 'Done ticket has correlated commits but no successful deployment-like pipeline'
                ),
                jsonb_build_object(
                    'done_window_hours', $2::int,
                    'pipeline_lookback_hours', $3::int,
                    'source', 'v2_advanced'
                )
            FROM candidates c
            "#,
        )
        .bind(org_id)
        .bind(done_window_hours as i32)
        .bind(pipeline_lookback_hours as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn detect_noncompliance_signals(&self, org_id: &str) -> Result<i64, DbError> {
        // Legacy SQL detector can be unavailable/misaligned when a deployment
        // has partial migrations. Keep detection resilient and continue with
        // V2 server-side rules instead of failing the endpoint/job.
        let mut total_created: i64 = match sqlx::query(
            "SELECT detect_noncompliance_signals($1::uuid)::bigint as count",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => row.get("count"),
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "Legacy SQL detect_noncompliance_signals failed; continuing with V2 fallback detection"
                );
                0
            }
        };
        let commit_window_hours = 24 * 7;
        let ticket_window_hours = 24 * 30;
        let pipeline_streak_window_hours = 24 * 14;
        let stale_in_progress_window_hours = 24 * 30;
        let done_ticket_window_hours = 24 * 45;
        let pipeline_lookback_hours = 24 * 45;

        match self
            .detect_v2_commit_no_ticket_signals(org_id, commit_window_hours)
            .await
        {
            Ok(count) => {
                total_created += count;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "V2 commit_no_ticket detection skipped due to database error"
                );
            }
        }

        match self
            .detect_v2_ticket_no_coverage_signals(org_id, ticket_window_hours)
            .await
        {
            Ok(count) => {
                total_created += count;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "V2 ticket_no_coverage detection skipped due to database error"
                );
            }
        }

        match self
            .detect_v2_pipeline_failure_streak_signals(org_id, pipeline_streak_window_hours, 3)
            .await
        {
            Ok(count) => {
                total_created += count;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "V2 pipeline_failure_streak detection skipped due to database error"
                );
            }
        }

        match self
            .detect_v2_stale_in_progress_signals(org_id, stale_in_progress_window_hours, 3)
            .await
        {
            Ok(count) => {
                total_created += count;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "V2 stale_in_progress detection skipped due to database error"
                );
            }
        }

        match self
            .detect_v2_done_not_deployed_signals(
                org_id,
                done_ticket_window_hours,
                pipeline_lookback_hours,
            )
            .await
        {
            Ok(count) => {
                total_created += count;
            }
            Err(e) => {
                tracing::warn!(
                    org_id = %org_id,
                    error = %e,
                    "V2 done_not_deployed detection skipped due to database error"
                );
            }
        }

        Ok(total_created)
    }

    // ========================================================================
    // COMPLIANCE DASHBOARD
    // ========================================================================
}
