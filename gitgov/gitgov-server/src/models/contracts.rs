use super::*;

/// Keeps public contract types referenced under strict `-D warnings` builds.
/// Some of these are legacy/compat structs that may not be instantiated by
/// the current runtime paths, but are still part of the shared API model.
pub fn touch_contract_types() {
    let _ = (
        std::mem::size_of::<Member>(),
        std::mem::size_of::<GitHubWebhookPayload>(),
        std::mem::size_of::<Violation>(),
        std::mem::size_of::<ViolationType>(),
        std::mem::size_of::<Severity>(),
        std::mem::size_of::<AuditLogEntry>(),
        std::mem::size_of::<AuditAction>(),
        std::mem::size_of::<AuditStatus>(),
        std::mem::size_of::<AuditFilter>(),
        std::mem::size_of::<Claims>(),
        std::mem::size_of::<CorrelationConfig>(),
        std::mem::size_of::<FeatureRequestRecord>(),
    );
}

// ============================================================================
