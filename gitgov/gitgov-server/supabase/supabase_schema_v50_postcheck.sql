DO $$
DECLARE
    missing_columns integer;
    invalid_status_rows integer;
BEGIN
    SELECT COUNT(*) INTO missing_columns
    FROM (
        VALUES
            ('review_status'),
            ('reviewed_by_user_id'),
            ('reviewed_at'),
            ('review_notes_safe')
    ) AS required(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_name = 'compliance_framework_review_reports'
          AND c.column_name = required.column_name
    );

    IF missing_columns > 0 THEN
        RAISE EXCEPTION 'KAN-107 postcheck failed: % review metadata columns missing', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = current_schema()
          AND tablename = 'compliance_framework_review_reports'
          AND indexname = 'idx_compliance_framework_review_reports_review_status'
    ) THEN
        RAISE EXCEPTION 'KAN-107 postcheck failed: review status index missing';
    END IF;

    SELECT COUNT(*) INTO invalid_status_rows
    FROM compliance_framework_review_reports
    WHERE review_status NOT IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')
       OR compliance_claim IS DISTINCT FROM FALSE
       OR regulatory_claim IS DISTINCT FROM FALSE
       OR requires_auditor_review IS DISTINCT FROM TRUE
       OR certification IS DISTINCT FROM FALSE;

    IF invalid_status_rows > 0 THEN
        RAISE EXCEPTION 'KAN-107 postcheck failed: % report rows violate review/no-claim constraints', invalid_status_rows;
    END IF;

    RAISE NOTICE 'KAN-107 postcheck passed: framework review report review metadata is present and no-claim constraints hold.';
END
$$;

