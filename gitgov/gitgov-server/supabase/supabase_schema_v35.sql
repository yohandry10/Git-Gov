-- KAN-83: Deployment Gates authorization history.
-- Records every CI/CD deployment authorization decision returned by GitGov.

CREATE TABLE IF NOT EXISTS deployment_gate_authorizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    authorization_id TEXT NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    target_sha TEXT NOT NULL,
    environment TEXT NOT NULL,
    deployer TEXT NOT NULL,
    ticket_id TEXT,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT,
    decision TEXT NOT NULL,
    approved BOOLEAN NOT NULL,
    blocking BOOLEAN NOT NULL,
    would_block BOOLEAN NOT NULL,
    reason TEXT NOT NULL,
    blocked_by JSONB NOT NULL DEFAULT '[]'::jsonb,
    warnings JSONB NOT NULL DEFAULT '[]'::jsonb,
    policy_checksum TEXT NOT NULL,
    break_glass_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    evaluation JSONB NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    requested_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'deployment_gate_authorizations_decision_check'
          AND conrelid = 'deployment_gate_authorizations'::regclass
    ) THEN
        ALTER TABLE deployment_gate_authorizations
            ADD CONSTRAINT deployment_gate_authorizations_decision_check
            CHECK (decision IN ('approved', 'advisory', 'blocked'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_org_created
    ON deployment_gate_authorizations(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_scope
    ON deployment_gate_authorizations(
        org_id,
        repository_full_name,
        branch,
        environment,
        created_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_release
    ON deployment_gate_authorizations(org_id, release_id, environment, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_target
    ON deployment_gate_authorizations(org_id, target_sha, created_at DESC);
