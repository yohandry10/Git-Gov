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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePacket {
    pub packet_type: String,
    pub subject: String,
    pub generated_at: i64,
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub branch: Option<String>,
    pub period: String,
    pub ticket: Option<ProjectTicket>,
    #[serde(default)]
    pub commits: Vec<TicketFlowCorrelation>,
    #[serde(default)]
    pub pull_requests: Vec<PrMergeEvidenceEntry>,
    #[serde(default)]
    pub quality_gates: Vec<CommitPipelineRun>,
    pub completeness: EvidencePacketCompleteness,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidencePacketResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<EvidencePacket>,
}
