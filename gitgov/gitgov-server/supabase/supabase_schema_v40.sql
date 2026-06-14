CREATE TABLE IF NOT EXISTS agent_governance_agent_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_id TEXT NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL DEFAULT 'ggag_',
    token_last4 TEXT NOT NULL,
    token_preview TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    environment TEXT,
    scopes JSONB NOT NULL DEFAULT '["agent_governance:evaluate"]'::jsonb,
    allowed_actions JSONB NOT NULL DEFAULT '["commit","push","open_pr","merge_pr","deploy"]'::jsonb,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_by TEXT NOT NULL,
    revoked_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (key_id LIKE 'agk_%'),
    CHECK (token_prefix = 'ggag_'),
    CHECK (jsonb_typeof(scopes) = 'array'),
    CHECK (jsonb_typeof(allowed_actions) = 'array')
);

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_org_created
    ON agent_governance_agent_keys(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_active
    ON agent_governance_agent_keys(org_id, revoked_at, expires_at);

ALTER TABLE agent_governance_evaluations
    ADD COLUMN IF NOT EXISTS principal_type TEXT,
    ADD COLUMN IF NOT EXISTS agent_key_id TEXT,
    ADD COLUMN IF NOT EXISTS agent_display_name TEXT;
