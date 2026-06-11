use super::*;

impl Database {
    pub async fn insert_pr_merge(&self, record: &PrMergeRecord) -> Result<(), DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO pull_request_merges (
                id, org_id, repo_id, delivery_id, pr_number, pr_title,
                author_login, merged_by_login, head_sha, base_branch, payload
            )
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, $8, $9, $10, $11::jsonb)
            ON CONFLICT (delivery_id) DO NOTHING
            "#,
        )
        .bind(&record.id)
        .bind(&record.org_id)
        .bind(&record.repo_id)
        .bind(&record.delivery_id)
        .bind(record.pr_number)
        .bind(&record.pr_title)
        .bind(&record.author_login)
        .bind(&record.merged_by_login)
        .bind(&record.head_sha)
        .bind(&record.base_branch)
        .bind(&record.payload)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) if res.rows_affected() == 0 => Err(DbError::Duplicate(format!(
                "delivery_id: {}",
                record.delivery_id
            ))),
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate") => Err(DbError::Duplicate(format!(
                "delivery_id: {}",
                record.delivery_id
            ))),
            Err(e) => Err(DbError::DatabaseError(e.to_string())),
        }
    }

    pub async fn list_pr_merge_evidence(
        &self,
        scope_org_id: Option<&str>,
        org_name: Option<&str>,
        repo_full_name: Option<&str>,
        merged_by: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PrMergeEvidenceEntry>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                prm.id::text AS id,
                prm.org_id::text AS org_id,
                o.login AS org_name,
                prm.repo_id::text AS repo_id,
                r.full_name AS repo_full_name,
                prm.delivery_id,
                prm.pr_number,
                prm.pr_title,
                prm.author_login,
                prm.merged_by_login,
                prm.head_sha,
                prm.base_branch,
                prm.payload,
                EXTRACT(EPOCH FROM prm.created_at)::bigint * 1000 AS created_at_ms
            FROM pull_request_merges prm
            LEFT JOIN orgs o ON o.id = prm.org_id
            LEFT JOIN repos r ON r.id = prm.repo_id
            WHERE ($1::uuid IS NULL OR prm.org_id = $1::uuid)
              AND ($2::text IS NULL OR o.login = $2)
              AND ($3::text IS NULL OR r.full_name = $3)
              AND ($4::text IS NULL OR prm.merged_by_login = $4)
            ORDER BY prm.created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(scope_org_id)
        .bind(org_name)
        .bind(repo_full_name)
        .bind(merged_by)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM pull_request_merges prm
            LEFT JOIN orgs o ON o.id = prm.org_id
            LEFT JOIN repos r ON r.id = prm.repo_id
            WHERE ($1::uuid IS NULL OR prm.org_id = $1::uuid)
              AND ($2::text IS NULL OR o.login = $2)
              AND ($3::text IS NULL OR r.full_name = $3)
              AND ($4::text IS NULL OR prm.merged_by_login = $4)
            "#,
        )
        .bind(scope_org_id)
        .bind(org_name)
        .bind(repo_full_name)
        .bind(merged_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");

        let entries = rows
            .into_iter()
            .map(|row| {
                let payload: serde_json::Value = row.get("payload");
                let approvers = payload
                    .pointer("/gitgov/approvers")
                    .cloned()
                    .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
                    .unwrap_or_default();
                let approvals_count = payload
                    .pointer("/gitgov/approvals_count")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(approvers.len() as i32);
                let merge_commit_sha = payload
                    .pointer("/pull_request/merge_commit_sha")
                    .or_else(|| payload.pointer("/gitgov/merge_commit_sha"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string);

                PrMergeEvidenceEntry {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    org_name: row.get("org_name"),
                    repo_id: row.get("repo_id"),
                    repo_full_name: row.get("repo_full_name"),
                    delivery_id: row.get("delivery_id"),
                    pr_number: row.get("pr_number"),
                    pr_title: row.get("pr_title"),
                    author_login: row.get("author_login"),
                    merged_by_login: row.get("merged_by_login"),
                    approvers,
                    approvals_count,
                    head_sha: row.get("head_sha"),
                    merge_commit_sha,
                    base_branch: row.get("base_branch"),
                    created_at: row.get::<i64, _>("created_at_ms"),
                }
            })
            .collect();

        Ok((entries, total))
    }

    // ========================================================================
    // ADMIN AUDIT LOG
    // ========================================================================

    pub async fn insert_admin_audit_log(&self, entry: &AdminAuditLogEntry) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO admin_audit_log (id, actor_client_id, action, target_type, target_id, metadata)
            VALUES ($1::uuid, $2, $3, $4, $5, $6::jsonb)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.actor_client_id)
        .bind(&entry.action)
        .bind(&entry.target_type)
        .bind(&entry.target_id)
        .bind(&entry.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_admin_audit_logs(
        &self,
        actor: Option<&str>,
        action_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AdminAuditLogEntry>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                actor_client_id,
                action,
                target_type,
                target_id,
                COALESCE(metadata, '{}') AS metadata,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM admin_audit_log
            WHERE ($1::text IS NULL OR actor_client_id = $1)
              AND ($2::text IS NULL OR action = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(actor)
        .bind(action_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM admin_audit_log
            WHERE ($1::text IS NULL OR actor_client_id = $1)
              AND ($2::text IS NULL OR action = $2)
            "#,
        )
        .bind(actor)
        .bind(action_filter)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");

        let entries = rows
            .iter()
            .map(|row| AdminAuditLogEntry {
                id: row.get("id"),
                actor_client_id: row.get("actor_client_id"),
                action: row.get("action"),
                target_type: row.get("target_type"),
                target_id: row.get("target_id"),
                metadata: row.get("metadata"),
                created_at: row.get::<i64, _>("created_at_ms"),
            })
            .collect();

        Ok((entries, total))
    }

    // ========================================================================
    // BOOTSTRAP
    // ========================================================================

    pub async fn bootstrap_admin_key(&self) -> Result<Option<String>, DbError> {
        let count = self.count_api_keys().await?;

        if count > 0 {
            return Ok(None);
        }

        let api_key = uuid::Uuid::new_v4().to_string();
        let key_hash = format!("{:x}", sha2::Sha256::digest(api_key.as_bytes()));
        let client_id = "bootstrap-admin";

        self.create_api_key(&key_hash, client_id, None, &UserRole::Admin)
            .await?;

        Ok(Some(api_key))
    }

    // ========================================================================
    // HEALTH CHECK
    // ========================================================================

    pub async fn health_check(&self) -> Result<(bool, i64), DbError> {
        let result =
            sqlx::query("SELECT COUNT(*) as count FROM client_events WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = result.get("count");
        Ok((true, count))
    }

    // ========================================================================
    // NONCOMPLIANCE SIGNALS
    // ========================================================================
}
