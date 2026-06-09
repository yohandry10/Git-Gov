use super::*;

// SERVER HEALTH
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseHealth,
    pub uptime_seconds: i64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub latency_ms: Option<i64>,
    pub pending_events: Option<i64>,
}

// ============================================================================
