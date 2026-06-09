use super::*;

impl Database {
    fn row_to_org_invitation(row: &sqlx::postgres::PgRow) -> OrgInvitation {
        OrgInvitation {
            id: row.get("id"),
            org_id: row.get("org_id"),
            invite_email: row.get("invite_email"),
            invite_login: row.get("invite_login"),
            role: row.get("role"),
            status: row.get("status"),
            invited_by: row.get("invited_by"),
            accepted_by: row.get("accepted_by"),
            accepted_at: row.get("accepted_at_ms"),
            revoked_by: row.get("revoked_by"),
            revoked_at: row.get("revoked_at_ms"),
            expires_at: row.get("expires_at_ms"),
            created_at: row.get("created_at_ms"),
            updated_at: row.get("updated_at_ms"),
        }
    }

    pub async fn create_org_invitation(
        &self,
        input: &CreateOrgInvitationInput<'_>,
    ) -> Result<OrgInvitation, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO org_invitations (
                org_id,
                invite_email,
                invite_login,
                role,
                token_hash,
                invited_by,
                expires_at
            )
            VALUES ($1::uuid, $2, $3, $4, $5, $6, $7)
            RETURNING
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.invite_email)
        .bind(input.invite_login)
        .bind(input.role)
        .bind(input.token_hash)
        .bind(input.invited_by)
        .bind(input.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(Self::row_to_org_invitation(&row))
    }

    pub async fn list_org_invitations(
        &self,
        org_id: &str,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrgInvitation>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                CASE
                    WHEN status = 'pending' AND expires_at < NOW() THEN 'expired'
                    ELSE status
                END AS status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            FROM org_invitations
            WHERE org_id = $1::uuid
              AND (
                    $2::text IS NULL
                    OR (
                        CASE
                            WHEN status = 'pending' AND expires_at < NOW() THEN 'expired'
                            ELSE status
                        END
                    ) = $2
              )
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
            FROM org_invitations
            WHERE org_id = $1::uuid
              AND (
                    $2::text IS NULL
                    OR (
                        CASE
                            WHEN status = 'pending' AND expires_at < NOW() THEN 'expired'
                            ELSE status
                        END
                    ) = $2
              )
            "#,
        )
        .bind(org_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");
        let entries = rows.iter().map(Self::row_to_org_invitation).collect();
        Ok((entries, total))
    }

    pub async fn resend_org_invitation(
        &self,
        invitation_id: &str,
        scope_org_id: Option<&str>,
        token_hash: &str,
        actor: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<OrgInvitation>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE org_invitations
            SET
                token_hash = $3,
                status = 'pending',
                invited_by = $4,
                expires_at = $5,
                accepted_by = NULL,
                accepted_at = NULL,
                revoked_by = NULL,
                revoked_at = NULL,
                updated_at = NOW()
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
              AND status <> 'accepted'
            RETURNING
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            "#,
        )
        .bind(invitation_id)
        .bind(scope_org_id)
        .bind(token_hash)
        .bind(actor)
        .bind(expires_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_org_invitation(&r)))
    }

    pub async fn revoke_org_invitation(
        &self,
        invitation_id: &str,
        scope_org_id: Option<&str>,
        actor: &str,
    ) -> Result<Option<OrgInvitation>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE org_invitations
            SET
                status = 'revoked',
                revoked_by = $3,
                revoked_at = NOW(),
                updated_at = NOW()
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
              AND status = 'pending'
            RETURNING
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            "#,
        )
        .bind(invitation_id)
        .bind(scope_org_id)
        .bind(actor)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_org_invitation(&r)))
    }

    pub async fn get_org_invitation_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<OrgInvitation>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                CASE
                    WHEN status = 'pending' AND expires_at < NOW() THEN 'expired'
                    ELSE status
                END AS status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            FROM org_invitations
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_org_invitation(&r)))
    }

    pub async fn accept_org_invitation(
        &self,
        token_hash: &str,
        requested_login: Option<&str>,
    ) -> Result<Option<AcceptedOrgInvitation>, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let invite_row = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            FROM org_invitations
            WHERE token_hash = $1
            FOR UPDATE
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(invite_row) = invite_row else {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Ok(None);
        };

        let invitation = Self::row_to_org_invitation(&invite_row);
        let now_ms = chrono::Utc::now().timestamp_millis();
        let is_pending = invitation.status == "pending";
        let is_not_expired = invitation.expires_at > now_ms;
        if !is_pending || !is_not_expired {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Ok(None);
        }

        let resolved_login = requested_login
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                invitation
                    .invite_login
                    .clone()
                    .map(|s| s.trim().to_string())
            })
            .or_else(|| {
                invitation
                    .invite_email
                    .as_ref()
                    .and_then(|email| email.split('@').next().map(str::trim))
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
            });

        let Some(login) = resolved_login else {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Err(DbError::DatabaseError(
                "Invitation does not have a resolvable login".to_string(),
            ));
        };

        let existing_id = sqlx::query(
            r#"
            SELECT id::text AS id
            FROM org_users
            WHERE org_id = $1::uuid
              AND login = $2
            "#,
        )
        .bind(&invitation.org_id)
        .bind(&login)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?
        .map(|r| r.get::<String, _>("id"));

        let org_user_row = if let Some(id) = existing_id {
            sqlx::query(
                r#"
                UPDATE org_users
                SET
                    email = COALESCE($2, email),
                    role = $3,
                    status = 'active',
                    updated_by = $4,
                    updated_at = NOW()
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
            .bind(id)
            .bind(invitation.invite_email.as_deref())
            .bind(&invitation.role)
            .bind(&login)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"
                INSERT INTO org_users (
                    org_id, login, display_name, email, role, status, created_by, updated_by
                )
                VALUES ($1::uuid, $2, NULL, $3, $4, 'active', $2, $2)
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
            .bind(&invitation.org_id)
            .bind(&login)
            .bind(invitation.invite_email.as_deref())
            .bind(&invitation.role)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?
        };
        let org_user = Self::row_to_org_user(&org_user_row);

        let api_key = uuid::Uuid::new_v4().to_string();
        let key_hash = format!("{:x}", sha2::Sha256::digest(api_key.as_bytes()));
        sqlx::query(
            r#"
            INSERT INTO api_keys (key_hash, client_id, org_id, role)
            VALUES ($1, $2, $3::uuid, $4)
            "#,
        )
        .bind(&key_hash)
        .bind(&org_user.login)
        .bind(&invitation.org_id)
        .bind(&invitation.role)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let accepted_row = sqlx::query(
            r#"
            UPDATE org_invitations
            SET
                status = 'accepted',
                accepted_by = $2,
                accepted_at = NOW(),
                updated_at = NOW()
            WHERE id = $1::uuid
            RETURNING
                id::text,
                org_id::text,
                invite_email,
                invite_login,
                role,
                status,
                invited_by,
                accepted_by,
                EXTRACT(EPOCH FROM accepted_at)::bigint * 1000 AS accepted_at_ms,
                revoked_by,
                EXTRACT(EPOCH FROM revoked_at)::bigint * 1000 AS revoked_at_ms,
                EXTRACT(EPOCH FROM expires_at)::bigint * 1000 AS expires_at_ms,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM updated_at)::bigint * 1000 AS updated_at_ms
            "#,
        )
        .bind(&invitation.id)
        .bind(&org_user.login)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(Some(AcceptedOrgInvitation {
            invitation: Self::row_to_org_invitation(&accepted_row),
            org_user,
            api_key,
        }))
    }

    // ========================================================================
    // CHAT QUERY ENGINE — Conversational MVP
    // ========================================================================
}
