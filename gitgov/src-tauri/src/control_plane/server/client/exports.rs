use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

impl ControlPlaneClient {
    pub fn export_events(
        &self,
        export_type: &str,
        start_date: Option<i64>,
        end_date: Option<i64>,
        org_name: Option<&str>,
    ) -> Result<ExportResponse, ServerError> {
        let url = format!("{}/export", self.config.url);
        let body = serde_json::json!({
            "export_type": export_type,
            "start_date": start_date,
            "end_date": end_date,
            "org_name": org_name,
        });
        let mut request = self.client.post(&url).json(&body);
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

    pub fn list_exports(&self) -> Result<Vec<ExportLogEntry>, ServerError> {
        let url = format!("{}/exports", self.config.url);
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
}
