ALTER TABLE agent_governance_agent_keys
    ADD COLUMN IF NOT EXISTS rotated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rotated_from_key_id TEXT,
    ADD COLUMN IF NOT EXISTS replaced_by_key_id TEXT,
    ADD COLUMN IF NOT EXISTS rotation_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_rotation_from
    ON agent_governance_agent_keys(org_id, rotated_from_key_id)
    WHERE rotated_from_key_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_replaced_by
    ON agent_governance_agent_keys(org_id, replaced_by_key_id)
    WHERE replaced_by_key_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_expiry
    ON agent_governance_agent_keys(org_id, expires_at)
    WHERE revoked_at IS NULL AND expires_at IS NOT NULL;
