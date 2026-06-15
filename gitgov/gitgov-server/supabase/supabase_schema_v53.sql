-- KAN-110: Reviewed Framework Review Report provenance manifests.

CREATE TABLE IF NOT EXISTS compliance_framework_review_report_manifests (
    manifest_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
    generated_by_user_id TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    previous_manifest_hash TEXT,
    signature_algorithm TEXT NOT NULL DEFAULT 'sha256-provenance-manifest-v1',
    payload_json_redacted JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT compliance_framework_review_report_manifests_id_check
        CHECK (manifest_id ~ '^frrm_[0-9a-f]{32}$'),
    CONSTRAINT compliance_framework_review_report_manifests_hash_check
        CHECK (manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT compliance_framework_review_report_manifests_prev_hash_check
        CHECK (previous_manifest_hash IS NULL OR previous_manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT compliance_framework_review_report_manifests_signature_check
        CHECK (signature_algorithm = 'sha256-provenance-manifest-v1'),
    CONSTRAINT compliance_framework_review_report_manifests_payload_check
        CHECK (
            payload_json_redacted ? 'schema_version'
            AND payload_json_redacted ? 'hash_chain'
            AND payload_json_redacted ? 'claims'
            AND payload_json_redacted ? 'audit_metadata'
            AND COALESCE((payload_json_redacted #>> '{claims,compliance_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,regulatory_claim}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{claims,certification}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,agent_governance_required}')::boolean, true) = false
            AND COALESCE((payload_json_redacted #>> '{audit_metadata,source_report_artifact_mutated}')::boolean, true) = false
        )
);

CREATE INDEX IF NOT EXISTS idx_cfr_report_manifests_report_created
    ON compliance_framework_review_report_manifests(org_id, report_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_cfr_report_manifests_hash
    ON compliance_framework_review_report_manifests(org_id, manifest_hash);

COMMENT ON TABLE compliance_framework_review_report_manifests IS
    'KAN-110 append-only provenance manifests for reviewed Framework Review Reports. Manifests hash existing report/review/collaboration evidence and do not mutate artifacts or create compliance claims.';
