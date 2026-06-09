use super::*;

// COMPLIANCE DASHBOARD
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComplianceDashboard {
    pub signals: SignalStats,
    pub correlation: CorrelationStats,
    pub policy: PolicyStats,
    pub exports: ExportStats,
    pub timeline: Vec<ComplianceTimelinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SignalStats {
    pub total: i64,
    pub pending: i64,
    pub high_confidence: i64,
    #[serde(default)]
    pub by_type: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CorrelationStats {
    pub github_pushes_24h: i64,
    pub client_pushes_24h: i64,
    pub correlation_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PolicyStats {
    pub repos_with_policy: i64,
    pub total_repos: i64,
    pub recent_changes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExportStats {
    pub total: i64,
    pub last_7_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComplianceTimelinePoint {
    pub month: String,
    pub signals_detected: i64,
    pub violations_confirmed: i64,
    pub commits_total: i64,
    pub commits_with_ticket: i64,
    pub ticket_coverage_pct: f64,
    pub pipeline_runs_total: i64,
    pub pipeline_success_pct: f64,
}

// ============================================================================
