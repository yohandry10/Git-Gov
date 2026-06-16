-- KAN-122 postcheck: Change Risk Rule Catalog & Evaluation Trace.

DO $$
DECLARE
    missing_columns INTEGER;
BEGIN
    IF to_regclass('public.change_risk_evaluations') IS NULL THEN
        RAISE EXCEPTION 'KAN-122 postcheck failed: change_risk_evaluations table missing';
    END IF;

    SELECT COUNT(*)
    INTO missing_columns
    FROM (
        VALUES
            ('ruleset_version'),
            ('triggered_rules'),
            ('non_triggered_rules'),
            ('evaluation_trace'),
            ('trace_hash'),
            ('advisory_only'),
            ('llm_used'),
            ('agent_governance_used'),
            ('compliance_claim'),
            ('certification')
    ) AS required(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_evaluations'
          AND columns.column_name = required.column_name
    );

    IF missing_columns <> 0 THEN
        RAISE EXCEPTION 'KAN-122 postcheck failed: change_risk_evaluations missing % columns', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE t.relname = 'change_risk_evaluations'
          AND n.nspname = 'public'
          AND c.conname IN (
              'change_risk_evaluations_ruleset_version_check',
              'change_risk_evaluations_triggered_rules_check',
              'change_risk_evaluations_non_triggered_rules_check',
              'change_risk_evaluations_trace_json_check',
              'change_risk_evaluations_trace_hash_check'
          )
        HAVING COUNT(*) = 5
    ) THEN
        RAISE EXCEPTION 'KAN-122 postcheck failed: rule trace constraints missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'change_risk_evaluations'
          AND indexname = 'idx_change_risk_evaluations_ruleset'
    ) THEN
        RAISE EXCEPTION 'KAN-122 postcheck failed: ruleset index missing';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM change_risk_evaluations
        WHERE advisory_only IS DISTINCT FROM TRUE
           OR llm_used IS DISTINCT FROM FALSE
           OR agent_governance_used IS DISTINCT FROM FALSE
           OR compliance_claim IS DISTINCT FROM FALSE
           OR certification IS DISTINCT FROM FALSE
    ) THEN
        RAISE EXCEPTION 'KAN-122 postcheck failed: advisory/no-claim invariants violated';
    END IF;
END $$;
