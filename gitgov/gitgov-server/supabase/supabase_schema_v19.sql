-- ===================================================================
-- GitGov schema migration v19
-- Date: 2026-03-17
-- Purpose:
--   1) Reinstate strict append-only guarantees for violations.
--   2) Ensure violation resolution is tracked in append-only decisions.
--   3) Add missing policy_drift_events runtime table as append-only.
--   4) Keep get_audit_stats unresolved/critical compatible via decisions.
-- ===================================================================

-- -------------------------------------------------------------------
-- Violation decisions (append-only audit trail)
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS violation_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    violation_id UUID NOT NULL REFERENCES violations(id) ON DELETE CASCADE,
    decision_type TEXT NOT NULL CHECK (decision_type IN (
        'acknowledged',
        'false_positive',
        'resolved',
        'escalated',
        'dismissed',
        'wont_fix'
    )),
    decided_by TEXT NOT NULL,
    decided_at TIMESTAMPTZ DEFAULT NOW(),
    notes TEXT,
    evidence JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT violation_decisions_once_per_type UNIQUE (violation_id, decision_type)
);

CREATE INDEX IF NOT EXISTS idx_violation_decisions_violation_id
    ON violation_decisions(violation_id);
CREATE INDEX IF NOT EXISTS idx_violation_decisions_decided_by
    ON violation_decisions(decided_by);
CREATE INDEX IF NOT EXISTS idx_violation_decisions_decided_at
    ON violation_decisions(decided_at DESC);

-- Migrate legacy resolved flags into decisions (idempotent)
INSERT INTO violation_decisions (violation_id, decision_type, decided_by, decided_at, notes)
SELECT
    v.id,
    'resolved',
    COALESCE(v.resolved_by, 'system'),
    COALESCE(v.resolved_at, v.created_at, NOW()),
    'Migrated from legacy resolved fields'
FROM violations v
WHERE COALESCE(v.resolved, FALSE) = TRUE
ON CONFLICT (violation_id, decision_type) DO NOTHING;

-- Strict append-only for violations (no UPDATE/DELETE)
DROP TRIGGER IF EXISTS violations_limited_update ON violations;
DROP TRIGGER IF EXISTS violations_no_delete ON violations;
DROP FUNCTION IF EXISTS violations_limited_update();
DROP TRIGGER IF EXISTS violations_append_only ON violations;
CREATE TRIGGER violations_append_only
    BEFORE UPDATE OR DELETE ON violations
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- Strict append-only for violation_decisions
DROP TRIGGER IF EXISTS violation_decisions_append_only ON violation_decisions;
CREATE TRIGGER violation_decisions_append_only
    BEFORE UPDATE OR DELETE ON violation_decisions
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- Helper used by runtime; insert-only/idempotent by (violation_id, decision_type)
CREATE OR REPLACE FUNCTION add_violation_decision(
    p_violation_id UUID,
    p_decision_type TEXT,
    p_decided_by TEXT,
    p_notes TEXT DEFAULT NULL,
    p_evidence JSONB DEFAULT '{}'::jsonb
) RETURNS UUID AS $$
DECLARE
    decision_id UUID;
BEGIN
    IF p_decision_type NOT IN ('acknowledged', 'false_positive', 'resolved', 'escalated', 'dismissed', 'wont_fix') THEN
        RAISE EXCEPTION 'Invalid decision_type: %. Must be one of: acknowledged, false_positive, resolved, escalated, dismissed, wont_fix', p_decision_type;
    END IF;

    INSERT INTO violation_decisions (
        violation_id, decision_type, decided_by, notes, evidence
    ) VALUES (
        p_violation_id, p_decision_type, p_decided_by, p_notes, COALESCE(p_evidence, '{}'::jsonb)
    )
    ON CONFLICT (violation_id, decision_type) DO NOTHING
    RETURNING id INTO decision_id;

    IF decision_id IS NULL THEN
        SELECT id INTO decision_id
        FROM violation_decisions
        WHERE violation_id = p_violation_id
          AND decision_type = p_decision_type
        LIMIT 1;
    END IF;

    RETURN decision_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Avoid 42P16 when an older view definition has a different column layout.
DROP VIEW IF EXISTS violation_current_status;
CREATE OR REPLACE VIEW violation_current_status
WITH (security_invoker = true) AS
SELECT
    v.id AS violation_id,
    v.org_id,
    v.repo_id,
    v.severity,
    v.violation_type,
    v.user_login,
    v.branch,
    v.commit_sha,
    v.created_at AS violation_created_at,
    vd.decision_type AS current_status,
    vd.decided_by,
    vd.decided_at,
    vd.notes AS decision_notes,
    CASE
        WHEN vd.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix') THEN true
        ELSE false
    END AS is_closed
