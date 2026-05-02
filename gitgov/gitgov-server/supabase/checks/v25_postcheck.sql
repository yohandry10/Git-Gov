WITH checks AS (
    SELECT
        'enterprise_onboarding_checklist_tracking.table_exists' AS check_name,
        CASE WHEN to_regclass('public.enterprise_onboarding_checklist_tracking') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.enterprise_onboarding_checklist_tracking')::text, 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_onboarding_checklist_tracking.primary_key' AS check_name,
        CASE WHEN EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_onboarding_checklist_tracking')
              AND contype = 'p'
        ) THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE((
            SELECT conname
            FROM pg_constraint
            WHERE conrelid = to_regclass('public.enterprise_onboarding_checklist_tracking')
              AND contype = 'p'
            LIMIT 1
        ), 'missing') AS observed

    UNION ALL

    SELECT
        'enterprise_onboarding_checklist_tracking.updated_at_index' AS check_name,
        CASE WHEN to_regclass('public.idx_enterprise_onboarding_checklist_tracking_updated_at') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status,
        COALESCE(to_regclass('public.idx_enterprise_onboarding_checklist_tracking_updated_at')::text, 'missing') AS observed
)
SELECT *
FROM checks;
