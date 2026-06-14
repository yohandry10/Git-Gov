-- KAN-88: Break-glass approval routing for deployment gates.
-- Adds pre-approved, evidence-bound break-glass approvals and links deployment
-- authorization records to the approval that authorized the exception.

CREATE TABLE IF NOT EXISTS deployment_gate_break_glass_approvals (
    approval_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    target_sha TEXT NOT NULL,
    environment TEXT NOT NULL,
    ticket_id TEXT,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT,
    reason TEXT NOT NULL,
    approver TEXT NOT NULL,
    approver_role TEXT NOT NULL DEFAULT 'incident_commander'
        CHECK (approver_role IN ('incident_commander', 'security', 'release_manager', 'platform_admin')),
    expires_at TIMESTAMPTZ NOT NULL,
    approval_hash TEXT NOT NULL UNIQUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (approval_id LIKE 'dgbga_%'),
    CHECK (length(trim(reason)) >= 16),
    CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_break_glass_approvals_org_created
    ON deployment_gate_break_glass_approvals(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_break_glass_approvals_scope
    ON deployment_gate_break_glass_approvals(
        org_id,
        repository_full_name,
        branch,
        environment,
        target_sha,
        evidence_packet_hash,
        expires_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_deployment_gate_break_glass_approvals_expiry
    ON deployment_gate_break_glass_approvals(org_id, expires_at DESC);

ALTER TABLE deployment_gate_authorizations
    ADD COLUMN IF NOT EXISTS break_glass_approval_id TEXT,
    ADD COLUMN IF NOT EXISTS break_glass_approval_hash TEXT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'deployment_gate_break_glass_approval_link_check'
          AND conrelid = 'deployment_gate_authorizations'::regclass
    ) THEN
        ALTER TABLE deployment_gate_authorizations
            ADD CONSTRAINT deployment_gate_break_glass_approval_link_check
            CHECK (
                (break_glass_used = FALSE AND break_glass_approval_id IS NULL AND break_glass_approval_hash IS NULL)
                OR
                (break_glass_used = TRUE AND break_glass_approval_id IS NOT NULL AND break_glass_approval_hash IS NOT NULL)
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_break_glass_approval
    ON deployment_gate_authorizations(org_id, break_glass_approval_id, created_at DESC)
    WHERE break_glass_approval_id IS NOT NULL;
