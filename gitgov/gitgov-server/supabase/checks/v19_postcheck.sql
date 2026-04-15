-- GitGov v19 post-migration checks
-- Run in Supabase SQL Editor (project/db already migrated)

-- ============================================================================
-- 1) RELATIONS (tables + view)
-- ============================================================================
WITH expected_relations AS (
    SELECT *
    FROM (VALUES
        ('table', 'violations'),
        ('table', 'violation_decisions'),
        ('table', 'policy_drift_events'),
        ('view',  'violation_current_status')
    ) AS v(kind, rel_name)
)
SELECT
    'relations' AS check_group,
    kind || ':' || rel_name AS check_name,
    CASE
        WHEN kind = 'table'
             AND to_regclass('public.' || rel_name) IS NOT NULL
             AND EXISTS (
                 SELECT 1
                 FROM pg_class c
                 JOIN pg_namespace n ON n.oid = c.relnamespace
                 WHERE n.nspname = 'public'
                   AND c.relname = rel_name
                   AND c.relkind = 'r'
             ) THEN 'PASS'
        WHEN kind = 'view'
             AND to_regclass('public.' || rel_name) IS NOT NULL
             AND EXISTS (
                 SELECT 1
                 FROM pg_class c
                 JOIN pg_namespace n ON n.oid = c.relnamespace
                 WHERE n.nspname = 'public'
                   AND c.relname = rel_name
                   AND c.relkind = 'v'
             ) THEN 'PASS'
        ELSE 'FAIL'
    END AS status,
    COALESCE(to_regclass('public.' || rel_name)::text, 'NULL') AS observed
FROM expected_relations
ORDER BY check_name;

-- ============================================================================
-- 2) VIEW SECURITY (security_invoker must be enabled)
-- ============================================================================
SELECT
    'view_security' AS check_group,
    'violation_current_status.security_invoker' AS check_name,
    CASE
        WHEN EXISTS (
            SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relname = 'violation_current_status'
              AND c.relkind = 'v'
              AND array_position(COALESCE(c.reloptions, ARRAY[]::text[]), 'security_invoker=true') IS NOT NULL
        ) THEN 'PASS'
        ELSE 'FAIL'
    END AS status,
    COALESCE((
        SELECT array_to_string(COALESCE(c.reloptions, ARRAY[]::text[]), ',')
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relname = 'violation_current_status'
          AND c.relkind = 'v'
        LIMIT 1
    ), 'NULL') AS observed;

-- ============================================================================
-- 3) FUNCTIONS
-- ============================================================================
WITH expected_functions AS (
    SELECT *
    FROM (VALUES
        ('add_violation_decision'),
        ('get_audit_stats')
    ) AS v(fn_name)
)
SELECT
    'functions' AS check_group,
    fn_name AS check_name,
    CASE
        WHEN EXISTS (
            SELECT 1
            FROM pg_proc p
            JOIN pg_namespace n ON n.oid = p.pronamespace
            WHERE n.nspname = 'public'
              AND p.proname = fn_name
        ) THEN 'PASS'
        ELSE 'FAIL'
    END AS status,
    COALESCE((
        SELECT p.oid::regprocedure::text
        FROM pg_proc p
        JOIN pg_namespace n ON n.oid = p.pronamespace
        WHERE n.nspname = 'public'
          AND p.proname = fn_name
        ORDER BY p.oid
        LIMIT 1
    ), 'NULL') AS observed
FROM expected_functions
ORDER BY check_name;

-- ============================================================================
-- 4) APPEND-ONLY TRIGGERS
-- ============================================================================
WITH expected_triggers AS (
    SELECT *
    FROM (VALUES
        ('violations_append_only', 'violations'),
        ('violation_decisions_append_only', 'violation_decisions'),
        ('policy_drift_events_append_only', 'policy_drift_events')
    ) AS v(tgname, relname)
)
SELECT
    'triggers' AS check_group,
    tgname AS check_name,
    CASE
        WHEN EXISTS (
            SELECT 1
            FROM pg_trigger t
            JOIN pg_class c ON c.oid = t.tgrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND t.tgname = expected_triggers.tgname
              AND c.relname = expected_triggers.relname
              AND NOT t.tgisinternal
        ) THEN 'PASS'
        ELSE 'FAIL'
    END AS status,
    COALESCE((
        SELECT t.tgname || ' -> ' || t.tgrelid::regclass::text
        FROM pg_trigger t
        JOIN pg_class c ON c.oid = t.tgrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND t.tgname = expected_triggers.tgname
          AND c.relname = expected_triggers.relname
          AND NOT t.tgisinternal
        LIMIT 1
    ), 'NULL') AS observed
FROM expected_triggers
ORDER BY check_name;

-- ============================================================================
-- 5) SMOKE: get_audit_stats(NULL) returns JSON with expected keys
-- ============================================================================
WITH smoke AS (
    SELECT get_audit_stats(NULL::uuid)::jsonb AS payload
),
smoke_checks AS (
    SELECT
        'smoke.json_root_object' AS check_name,
        CASE
            WHEN jsonb_typeof(payload) = 'object' THEN 'PASS'
            ELSE 'FAIL'
        END AS status,
        COALESCE(jsonb_typeof(payload), 'NULL') AS observed
    FROM smoke

    UNION ALL

    SELECT
        'smoke.keys.top_level' AS check_name,
        CASE
            WHEN payload ? 'github_events'
              AND payload ? 'client_events'
              AND payload ? 'violations'
              AND payload ? 'active_devs_week'
              AND payload ? 'active_repos'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        payload::text AS observed
    FROM smoke

    UNION ALL

    SELECT
        'smoke.keys.github_events' AS check_name,
        CASE
            WHEN jsonb_typeof(payload -> 'github_events') = 'object'
              AND (payload -> 'github_events') ? 'total'
              AND (payload -> 'github_events') ? 'today'
              AND (payload -> 'github_events') ? 'pushes_today'
              AND (payload -> 'github_events') ? 'by_type'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        (payload -> 'github_events')::text AS observed
    FROM smoke

    UNION ALL

    SELECT
        'smoke.keys.client_events' AS check_name,
        CASE
            WHEN jsonb_typeof(payload -> 'client_events') = 'object'
              AND (payload -> 'client_events') ? 'total'
              AND (payload -> 'client_events') ? 'today'
              AND (payload -> 'client_events') ? 'blocked_today'
              AND (payload -> 'client_events') ? 'by_type'
              AND (payload -> 'client_events') ? 'by_status'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        (payload -> 'client_events')::text AS observed
    FROM smoke

    UNION ALL

    SELECT
        'smoke.keys.violations' AS check_name,
        CASE
            WHEN jsonb_typeof(payload -> 'violations') = 'object'
              AND (payload -> 'violations') ? 'total'
              AND (payload -> 'violations') ? 'unresolved'
              AND (payload -> 'violations') ? 'critical'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        (payload -> 'violations')::text AS observed
    FROM smoke
)
SELECT
    'smoke' AS check_group,
    check_name,
    status,
    observed
FROM smoke_checks
ORDER BY check_name;
