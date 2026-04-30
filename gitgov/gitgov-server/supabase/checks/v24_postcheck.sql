\echo '=== GitGov v24 post-check ==='

WITH checks AS (
    SELECT
        'enterprise_release_approvals.table_exists' AS check_name,
        CASE WHEN to_regclass('public.enterprise_release_approvals') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.enterprise_release_approvals')::text, 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_release_approvals.primary_key' AS check_name,
        CASE WHEN EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_release_approvals')
              AND contype = 'p'
        ) THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE((
            SELECT conname
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_release_approvals')
              AND contype = 'p'
            LIMIT 1
        ), 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_release_approvals.decision_check' AS check_name,
        CASE WHEN EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_release_approvals')
              AND contype = 'c'
              AND pg_get_constraintdef(oid) LIKE '%accepted-risk%'
        ) THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE((
            SELECT conname
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_release_approvals')
              AND contype = 'c'
              AND pg_get_constraintdef(oid) LIKE '%accepted-risk%'
            LIMIT 1
        ), 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_release_approvals.org_created_index' AS check_name,
        CASE WHEN to_regclass('public.idx_enterprise_release_approvals_org_created') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.idx_enterprise_release_approvals_org_created')::text, 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_release_approvals.repo_release_index' AS check_name,
        CASE WHEN to_regclass('public.idx_enterprise_release_approvals_repo_release') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.idx_enterprise_release_approvals_repo_release')::text, 'missing') AS observed
)
SELECT check_name, status, observed
FROM checks
ORDER BY check_name;

\echo '=== v24 post-check complete ==='
