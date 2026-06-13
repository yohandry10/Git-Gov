-- KAN-83 postcheck: deployment gate authorization table is ready.

SELECT
    'deployment_gate_authorizations.table' AS check_name,
    CASE WHEN to_regclass('public.deployment_gate_authorizations') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status;

SELECT
    'deployment_gate_authorizations.decision_constraint' AS check_name,
    CASE WHEN COUNT(*) = 1 THEN 'PASS' ELSE 'FAIL' END AS status
FROM pg_constraint
WHERE conrelid = 'deployment_gate_authorizations'::regclass
  AND conname = 'deployment_gate_authorizations_decision_check';

SELECT
    'deployment_gate_authorizations.indexes' AS check_name,
    CASE WHEN COUNT(*) >= 4 THEN 'PASS' ELSE 'FAIL' END AS status
FROM pg_indexes
WHERE schemaname = 'public'
  AND tablename = 'deployment_gate_authorizations'
  AND indexname IN (
      'deployment_gate_authorizations_pkey',
      'deployment_gate_authorizations_authorization_id_key',
      'idx_deployment_gate_authorizations_org_created',
      'idx_deployment_gate_authorizations_scope',
      'idx_deployment_gate_authorizations_release',
      'idx_deployment_gate_authorizations_target'
  );
