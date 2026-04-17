\echo '=== GitGov v21 post-check ==='

-- chat trace tables
SELECT to_regclass('public.chat_query_events') AS chat_query_events;
SELECT to_regclass('public.chat_query_tool_calls') AS chat_query_tool_calls;

-- expected indexes
SELECT to_regclass('public.idx_chat_query_events_created') AS idx_events_created;
SELECT to_regclass('public.idx_chat_query_events_client_created') AS idx_events_client_created;
SELECT to_regclass('public.idx_chat_query_tool_calls_trace_created') AS idx_tool_calls_trace_created;

-- append-only triggers
SELECT tgname
FROM pg_trigger
WHERE tgname IN (
  'chat_query_events_append_only',
  'chat_query_tool_calls_append_only'
)
ORDER BY tgname;

\echo '=== v21 post-check complete ==='
