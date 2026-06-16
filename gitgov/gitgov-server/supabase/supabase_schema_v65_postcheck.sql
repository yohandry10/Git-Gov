DO $$
DECLARE
    missing_columns TEXT[];
    missing_indexes TEXT[];
BEGIN
    IF to_regclass('public.change_risk_cab_packets') IS NULL THEN
        RAISE EXCEPTION 'KAN-125 postcheck failed: change_risk_cab_packets table missing';
    END IF;

    SELECT array_agg(column_name ORDER BY column_name)
    INTO missing_columns
    FROM (
        VALUES
            ('packet_id'),
            ('org_id'),
            ('name'),
            ('filters_json'),
            ('evaluation_ids_json'),
            ('artifact_hash'),
            ('artifact_json'),
            ('status'),
            ('created_by_user_id'),
            ('created_at'),
            ('downloaded_at'),
            ('download_count'),
            ('archived_at'),
            ('archived_by_user_id')
    ) AS expected(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_cab_packets'
          AND columns.column_name = expected.column_name
    );

    IF missing_columns IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-125 postcheck failed: change_risk_cab_packets missing columns %', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_cab_packets'
          AND column_name = 'download_count'
          AND data_type = 'bigint'
    ) THEN
        RAISE EXCEPTION 'KAN-125 postcheck failed: change_risk_cab_packets.download_count must be bigint';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND pg_get_constraintdef(oid) LIKE '%gitgov_change_risk_cab_packet.v1%'
          AND pg_get_constraintdef(oid) LIKE '%source_evaluations_mutated%'
    ) THEN
        RAISE EXCEPTION 'KAN-125 postcheck failed: no-claim artifact constraint missing';
    END IF;

    SELECT array_agg(indexname ORDER BY indexname)
    INTO missing_indexes
    FROM (
        VALUES
            ('idx_change_risk_cab_packets_org_hash'),
            ('idx_change_risk_cab_packets_org_created'),
            ('idx_change_risk_cab_packets_status')
    ) AS expected(indexname)
    WHERE NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'change_risk_cab_packets'
          AND pg_indexes.indexname = expected.indexname
    );

    IF missing_indexes IS NOT NULL THEN
        RAISE EXCEPTION 'KAN-125 postcheck failed: change_risk_cab_packets missing indexes %', missing_indexes;
    END IF;
END $$;
