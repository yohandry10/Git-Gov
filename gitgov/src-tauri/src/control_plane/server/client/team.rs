use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

impl ControlPlaneClient {
    pub fn get_team_overview(
        &self,
        org_name: Option<&str>,
        status: Option<&str>,
        days: i64,
        limit: usize,
        offset: usize,
    ) -> Result<TeamOverviewResponse, ServerError> {
        let url = format!("{}/team/overview", self.config.url);
        let mut query_params: Vec<(String, String)> = vec![
            ("days".to_string(), days.to_string()),
            ("limit".to_string(), limit.to_string()),
            ("offset".to_string(), offset.to_string()),
        ];
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }
        if let Some(status) = status {
            query_params.push(("status".to_string(), status.to_string()));
        }

        let mut request = self.client.get(&url).query(&query_params);
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

    pub fn get_team_repos(
        &self,
        org_name: Option<&str>,
        days: i64,
        limit: usize,
        offset: usize,
    ) -> Result<TeamReposResponse, ServerError> {
        let url = format!("{}/team/repos", self.config.url);
        let mut query_params: Vec<(String, String)> = vec![
            ("days".to_string(), days.to_string()),
            ("limit".to_string(), limit.to_string()),
            ("offset".to_string(), offset.to_string()),
        ];
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }

        let mut request = self.client.get(&url).query(&query_params);
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
