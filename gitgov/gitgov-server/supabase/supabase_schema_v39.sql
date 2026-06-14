-- KAN-92: Agent Governance Control Boundary.
-- Agent Governance is manual-first and disabled by default per tenant.
-- Admins must opt in explicitly before /agent-governance/evaluate accepts requests.

CREATE TABLE IF NOT EXISTS agent_governance_settings (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mode TEXT NOT NULL DEFAULT 'manual_only'
        CHECK (mode IN ('manual_only', 'opt_in_enabled')),
    payload_mode TEXT NOT NULL DEFAULT 'minimized'
        CHECK (payload_mode IN ('minimized')),
    reason TEXT,
    updated_by TEXT NOT NULL DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (enabled = FALSE AND mode = 'manual_only')
        OR
        (enabled = TRUE AND mode = 'opt_in_enabled')
    )
);

CREATE INDEX IF NOT EXISTS idx_agent_governance_settings_enabled
    ON agent_governance_settings(enabled, updated_at DESC);
