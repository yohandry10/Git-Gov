use super::*;

pub struct PrMergeEvidenceForTicketPacketQuery<'a> {
    pub scope_org_id: Option<&'a str>,
    pub org_name: Option<&'a str>,
    pub repo_full_name: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub target_sha: Option<&'a str>,
    pub ticket_id: &'a str,
    pub commit_shas: &'a [String],
    pub hours: i64,
}

pub struct PipelineRunsForEvidencePacketQuery<'a> {
    pub scope_org_id: &'a str,
    pub repo_full_name: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub commit_shas: &'a [String],
    pub allow_legacy_scope_fallback: bool,
}

fn contains_ticket_id_with_ascii_boundary(value: &str, ticket: &str) -> bool {
    let haystack = value.to_ascii_uppercase();
    let needle = ticket.to_ascii_uppercase();
    if needle.is_empty() {
        return false;
    }

    haystack.match_indices(&needle).any(|(index, _)| {
        let bytes = haystack.as_bytes();
        let before_is_boundary = index == 0
            || bytes
                .get(index.saturating_sub(1))
                .is_some_and(|byte| !byte.is_ascii_alphanumeric());
        let after_index = index + needle.len();
        let after_is_boundary = after_index >= bytes.len()
            || bytes
                .get(after_index)
                .is_some_and(|byte| !byte.is_ascii_alphanumeric());

        before_is_boundary && after_is_boundary
    })
}

fn sha_matches_packet_scope(value: Option<&str>, commit_scope: &HashSet<String>) -> bool {
    value
        .map(str::trim)
        .filter(|sha| !sha.is_empty())
        .map(str::to_ascii_lowercase)
        .is_some_and(|sha| commit_scope.contains(&sha))
}

fn sha_matches_target(value: Option<&str>, target_sha: &str) -> bool {
    value
        .map(str::trim)
        .filter(|sha| !sha.is_empty())
        .map(str::to_ascii_lowercase)
        .is_some_and(|sha| sha == target_sha)
}

fn pr_merge_matches_ticket_packet(
    entry: &PrMergeEvidenceEntry,
    ticket: &str,
    commit_scope: &HashSet<String>,
    target_sha: Option<&str>,
) -> bool {
    if let Some(target_sha) = target_sha {
        return sha_matches_target(entry.head_sha.as_deref(), target_sha)
            || sha_matches_target(entry.merge_commit_sha.as_deref(), target_sha);
    }

    let title_match = entry
        .pr_title
        .as_deref()
        .is_some_and(|title| contains_ticket_id_with_ascii_boundary(title, ticket));
    let head_match = sha_matches_packet_scope(entry.head_sha.as_deref(), commit_scope);
    let merge_match = sha_matches_packet_scope(entry.merge_commit_sha.as_deref(), commit_scope);

    title_match || head_match || merge_match
}

