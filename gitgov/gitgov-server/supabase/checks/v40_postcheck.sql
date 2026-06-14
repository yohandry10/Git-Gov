WITH checks AS (
    SELECT
        'agent_governance_agent_keys.table' AS check_name,
        CASE
            WHEN to_regclass('public.agent_governance_agent_keys') IS NOT NULL
            THEN 'PASS' ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_agent_keys.constraints' AS check_name,
        CASE
            WHEN (
                SELECT COUNT(*)
                FROM pg_constraint
                WHERE conrelid = 'public.agent_governance_agent_keys'::regclass
                  AND conname IN (
                      'agent_governance_agent_keys_key_id_key',
                      'agent_governance_agent_keys_token_hash_key'
                  )
            ) = 2
            THEN 'PASS' ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_agent_keys.indexes' AS check_name,
        CASE
            WHEN to_regclass('public.idx_agent_governance_agent_keys_org_created') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_agent_keys_active') IS NOT NULL
            THEN 'PASS' ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_evaluations.agent_identity_columns' AS check_name,
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_governance_evaluations'
                  AND column_name = 'principal_type'
            )
            AND EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_governance_evaluations'
                  AND column_name = 'agent_key_id'
            )
            AND EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_governance_evaluations'
                  AND column_name = 'agent_display_name'
            )
            THEN 'PASS' ELSE 'FAIL'
        END AS status
)
SELECT *
FROM checks;
