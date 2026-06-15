-- KAN-117 postcheck: Period Compliance Report review/sign-off metadata.

DO $$
DECLARE
    review_column_count INTEGER;
    review_constraint_count INTEGER;
    review_index_count INTEGER;
    action_constraint_count INTEGER;
    artifact_type_constraint_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO review_column_count
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'compliance_period_reports'
      AND column_name IN (
          'review_status',
          'reviewed_by_user_id',
          'reviewed_at',
          'review_notes_safe'
      );

    IF review_column_count <> 4 THEN
        RAISE EXCEPTION 'KAN-117 postcheck failed: period report review columns missing';
    END IF;

    SELECT COUNT(*)
    INTO review_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_reports'
      AND c.conname IN (
          'compliance_period_reports_review_status_check',
          'compliance_period_reports_review_notes_safe_check',
          'compliance_period_reports_terminal_review_note_check'
      );

    IF review_constraint_count <> 3 THEN
        RAISE EXCEPTION 'KAN-117 postcheck failed: period report review constraints missing';
    END IF;

    SELECT COUNT(*)
    INTO review_index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename = 'compliance_period_reports'
      AND indexname = 'idx_compliance_period_reports_org_review_status';

    IF review_index_count <> 1 THEN
        RAISE EXCEPTION 'KAN-117 postcheck failed: period report review index missing';
    END IF;

    SELECT COUNT(*)
    INTO action_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_report_access_log'
      AND c.conname = 'compliance_period_report_access_log_action_check'
      AND pg_get_constraintdef(c.oid) LIKE '%review_updated%';

    IF action_constraint_count <> 1 THEN
        RAISE EXCEPTION 'KAN-117 postcheck failed: period report review access-log action missing';
    END IF;

    SELECT COUNT(*)
    INTO artifact_type_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_report_access_log'
      AND c.conname = 'compliance_period_report_access_log_artifact_type_check'
      AND pg_get_constraintdef(c.oid) LIKE '%review%';

    IF artifact_type_constraint_count <> 1 THEN
        RAISE EXCEPTION 'KAN-117 postcheck failed: period report review access-log artifact type missing';
    END IF;

    RAISE NOTICE 'KAN-117 postcheck passed: Period Compliance Report review/sign-off metadata is installed.';
END $$;
