use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

impl ControlPlaneClient {
    pub fn ingest_cli_command(
        &self,
        payload: &CliCommandInput,
    ) -> Result<CliCommandResponse, ServerError> {
        let url = self.endpoint_url(&["cli", "commands"])?;
        let mut request = self.client.post(url).json(payload);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn list_cli_commands(
        &self,
        user_login: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<CliCommandListResponse, ServerError> {
        let url = self.endpoint_url(&["cli", "commands"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(user_login) = user_login {
            query_params.push(("user_login".to_string(), user_login.to_string()));
        }
        query_params.push(("limit".to_string(), limit.to_string()));
        query_params.push(("offset".to_string(), offset.to_string()));

        let mut request = self.client.get(url).query(&query_params);
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }
}
