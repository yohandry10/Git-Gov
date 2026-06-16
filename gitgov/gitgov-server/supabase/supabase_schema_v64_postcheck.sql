DO $$
DECLARE
    missing_columns TEXT[];
    constraint_count INTEGER;
    index_count INTEGER;
BEGIN
    SELECT ARRAY(
        SELECT column_name
        FROM (VALUES
            ('review_status'),
            ('reviewed_by_user_id'),
            ('reviewed_at'),
            ('review_notes_safe'),
            ('mitigation_notes_safe'),
            ('decision_reason_safe'),
            ('review_updated_at')
        ) AS expected(column_name)
        WHERE NOT EXISTS (
            SELECT 1
            FROM information_schema.columns c
            WHERE c.table_schema = 'public'
              AND c.table_name = 'change_risk_evaluations'
              AND c.column_name = expected.column_name
        )
    ) INTO missing_columns;

    IF array_length(missing_columns, 1) IS NOT NULL THEN
        RAISE EXCEPTION 'v64 postcheck failed: missing columns %', missing_columns;
    END IF;

    SELECT COUNT(*)
    INTO constraint_count
    FROM pg_constraint
    WHERE conrelid = 'public.change_risk_evaluations'::regclass
      AND conname IN (
          'change_risk_evaluations_review_status_check',
          'change_risk_evaluations_review_notes_safe_len_check'
      );

    IF constraint_count <> 2 THEN
        RAISE EXCEPTION 'v64 postcheck failed: expected 2 review constraints, found %', constraint_count;
    END IF;

    SELECT COUNT(*)
    INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename = 'change_risk_evaluations'
      AND indexname = 'idx_change_risk_evaluations_review';

    IF index_count <> 1 THEN
        RAISE EXCEPTION 'v64 postcheck failed: review index missing';
    END IF;
END $$;
