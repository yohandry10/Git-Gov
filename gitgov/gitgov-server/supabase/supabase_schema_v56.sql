CREATE TABLE IF NOT EXISTS compliance_period_report_pdf_exports (
    pdf_export_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    source_period_report_hash TEXT NOT NULL,
    pdf_artifact_hash TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/pdf',
    page_count INTEGER NOT NULL DEFAULT 1 CHECK (page_count BETWEEN 1 AND 200),
    pdf_bytes BYTEA NOT NULL,
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
    regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
    requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
    certification BOOLEAN NOT NULL DEFAULT FALSE CHECK (certification = FALSE),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    downloaded_at TIMESTAMPTZ,
    CHECK (pdf_export_id LIKE 'cprpdf_%'),
    CHECK (source_period_report_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (pdf_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (content_type = 'application/pdf')
);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_pdf_exports_report_created
    ON compliance_period_report_pdf_exports(org_id, period_report_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_pdf_exports_hash
    ON compliance_period_report_pdf_exports(org_id, pdf_artifact_hash);

COMMENT ON TABLE compliance_period_report_pdf_exports IS
    'KAN-114 append-only PDF exports for manual Period Compliance Reports. Non-certifying, non-regulatory, auditor-review-required artifacts bound to source period report hashes.';
