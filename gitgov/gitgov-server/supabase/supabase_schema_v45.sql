-- GitGov Control Plane Schema v45 - Control Mapping Review Package
-- =====================================================================
-- KAN-101: Persist JSON-only, hashable review packages over KAN-100
-- Evidence-to-Control mappings. Packages are evidence for customer/auditor
-- review, not compliance certification or official regulatory claims.

CREATE TABLE IF NOT EXISTS compliance_review_packages (
    review_package_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    mapping_id TEXT NOT NULL REFERENCES compliance_evidence_mappings(mapping_id) ON DELETE RESTRICT,
    evidence_export_id TEXT NOT NULL REFERENCES compliance_evidence_exports(export_id) ON DELETE RESTRICT,
    evidence_export_hash TEXT NOT NULL,
    mapping_hash TEXT NOT NULL,
    framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE RESTRICT,
    framework_version TEXT NOT NULL,
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
    CHECK (review_package_id LIKE 'crp_%'),
    CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (evidence_export_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (mapping_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (compliance_claim = FALSE),
    CHECK (regulatory_claim = FALSE),
    CHECK (requires_auditor_review = TRUE),
    CHECK (certification = FALSE)
);

CREATE INDEX IF NOT EXISTS idx_compliance_review_packages_org_created
    ON compliance_review_packages(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_review_packages_mapping
    ON compliance_review_packages(org_id, mapping_id);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT, UPDATE ON compliance_review_packages TO gitgov_server;
    END IF;
END $$;

COMMENT ON TABLE compliance_review_packages IS
    'KAN-101 JSON-only, non-certifying control mapping review packages over KAN-100 mappings.';
