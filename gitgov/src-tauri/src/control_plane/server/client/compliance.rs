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
        if let Some(assigned_to_me) = query.assigned_to_me {
            query_params.push(("assigned_to_me".to_string(), assigned_to_me.to_string()));
        }
    }
    query_params
}

fn framework_review_report_assignment_query_params(
    query: &ComplianceFrameworkReviewReportAssignmentQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    query_params
}

fn framework_review_report_comments_query_params(
    query: &ComplianceFrameworkReviewReportCommentsQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    query_params
}

fn framework_review_report_pdf_query_params(
    query: &ComplianceFrameworkReviewReportPdfExportQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    if let Some(pdf_export_id) = &query.pdf_export_id {
        query_params.push(("pdf_export_id".to_string(), pdf_export_id.clone()));
    }
    query_params
}

fn compliance_period_report_query_params(
    query: &CompliancePeriodReportQuery,
    include_filters: bool,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    if include_filters {
        if let Some(framework_id) = &query.framework_id {
            query_params.push(("framework_id".to_string(), framework_id.clone()));
        }
        if let Some(limit) = query.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
    }
    query_params
}

fn compliance_period_report_pdf_query_params(
    query: &CompliancePeriodReportPdfExportQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    if let Some(pdf_export_id) = &query.pdf_export_id {
        query_params.push(("pdf_export_id".to_string(), pdf_export_id.clone()));
    }
    query_params
}

fn compliance_period_report_access_log_query_params(
    query: &CompliancePeriodReportAccessLogQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
    }
    if let Some(limit) = query.limit {
        query_params.push(("limit".to_string(), limit.to_string()));
    }
    query_params
}

