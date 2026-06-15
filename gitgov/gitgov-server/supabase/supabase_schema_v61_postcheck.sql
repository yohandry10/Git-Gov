-- KAN-119 postcheck: Period Compliance Report share packages.

DO $$
DECLARE
    missing_columns INTEGER;
BEGIN
    IF to_regclass('public.compliance_period_report_share_packages') IS NULL THEN
        RAISE EXCEPTION 'KAN-119 postcheck failed: compliance_period_report_share_packages table missing';
    END IF;

    SELECT COUNT(*)
    INTO missing_columns
    FROM (
        VALUES
            ('share_package_id'),
            ('org_id'),
            ('period_report_id'),
            ('created_by_user_id'),
            ('package_format'),
            ('status'),
            ('artifact_hash'),
            ('payload_json_redacted'),
            ('period_report_artifact_hash'),
            ('pdf_export_id'),
            ('pdf_artifact_hash'),
            ('manifest_id'),
            ('manifest_hash'),
            ('no_claims_snapshot'),
            ('source_hashes'),
            ('review_snapshot'),
            ('retention_snapshot'),
            ('download_count'),
            ('downloaded_at'),
            ('last_downloaded_at'),
            ('revoked_at'),
            ('revoked_by_user_id'),
            ('created_at'),
            ('error_message_safe')
    ) AS required(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'compliance_period_report_share_packages'
          AND columns.column_name = required.column_name
    );

    IF missing_columns <> 0 THEN
        RAISE EXCEPTION 'KAN-119 postcheck failed: compliance_period_report_share_packages missing % columns', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE t.relname = 'compliance_period_report_share_packages'
          AND n.nspname = 'public'
          AND c.conname IN (
              'compliance_period_report_share_packages_no_claims_check',
              'compliance_period_report_share_packages_payload_check',
              'compliance_period_report_share_packages_status_check',
              'compliance_period_report_share_packages_hash_check'
          )
        HAVING COUNT(*) = 4
    ) THEN
        RAISE EXCEPTION 'KAN-119 postcheck failed: share package constraints missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'compliance_period_report_share_packages'
          AND indexname IN (
              'idx_compliance_period_report_share_packages_report_created',
              'idx_compliance_period_report_share_packages_status',
              'idx_compliance_period_report_share_packages_hash'
          )
        HAVING COUNT(*) = 3
    ) THEN
        RAISE EXCEPTION 'KAN-119 postcheck failed: share package indexes missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE t.relname = 'compliance_period_report_access_log'
          AND n.nspname = 'public'
          AND c.conname = 'compliance_period_report_access_log_action_check'
          AND pg_get_constraintdef(c.oid) LIKE '%share_package_created%'
          AND pg_get_constraintdef(c.oid) LIKE '%share_package_downloaded%'
          AND pg_get_constraintdef(c.oid) LIKE '%share_package_revoked%'
    ) THEN
        RAISE EXCEPTION 'KAN-119 postcheck failed: access log action constraint missing share package actions';
    END IF;
END $$;
