-- KAN-100 postcheck for Evidence-to-Control Mapping.

DO $$
DECLARE
    missing_columns INTEGER;
    baseline_controls INTEGER;
BEGIN
    SELECT COUNT(*) INTO missing_columns
    FROM (
        VALUES
            ('compliance_control_frameworks', 'framework_id'),
            ('compliance_controls', 'control_id'),
            ('compliance_evidence_mappings', 'mapping_id'),
            ('compliance_evidence_mappings', 'evidence_export_hash'),
            ('compliance_evidence_mapping_items', 'mapping_id'),
            ('compliance_evidence_mapping_items', 'status')
    ) AS required(table_name, column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_schema = 'public'
          AND c.table_name = required.table_name
          AND c.column_name = required.column_name
    );

    SELECT COUNT(*) INTO baseline_controls
    FROM compliance_controls
    WHERE framework_id = 'gitgov_release_governance_baseline_v1';

    IF missing_columns <> 0 THEN
        RAISE EXCEPTION 'KAN-100 postcheck failed: % required columns missing', missing_columns;
    END IF;

    IF baseline_controls <> 10 THEN
        RAISE EXCEPTION 'KAN-100 postcheck failed: expected 10 baseline controls, found %', baseline_controls;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_evidence_mappings_export'
    ) THEN
        RAISE EXCEPTION 'KAN-100 postcheck failed: mapping export index missing';
    END IF;

    RAISE NOTICE 'KAN-100 postcheck PASS: framework, controls, mapping tables, and indexes exist';
END $$;
