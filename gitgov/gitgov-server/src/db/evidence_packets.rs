use super::*;

impl Database {
    pub async fn get_pr_merge_evidence_for_ticket_packet(
        &self,
        scope_org_id: Option<&str>,
        org_name: Option<&str>,
        repo_full_name: Option<&str>,
        ticket_id: &str,
        commit_shas: &[String],
        hours: i64,
    ) -> Result<Vec<PrMergeEvidenceEntry>, DbError> {
        let org_id = if let Some(name) = org_name {
            self.get_org_by_login(name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(name) = repo_full_name {
            self.get_repo_by_full_name(name).await?.map(|r| r.id)
        } else {
            None
        };

        if org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let ticket = ticket_id.trim().to_ascii_uppercase();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (prm.id)
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
            WHERE prm.created_at >= NOW() - make_interval(hours => $6::int)
              AND ($1::uuid IS NULL OR prm.org_id = $1::uuid)
              AND ($2::uuid IS NULL OR prm.org_id = $2::uuid)
              AND ($3::uuid IS NULL OR prm.repo_id = $3::uuid)
              AND (
                UPPER(COALESCE(prm.pr_title, '')) LIKE '%' || $4 || '%'
                OR UPPER(COALESCE(prm.payload::text, '')) LIKE '%' || $4 || '%'
                OR (
                  cardinality($5::text[]) > 0
                  AND (
                    prm.head_sha = ANY($5::text[])
                    OR COALESCE(
                      NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                      NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', '')
                    ) = ANY($5::text[])
                  )
                )
              )
            ORDER BY prm.id, prm.created_at DESC
            LIMIT 50
            "#,
        )
        .bind(scope_org_id)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket)
        .bind(commit_shas)
        .bind(hours as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
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
                    base_branch: row.get("base_branch"),
                    created_at: row.get::<i64, _>("created_at_ms"),
                }
            })
            .collect())
    }
}
