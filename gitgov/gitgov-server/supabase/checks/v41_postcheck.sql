WITH checks AS (
    SELECT
        'agent_governance_evaluations.attribution_columns' AS check_name,
        CASE
            WHEN (
                SELECT COUNT(*)
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'agent_governance_evaluations'
                  AND column_name IN (
                      'attribution_id',
                      'correlation_id',
                      'parent_correlation_id',
                      'session_id',
                      'tool_name',
                      'tool_version',
                      'agent_name',
                      'external_run_id',
                      'consumer_type'
                  )
            ) = 9
            THEN 'PASS' ELSE 'FAIL'
        END AS status
    UNION ALL
    SELECT
        'agent_governance_evaluations.attribution_indexes' AS check_name,
        CASE
            WHEN to_regclass('public.idx_agent_governance_evaluations_correlation') IS NOT NULL
             AND to_regclass('public.idx_agent_governance_evaluations_session') IS NOT NULL
            THEN 'PASS' ELSE 'FAIL'
        END AS status
)
SELECT *
FROM checks;
