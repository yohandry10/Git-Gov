use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};
use serde::Deserialize;

impl ControlPlaneClient {
    pub fn send_event(&self, payload: &EventPayload) -> Result<EventResponse, ServerError> {
        let url = format!("{}/events", self.config.url);

        let mut request = self.client.post(&url).json(payload);

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

    pub fn get_logs(&self, filter: &AuditFilter) -> Result<Vec<CombinedEvent>, ServerError> {
        let url = format!("{}/logs", self.config.url);

        let effective_user_login = filter
            .user_login
            .as_ref()
            .or(filter.developer_login.as_ref());
        let effective_event_type = filter.event_type.as_ref().or(filter.action.as_ref());
        let effective_repo_full_name = filter.repo_full_name.as_ref().or(filter.repo_name.as_ref());

        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(source) = &filter.source {
            query_params.push(("source".to_string(), source.clone()));
        }
        if let Some(start_date) = filter.start_date {
            query_params.push(("start_date".to_string(), start_date.to_string()));
        }
        if let Some(end_date) = filter.end_date {
            query_params.push(("end_date".to_string(), end_date.to_string()));
        }
        if let Some(user_login) = effective_user_login {
            query_params.push(("user_login".to_string(), user_login.clone()));
        }
        if let Some(event_type) = effective_event_type {
            query_params.push(("event_type".to_string(), event_type.clone()));
        }
        if let Some(status) = &filter.status {
            query_params.push(("status".to_string(), status.clone()));
        }
        if let Some(branch) = &filter.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(repo_full_name) = effective_repo_full_name {
            query_params.push(("repo_full_name".to_string(), repo_full_name.clone()));
        }
        if let Some(org_name) = &filter.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(before_created_at) = filter.before_created_at {
            query_params.push((
                "before_created_at".to_string(),
                before_created_at.to_string(),
            ));
        }
        if let Some(before_id) = &filter.before_id {
            query_params.push(("before_id".to_string(), before_id.clone()));
        }
        query_params.push(("limit".to_string(), filter.limit.to_string()));
        if filter.offset > 0 {
            query_params.push(("offset".to_string(), filter.offset.to_string()));
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

        #[derive(Deserialize)]
        struct LogsResponse {
            events: Vec<CombinedEvent>,
        }

        let result: LogsResponse = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        Ok(result.events)
    }

    pub fn get_stats(&self) -> Result<ServerStats, ServerError> {
        let url = format!("{}/stats", self.config.url);

        let mut request = self.client.get(&url);

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

    pub fn get_daily_activity(
        &self,
        filter: &DailyActivityFilter,
    ) -> Result<Vec<DailyActivityPoint>, ServerError> {
        let url = format!("{}/stats/daily", self.config.url);
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(days) = filter.days {
            query_params.push(("days".to_string(), days.to_string()));
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

    pub fn health_check(&self) -> Result<bool, ServerError> {
        let url = format!("{}/health", self.config.url);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
