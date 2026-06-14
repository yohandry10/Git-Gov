-- GitGov Control Plane Schema v48 - Framework-specific Review Report Export
-- KAN-105: Persist JSON-only, non-certifying framework review reports over KAN-101 packages.

CREATE TABLE IF NOT EXISTS compliance_framework_review_reports (
    report_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    mapping_id TEXT NOT NULL REFERENCES compliance_evidence_mappings(mapping_id) ON DELETE RESTRICT,
    review_package_id TEXT NOT NULL REFERENCES compliance_review_packages(review_package_id) ON DELETE RESTRICT,
    evidence_export_id TEXT NOT NULL REFERENCES compliance_evidence_exports(export_id) ON DELETE RESTRICT,
    evidence_export_hash TEXT NOT NULL,
    mapping_hash TEXT NOT NULL,
    review_package_hash TEXT NOT NULL,
    framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE RESTRICT,
    framework_version TEXT NOT NULL,
    framework_owner_type TEXT NOT NULL CHECK (framework_owner_type IN ('gitgov', 'customer')),
    framework_review_status TEXT CHECK (
        framework_review_status IS NULL
        OR framework_review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected', 'archived')
    ),
    pack_hash TEXT,
    format TEXT NOT NULL CHECK (format IN ('json')),
    artifact_hash TEXT NOT NULL,
    payload_json_redacted JSONB NOT NULL,
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE,
    regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE,
    requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE,
    certification BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    downloaded_at TIMESTAMPTZ,
    error_message_safe TEXT,
    CHECK (report_id LIKE 'frr_%'),
    CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (evidence_export_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (mapping_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (review_package_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (pack_hash IS NULL OR pack_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (compliance_claim = FALSE),
    CHECK (regulatory_claim = FALSE),
    CHECK (requires_auditor_review = TRUE),
    CHECK (certification = FALSE)
);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_org_created
    ON compliance_framework_review_reports(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_mapping
    ON compliance_framework_review_reports(org_id, mapping_id);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_package
    ON compliance_framework_review_reports(org_id, review_package_id);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT, UPDATE ON compliance_framework_review_reports TO gitgov_server;
    END IF;
END $$;

COMMENT ON TABLE compliance_framework_review_reports IS
    'KAN-105 JSON-only, non-certifying framework-specific review reports over KAN-101 review packages.';
