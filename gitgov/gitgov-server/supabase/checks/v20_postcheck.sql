\echo '=== GitGov v20 post-check ==='

-- policy change request tables
SELECT to_regclass('public.policy_change_requests') AS policy_change_requests;
SELECT to_regclass('public.policy_change_request_decisions') AS policy_change_request_decisions;

-- expected indexes
SELECT to_regclass('public.idx_policy_change_requests_org_created') AS idx_req_org_created;
SELECT to_regclass('public.idx_policy_change_requests_repo_name_created') AS idx_req_repo_name_created;
SELECT to_regclass('public.idx_policy_change_request_decisions_org_created') AS idx_dec_org_created;

-- append-only triggers
SELECT tgname
FROM pg_trigger
WHERE tgname IN (
  'policy_change_requests_append_only',
  'policy_change_request_decisions_append_only'
)
ORDER BY tgname;

\echo '=== v20 post-check complete ==='
