\echo '=== GitGov v22 post-check ==='

WITH payload AS (
    SELECT get_audit_stats(NULL::uuid)::jsonb AS stats
),
github_expected AS (
    SELECT
        COUNT(*)::bigint AS total,
        COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('day', NOW()))::bigint AS today,
        COUNT(*) FILTER (
            WHERE event_type = 'push'
              AND created_at >= DATE_TRUNC('day', NOW())
        )::bigint AS pushes_today
    FROM github_events
),
checks AS (
    SELECT
        'github_events.shape' AS check_name,
        CASE
            WHEN jsonb_typeof(stats -> 'github_events') = 'object'
              AND (stats -> 'github_events') ? 'total'
              AND (stats -> 'github_events') ? 'today'
              AND (stats -> 'github_events') ? 'pushes_today'
              AND jsonb_typeof(stats -> 'github_events' -> 'by_type') = 'object'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        (stats -> 'github_events')::text AS observed
    FROM payload

    UNION ALL

    SELECT
        'github_events.total_matches_table' AS check_name,
        CASE
            WHEN ((stats -> 'github_events' ->> 'total')::bigint = github_expected.total)
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        jsonb_build_object(
            'stats_total', (stats -> 'github_events' ->> 'total')::bigint,
            'table_total', github_expected.total
        )::text AS observed
    FROM payload, github_expected

    UNION ALL

    SELECT
        'github_events.today_matches_table' AS check_name,
        CASE
            WHEN ((stats -> 'github_events' ->> 'today')::bigint = github_expected.today)
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        jsonb_build_object(
            'stats_today', (stats -> 'github_events' ->> 'today')::bigint,
            'table_today', github_expected.today
        )::text AS observed
    FROM payload, github_expected

    UNION ALL

    SELECT
        'github_events.pushes_today_matches_table' AS check_name,
        CASE
            WHEN ((stats -> 'github_events' ->> 'pushes_today')::bigint = github_expected.pushes_today)
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        jsonb_build_object(
            'stats_pushes_today', (stats -> 'github_events' ->> 'pushes_today')::bigint,
            'table_pushes_today', github_expected.pushes_today
        )::text AS observed
    FROM payload, github_expected

    UNION ALL

    SELECT
        'active_repos.is_number' AS check_name,
        CASE
            WHEN jsonb_typeof(stats -> 'active_repos') = 'number'
            THEN 'PASS' ELSE 'FAIL'
        END AS status,
        (stats -> 'active_repos')::text AS observed
    FROM payload
)
SELECT check_name, status, observed
FROM checks
ORDER BY check_name;

\echo '=== v22 post-check complete ==='
