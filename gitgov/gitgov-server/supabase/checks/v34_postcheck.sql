-- KAN-82 postcheck: platform founder principal exists outside tenant scope.

SELECT
    'platform_principals.table' AS check_name,
    CASE WHEN to_regclass('public.platform_principals') IS NOT NULL THEN 'PASS' ELSE 'FAIL' END AS status;

SELECT
    'platform_principals.constraints' AS check_name,
    CASE WHEN COUNT(*) = 3 THEN 'PASS' ELSE 'FAIL' END AS status
FROM pg_constraint
WHERE conrelid = 'platform_principals'::regclass
  AND conname IN (
      'platform_principals_type_check',
      'platform_principals_status_check',
      'platform_principals_auth_method_check'
  );

SELECT
    'platform_principals.bootstrap_founder' AS check_name,
    CASE WHEN COUNT(*) = 1 THEN 'PASS' ELSE 'FAIL' END AS status
FROM platform_principals
WHERE client_id = 'bootstrap-admin'
  AND principal_type = 'platform_founder'
  AND status = 'active'
  AND auth_method = 'api_key';
