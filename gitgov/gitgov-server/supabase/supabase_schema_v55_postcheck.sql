DO $$
DECLARE
    missing TEXT[];
BEGIN
    SELECT array_agg(name)
    INTO missing
    FROM (
        VALUES
            ('compliance_period_reports.table'),
            ('idx_compliance_period_reports_org_created.index'),
            ('idx_compliance_period_reports_framework_created.index'),
            ('idx_compliance_period_reports_date_range.index'),
            ('idx_compliance_period_reports_artifact_hash.index'),
            ('compliance_period_reports.claim_constraints'),
            ('compliance_period_reports.source_report_ids_constraint')
    ) AS expected(name)
    WHERE NOT CASE expected.name
        WHEN 'compliance_period_reports.table' THEN EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_name = 'compliance_period_reports'
        )
        WHEN 'idx_compliance_period_reports_org_created.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_compliance_period_reports_org_created'
        )
        WHEN 'idx_compliance_period_reports_framework_created.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_compliance_period_reports_framework_created'
        )
        WHEN 'idx_compliance_period_reports_date_range.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_compliance_period_reports_date_range'
        )
        WHEN 'idx_compliance_period_reports_artifact_hash.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_compliance_period_reports_artifact_hash'
        )
        WHEN 'compliance_period_reports.claim_constraints' THEN EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_period_reports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%compliance_claim = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_period_reports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%regulatory_claim = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_period_reports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%certification = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_period_reports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%requires_auditor_review = true%'
        )
        WHEN 'compliance_period_reports.source_report_ids_constraint' THEN EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_period_reports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%jsonb_array_length(source_report_ids) = report_count%'
        )
        ELSE FALSE
    END;

    IF missing IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-113 postcheck failed, missing: %', missing;
    END IF;

    RAISE NOTICE 'KAN-113 postcheck passed: Period Compliance Report Generator storage is installed.';
END $$;
