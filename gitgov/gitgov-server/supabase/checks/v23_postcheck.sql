\echo '=== GitGov v23 post-check ==='

WITH checks AS (
    SELECT
        'enterprise_adoption_profiles.table_exists' AS check_name,
        CASE WHEN to_regclass('public.enterprise_adoption_profiles') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.enterprise_adoption_profiles')::text, 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_adoption_profiles.primary_key' AS check_name,
        CASE WHEN EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_adoption_profiles')
              AND contype = 'p'
        ) THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE((
            SELECT conname
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_adoption_profiles')
              AND contype = 'p'
            LIMIT 1
        ), 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_adoption_profiles.updated_at_index' AS check_name,
        CASE WHEN to_regclass('public.idx_enterprise_adoption_profiles_updated_at') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.idx_enterprise_adoption_profiles_updated_at')::text, 'missing') AS observed
)
SELECT check_name, status, observed
FROM checks
ORDER BY check_name;

\echo '=== v23 post-check complete ==='
