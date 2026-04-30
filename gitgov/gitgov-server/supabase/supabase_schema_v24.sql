-- v24: Formal enterprise release approval evidence.
-- Stores append-only approval decisions tied to release, risk and evidence packet hash.

CREATE TABLE IF NOT EXISTS enterprise_release_approvals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT,
    target_sha TEXT,
    environment TEXT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'accepted-risk')),
    approver TEXT NOT NULL,
    ticket_id TEXT,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT,
    evidence_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    risk_severity TEXT NOT NULL DEFAULT 'none' CHECK (risk_severity IN ('none', 'low', 'medium', 'high', 'critical')),
    risk_acceptance_reason TEXT,
    expires_at TIMESTAMPTZ,
    approval_hash TEXT NOT NULL UNIQUE,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_org_created
    ON enterprise_release_approvals(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_repo_release
    ON enterprise_release_approvals(org_id, repository_full_name, release_id);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_expiry
    ON enterprise_release_approvals(expires_at)
    WHERE expires_at IS NOT NULL;
