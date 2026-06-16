SELECT
    'change_risk_evaluations.table' AS check_name,
    CASE WHEN to_regclass('public.change_risk_evaluations') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status;

SELECT
    'change_risk_evaluations.no_claim_constraints' AS check_name,
    CASE WHEN COUNT(*) = 5 THEN 'PASS' ELSE 'FAIL' END AS status
FROM (
    SELECT lower(pg_get_constraintdef(oid)) AS definition
    FROM pg_constraint
    WHERE conrelid = 'public.change_risk_evaluations'::regclass
      AND contype = 'c'
) constraints
WHERE (
      definition LIKE '%advisory_only = true%'
      OR definition LIKE '%llm_used = false%'
      OR definition LIKE '%agent_governance_used = false%'
      OR definition LIKE '%compliance_claim = false%'
      OR definition LIKE '%certification = false%'
  );

SELECT
    'change_risk_evaluations.indexes' AS check_name,
    CASE WHEN COUNT(*) = 6 THEN 'PASS' ELSE 'FAIL' END AS status
FROM pg_indexes
WHERE schemaname = 'public'
  AND tablename = 'change_risk_evaluations'
  AND indexname IN (
      'change_risk_evaluations_pkey',
      'idx_change_risk_evaluations_org_created',
      'idx_change_risk_evaluations_scope',
      'idx_change_risk_evaluations_gate',
      'idx_change_risk_evaluations_release',
      'idx_change_risk_evaluations_commit'
  );
