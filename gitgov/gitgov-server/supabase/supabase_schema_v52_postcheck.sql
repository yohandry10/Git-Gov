DO $$
BEGIN
    IF to_regclass('compliance_framework_review_report_assignments') IS NULL THEN
        RAISE EXCEPTION 'KAN-109 postcheck failed: assignments table missing';
    END IF;

    IF to_regclass('compliance_framework_review_report_comments') IS NULL THEN
        RAISE EXCEPTION 'KAN-109 postcheck failed: comments table missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE tablename = 'compliance_framework_review_report_assignments'
          AND indexname = 'idx_cfr_report_assignments_auditor_status'
    ) THEN
        RAISE EXCEPTION 'KAN-109 postcheck failed: assigned-to-me index missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE tablename = 'compliance_framework_review_report_comments'
          AND indexname = 'idx_cfr_report_comments_report_created'
    ) THEN
        RAISE EXCEPTION 'KAN-109 postcheck failed: comments index missing';
    END IF;

    RAISE NOTICE 'KAN-109 postcheck passed: assignment/comment tables are installed.';
END
$$;
