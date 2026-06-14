WITH checks AS (
    SELECT
        'agent_governance_evaluations.table' AS check_name,
        CASE
            WHEN to_regclass('public.agent_governance_evaluations') IS NOT NULL
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_evaluations.constraints' AS check_name,
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM pg_constraint
                WHERE conrelid = 'public.agent_governance_evaluations'::regclass
                  AND conname LIKE '%action%'
            )
            AND EXISTS (
                SELECT 1
                FROM pg_constraint
                WHERE conrelid = 'public.agent_governance_evaluations'::regclass
                  AND conname LIKE '%decision%'
            )
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_evaluations.indexes' AS check_name,
        CASE
            WHEN to_regclass('public.idx_agent_governance_evaluations_org_created') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_evaluations_scope') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_evaluations_agent') IS NOT NULL
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
)
SELECT * FROM checks ORDER BY check_name;
