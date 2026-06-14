-- KAN-105 postcheck for Framework-specific Review Report Export.

DO $$
DECLARE
    missing_columns INTEGER;
    violated_claims INTEGER;
BEGIN
    SELECT COUNT(*) INTO missing_columns
    FROM (
        VALUES
            ('report_id'),
            ('org_id'),
            ('mapping_id'),
            ('review_package_id'),
            ('evidence_export_hash'),
            ('mapping_hash'),
            ('review_package_hash'),
            ('framework_owner_type'),
            ('framework_review_status'),
            ('pack_hash'),
            ('artifact_hash'),
            ('payload_json_redacted'),
            ('compliance_claim'),
            ('regulatory_claim'),
            ('requires_auditor_review'),
            ('certification')
    ) AS required(column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_name = 'compliance_framework_review_reports'
          AND c.column_name = required.column_name
    );

    IF missing_columns > 0 THEN
        RAISE EXCEPTION 'KAN-105 postcheck failed: % required columns missing', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE indexname = 'idx_compliance_framework_review_reports_mapping'
    ) THEN
        RAISE EXCEPTION 'KAN-105 postcheck failed: framework review report mapping index missing';
    END IF;

    SELECT COUNT(*) INTO violated_claims
    FROM compliance_framework_review_reports
    WHERE compliance_claim IS DISTINCT FROM FALSE
       OR regulatory_claim IS DISTINCT FROM FALSE
       OR requires_auditor_review IS DISTINCT FROM TRUE
       OR certification IS DISTINCT FROM FALSE;

    IF violated_claims > 0 THEN
        RAISE EXCEPTION 'KAN-105 postcheck failed: % report rows violate no-claims constraints', violated_claims;
    END IF;

    RAISE NOTICE 'KAN-105 postcheck PASS: framework review report table, indexes, and no-claim constraints exist';
END $$;
