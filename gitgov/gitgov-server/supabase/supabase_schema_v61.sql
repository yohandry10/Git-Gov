-- KAN-119: Period Compliance Report share packages / offline verification bundles.

ALTER TABLE compliance_period_report_access_log
    DROP CONSTRAINT IF EXISTS compliance_period_report_access_log_action_check;

ALTER TABLE compliance_period_report_access_log
    ADD CONSTRAINT compliance_period_report_access_log_action_check
    CHECK (
        action IN (
            'viewed',
            'downloaded_json',
            'downloaded_pdf',
            'archived',
            'retention_updated',
            'manifest_created',
            'manifest_downloaded',
            'review_updated',
            'share_package_created',
            'share_package_downloaded',
            'share_package_revoked'
        )
    );

ALTER TABLE compliance_period_report_access_log
    DROP CONSTRAINT IF EXISTS compliance_period_report_access_log_artifact_type_check;

ALTER TABLE compliance_period_report_access_log
    ADD CONSTRAINT compliance_period_report_access_log_artifact_type_check
    CHECK (artifact_type IN ('metadata', 'json', 'pdf', 'retention', 'manifest', 'review', 'share_package'));

CREATE TABLE IF NOT EXISTS compliance_period_report_share_packages (
    share_package_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    package_format TEXT NOT NULL DEFAULT 'json_bundle',
    status TEXT NOT NULL DEFAULT 'active',
    artifact_hash TEXT NOT NULL,
    payload_json_redacted JSONB NOT NULL,
    period_report_artifact_hash TEXT NOT NULL,
    pdf_export_id TEXT NOT NULL,
    pdf_artifact_hash TEXT NOT NULL,
    manifest_id TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    no_claims_snapshot JSONB NOT NULL,
    source_hashes JSONB NOT NULL,
    review_snapshot JSONB NOT NULL,
    retention_snapshot JSONB NOT NULL,
    download_count INTEGER NOT NULL DEFAULT 0,
    downloaded_at TIMESTAMPTZ,
    last_downloaded_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by_user_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    error_message_safe TEXT,
    CONSTRAINT compliance_period_report_share_packages_id_check
        CHECK (share_package_id LIKE 'cprsp_%'),
    CONSTRAINT compliance_period_report_share_packages_format_check
        CHECK (package_format = 'json_bundle'),
    CONSTRAINT compliance_period_report_share_packages_status_check
        CHECK (status IN ('active', 'revoked')),
    CONSTRAINT compliance_period_report_share_packages_hash_check
        CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT compliance_period_report_share_packages_period_hash_check
        CHECK (period_report_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT compliance_period_report_share_packages_pdf_hash_check
        CHECK (pdf_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT compliance_period_report_share_packages_manifest_hash_check
        CHECK (manifest_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT compliance_period_report_share_packages_download_count_check
        CHECK (download_count >= 0),
    CONSTRAINT compliance_period_report_share_packages_no_claims_check
        CHECK (
            COALESCE((no_claims_snapshot ->> 'compliance_claim')::boolean, true) = false
            AND COALESCE((no_claims_snapshot ->> 'regulatory_claim')::boolean, true) = false
            AND COALESCE((no_claims_snapshot ->> 'certification')::boolean, true) = false
            AND COALESCE((no_claims_snapshot ->> 'compliance_score')::boolean, true) = false
            AND COALESCE((no_claims_snapshot ->> 'requires_auditor_review')::boolean, false) = true
            AND COALESCE((no_claims_snapshot ->> 'agent_governance_required')::boolean, true) = false
        ),
    CONSTRAINT compliance_period_report_share_packages_payload_check
        CHECK (
            payload_json_redacted ->> 'schema_version' = 'gitgov_period_compliance_report_share_package.v1'
            AND COALESCE((payload_json_redacted #>> '{claims,compliance_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,regulatory_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,certification}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,compliance_score}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,requires_auditor_review}')::boolean, false) = true
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,agent_governance_required}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,source_period_report_artifact_mutated}')::boolean, true) = false
        )
);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_share_packages_report_created
    ON compliance_period_report_share_packages(org_id, period_report_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_share_packages_status
    ON compliance_period_report_share_packages(org_id, status, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_compliance_period_report_share_packages_hash
    ON compliance_period_report_share_packages(org_id, artifact_hash);

COMMENT ON TABLE compliance_period_report_share_packages IS
    'KAN-119 append-only/revocable manual share packages for already reviewed Period Compliance Reports. Packages bind existing JSON/PDF/manifest hashes for offline auditor/customer verification without public links, email, certification, regulatory claims, or Agent Governance dependency.';
