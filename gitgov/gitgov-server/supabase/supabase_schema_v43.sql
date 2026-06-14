-- GitGov Control Plane Schema v43 - Compliance Evidence Exports
-- =====================================================================
-- KAN-99: Persist read-only JSON evidence packages generated from
-- Deployment Gate authorizations for audit/review.

CREATE TABLE IF NOT EXISTS compliance_evidence_exports (
    export_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    scope TEXT NOT NULL CHECK (scope IN ('deployment_gate')),
    deployment_gate_id TEXT,
    release_id TEXT,
    status TEXT NOT NULL CHECK (status IN ('completed', 'failed')),
    format TEXT NOT NULL CHECK (format IN ('json')),
    artifact_hash TEXT NOT NULL,
    policy_checksum TEXT,
    gate_decision TEXT,
    payload_json_redacted JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error_message_safe TEXT,
    CONSTRAINT compliance_evidence_exports_deployment_gate_required
        CHECK (scope <> 'deployment_gate' OR deployment_gate_id IS NOT NULL),
    CONSTRAINT compliance_evidence_exports_hash_shape
        CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$')
);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_exports_org_created
    ON compliance_evidence_exports(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_exports_deployment_gate
    ON compliance_evidence_exports(org_id, deployment_gate_id);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_exports_release
    ON compliance_evidence_exports(org_id, release_id);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT ON compliance_evidence_exports TO gitgov_server;
    END IF;
END $$;

COMMENT ON TABLE compliance_evidence_exports IS
    'KAN-99 read-only JSON evidence packages generated from Deployment Gate authorizations.';
