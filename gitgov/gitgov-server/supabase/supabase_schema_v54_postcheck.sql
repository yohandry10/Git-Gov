DO $$
DECLARE
    missing TEXT[];
BEGIN
    SELECT array_agg(name)
    INTO missing
    FROM (
        VALUES
            ('compliance_framework_review_report_pdf_exports.table'),
            ('idx_cfr_report_pdf_exports_report_created.index'),
            ('idx_cfr_report_pdf_exports_manifest.index'),
            ('compliance_framework_review_report_pdf_exports.claim_constraints')
    ) AS expected(name)
    WHERE NOT CASE expected.name
        WHEN 'compliance_framework_review_report_pdf_exports.table' THEN EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_name = 'compliance_framework_review_report_pdf_exports'
        )
        WHEN 'idx_cfr_report_pdf_exports_report_created.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_cfr_report_pdf_exports_report_created'
        )
        WHEN 'idx_cfr_report_pdf_exports_manifest.index' THEN EXISTS (
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'idx_cfr_report_pdf_exports_manifest'
        )
        WHEN 'compliance_framework_review_report_pdf_exports.claim_constraints' THEN EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_framework_review_report_pdf_exports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%compliance_claim = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_framework_review_report_pdf_exports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%regulatory_claim = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_framework_review_report_pdf_exports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%certification = false%'
        ) AND EXISTS (
            SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            WHERE t.relname = 'compliance_framework_review_report_pdf_exports'
              AND c.contype = 'c'
              AND pg_get_constraintdef(c.oid) LIKE '%requires_auditor_review = true%'
        )
        ELSE FALSE
    END;

    IF missing IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-111 postcheck failed, missing: %', missing;
    END IF;

    RAISE NOTICE 'KAN-111 postcheck passed: Framework Review Report PDF exports are installed.';
END $$;
