WITH checks AS (
    SELECT
        'agent_governance_settings.table' AS check_name,
        CASE
            WHEN to_regclass('public.agent_governance_settings') IS NOT NULL
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_settings.constraints' AS check_name,
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM pg_constraint
                WHERE conrelid = 'public.agent_governance_settings'::regclass
                  AND conname LIKE '%mode%'
            )
            AND EXISTS (
                SELECT 1
                FROM pg_constraint
                WHERE conrelid = 'public.agent_governance_settings'::regclass
                  AND conname LIKE '%payload_mode%'
            )
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_settings.indexes' AS check_name,
        CASE
            WHEN to_regclass('public.idx_agent_governance_settings_enabled') IS NOT NULL
            THEN 'PASS'
            ELSE 'FAIL'
        END AS status
)
SELECT * FROM checks ORDER BY check_name;
