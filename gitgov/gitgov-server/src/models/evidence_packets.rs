use super::*;

// EVIDENCE PACKETS (KAN-23)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub repo_full_name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketCompleteness {
    pub ticket_found: bool,
    pub commits: i64,
    pub pull_requests: i64,
    pub pipelines: i64,
    pub quality_gates: i64,
    #[serde(default)]
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketReconstructionFilters {
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub target_sha: Option<String>,
    pub ticket_id: String,
    pub hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketReconstructionSources {
    pub commit_correlations: i64,
    pub client_events: i64,
    pub pull_request_merge_commits: i64,
    pub pull_request_merges: i64,
    pub pipeline_events: i64,
    pub quality_gate_pipeline_events: i64,
    pub legacy_pipeline_scope_fallbacks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketReconstruction {
    pub filters: EvidencePacketReconstructionFilters,
    pub sources: EvidencePacketReconstructionSources,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePacket {
    pub packet_type: String,
    pub subject: String,
    pub generated_at: i64,
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub period: String,
    pub ticket: Option<ProjectTicket>,
    #[serde(default)]
    pub commits: Vec<TicketFlowCorrelation>,
    #[serde(default)]
    pub pull_requests: Vec<PrMergeEvidenceEntry>,
    #[serde(default)]
    pub pipelines: Vec<CommitPipelineRun>,
    #[serde(default)]
    pub quality_gates: Vec<CommitPipelineRun>,
    #[serde(default)]
    pub reconstruction: EvidencePacketReconstruction,
    pub completeness: EvidencePacketCompleteness,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<EvidencePacket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEvidencePacketBinding {
    pub id: String,
    pub org_id: String,
    pub ticket_id: String,
    pub release_id: String,
    pub repository_full_name: String,
    pub branch: String,
    pub target_sha: String,
    pub environment: String,
    pub evidence_packet_hash: String,
    pub evidence_packet_uri: String,
    pub packet: serde_json::Value,
    pub generated_by: String,
    pub generated_at: i64,
    pub created_at: i64,
}
