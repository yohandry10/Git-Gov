CREATE TABLE IF NOT EXISTS compliance_period_reports (
    period_report_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    framework_id TEXT,
    date_range_start TIMESTAMPTZ NOT NULL,
    date_range_end TIMESTAMPTZ NOT NULL,
    report_count INTEGER NOT NULL CHECK (report_count > 0),
    source_report_ids JSONB NOT NULL,
    format TEXT NOT NULL DEFAULT 'json',
    status TEXT NOT NULL DEFAULT 'generated',
    artifact_hash TEXT NOT NULL,
    payload_json_redacted JSONB NOT NULL,
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
    regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
    requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
    certification BOOLEAN NOT NULL DEFAULT FALSE CHECK (certification = FALSE),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    downloaded_at TIMESTAMPTZ,
    error_message_safe TEXT,
    CHECK (period_report_id LIKE 'cpr_%'),
    CHECK (date_range_end > date_range_start),
    CHECK (jsonb_typeof(source_report_ids) = 'array'),
    CHECK (jsonb_array_length(source_report_ids) = report_count),
    CHECK (format = 'json'),
    CHECK (status IN ('generated', 'failed')),
    CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$')
);

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_org_created
    ON compliance_period_reports(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_framework_created
    ON compliance_period_reports(org_id, framework_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_date_range
    ON compliance_period_reports(org_id, date_range_start, date_range_end);

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_artifact_hash
    ON compliance_period_reports(org_id, artifact_hash);

COMMENT ON TABLE compliance_period_reports IS
    'KAN-113 manual period compliance report generator artifacts. Non-certifying, non-regulatory, auditor-review-required JSON summaries over reviewed Framework Review Reports.';
