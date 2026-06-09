use super::super::models::*;
use super::ControlPlaneClient;
use serde::Deserialize;

impl ControlPlaneClient {
    pub fn get_jenkins_correlations(
        &self,
        filter: &JenkinsCorrelationFilter,
    ) -> Result<Vec<CommitPipelineCorrelation>, ServerError> {
        let url = format!("{}/integrations/jenkins/correlations", self.config.url);

        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &filter.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(repo_full_name) = &filter.repo_full_name {
            query_params.push(("repo_full_name".to_string(), repo_full_name.clone()));
        }
        if let Some(branch) = &filter.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(user_login) = &filter.user_login {
            query_params.push(("user_login".to_string(), user_login.clone()));
        }
        query_params.push(("limit".to_string(), filter.limit.to_string()));
        query_params.push(("offset".to_string(), filter.offset.to_string()));

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

        #[derive(Deserialize)]
        struct CorrelationsResponse {
            correlations: Vec<CommitPipelineCorrelation>,
        }

        let result: CorrelationsResponse = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        Ok(result.correlations)
    }

    pub fn get_pr_merges(
        &self,
        filter: &PrMergeEvidenceFilter,
    ) -> Result<Vec<PrMergeEvidenceEntry>, ServerError> {
        let url = format!("{}/pr-merges", self.config.url);

        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &filter.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(repo_full_name) = &filter.repo_full_name {
            query_params.push(("repo_full_name".to_string(), repo_full_name.clone()));
        }
        if let Some(merged_by) = &filter.merged_by {
            query_params.push(("merged_by".to_string(), merged_by.clone()));
        }
        query_params.push(("limit".to_string(), filter.limit.to_string()));
        query_params.push(("offset".to_string(), filter.offset.to_string()));

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

        #[derive(Deserialize)]
        struct PrMergesResponse {
            entries: Vec<PrMergeEvidenceEntry>,
        }

        let result: PrMergesResponse = response
            .json()
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        Ok(result.entries)
    }

    pub fn get_jira_ticket_coverage(
        &self,
        query: &TicketCoverageQuery,
    ) -> Result<TicketCoverageResponse, ServerError> {
        let url = format!("{}/integrations/jira/ticket-coverage", self.config.url);

        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(repo_full_name) = &query.repo_full_name {
            query_params.push(("repo_full_name".to_string(), repo_full_name.clone()));
        }
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(hours) = query.hours {
            query_params.push(("hours".to_string(), hours.to_string()));
        }

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

    pub fn correlate_jira_tickets(
        &self,
        request_body: &JiraCorrelateRequest,
    ) -> Result<JiraCorrelateResponse, ServerError> {
        let url = format!("{}/integrations/jira/correlate", self.config.url);

        let mut request = self.client.post(&url).json(request_body);
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

    pub fn get_jira_ticket_detail(
        &self,
        ticket_id: &str,
    ) -> Result<JiraTicketDetailResponse, ServerError> {
        let url = self.endpoint_url(&["integrations", "jira", "tickets", ticket_id])?;
        let mut request = self.client.get(url);
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Ok(JiraTicketDetailResponse {
                found: false,
                ticket: None,
            });
        }
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

    pub fn get_ticket_evidence_packet(
        &self,
        ticket_id: &str,
        query: &EvidencePacketQuery,
    ) -> Result<EvidencePacketResponse, ServerError> {
        let url = self.endpoint_url(&["evidence", "packets", "tickets", ticket_id])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        if let Some(repo_full_name) = &query.repo_full_name {
            query_params.push(("repo_full_name".to_string(), repo_full_name.clone()));
        }
        if let Some(branch) = &query.branch {
            query_params.push(("branch".to_string(), branch.clone()));
        }
        if let Some(hours) = query.hours {
            query_params.push(("hours".to_string(), hours.to_string()));
        }

        let mut request = self.client.get(url).query(&query_params);
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Ok(EvidencePacketResponse {
                found: false,
                packet: None,
            });
        }
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
