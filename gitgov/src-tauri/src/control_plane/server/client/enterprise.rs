use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

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
            return Err(server_error_from_response(response));
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
            return Err(server_error_from_response(response));
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
            return Err(server_error_from_response(response));
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn get_first_governed_repo_setup(
        &self,
        org_name: Option<&str>,
    ) -> Result<FirstGovernedRepoSetupResponse, ServerError> {
        let url = self.endpoint_url(&["enterprise", "first-governed-repo-setup"])?;
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn upsert_first_governed_repo_setup(
        &self,
        payload: &UpsertFirstGovernedRepoSetupRequest,
    ) -> Result<FirstGovernedRepoSetupRecord, ServerError> {
        let url = self.endpoint_url(&["enterprise", "first-governed-repo-setup"])?;
        let mut request = self.client.put(url).json(payload);
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

    pub fn get_first_governed_repo_wizard_state(
        &self,
        org_name: Option<&str>,
    ) -> Result<FirstGovernedRepoWizardStateResponse, ServerError> {
        let url = self.endpoint_url(&["onboarding", "first-governed-repo", "state"])?;
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn create_first_governed_repo_wizard_run(
        &self,
        payload: &FirstGovernedRepoWizardActionRequest,
    ) -> Result<FirstGovernedRepoWizardRunResponse, ServerError> {
        let url = self.endpoint_url(&["onboarding", "first-governed-repo", "runs"])?;
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

    pub fn update_first_governed_repo_wizard_run(
        &self,
        run_id: &str,
        payload: &FirstGovernedRepoWizardActionRequest,
    ) -> Result<FirstGovernedRepoWizardRunResponse, ServerError> {
        let url = self.endpoint_url(&["onboarding", "first-governed-repo", "runs", run_id])?;
        let mut request = self.client.patch(url).json(payload);
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

    pub fn validate_first_governed_repo_wizard_run(
        &self,
        run_id: &str,
        payload: &FirstGovernedRepoWizardActionRequest,
    ) -> Result<FirstGovernedRepoWizardRunResponse, ServerError> {
        let url = self.endpoint_url(&[
            "onboarding",
            "first-governed-repo",
            "runs",
            run_id,
            "validate",
        ])?;
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

    pub fn plan_first_governed_repo_wizard_run(
        &self,
        run_id: &str,
        payload: &FirstGovernedRepoWizardActionRequest,
    ) -> Result<FirstGovernedRepoWizardRunResponse, ServerError> {
        let url =
            self.endpoint_url(&["onboarding", "first-governed-repo", "runs", run_id, "plan"])?;
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

    pub fn complete_first_governed_repo_wizard_run(
        &self,
        run_id: &str,
        payload: &FirstGovernedRepoWizardActionRequest,
    ) -> Result<FirstGovernedRepoWizardRunResponse, ServerError> {
        let url = self.endpoint_url(&[
            "onboarding",
            "first-governed-repo",
            "runs",
            run_id,
            "complete",
        ])?;
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
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(target_sha) = &query.target_sha {
            query_params.push(("target_sha".to_string(), target_sha.clone()));
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
        if let Some(evidence_packet_hash) = &query.evidence_packet_hash {
            query_params.push((
                "evidence_packet_hash".to_string(),
                evidence_packet_hash.clone(),
            ));
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
            return Err(server_error_from_response(response));
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
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(target_sha) = &query.target_sha {
            query_params.push(("target_sha".to_string(), target_sha.clone()));
        }
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn list_deployment_gate_authorizations(
        &self,
        query: &DeploymentGateAuthorizationQuery,
    ) -> Result<DeploymentGateAuthorizationListResponse, ServerError> {
        let url = self.endpoint_url(&["deployment-gates", "authorizations"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(authorization_id) = &query.authorization_id {
            query_params.push(("authorization_id".to_string(), authorization_id.clone()));
        }
        if let Some(repository_full_name) = &query.repository_full_name {
            query_params.push((
                "repository_full_name".to_string(),
                repository_full_name.clone(),
            ));
        }
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(target_sha) = &query.target_sha {
            query_params.push(("target_sha".to_string(), target_sha.clone()));
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
        if let Some(deployer) = &query.deployer {
            query_params.push(("deployer".to_string(), deployer.clone()));
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn list_change_risk_evaluations(
        &self,
        query: &ChangeRiskEvaluationQuery,
    ) -> Result<ChangeRiskEvaluationListResponse, ServerError> {
        let url = self.endpoint_url(&["change-risk", "evaluations"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(evaluation_id) = &query.evaluation_id {
            query_params.push(("evaluation_id".to_string(), evaluation_id.clone()));
        }
        if let Some(deployment_gate_id) = &query.deployment_gate_id {
            query_params.push(("deployment_gate_id".to_string(), deployment_gate_id.clone()));
        }
        if let Some(release_id) = &query.release_id {
            query_params.push(("release_id".to_string(), release_id.clone()));
        }
        if let Some(repository_full_name) = &query.repository_full_name {
            query_params.push((
                "repository_full_name".to_string(),
                repository_full_name.clone(),
            ));
        }
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(environment) = &query.environment {
            query_params.push(("environment".to_string(), environment.clone()));
        }
        if let Some(change_id) = &query.change_id {
            query_params.push(("change_id".to_string(), change_id.clone()));
        }
        if let Some(commit_sha) = &query.commit_sha {
            query_params.push(("commit_sha".to_string(), commit_sha.clone()));
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }

    pub fn get_change_risk_evaluation(
        &self,
        evaluation_id: &str,
        query: &ChangeRiskEvaluationQuery,
    ) -> Result<ChangeRiskEvaluationRecord, ServerError> {
        let url = self.endpoint_url(&["change-risk", "evaluations", evaluation_id])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }

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

    pub fn create_change_risk_evaluation(
        &self,
        payload: &ChangeRiskEvaluationRequest,
    ) -> Result<ChangeRiskEvaluationRecord, ServerError> {
        let url = self.endpoint_url(&["change-risk", "evaluations"])?;
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
            return Err(server_error_from_response(response));
        }

        response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))
    }
}
