use super::super::models::*;
use super::ControlPlaneClient;

impl ControlPlaneClient {
    pub fn get_enterprise_adoption_profile(
        &self,
        org_name: Option<&str>,
    ) -> Result<EnterpriseAdoptionProfileResponse, ServerError> {
        let url = self.endpoint_url(&["enterprise", "adoption-profile"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }

        let mut request = self.client.get(url).query(&query_params);
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

    pub fn upsert_enterprise_adoption_profile(
        &self,
        payload: &UpsertEnterpriseAdoptionProfileRequest,
    ) -> Result<EnterpriseAdoptionProfileRecord, ServerError> {
        let url = self.endpoint_url(&["enterprise", "adoption-profile"])?;
        let mut request = self.client.put(url).json(payload);
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

    pub fn get_enterprise_onboarding_checklist_tracking(
        &self,
        org_name: Option<&str>,
    ) -> Result<EnterpriseOnboardingChecklistTrackingResponse, ServerError> {
        let url = self.endpoint_url(&["enterprise", "onboarding-checklist-tracking"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = org_name {
            query_params.push(("org_name".to_string(), org_name.to_string()));
        }

        let mut request = self.client.get(url).query(&query_params);
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

    pub fn upsert_enterprise_onboarding_checklist_tracking(
        &self,
        payload: &UpsertEnterpriseOnboardingChecklistTrackingRequest,
    ) -> Result<EnterpriseOnboardingChecklistTrackingRecord, ServerError> {
        let url = self.endpoint_url(&["enterprise", "onboarding-checklist-tracking"])?;
        let mut request = self.client.put(url).json(payload);
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

    pub fn list_enterprise_release_approvals(
        &self,
        query: &EnterpriseReleaseApprovalQuery,
    ) -> Result<EnterpriseReleaseApprovalListResponse, ServerError> {
        let url = self.endpoint_url(&["enterprise", "release-approvals"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(repository_full_name) = &query.repository_full_name {
            query_params.push((
                "repository_full_name".to_string(),
                repository_full_name.clone(),
            ));
        }
        if let Some(release_id) = &query.release_id {
            query_params.push(("release_id".to_string(), release_id.clone()));
        }
        if let Some(environment) = &query.environment {
            query_params.push(("environment".to_string(), environment.clone()));
        }
        if let Some(decision) = &query.decision {
            query_params.push(("decision".to_string(), decision.clone()));
        }
        if let Some(limit) = query.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = query.offset {
            query_params.push(("offset".to_string(), offset.to_string()));
        }

        let mut request = self.client.get(url).query(&query_params);
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

    pub fn evaluate_enterprise_release_governance(
        &self,
        query: &EnterpriseReleaseGovernanceEvaluationQuery,
    ) -> Result<EnterpriseReleaseGovernanceEvaluationResponse, ServerError> {
        let url = self.endpoint_url(&["enterprise", "release-governance", "evaluate"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        query_params.push((
            "repository_full_name".to_string(),
            query.repository_full_name.clone(),
        ));
        query_params.push(("release_id".to_string(), query.release_id.clone()));
        query_params.push(("environment".to_string(), query.environment.clone()));
        if let Some(evidence_packet_hash) = &query.evidence_packet_hash {
            query_params.push((
                "evidence_packet_hash".to_string(),
                evidence_packet_hash.clone(),
            ));
        }

        let mut request = self.client.get(url).query(&query_params);
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

    pub fn create_enterprise_release_approval(
        &self,
        payload: &CreateEnterpriseReleaseApprovalRequest,
    ) -> Result<EnterpriseReleaseApprovalRecord, ServerError> {
        let url = self.endpoint_url(&["enterprise", "release-approvals"])?;
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
}
