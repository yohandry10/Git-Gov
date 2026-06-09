use super::super::models::*;
use super::ControlPlaneClient;

impl ControlPlaneClient {
    pub fn get_me(&self) -> Result<MeResponse, ServerError> {
        let url = format!("{}/me", self.config.url);
        let mut request = self.client.get(&url);
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

    pub fn create_org(&self, payload: &CreateOrgRequest) -> Result<CreateOrgResponse, ServerError> {
        let url = format!("{}/orgs", self.config.url);
        let mut request = self.client.post(&url).json(payload);
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

    pub fn create_org_user(
        &self,
        payload: &CreateOrgUserRequest,
    ) -> Result<CreateOrgUserResponse, ServerError> {
        let url = format!("{}/org-users", self.config.url);
        let mut request = self.client.post(&url).json(payload);
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

    pub fn list_org_users(
        &self,
        org_name: Option<&str>,
        status: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<OrgUsersResponse, ServerError> {
        let url = format!("{}/org-users", self.config.url);
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }
        if let Some(status) = status {
            query_params.push(("status".to_string(), status.to_string()));
        }
        query_params.push(("limit".to_string(), limit.to_string()));
        query_params.push(("offset".to_string(), offset.to_string()));

        let mut request = self.client.get(&url).query(&query_params);
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

    pub fn update_org_user_status(
        &self,
        user_id: &str,
        status: &str,
    ) -> Result<OrgUser, ServerError> {
        let url = self.endpoint_url(&["org-users", user_id, "status"])?;
        let mut request = self.client.patch(url).json(&UpdateOrgUserStatusRequest {
            status: status.to_string(),
        });
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

    pub fn create_api_key_for_org_user(
        &self,
        user_id: &str,
    ) -> Result<ApiKeyResponse, ServerError> {
        let url = self.endpoint_url(&["org-users", user_id, "api-key"])?;
        let mut request = self.client.post(url).body("");
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

    pub fn create_org_invitation(
        &self,
        payload: &CreateOrgInvitationRequest,
    ) -> Result<CreateOrgInvitationResponse, ServerError> {
        let url = format!("{}/org-invitations", self.config.url);
        let mut request = self.client.post(&url).json(payload);
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

    pub fn list_org_invitations(
        &self,
        org_name: Option<&str>,
        status: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<OrgInvitationsResponse, ServerError> {
        let url = format!("{}/org-invitations", self.config.url);
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }
        if let Some(status) = status {
            query_params.push(("status".to_string(), status.to_string()));
        }
        query_params.push(("limit".to_string(), limit.to_string()));
        query_params.push(("offset".to_string(), offset.to_string()));

        let mut request = self.client.get(&url).query(&query_params);
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

    pub fn resend_org_invitation(
        &self,
        invitation_id: &str,
        payload: &ResendOrgInvitationRequest,
    ) -> Result<CreateOrgInvitationResponse, ServerError> {
        let url = self.endpoint_url(&["org-invitations", invitation_id, "resend"])?;
        let mut request = self.client.post(url).json(payload);
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

    pub fn revoke_org_invitation(&self, invitation_id: &str) -> Result<OrgInvitation, ServerError> {
        let url = self.endpoint_url(&["org-invitations", invitation_id, "revoke"])?;
        let mut request = self.client.post(url).body("");
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

    pub fn preview_org_invitation(&self, token: &str) -> Result<OrgInvitation, ServerError> {
        let url = self.endpoint_url(&["org-invitations", "preview", token])?;
        let response = self
            .client
            .get(url)
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

    pub fn accept_org_invitation(
        &self,
        payload: &AcceptOrgInvitationRequest,
    ) -> Result<AcceptOrgInvitationResponse, ServerError> {
        let url = format!("{}/org-invitations/accept", self.config.url);
        let response = self
            .client
            .post(url)
            .json(payload)
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

    pub fn list_api_keys(&self) -> Result<Vec<ApiKeyInfo>, ServerError> {
        let url = format!("{}/api-keys", self.config.url);
        let mut request = self.client.get(&url);
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

    pub fn revoke_api_key(&self, key_id: &str) -> Result<RevokeApiKeyResponse, ServerError> {
        let url = self.endpoint_url(&["api-keys", key_id, "revoke"])?;
        let mut request = self.client.post(url).body("");
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        if !response.status().is_success() && response.status().as_u16() != 404 {
            return Err(ServerError::ServerError(format!(
                "Server returned status: {}",
                response.status()
            )));
        }
        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    // ── Chat & Feature Requests ─────────────────────────────────────────────
}