FROM violations v
LEFT JOIN LATERAL (
    SELECT decision_type, decided_by, decided_at, notes
    FROM violation_decisions
    WHERE violation_id = v.id
    ORDER BY decided_at DESC, created_at DESC
    LIMIT 1
) vd ON true;

-- -------------------------------------------------------------------
-- policy_drift_events (missing runtime table)
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS policy_drift_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    user_login TEXT NOT NULL,
    action TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    result TEXT NOT NULL,
    before_checksum TEXT,
    after_checksum TEXT,
    duration_ms BIGINT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_drift_events_org_created
    ON policy_drift_events(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_repo_created
    ON policy_drift_events(repo_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_user_created
    ON policy_drift_events(user_login, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_action
    ON policy_drift_events(action);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_result
    ON policy_drift_events(result);

DROP TRIGGER IF EXISTS policy_drift_events_append_only ON policy_drift_events;
CREATE TRIGGER policy_drift_events_append_only
    BEFORE UPDATE OR DELETE ON policy_drift_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- -------------------------------------------------------------------
-- get_audit_stats compatibility: unresolved/critical via latest decision
-- -------------------------------------------------------------------
CREATE OR REPLACE FUNCTION get_audit_stats(p_org_id UUID DEFAULT NULL)
RETURNS JSON
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    result JSON;
    ce_total bigint;
    ce_today bigint;
    ce_blocked bigint;
    ce_by_type json;
    ce_by_status json;
    v_total bigint;
    v_unresolved bigint;
    v_critical bigint;
    active_devs bigint;
BEGIN
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('day', NOW())),
        COUNT(*) FILTER (WHERE status = 'blocked' AND created_at >= DATE_TRUNC('day', NOW()))
    INTO ce_total, ce_today, ce_blocked
    FROM client_events
    WHERE (p_org_id IS NULL OR org_id = p_org_id);

    SELECT COALESCE(json_object_agg(event_type, cnt), '{}'::json)
    INTO ce_by_type
    FROM (
        SELECT event_type, COUNT(*) AS cnt
        FROM client_events
        WHERE (p_org_id IS NULL OR org_id = p_org_id)
        GROUP BY event_type
    ) t;

    SELECT COALESCE(json_object_agg(status, cnt), '{}'::json)
    INTO ce_by_status
    FROM (
        SELECT status, COUNT(*) AS cnt
        FROM client_events
        WHERE (p_org_id IS NULL OR org_id = p_org_id)
        GROUP BY status
    ) t;

    SELECT COUNT(DISTINCT user_login)
    INTO active_devs
    FROM client_events
    WHERE (p_org_id IS NULL OR org_id = p_org_id)
      AND created_at >= NOW() - INTERVAL '7 days';

    WITH latest_decisions AS (
        SELECT DISTINCT ON (vd.violation_id)
            vd.violation_id,
            vd.decision_type
        FROM violation_decisions vd
        ORDER BY vd.violation_id, vd.decided_at DESC, vd.created_at DESC
    )
    SELECT
        COUNT(*),
        COUNT(*) FILTER (
            WHERE NOT (
                COALESCE(v.resolved, FALSE)
                OR COALESCE(ld.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix'), FALSE)
            )
        ),
        COUNT(*) FILTER (
            WHERE v.severity = 'critical'
              AND NOT (
                  COALESCE(v.resolved, FALSE)
                  OR COALESCE(ld.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix'), FALSE)
              )
        )
    INTO v_total, v_unresolved, v_critical
    FROM violations v
    LEFT JOIN latest_decisions ld
        ON ld.violation_id = v.id
    WHERE (p_org_id IS NULL OR v.org_id = p_org_id);

    SELECT json_build_object(
        'github_events', json_build_object(
            'total', 0,
            'today', 0,
            'pushes_today', 0,
            'by_type', '{}'::json
        ),
        'client_events', json_build_object(
            'total', ce_total,
            'today', ce_today,
            'blocked_today', ce_blocked,
            'by_type', ce_by_type,
            'by_status', ce_by_status
        ),
        'violations', json_build_object(
            'total', v_total,
            'unresolved', v_unresolved,
            'critical', v_critical
        ),
        'active_devs_week', active_devs,
        'active_repos', 0
    ) INTO result;

    RETURN result;
END;
$$;

-- Optional grants if role exists in target environment
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT ON violation_decisions TO gitgov_server;
        GRANT SELECT, INSERT ON policy_drift_events TO gitgov_server;
        GRANT SELECT ON violation_current_status TO gitgov_server;
        GRANT EXECUTE ON FUNCTION add_violation_decision(UUID, TEXT, TEXT, TEXT, JSONB) TO gitgov_server;
    END IF;
END;
$$;
