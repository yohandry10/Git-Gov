-- KAN-116 postcheck: Period Compliance Report provenance manifests.

DO $$
DECLARE
    manifest_table_count INTEGER;
    manifest_index_count INTEGER;
    action_constraint_count INTEGER;
    artifact_type_constraint_count INTEGER;
    payload_constraint_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO manifest_table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
      AND table_name = 'compliance_period_report_manifests';

    IF manifest_table_count <> 1 THEN
        RAISE EXCEPTION 'compliance_period_report_manifests table missing';
    END IF;

    SELECT COUNT(*)
    INTO manifest_index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename = 'compliance_period_report_manifests'
      AND indexname IN (
          'idx_compliance_period_report_manifests_report_created',
          'idx_compliance_period_report_manifests_hash'
      );

    IF manifest_index_count <> 2 THEN
        RAISE EXCEPTION 'compliance_period_report_manifests indexes missing';
    END IF;

    SELECT COUNT(*)
    INTO action_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_report_access_log'
      AND c.conname = 'compliance_period_report_access_log_action_check'
      AND pg_get_constraintdef(c.oid) LIKE '%manifest_created%'
      AND pg_get_constraintdef(c.oid) LIKE '%manifest_downloaded%';

    IF action_constraint_count <> 1 THEN
        RAISE EXCEPTION 'period report access log manifest actions missing';
    END IF;

    SELECT COUNT(*)
    INTO artifact_type_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_report_access_log'
      AND c.conname = 'compliance_period_report_access_log_artifact_type_check'
      AND pg_get_constraintdef(c.oid) LIKE '%manifest%';

    IF artifact_type_constraint_count <> 1 THEN
        RAISE EXCEPTION 'period report access log manifest artifact type missing';
    END IF;

    SELECT COUNT(*)
    INTO payload_constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    WHERE t.relname = 'compliance_period_report_manifests'
      AND c.conname = 'compliance_period_report_manifests_payload_check'
      AND pg_get_constraintdef(c.oid) LIKE '%agent_governance_required%'
      AND pg_get_constraintdef(c.oid) LIKE '%source_period_report_artifact_mutated%';

    IF payload_constraint_count <> 1 THEN
        RAISE EXCEPTION 'period report manifest payload no-claim constraint missing';
    END IF;
END $$;