impl Database {
    pub async fn store_release_evidence_packet_binding(
        &self,
        binding: &ReleaseEvidencePacketBinding,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO release_evidence_packets (
                id,
                org_id,
                ticket_id,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                evidence_packet_hash,
                evidence_packet_uri,
                packet,
                generated_by,
                generated_at
            )
            VALUES (
                $1::uuid,
                $2::uuid,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11::jsonb,
                $12,
                to_timestamp(($13::BIGINT)::double precision / 1000.0)
            )
            ON CONFLICT (org_id, evidence_packet_hash) DO NOTHING
            "#,
        )
        .bind(&binding.id)
        .bind(&binding.org_id)
        .bind(&binding.ticket_id)
        .bind(&binding.release_id)
        .bind(&binding.repository_full_name)
        .bind(&binding.branch)
        .bind(&binding.target_sha)
        .bind(&binding.environment)
        .bind(&binding.evidence_packet_hash)
        .bind(&binding.evidence_packet_uri)
        .bind(&binding.packet)
        .bind(&binding.generated_by)
        .bind(binding.generated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_release_evidence_packet_binding(
        &self,
        org_id: &str,
        evidence_packet_hash: &str,
    ) -> Result<Option<ReleaseEvidencePacketBinding>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                ticket_id,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                evidence_packet_hash,
                evidence_packet_uri,
                packet,
                generated_by,
                ROUND(EXTRACT(EPOCH FROM generated_at) * 1000)::BIGINT AS generated_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM release_evidence_packets
            WHERE org_id = $1::uuid
              AND evidence_packet_hash = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(evidence_packet_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| ReleaseEvidencePacketBinding {
            id: row.get("id"),
            org_id: row.get("org_id"),
            ticket_id: row.get("ticket_id"),
            release_id: row.get("release_id"),
            repository_full_name: row.get("repository_full_name"),
            branch: row.get("branch"),
            target_sha: row.get("target_sha"),
            environment: row.get("environment"),
            evidence_packet_hash: row.get("evidence_packet_hash"),
            evidence_packet_uri: row.get("evidence_packet_uri"),
            packet: row.get("packet"),
            generated_by: row.get("generated_by"),
            generated_at: row.get("generated_at_ms"),
            created_at: row.get("created_at_ms"),
        }))
    }

    pub async fn get_pr_merge_evidence_for_ticket_packet(
        &self,
        query: PrMergeEvidenceForTicketPacketQuery<'_>,
    ) -> Result<Vec<PrMergeEvidenceEntry>, DbError> {
        let org_id = if let Some(name) = query.org_name {
            self.get_org_by_login(name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(name) = query.repo_full_name {
            self.get_repo_by_full_name(name).await?.map(|r| r.id)
        } else {
            None
        };

        if query.org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if query.repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let ticket = query.ticket_id.trim().to_ascii_uppercase();
        let rows = sqlx::query(
            r#"
            WITH commit_scope AS (
                SELECT LOWER(value) AS commit_sha
                FROM unnest($5::text[]) AS value
                WHERE value IS NOT NULL
                  AND value <> ''
            )
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
              AND ($7::text IS NULL OR prm.base_branch = $7)
              AND (
                (
                  $8::text IS NULL
                  AND (
                    POSITION($4 IN UPPER(COALESCE(prm.pr_title, ''))) > 0
                    OR EXISTS (
                      SELECT 1
                      FROM commit_scope cs
                      WHERE LOWER(COALESCE(NULLIF(prm.head_sha, ''), '')) = cs.commit_sha
                         OR LOWER(COALESCE(
                              NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                              NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                              ''
                            )) = cs.commit_sha
                    )
                  )
                )
                OR (
                  $8::text IS NOT NULL
                  AND (
                    LOWER(COALESCE(NULLIF(prm.head_sha, ''), '')) = LOWER($8)
                    OR LOWER(COALESCE(
                         NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                         NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                         ''
                       )) = LOWER($8)
                  )
                )
              )
            ORDER BY prm.id, prm.created_at DESC
            LIMIT 50
            "#,
        )
        .bind(query.scope_org_id)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket)
        .bind(query.commit_shas)
        .bind(query.hours as i32)
        .bind(query.branch)
        .bind(query.target_sha)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let commit_scope: HashSet<String> = query
            .commit_shas
            .iter()
            .map(|sha| sha.trim().to_ascii_lowercase())
            .filter(|sha| !sha.is_empty())
            .collect();
        let target_sha = query
            .target_sha
            .map(str::trim)
            .filter(|sha| !sha.is_empty())
            .map(str::to_ascii_lowercase);

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
            .filter(|entry| {
                pr_merge_matches_ticket_packet(entry, &ticket, &commit_scope, target_sha.as_deref())
            })
            .collect())
    }

    pub async fn get_pipeline_runs_for_evidence_packet(
        &self,
        query: PipelineRunsForEvidencePacketQuery<'_>,
    ) -> Result<Vec<CommitPipelineRun>, DbError> {
        if query.commit_shas.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query(
            r#"
            WITH commit_scope AS (
                SELECT LOWER(value) AS commit_sha
                FROM unnest($4::text[]) AS value
                WHERE value IS NOT NULL
                  AND value <> ''
            )
            SELECT
                pe.id::text AS pipeline_event_id,
                pe.pipeline_id,
                pe.job_name,
                pe.status AS pipeline_status,
                pe.branch AS pipeline_branch,
                pe.repo_full_name AS pipeline_repo_full_name,
                pe.duration_ms,
                pe.triggered_by,
                EXTRACT(EPOCH FROM pe.ingested_at)::bigint * 1000 AS ingested_at_ms
            FROM pipeline_events pe
            WHERE pe.org_id = $1::uuid
              AND pe.commit_sha IS NOT NULL
              AND (
                ($5::boolean AND ($2::text IS NULL OR pe.repo_full_name IS NULL OR pe.repo_full_name = $2))
                OR (NOT $5::boolean AND ($2::text IS NULL OR pe.repo_full_name = $2))
              )
              AND (
                ($5::boolean AND ($3::text IS NULL OR pe.branch IS NULL OR pe.branch = $3))
                OR (NOT $5::boolean AND ($3::text IS NULL OR pe.branch = $3))
              )
              AND EXISTS (
                SELECT 1
                FROM commit_scope cs
                WHERE LOWER(pe.commit_sha) = cs.commit_sha
                   OR (
                     length(pe.commit_sha) < 40
                     AND cs.commit_sha LIKE LOWER(pe.commit_sha) || '%'
                   )
                   OR (
                     length(cs.commit_sha) < 40
                     AND LOWER(pe.commit_sha) LIKE cs.commit_sha || '%'
                   )
              )
            ORDER BY pe.ingested_at DESC, pe.id
            LIMIT 100
            "#,
        )
        .bind(query.scope_org_id)
        .bind(query.repo_full_name)
        .bind(query.branch)
        .bind(query.commit_shas)
        .bind(query.allow_legacy_scope_fallback)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| CommitPipelineRun {
                pipeline_event_id: row.get("pipeline_event_id"),
                pipeline_id: row.get("pipeline_id"),
                job_name: row.get("job_name"),
                status: row.get("pipeline_status"),
                branch: row.get("pipeline_branch"),
                repo_full_name: row.get("pipeline_repo_full_name"),
                duration_ms: row.get("duration_ms"),
                triggered_by: row.get("triggered_by"),
                ingested_at: row.get("ingested_at_ms"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::contains_ticket_id_with_ascii_boundary;

    #[test]
    fn ticket_title_match_requires_ascii_boundaries() {
        assert!(contains_ticket_id_with_ascii_boundary(
            "fix(KAN-702): governed change",
            "KAN-702"
        ));
        assert!(!contains_ticket_id_with_ascii_boundary(
            "fix(KAN-7020): unrelated change",
            "KAN-702"
        ));
    }
}
