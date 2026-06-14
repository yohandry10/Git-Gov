use super::super::models::*;
use super::{server_error_from_response, ControlPlaneClient};

fn framework_review_report_query_params(
    query: &ComplianceFrameworkReviewReportQuery,
    include_filters: bool,
) -> Vec<(String, String)> {
    let mut query_params: Vec<(String, String)> = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    if include_filters {
        if let Some(framework_id) = &query.framework_id {
            query_params.push(("framework_id".to_string(), framework_id.clone()));
        }
        if let Some(mapping_id) = &query.mapping_id {
            query_params.push(("mapping_id".to_string(), mapping_id.clone()));
        }
        if let Some(review_package_id) = &query.review_package_id {
            query_params.push(("review_package_id".to_string(), review_package_id.clone()));
        }
        if let Some(limit) = query.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
    }
    query_params
}

impl ControlPlaneClient {
    pub fn list_compliance_control_frameworks(
        &self,
        query: &ComplianceFrameworkPackQuery,
    ) -> Result<ComplianceControlFrameworkListResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "control-frameworks"])?;
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

    pub fn import_compliance_framework_pack(
        &self,
        payload: &ComplianceFrameworkPackImportRequest,
    ) -> Result<ComplianceFrameworkPackImportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-packs", "import"])?;
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

    pub fn list_compliance_framework_packs(
        &self,
        query: &ComplianceFrameworkPackQuery,
    ) -> Result<ComplianceFrameworkPackListResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-packs"])?;
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

    pub fn review_compliance_framework_pack(
        &self,
        framework_pack_id: &str,
        payload: &ComplianceFrameworkPackReviewRequest,
    ) -> Result<ComplianceFrameworkPackRecord, ServerError> {
        let url =
            self.endpoint_url(&["compliance", "framework-packs", framework_pack_id, "review"])?;
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

    pub fn create_compliance_evidence_export(
        &self,
        payload: &ComplianceEvidenceExportRequest,
    ) -> Result<ComplianceEvidenceExportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "evidence-exports"])?;
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

    pub fn get_compliance_evidence_export(
        &self,
        export_id: &str,
        query: &ComplianceEvidenceExportQuery,
    ) -> Result<ComplianceEvidenceExportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "evidence-exports", export_id])?;
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

    pub fn download_compliance_evidence_export(
        &self,
        export_id: &str,
        query: &ComplianceEvidenceExportQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url = self.endpoint_url(&["compliance", "evidence-exports", export_id, "download"])?;
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

    pub fn create_compliance_evidence_mapping(
        &self,
        payload: &ComplianceEvidenceMappingRequest,
    ) -> Result<ComplianceEvidenceMappingResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "evidence-mappings"])?;
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

    pub fn get_compliance_evidence_mapping(
        &self,
        mapping_id: &str,
        query: &ComplianceEvidenceMappingQuery,
    ) -> Result<ComplianceEvidenceMappingResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "evidence-mappings", mapping_id])?;
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

    pub fn create_compliance_review_package(
        &self,
        payload: &ComplianceReviewPackageRequest,
    ) -> Result<ComplianceReviewPackageResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "review-packages"])?;
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

    pub fn get_compliance_review_package(
        &self,
        review_package_id: &str,
        query: &ComplianceReviewPackageQuery,
    ) -> Result<ComplianceReviewPackageResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "review-packages", review_package_id])?;
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

    pub fn download_compliance_review_package(
        &self,
        review_package_id: &str,
        query: &ComplianceReviewPackageQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "review-packages",
            review_package_id,
            "download",
        ])?;
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

    pub fn create_compliance_framework_review_report(
        &self,
        payload: &ComplianceFrameworkReviewReportRequest,
    ) -> Result<ComplianceFrameworkReviewReportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-review-reports"])?;
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

    pub fn list_compliance_framework_review_reports(
        &self,
        query: &ComplianceFrameworkReviewReportQuery,
    ) -> Result<ComplianceFrameworkReviewReportListResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-review-reports"])?;
        let query_params = framework_review_report_query_params(query, true);
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

    pub fn get_compliance_framework_review_report(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportQuery,
    ) -> Result<ComplianceFrameworkReviewReportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-review-reports", report_id])?;
        let query_params = framework_review_report_query_params(query, false);

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

    pub fn review_compliance_framework_review_report(
        &self,
        report_id: &str,
        payload: &ComplianceFrameworkReviewReportReviewRequest,
    ) -> Result<ComplianceFrameworkReviewReportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "review",
        ])?;
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

    pub fn download_compliance_framework_review_report(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "download",
        ])?;
        let query_params = framework_review_report_query_params(query, false);

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
