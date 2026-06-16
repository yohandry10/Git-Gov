-- KAN-122: Change Risk Rule Catalog & Evaluation Trace.
-- Extends the advisory-only KAN-121 table with deterministic rule trace
-- evidence. These fields are audit metadata only; they do not approve,
-- block, certify, deploy, call AI/LLMs, or depend on Agent Governance.

ALTER TABLE change_risk_evaluations
    ADD COLUMN IF NOT EXISTS ruleset_version TEXT NOT NULL DEFAULT 'change_risk_rules.v1',
    ADD COLUMN IF NOT EXISTS triggered_rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS non_triggered_rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS evaluation_trace JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS trace_hash TEXT NOT NULL DEFAULT 'sha256:0000000000000000000000000000000000000000000000000000000000000000';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_ruleset_version_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_ruleset_version_check
            CHECK (ruleset_version <> '');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_triggered_rules_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_triggered_rules_check
            CHECK (jsonb_typeof(triggered_rules) = 'array');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_non_triggered_rules_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_non_triggered_rules_check
            CHECK (jsonb_typeof(non_triggered_rules) = 'array');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_trace_json_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_trace_json_check
            CHECK (jsonb_typeof(evaluation_trace) = 'object');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_trace_hash_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_trace_hash_check
            CHECK (trace_hash ~ '^sha256:[a-f0-9]{64}$');
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_ruleset
    ON change_risk_evaluations(org_id, ruleset_version, created_at DESC);
