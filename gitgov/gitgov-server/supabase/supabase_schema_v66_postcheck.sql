DO $$
DECLARE
    missing_columns TEXT[];
BEGIN
    IF to_regclass('public.change_risk_cab_packets') IS NULL THEN
        RAISE EXCEPTION 'KAN-126 postcheck failed: change_risk_cab_packets table missing';
    END IF;

    SELECT array_agg(column_name ORDER BY column_name)
    INTO missing_columns
    FROM (
        VALUES
            ('review_status'),
            ('reviewed_by_user_id'),
            ('reviewed_at'),
            ('review_notes_safe'),
            ('mitigation_notes_safe'),
            ('decision_reason_safe'),
            ('follow_up_required'),
            ('follow_up_owner_safe'),
            ('review_updated_at')
    ) AS expected(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_cab_packets'
          AND columns.column_name = expected.column_name
    );

    IF missing_columns IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-126 postcheck failed: change_risk_cab_packets missing columns %', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND conname = 'change_risk_cab_packets_review_status_check'
          AND pg_get_constraintdef(oid) LIKE '%pending_review%'
          AND pg_get_constraintdef(oid) LIKE '%returned_to_owner%'
    ) THEN
        RAISE EXCEPTION 'KAN-126 postcheck failed: review status constraint missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND conname = 'change_risk_cab_packets_follow_up_check'
    ) THEN
        RAISE EXCEPTION 'KAN-126 postcheck failed: follow-up constraint missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'change_risk_cab_packets'
          AND indexname = 'idx_change_risk_cab_packets_review'
    ) THEN
        RAISE EXCEPTION 'KAN-126 postcheck failed: review index missing';
    END IF;
END $$;
