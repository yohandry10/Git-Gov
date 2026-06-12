-- v31: Policy-as-Code source metadata.
--
-- Stores where the active policy came from: Control Plane managed policy, repo
-- policy file, or hybrid advisory state. The source metadata is JSONB so the PR
-- activation path can add commit/blob/reviewer evidence without another schema
-- churn cycle.

ALTER TABLE policies
    ADD COLUMN IF NOT EXISTS source_metadata JSONB NOT NULL DEFAULT
        '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb;

ALTER TABLE policy_history
    ADD COLUMN IF NOT EXISTS source_metadata JSONB NOT NULL DEFAULT
        '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb;

ALTER TABLE policy_change_requests
    ADD COLUMN IF NOT EXISTS source_metadata JSONB NOT NULL DEFAULT
        '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_policies_source_mode
    ON policies ((source_metadata ->> 'source_mode'));

CREATE INDEX IF NOT EXISTS idx_policy_history_source_mode
    ON policy_history ((source_metadata ->> 'source_mode'));

DROP TRIGGER IF EXISTS policy_history_trigger ON policies;

CREATE OR REPLACE FUNCTION record_policy_change()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO policy_history (
        repo_id,
        config,
        checksum,
        source_metadata,
        changed_by,
        change_type,
        previous_checksum
    )
    VALUES (
        NEW.repo_id,
        NEW.config,
        NEW.checksum,
        NEW.source_metadata,
        COALESCE(NEW.override_actor, 'system'),
        CASE WHEN TG_OP = 'INSERT' THEN 'create' ELSE 'update' END,
        CASE WHEN TG_OP = 'UPDATE' THEN OLD.checksum ELSE NULL END
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE TRIGGER policy_history_trigger
    AFTER INSERT OR UPDATE ON policies
    FOR EACH ROW EXECUTE FUNCTION record_policy_change();

CREATE OR REPLACE FUNCTION get_policy_history(
    p_repo_id UUID,
    p_limit INTEGER DEFAULT 50
) RETURNS TABLE (
    id TEXT,
    config JSONB,
    checksum TEXT,
    source_metadata JSONB,
    changed_by TEXT,
    change_type TEXT,
    previous_checksum TEXT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        ph.id::TEXT,
        ph.config,
        ph.checksum,
        ph.source_metadata,
        ph.changed_by,
        ph.change_type,
        ph.previous_checksum,
        ph.created_at
    FROM policy_history ph
    WHERE ph.repo_id = p_repo_id
    ORDER BY ph.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
