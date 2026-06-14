DO $$
DECLARE
    missing text[];
BEGIN
    SELECT array_agg(expected.indexname ORDER BY expected.indexname)
    INTO missing
    FROM (
        VALUES
            ('idx_compliance_framework_review_reports_framework_created'),
            ('idx_compliance_framework_review_reports_framework_mapping'),
            ('idx_compliance_framework_review_reports_framework_package')
    ) AS expected(indexname)
    WHERE NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = current_schema()
          AND tablename = 'compliance_framework_review_reports'
          AND indexname = expected.indexname
    );

    IF missing IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-106 postcheck failed; missing framework review report inventory indexes: %', missing;
    END IF;

    RAISE NOTICE 'KAN-106 postcheck passed: framework review report inventory indexes are present.';
END
$$;

