-- KAN-99 Compliance Evidence Exports postcheck

DO $$
DECLARE
    missing_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO missing_count
    FROM (
        VALUES
            ('compliance_evidence_exports', 'export_id'),
            ('compliance_evidence_exports', 'org_id'),
            ('compliance_evidence_exports', 'deployment_gate_id'),
            ('compliance_evidence_exports', 'artifact_hash'),
            ('compliance_evidence_exports', 'payload_json_redacted')
    ) AS expected(table_name, column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_schema = 'public'
          AND c.table_name = expected.table_name
          AND c.column_name = expected.column_name
    );

    IF missing_count <> 0 THEN
        RAISE EXCEPTION 'KAN-99 postcheck failed: missing % required columns', missing_count;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_evidence_exports_deployment_gate'
    ) THEN
        RAISE EXCEPTION 'KAN-99 postcheck failed: missing deployment gate index';
    END IF;

    RAISE NOTICE 'KAN-99 postcheck PASS: compliance evidence exports table, columns, and indexes exist';
END $$;
