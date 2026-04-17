-- ===================================================================
-- GitGov schema migration v21
-- Date: 2026-04-16
-- Purpose:
--   1) Persist chat query responses as append-only evidence.
--   2) Persist chat tool execution traces linked by trace_id.
--   3) Add indexes and grants required by runtime handlers.
-- ===================================================================

-- -------------------------------------------------------------------
-- Chat query events (append-only audit evidence)
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chat_query_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trace_id TEXT NOT NULL UNIQUE,
    conversation_key TEXT NOT NULL,
    client_id TEXT NOT NULL,
    org_scope TEXT,
    question TEXT NOT NULL,
    intent TEXT NOT NULL,
    response_status TEXT NOT NULL CHECK (response_status IN ('ok', 'insufficient_data', 'feature_not_available', 'error')),
    confidence REAL,
    language TEXT,
    entities_detected JSONB NOT NULL DEFAULT '[]'::jsonb,
    time_range_used TEXT,
    sources JSONB NOT NULL DEFAULT '[]'::jsonb,
    actions_recommended JSONB NOT NULL DEFAULT '[]'::jsonb,
    answer_preview TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_query_events_created
    ON chat_query_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_query_events_client_created
    ON chat_query_events(client_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_query_events_org_created
    ON chat_query_events(org_scope, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_query_events_status_created
    ON chat_query_events(response_status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_query_events_intent_created
    ON chat_query_events(intent, created_at DESC);

-- -------------------------------------------------------------------
-- Chat tool calls (append-only trace rows)
-- -------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chat_query_tool_calls (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trace_id TEXT NOT NULL REFERENCES chat_query_events(trace_id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    tool_status TEXT NOT NULL CHECK (tool_status IN ('ok', 'error', 'skipped')),
    duration_ms INTEGER,
    input_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    output_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_query_tool_calls_trace_created
    ON chat_query_tool_calls(trace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_query_tool_calls_tool_created
    ON chat_query_tool_calls(tool_name, created_at DESC);

-- -------------------------------------------------------------------
-- Append-only protections
-- -------------------------------------------------------------------
DROP TRIGGER IF EXISTS chat_query_events_append_only ON chat_query_events;
CREATE TRIGGER chat_query_events_append_only
    BEFORE UPDATE OR DELETE ON chat_query_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

DROP TRIGGER IF EXISTS chat_query_tool_calls_append_only ON chat_query_tool_calls;
CREATE TRIGGER chat_query_tool_calls_append_only
    BEFORE UPDATE OR DELETE ON chat_query_tool_calls
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- -------------------------------------------------------------------
-- Optional grants if role exists in target environment
-- -------------------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT ON chat_query_events TO gitgov_server;
        GRANT SELECT, INSERT ON chat_query_tool_calls TO gitgov_server;
    END IF;
END;
$$;
