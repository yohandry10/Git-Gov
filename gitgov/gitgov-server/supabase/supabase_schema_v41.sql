ALTER TABLE agent_governance_evaluations
    ADD COLUMN IF NOT EXISTS attribution_id TEXT,
    ADD COLUMN IF NOT EXISTS correlation_id TEXT,
    ADD COLUMN IF NOT EXISTS parent_correlation_id TEXT,
    ADD COLUMN IF NOT EXISTS session_id TEXT,
    ADD COLUMN IF NOT EXISTS tool_name TEXT,
    ADD COLUMN IF NOT EXISTS tool_version TEXT,
    ADD COLUMN IF NOT EXISTS agent_name TEXT,
    ADD COLUMN IF NOT EXISTS external_run_id TEXT,
    ADD COLUMN IF NOT EXISTS consumer_type TEXT;

UPDATE agent_governance_evaluations
SET consumer_type = 'agent_governance'
WHERE consumer_type IS NULL;

ALTER TABLE agent_governance_evaluations
    DROP CONSTRAINT IF EXISTS agent_governance_evaluations_attribution_id_format,
    DROP CONSTRAINT IF EXISTS agent_governance_evaluations_correlation_id_format,
    DROP CONSTRAINT IF EXISTS agent_governance_evaluations_consumer_type_check;

ALTER TABLE agent_governance_evaluations
    ADD CONSTRAINT agent_governance_evaluations_attribution_id_format
        CHECK (attribution_id IS NULL OR attribution_id LIKE 'attr_%') NOT VALID,
    ADD CONSTRAINT agent_governance_evaluations_correlation_id_format
        CHECK (correlation_id IS NULL OR length(correlation_id) BETWEEN 1 AND 128) NOT VALID,
    ADD CONSTRAINT agent_governance_evaluations_consumer_type_check
        CHECK (consumer_type IS NULL OR consumer_type IN ('agent_governance', 'agent_dry_run')) NOT VALID;

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_correlation
    ON agent_governance_evaluations(org_id, correlation_id, created_at DESC)
    WHERE correlation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_session
    ON agent_governance_evaluations(org_id, session_id, created_at DESC)
    WHERE session_id IS NOT NULL;
