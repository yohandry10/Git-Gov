use super::*;

impl Database {
    pub(super) fn row_to_org_user(row: &sqlx::postgres::PgRow) -> OrgUser {
        OrgUser {
            id: row.get("id"),
            org_id: row.get("org_id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            email: row.get("email"),
            role: row.get("role"),
            status: row.get("status"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at_ms"),
            updated_at: row.get("updated_at_ms"),
        }
    }

    pub async fn upsert_org_user(
        &self,
        input: &UpsertOrgUserInput<'_>,
    ) -> Result<(OrgUser, bool), DbError> {
        let existing_id = sqlx::query(
            r#"
            SELECT id::text AS id
            FROM org_users
            WHERE org_id = $1::uuid
              AND login = $2
            "#,
        )
        .bind(input.org_id)
        .bind(input.login)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?
        .map(|r| r.get::<String, _>("id"));

        let created = existing_id.is_none();
        let row = if let Some(id) = existing_id {
            sqlx::query(
                r#"
                UPDATE org_users
                SET
                    display_name = COALESCE($2, display_name),
                    email        = COALESCE($3, email),
                    role         = $4,
                    status       = $5,
                    updated_by   = $6,
                    updated_at   = NOW()
                WHERE id = $1::uuid
                RETURNING
                    id::text,
                    org_id::text,
                    login,
                    display_name,
                    email,
                    role,
                    status,
                    created_by,
                    updated_by,
                    EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                    EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
                "#,
            )
            .bind(&id)
            .bind(input.display_name)
            .bind(input.email)
            .bind(input.role)
            .bind(input.status)
            .bind(input.actor)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"
                INSERT INTO org_users (
                    org_id, login, display_name, email, role, status, created_by, updated_by
                )
                VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $7)
                RETURNING
                    id::text,
                    org_id::text,
                    login,
                    display_name,
                    email,
                    role,
                    status,
                    created_by,
                    updated_by,
                    EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                    EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
                "#,
            )
            .bind(input.org_id)
            .bind(input.login)
            .bind(input.display_name)
            .bind(input.email)
            .bind(input.role)
            .bind(input.status)
            .bind(input.actor)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?
        };

        Ok((Self::row_to_org_user(&row), created))
    }