fn compliance_period_report_manifest_query_params(
    query: &CompliancePeriodReportProvenanceManifestQuery,
) -> Vec<(String, String)> {
    let mut query_params = Vec::new();
    if let Some(org_name) = &query.org_name {
        query_params.push(("org_name".to_string(), org_name.clone()));
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

    pub fn diff_compliance_framework_packs(
        &self,
        query: &ComplianceFrameworkPackDiffQuery,
    ) -> Result<ComplianceFrameworkPackDiffResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "framework-packs", "diff"])?;
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(org_name) = &query.org_name {
            query_params.push(("org_name".to_string(), org_name.clone()));
        }
        query_params.push(("base_pack_id".to_string(), query.base_pack_id.clone()));
        query_params.push(("target_pack_id".to_string(), query.target_pack_id.clone()));

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

    pub fn list_assigned_compliance_framework_review_reports(
        &self,
        query: &ComplianceFrameworkReviewReportQuery,
    ) -> Result<ComplianceFrameworkReviewReportListResponse, ServerError> {
        let url =
            self.endpoint_url(&["compliance", "framework-review-reports", "assigned-to-me"])?;
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

    pub fn list_compliance_framework_review_report_assignments(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportAssignmentQuery,
    ) -> Result<ComplianceFrameworkReviewReportAssignmentsResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "assignments",
        ])?;
        let query_params = framework_review_report_assignment_query_params(query);
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

    pub fn upsert_compliance_framework_review_report_assignments(
        &self,
        report_id: &str,
        payload: &ComplianceFrameworkReviewReportAssignmentsRequest,
    ) -> Result<ComplianceFrameworkReviewReportAssignmentsResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "assignments",
        ])?;
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

    pub fn list_compliance_framework_review_report_comments(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportCommentsQuery,
    ) -> Result<ComplianceFrameworkReviewReportCommentsResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "comments",
        ])?;
        let query_params = framework_review_report_comments_query_params(query);
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

    pub fn create_compliance_framework_review_report_comment(
        &self,
        report_id: &str,
        payload: &ComplianceFrameworkReviewReportCommentRequest,
    ) -> Result<ComplianceFrameworkReviewReportCommentRecord, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "comments",
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

    pub fn create_compliance_framework_review_report_provenance_manifest(
        &self,
        report_id: &str,
        payload: &ComplianceFrameworkReviewReportProvenanceManifestRequest,
    ) -> Result<ComplianceFrameworkReviewReportProvenanceManifestResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "provenance-manifests",
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

    pub fn download_compliance_framework_review_report_provenance_manifest(
        &self,
        report_id: &str,
        manifest_id: &str,
        query: &ComplianceFrameworkReviewReportQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "provenance-manifests",
            manifest_id,
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

    pub fn create_compliance_framework_review_report_pdf_export(
        &self,
        report_id: &str,
        payload: &ComplianceFrameworkReviewReportPdfExportRequest,
    ) -> Result<ComplianceFrameworkReviewReportPdfExportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "pdf-export",
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

    pub fn get_compliance_framework_review_report_pdf_export(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportPdfExportQuery,
    ) -> Result<ComplianceFrameworkReviewReportPdfExportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "pdf-export",
        ])?;
        let query_params = framework_review_report_pdf_query_params(query);

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

    pub fn download_compliance_framework_review_report_pdf_export(
        &self,
        report_id: &str,
        query: &ComplianceFrameworkReviewReportPdfExportQuery,
    ) -> Result<ComplianceFrameworkReviewReportPdfDownloadResponse, ServerError> {
        let metadata = self.get_compliance_framework_review_report_pdf_export(report_id, query)?;
        let url = self.endpoint_url(&[
            "compliance",
            "framework-review-reports",
            report_id,
            "pdf-export",
            "download",
        ])?;
        let query_params = framework_review_report_pdf_query_params(query);

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

        let bytes = response
            .bytes()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        Ok(ComplianceFrameworkReviewReportPdfDownloadResponse {
            pdf_export: metadata.pdf_export,
            pdf_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
        })
    }

    pub fn create_compliance_period_report(
        &self,
        payload: &CompliancePeriodReportRequest,
    ) -> Result<CompliancePeriodReportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "period-reports"])?;
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

    pub fn list_compliance_period_reports(
        &self,
        query: &CompliancePeriodReportQuery,
    ) -> Result<CompliancePeriodReportListResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "period-reports"])?;
        let query_params = compliance_period_report_query_params(query, true);
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

    pub fn get_compliance_period_report(
        &self,
        period_report_id: &str,
        query: &CompliancePeriodReportQuery,
    ) -> Result<CompliancePeriodReportResponse, ServerError> {
        let url = self.endpoint_url(&["compliance", "period-reports", period_report_id])?;
        let query_params = compliance_period_report_query_params(query, false);
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

    pub fn download_compliance_period_report(
        &self,
        period_report_id: &str,
        query: &CompliancePeriodReportQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url =
            self.endpoint_url(&["compliance", "period-reports", period_report_id, "download"])?;
        let query_params = compliance_period_report_query_params(query, false);
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

    pub fn update_compliance_period_report_retention(
        &self,
        period_report_id: &str,
        payload: &CompliancePeriodReportRetentionRequest,
    ) -> Result<CompliancePeriodReportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "retention",
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

    pub fn list_compliance_period_report_access_log(
        &self,
        period_report_id: &str,
        query: &CompliancePeriodReportAccessLogQuery,
    ) -> Result<CompliancePeriodReportAccessLogResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "access-log",
        ])?;
        let query_params = compliance_period_report_access_log_query_params(query);
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

    pub fn create_compliance_period_report_pdf_export(
        &self,
        period_report_id: &str,
        payload: &CompliancePeriodReportPdfExportRequest,
    ) -> Result<CompliancePeriodReportPdfExportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "pdf-export",
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

    pub fn get_compliance_period_report_pdf_export(
        &self,
        period_report_id: &str,
        query: &CompliancePeriodReportPdfExportQuery,
    ) -> Result<CompliancePeriodReportPdfExportResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "pdf-export",
        ])?;
        let query_params = compliance_period_report_pdf_query_params(query);
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

    pub fn download_compliance_period_report_pdf_export(
        &self,
        period_report_id: &str,
        query: &CompliancePeriodReportPdfExportQuery,
    ) -> Result<CompliancePeriodReportPdfDownloadResponse, ServerError> {
        let metadata = self.get_compliance_period_report_pdf_export(period_report_id, query)?;
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "pdf-export",
            "download",
        ])?;
        let download_query = CompliancePeriodReportPdfExportQuery {
            org_name: query.org_name.clone(),
            pdf_export_id: Some(metadata.pdf_export.pdf_export_id.clone()),
        };
        let query_params = compliance_period_report_pdf_query_params(&download_query);
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

        let bytes = response
            .bytes()
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        Ok(CompliancePeriodReportPdfDownloadResponse {
            pdf_export: metadata.pdf_export,
            pdf_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
        })
    }

    pub fn create_compliance_period_report_provenance_manifest(
        &self,
        period_report_id: &str,
        payload: &CompliancePeriodReportProvenanceManifestRequest,
    ) -> Result<CompliancePeriodReportProvenanceManifestResponse, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "provenance-manifests",
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

    pub fn download_compliance_period_report_provenance_manifest(
        &self,
        period_report_id: &str,
        manifest_id: &str,
        query: &CompliancePeriodReportProvenanceManifestQuery,
    ) -> Result<serde_json::Value, ServerError> {
        let url = self.endpoint_url(&[
            "compliance",
            "period-reports",
            period_report_id,
            "provenance-manifests",
            manifest_id,
        ])?;
        let query_params = compliance_period_report_manifest_query_params(query);
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
