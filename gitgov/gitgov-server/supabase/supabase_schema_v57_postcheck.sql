DO $$
DECLARE
    missing TEXT[];
BEGIN
    SELECT ARRAY_AGG(name)
    INTO missing
    FROM (
        VALUES
            ('compliance_period_reports.retention_status'),
            ('compliance_period_reports.retention_until'),
            ('compliance_period_reports.download_count'),
            ('compliance_period_reports.last_downloaded_at'),
            ('compliance_period_reports.archived_at'),
            ('compliance_period_report_access_log.access_log_id'),
            ('compliance_period_report_access_log.org_id'),
            ('compliance_period_report_access_log.period_report_id'),
            ('compliance_period_report_access_log.actor_client_id'),
            ('compliance_period_report_access_log.action'),
            ('compliance_period_report_access_log.artifact_type'),
            ('compliance_period_report_access_log.artifact_id'),
            ('compliance_period_report_access_log.artifact_hash'),
            ('compliance_period_report_access_log.metadata'),
            ('compliance_period_report_access_log.created_at')
    ) AS required(name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_schema = 'public'
          AND c.table_name = split_part(required.name, '.', 1)
          AND c.column_name = split_part(required.name, '.', 2)
    );

    IF missing IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing columns: %', array_to_string(missing, ', ');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_retention_status_check'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing retention status constraint';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_download_count_check'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing download count constraint';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_archived_status_check'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing archived status constraint';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_period_reports_retention'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing retention index';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_period_report_access_log_report_created'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing report access log index';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_period_report_access_log_actor_created'
    ) THEN
        RAISE EXCEPTION 'KAN-115 postcheck missing actor access log index';
    END IF;

    RAISE NOTICE 'KAN-115 postcheck passed: Period Compliance Report retention and access log are installed.';
END $$;
