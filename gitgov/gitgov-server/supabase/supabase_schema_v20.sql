-- ===================================================================
-- GitGov schema migration v20
-- Date: 2026-04-16
-- Purpose:
--   1) Persist policy change requests in versioned SQL migrations.
--   2) Persist policy change decisions in append-only audit form.
--   3) Add indexes and role grants required by runtime handlers.
-- ===================================================================

-- -------------------------------------------------------------------
-- Policy change requests
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS policy_change_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    repo_name TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    requested_config JSONB NOT NULL,
    requested_checksum TEXT NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_change_requests_org_created
    ON policy_change_requests(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_change_requests_repo_name_created
    ON policy_change_requests(repo_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_change_requests_requested_by_created
    ON policy_change_requests(requested_by, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_change_requests_repo_id_created
    ON policy_change_requests(repo_id, created_at DESC);

-- -------------------------------------------------------------------
-- Policy change request decisions (single decision per request)
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS policy_change_request_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    request_id UUID UNIQUE NOT NULL REFERENCES policy_change_requests(id) ON DELETE CASCADE,
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected')),
    decided_by TEXT NOT NULL,
    note TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_change_request_decisions_org_created
    ON policy_change_request_decisions(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_change_request_decisions_decision_created
    ON policy_change_request_decisions(decision, created_at DESC);

-- -------------------------------------------------------------------
-- Append-only protections
-- -------------------------------------------------------------------
DROP TRIGGER IF EXISTS policy_change_requests_append_only ON policy_change_requests;
CREATE TRIGGER policy_change_requests_append_only
    BEFORE UPDATE OR DELETE ON policy_change_requests
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

DROP TRIGGER IF EXISTS policy_change_request_decisions_append_only ON policy_change_request_decisions;
CREATE TRIGGER policy_change_request_decisions_append_only
    BEFORE UPDATE OR DELETE ON policy_change_request_decisions
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- -------------------------------------------------------------------
-- Optional grants if role exists in target environment
-- -------------------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT ON policy_change_requests TO gitgov_server;
        GRANT SELECT, INSERT ON policy_change_request_decisions TO gitgov_server;
    END IF;
END;
$$;
