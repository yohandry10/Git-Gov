-- ===================================================================
-- GitGov schema migration v22
-- Date: 2026-04-25
-- Purpose:
--   Restore GitHub event statistics in get_audit_stats after the v18/v19
--   runtime optimization accidentally returned zeroed GitHub evidence.
-- ===================================================================

CREATE OR REPLACE FUNCTION get_audit_stats(p_org_id UUID DEFAULT NULL)
RETURNS JSON
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    result JSON;
    gh_total bigint;
    gh_today bigint;
    gh_pushes_today bigint;
    gh_by_type json;
    ce_total bigint;
    ce_today bigint;
    ce_blocked bigint;
    ce_by_type json;
    ce_by_status json;
    v_total bigint;
    v_unresolved bigint;
    v_critical bigint;
    active_devs bigint;
    active_repos bigint;
BEGIN
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('day', NOW())),
        COUNT(*) FILTER (WHERE event_type = 'push' AND created_at >= DATE_TRUNC('day', NOW()))
    INTO gh_total, gh_today, gh_pushes_today
    FROM github_events
    WHERE (p_org_id IS NULL OR org_id = p_org_id);

    SELECT COALESCE(json_object_agg(event_type, cnt), '{}'::json)
    INTO gh_by_type
    FROM (
        SELECT event_type, COUNT(*) AS cnt
        FROM github_events
        WHERE (p_org_id IS NULL OR org_id = p_org_id)
        GROUP BY event_type
    ) t;

    SELECT COUNT(DISTINCT repo_id)
    INTO active_repos
    FROM github_events
    WHERE (p_org_id IS NULL OR org_id = p_org_id)
      AND created_at >= NOW() - INTERVAL '7 days';

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
            'total', gh_total,
            'today', gh_today,
            'pushes_today', gh_pushes_today,
            'by_type', gh_by_type
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
        'active_repos', active_repos
    ) INTO result;

    RETURN result;
END;
$$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT EXECUTE ON FUNCTION get_audit_stats(UUID) TO gitgov_server;
    END IF;
END;
$$;
