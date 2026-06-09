use super::super::models::*;
use super::ControlPlaneClient;
use crate::models::GitGovConfig;
use serde::Deserialize;

impl ControlPlaneClient {
    pub fn get_policy(&self, repo_name: &str) -> Result<Option<PolicyResponse>, ServerError> {
        let url = self.endpoint_url(&["policy", repo_name])?;

        let mut request = self.client.get(url);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(ServerError::ServerError(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct PolicyApiResponse {
            version: Option<String>,
            checksum: Option<String>,
            config: Option<GitGovConfig>,
            updated_at: Option<i64>,
        }

        let result: PolicyApiResponse = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        match (
            result.version,
            result.checksum,
            result.config,
            result.updated_at,
        ) {
            (Some(v), Some(c), Some(cfg), Some(u)) => Ok(Some(PolicyResponse {
                version: v,
                checksum: c,
                config: cfg,
                updated_at: u,
            })),
            _ => Ok(None),
        }
    }

    pub fn override_policy(
        &self,
        repo_name: &str,
        config: &GitGovConfig,
    ) -> Result<PolicyResponse, ServerError> {
        let url = self.endpoint_url(&["policy", repo_name, "override"])?;

        let mut request = self.client.put(url).json(config);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ServerError::ServerError(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct PolicyApiResp {
            version: Option<String>,
            checksum: Option<String>,
            config: Option<GitGovConfig>,
            updated_at: Option<i64>,
            error: Option<String>,
        }

        let result: PolicyApiResp = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        if let Some(err) = result.error {
            return Err(ServerError::ServerError(err));
        }

        match (
            result.version,
            result.checksum,
            result.config,
            result.updated_at,
        ) {
            (Some(v), Some(c), Some(cfg), Some(u)) => Ok(PolicyResponse {
                version: v,
                checksum: c,
                config: cfg,
                updated_at: u,
            }),
            _ => Err(ServerError::ServerError(
                "Incomplete policy response".to_string(),
            )),
        }
    }

    pub fn get_policy_history(
        &self,
        repo_name: &str,
    ) -> Result<Vec<PolicyHistoryEntry>, ServerError> {
        let url = self.endpoint_url(&["policy", repo_name, "history"])?;

        let mut request = self.client.get(url);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ServerError::ServerError(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct HistoryResp {
            history: Vec<PolicyHistoryEntry>,
        }

        let result: HistoryResp = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        Ok(result.history)
    }

    pub fn policy_check(
        &self,
        repo: &str,
        branch: &str,
        user_login: Option<&str>,
        commit: Option<&str>,
    ) -> Result<PolicyCheckResponse, ServerError> {
        let url = self.endpoint_url(&["policy", "check"])?;

        let payload = PolicyCheckRequest {
            repo: repo.to_string(),
            commit: commit.map(|s| s.to_string()),
            branch: branch.to_string(),
            user_login: user_login.map(|s| s.to_string()),
        };

        let mut request = self.client.post(url).json(&payload);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ServerError::ServerError(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }
}
