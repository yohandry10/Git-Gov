use super::*;

// CORRELATION CONFIG
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfig {
    pub org_id: Option<String>,
    pub correlation_window_minutes: i32,
    pub bypass_tolerance_minutes: i32,
    pub clock_skew_seconds: i32,
    pub auto_create_violations: bool,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            org_id: None,
            correlation_window_minutes: 15,
            bypass_tolerance_minutes: 30,
            clock_skew_seconds: 60,
            auto_create_violations: false,
        }
    }
}

// ============================================================================
