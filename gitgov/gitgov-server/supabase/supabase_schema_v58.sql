-- KAN-116: Period Compliance Report provenance manifests.

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
            'manifest_downloaded'
        )
    );

ALTER TABLE compliance_period_report_access_log
    DROP CONSTRAINT IF EXISTS compliance_period_report_access_log_artifact_type_check;

ALTER TABLE compliance_period_report_access_log
    ADD CONSTRAINT compliance_period_report_access_log_artifact_type_check
    CHECK (artifact_type IN ('metadata', 'json', 'pdf', 'retention', 'manifest'));

CREATE TABLE IF NOT EXISTS compliance_period_report_manifests (
    manifest_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
    generated_by_user_id TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    previous_manifest_hash TEXT,
    signature_algorithm TEXT NOT NULL DEFAULT 'sha256-period-report-provenance-manifest-v1',
    payload_json_redacted JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT compliance_period_report_manifests_id_check
        CHECK (manifest_id ~ '^cprm_[0-9a-f]{32}$'),
    CONSTRAINT compliance_period_report_manifests_hash_check
        CHECK (manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT compliance_period_report_manifests_prev_hash_check
        CHECK (previous_manifest_hash IS NULL OR previous_manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT compliance_period_report_manifests_signature_check
        CHECK (signature_algorithm = 'sha256-period-report-provenance-manifest-v1'),
    CONSTRAINT compliance_period_report_manifests_payload_check
        CHECK (
            payload_json_redacted ? 'schema_version'
            AND payload_json_redacted ? 'hash_chain'
            AND payload_json_redacted ? 'claims'
            AND payload_json_redacted ? 'audit_metadata'
            AND COALESCE((payload_json_redacted #>> '{claims,compliance_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,regulatory_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,certification}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,agent_governance_required}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,source_period_report_artifact_mutated}')::boolean, true) = false
        )
);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_manifests_report_created
    ON compliance_period_report_manifests(org_id, period_report_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_compliance_period_report_manifests_hash
    ON compliance_period_report_manifests(org_id, manifest_hash);

COMMENT ON TABLE compliance_period_report_manifests IS
    'KAN-116 append-only provenance manifests for Period Compliance Reports. Manifests bind report hash, PDF exports, retention/access custody, and source hashes without creating compliance, regulatory, certification, or Agent Governance claims.';
