WITH checks AS (
    SELECT
        'agent_governance_agent_keys.lifecycle_columns' AS check_name,
        CASE
            WHEN (
                SELECT COUNT(*)
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_governance_agent_keys'
                  AND column_name IN (
                      'rotated_at',
                      'rotated_from_key_id',
                      'replaced_by_key_id',
                      'rotation_reason'
                  )
            ) = 4
            THEN 'PASS' ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_agent_keys.lifecycle_indexes' AS check_name,
        CASE
            WHEN to_regclass('public.idx_agent_governance_agent_keys_rotation_from') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_agent_keys_replaced_by') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_agent_keys_expiry') IS NOT NULL
            THEN 'PASS' ELSE 'FAIL'
        END AS status
)
SELECT *
FROM checks;
