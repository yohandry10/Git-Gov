CREATE TABLE IF NOT EXISTS compliance_framework_review_report_pdf_exports (
    pdf_export_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
    manifest_id TEXT NOT NULL REFERENCES compliance_framework_review_report_manifests(manifest_id) ON DELETE RESTRICT,
    created_by_user_id TEXT NOT NULL,
    source_report_hash TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
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
    CHECK (pdf_export_id LIKE 'frrpdf_%'),
    CHECK (source_report_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (manifest_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (pdf_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (content_type = 'application/pdf')
);

CREATE INDEX IF NOT EXISTS idx_cfr_report_pdf_exports_report_created
    ON compliance_framework_review_report_pdf_exports(org_id, report_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_cfr_report_pdf_exports_manifest
    ON compliance_framework_review_report_pdf_exports(org_id, manifest_id);

COMMENT ON TABLE compliance_framework_review_report_pdf_exports IS
    'KAN-111 append-only PDF exports for reviewed Framework Review Reports bound to provenance manifests.';