    pub async fn list_org_users(
        &self,
        org_id: &str,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrgUser>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                login,
                display_name,
                email,
                role,
                status,
                created_by,
                updated_by,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            FROM org_users
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM org_users
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR status = $2)
            "#,
        )
        .bind(org_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");
        let entries = rows.iter().map(Self::row_to_org_user).collect();
        Ok((entries, total))
    }

    pub async fn get_org_user_by_id(
        &self,
        org_user_id: &str,
        scope_org_id: Option<&str>,
    ) -> Result<Option<OrgUser>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                login,
                display_name,
                email,
                role,
                status,
                created_by,
                updated_by,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            FROM org_users
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            "#,
        )
        .bind(org_user_id)
        .bind(scope_org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_org_user(&r)))
    }

    pub async fn update_org_user_status(
        &self,
        org_user_id: &str,
        scope_org_id: Option<&str>,
        status: &str,
        actor: &str,
    ) -> Result<Option<OrgUser>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE org_users
            SET
                status     = $3,
                updated_by = $4,
                updated_at = NOW()
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            RETURNING
                id::text,
                org_id::text,
                login,
                display_name,
                email,
                role,
                status,
                created_by,
                updated_by,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            "#,
        )
        .bind(org_user_id)
        .bind(scope_org_id)
        .bind(status)
        .bind(actor)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_org_user(&r)))
    }

    pub async fn get_team_overview(
        &self,
        org_id: &str,
        status: Option<&str>,
        days: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TeamDeveloperOverview>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            WITH filtered_users AS (
                SELECT
                    ou.id,
                    ou.login,
                    ou.display_name,
                    ou.email,
                    ou.role,
                    ou.status,
                    ou.created_at
                FROM org_users ou
                WHERE ou.org_id = $1::uuid
                  AND ($2::text IS NULL OR ou.status = $2)
                ORDER BY ou.created_at DESC
                LIMIT $3 OFFSET $4
            ),
            window_events AS (
                SELECT
                    COALESCE(ica.canonical_login, c.user_login) AS user_login,
                    COALESCE(r.full_name, c.metadata->>'repo_name') AS repo_name,
                    c.event_type,
                    c.status,
                    c.created_at
                FROM client_events c
                LEFT JOIN repos r ON r.id = c.repo_id
                LEFT JOIN identity_aliases ica
                    ON ica.alias_login = c.user_login
                   AND ica.org_id = $1::uuid
                WHERE c.org_id = $1::uuid
                  AND c.created_at >= NOW() - (($5::int || ' days')::interval)
            ),
            user_metrics AS (
                SELECT
                    we.user_login,
                    MAX(we.created_at) AS last_seen,
                    COUNT(*)::bigint AS total_events,
                    COUNT(*) FILTER (WHERE we.event_type = 'commit')::bigint AS commits,
                    COUNT(*) FILTER (WHERE we.event_type IN ('attempt_push', 'successful_push', 'push'))::bigint AS pushes,
                    COUNT(*) FILTER (WHERE we.event_type = 'blocked_push' OR we.status = 'blocked')::bigint AS blocked_pushes
                FROM window_events we
                GROUP BY we.user_login
            ),
            user_repo_metrics AS (
                SELECT
                    we.user_login,
                    we.repo_name,
                    COUNT(*)::bigint AS events,
                    COUNT(*) FILTER (WHERE we.event_type = 'commit')::bigint AS commits,
                    COUNT(*) FILTER (WHERE we.event_type IN ('attempt_push', 'successful_push', 'push'))::bigint AS pushes,
                    COUNT(*) FILTER (WHERE we.event_type = 'blocked_push' OR we.status = 'blocked')::bigint AS blocked_pushes,
                    EXTRACT(EPOCH FROM MAX(we.created_at))::bigint * 1000 AS last_seen_ms
                FROM window_events we
                WHERE we.repo_name IS NOT NULL AND we.repo_name <> ''
                GROUP BY we.user_login, we.repo_name
            )
            SELECT
                fu.login,
                fu.display_name,
                fu.email,
                fu.role,
                fu.status,
                EXTRACT(EPOCH FROM um.last_seen)::bigint * 1000 AS last_seen_ms,
                COALESCE(um.total_events, 0)::bigint AS total_events,
                COALESCE(um.commits, 0)::bigint AS commits,
                COALESCE(um.pushes, 0)::bigint AS pushes,
                COALESCE(um.blocked_pushes, 0)::bigint AS blocked_pushes,
                COALESCE((
                    SELECT COUNT(*)::bigint
                    FROM user_repo_metrics urm_cnt
                    WHERE urm_cnt.user_login = fu.login
                ), 0)::bigint AS repos_active_count,
                COALESCE((
                    SELECT jsonb_agg(
                        jsonb_build_object(
                            'repo_name', urm.repo_name,
                            'events', urm.events,
                            'commits', urm.commits,
                            'pushes', urm.pushes,
                            'blocked_pushes', urm.blocked_pushes,
                            'last_seen', urm.last_seen_ms
                        )
                        ORDER BY urm.events DESC, urm.repo_name ASC
                    )
                    FROM user_repo_metrics urm
                    WHERE urm.user_login = fu.login
                ), '[]'::jsonb) AS repos
            FROM filtered_users fu
            LEFT JOIN user_metrics um ON um.user_login = fu.login
            ORDER BY fu.created_at DESC
            "#,
        )
        .bind(org_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM org_users
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR status = $2)
            "#,
        )
        .bind(org_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");
        let entries = rows
            .iter()
            .map(|row| {
                let repos_json: serde_json::Value = row.get("repos");
                let repos: Vec<TeamRepoSummary> =
                    serde_json::from_value(repos_json).unwrap_or_default();
                TeamDeveloperOverview {
                    login: row.get("login"),
                    display_name: row.get("display_name"),
                    email: row.get("email"),
                    role: row.get("role"),
                    status: row.get("status"),
                    last_seen: row.get("last_seen_ms"),
                    total_events: row.get("total_events"),
                    commits: row.get("commits"),
                    pushes: row.get("pushes"),
                    blocked_pushes: row.get("blocked_pushes"),
                    repos_active_count: row.get("repos_active_count"),
                    repos,
                }
            })
            .collect();

        Ok((entries, total))
    }

    pub async fn get_team_repos(
        &self,
        org_id: &str,
        days: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TeamRepoOverview>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            WITH window_events AS (
                SELECT
                    COALESCE(ica.canonical_login, c.user_login) AS user_login,
                    COALESCE(r.full_name, c.metadata->>'repo_name') AS repo_name,
                    c.event_type,
                    c.status,
                    c.created_at
                FROM client_events c
                LEFT JOIN repos r ON r.id = c.repo_id
                LEFT JOIN identity_aliases ica
                    ON ica.alias_login = c.user_login
                   AND ica.org_id = $1::uuid
                WHERE c.org_id = $1::uuid
                  AND c.created_at >= NOW() - (($2::int || ' days')::interval)
            )
            SELECT
                we.repo_name,
                COUNT(DISTINCT we.user_login)::bigint AS developers_active,
                COUNT(*)::bigint AS total_events,
                COUNT(*) FILTER (WHERE we.event_type = 'commit')::bigint AS commits,
                COUNT(*) FILTER (WHERE we.event_type IN ('attempt_push', 'successful_push', 'push'))::bigint AS pushes,
                COUNT(*) FILTER (WHERE we.event_type = 'blocked_push' OR we.status = 'blocked')::bigint AS blocked_pushes,
                EXTRACT(EPOCH FROM MAX(we.created_at))::bigint * 1000 AS last_seen_ms
            FROM window_events we
            WHERE we.repo_name IS NOT NULL AND we.repo_name <> ''
            GROUP BY we.repo_name
            ORDER BY total_events DESC, we.repo_name ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id)
        .bind(days)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count_row = sqlx::query(
            r#"
            WITH window_events AS (
                SELECT
                    COALESCE(r.full_name, c.metadata->>'repo_name') AS repo_name
                FROM client_events c
                LEFT JOIN repos r ON r.id = c.repo_id
                WHERE c.org_id = $1::uuid
                  AND c.created_at >= NOW() - (($2::int || ' days')::interval)
            )
            SELECT COUNT(DISTINCT repo_name) AS total
            FROM window_events
            WHERE repo_name IS NOT NULL AND repo_name <> ''
            "#,
        )
        .bind(org_id)
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");
        let entries = rows
            .iter()
            .map(|row| TeamRepoOverview {
                repo_name: row.get("repo_name"),
                developers_active: row.get("developers_active"),
                total_events: row.get("total_events"),
                commits: row.get("commits"),
                pushes: row.get("pushes"),
                blocked_pushes: row.get("blocked_pushes"),
                last_seen: row.get("last_seen_ms"),
            })
            .collect();

        Ok((entries, total))
    }
}
