use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

impl ControlPlaneClient {
    pub fn chat_ask(&self, request: &ChatAskRequest) -> Result<ChatAskResponse, ServerError> {
        let url = format!("{}/chat/ask", self.config.url);
        let mut req = self.client.post(&url).json(request);
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = req
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if let Ok(parsed) = serde_json::from_str::<ChatAskResponse>(&body) {
            return Ok(parsed);
        }

        if !status.is_success() {
            let snippet = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("error")
                        .or_else(|| value.get("message"))
                        .and_then(|message| message.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| body.chars().take(180).collect::<String>());
            return Err(ServerError::ServerError(format!(
                "Server returned status: {} ({})",
                status, snippet
            )));
        }

        Err(ServerError::SerializationError(
            "Invalid chat response payload".to_string(),
        ))
    }

    pub fn create_feature_request(
        &self,
        input: &FeatureRequestInput,
    ) -> Result<FeatureRequestCreated, ServerError> {
        let url = format!("{}/feature-requests", self.config.url);
        let mut req = self.client.post(&url).json(input);
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = req
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
