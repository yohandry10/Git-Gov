use super::*;

impl Database {
    pub async fn upsert_org(
        &self,
        github_id: i64,
        login: &str,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<String, DbError> {
        if let Some(existing) = self.get_org_by_login(login).await? {
            let result = sqlx::query(
                r#"
                UPDATE orgs
                SET
                    github_id = COALESCE(orgs.github_id, $2),
                    name = COALESCE($3, orgs.name),
                    avatar_url = COALESCE($4, orgs.avatar_url),
                    updated_at = NOW()
                WHERE id = $1::uuid
                RETURNING id::text
                "#,
            )
            .bind(&existing.id)
            .bind(github_id)
            .bind(name)
            .bind(avatar_url)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

            return Ok(result.get("id"));
        }

        let result = sqlx::query(
            r#"
            INSERT INTO orgs (github_id, login, name, avatar_url)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (github_id) DO UPDATE SET
                login = EXCLUDED.login,
                name = COALESCE($3, orgs.name),
                avatar_url = COALESCE($4, orgs.avatar_url),
                updated_at = NOW()
            RETURNING id::text
            "#,
        )
        .bind(github_id)
        .bind(login)
        .bind(name)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.get("id"))
    }

    pub async fn upsert_org_by_login(
        &self,
        login: &str,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<String, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO orgs (login, name, avatar_url)
            VALUES ($1, $2, $3)
            ON CONFLICT (login) DO UPDATE SET
                name = COALESCE($2, orgs.name),
                avatar_url = COALESCE($3, orgs.avatar_url),
                updated_at = NOW()
            RETURNING id::text
            "#,
        )
        .bind(login)
        .bind(name)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.get("id"))
    }

    pub async fn get_org_by_login(&self, login: &str) -> Result<Option<Org>, DbError> {
        let result = sqlx::query(
            "SELECT id::text, github_id, login, name, avatar_url, created_at FROM orgs WHERE login = $1"
        )
        .bind(login)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                Ok(Some(Org {
                    id: row.get("id"),
                    github_id: row.get("github_id"),
                    login: row.get("login"),
                    name: row.get("name"),
                    avatar_url: row.get("avatar_url"),
                    created_at: created_at.timestamp_millis(),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_org_by_id(&self, org_id: &str) -> Result<Option<Org>, DbError> {
        let result = sqlx::query(
            "SELECT id::text, github_id, login, name, avatar_url, created_at FROM orgs WHERE id = $1::uuid",
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                Ok(Some(Org {
                    id: row.get("id"),
                    github_id: row.get("github_id"),
                    login: row.get("login"),
                    name: row.get("name"),
                    avatar_url: row.get("avatar_url"),
                    created_at: created_at.timestamp_millis(),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_orgs(&self, org_id: Option<&str>) -> Result<Vec<Org>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, github_id, login, name, avatar_url, created_at
            FROM orgs
            WHERE ($1::uuid IS NULL OR id = $1::uuid)
            ORDER BY login ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        rows.iter()
            .map(|row| {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                Ok(Org {
                    id: row.get("id"),
                    github_id: row.get("github_id"),
                    login: row.get("login"),
                    name: row.get("name"),
                    avatar_url: row.get("avatar_url"),
                    created_at: created_at.timestamp_millis(),
                })
            })
            .collect()
    }

    pub async fn upsert_repo(
        &self,
        org_id: Option<&str>,
        github_id: i64,
        full_name: &str,
        name: &str,
        private: bool,
    ) -> Result<String, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO repos (org_id, github_id, full_name, name, private)
            VALUES ($1::uuid, $2, $3, $4, $5)
            ON CONFLICT (full_name) DO UPDATE SET
                name = $4,
                private = $5,
                updated_at = NOW()
            RETURNING id::text
            "#,
        )
        .bind(org_id)
        .bind(github_id)
        .bind(full_name)
        .bind(name)
        .bind(private)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.get("id"))
    }

    pub async fn upsert_repo_by_full_name(
        &self,
        org_id: Option<&str>,
        full_name: &str,
        name: &str,
        private: bool,
    ) -> Result<String, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO repos (org_id, github_id, full_name, name, private)
            VALUES ($1::uuid, NULL, $2, $3, $4)
            ON CONFLICT (full_name) DO UPDATE SET
                org_id = COALESCE(repos.org_id, $1::uuid),
                name = $3,
                private = $4,
                updated_at = NOW()
            RETURNING id::text
            "#,
        )
        .bind(org_id)
        .bind(full_name)
        .bind(name)
        .bind(private)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.get("id"))
    }

    pub async fn get_repo_by_full_name(&self, full_name: &str) -> Result<Option<Repo>, DbError> {
        let result = sqlx::query(
            "SELECT id::text, org_id::text, github_id, full_name, name, private, created_at FROM repos WHERE full_name = $1"
        )
        .bind(full_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                Ok(Some(Repo {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    github_id: row.get("github_id"),
                    full_name: row.get("full_name"),
                    name: row.get("name"),
                    private: row.get("private"),
                    created_at: created_at.timestamp_millis(),
                }))
            }
            None => Ok(None),
        }
    }

    // ========================================================================
    // GITHUB EVENTS (Source of Truth)
    // ========================================================================
}
