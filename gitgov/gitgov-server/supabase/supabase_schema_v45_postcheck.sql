-- KAN-101 postcheck for Control Mapping Review Package.

DO $$
DECLARE
    missing_columns INTEGER;
    violated_claims INTEGER;
BEGIN
    SELECT COUNT(*) INTO missing_columns
    FROM (
        VALUES
            ('compliance_review_packages', 'review_package_id'),
            ('compliance_review_packages', 'mapping_id'),
            ('compliance_review_packages', 'evidence_export_id'),
            ('compliance_review_packages', 'evidence_export_hash'),
            ('compliance_review_packages', 'mapping_hash'),
            ('compliance_review_packages', 'artifact_hash'),
            ('compliance_review_packages', 'payload_json_redacted'),
            ('compliance_review_packages', 'compliance_claim'),
            ('compliance_review_packages', 'regulatory_claim'),
            ('compliance_review_packages', 'requires_auditor_review'),
            ('compliance_review_packages', 'certification'),
            ('compliance_review_packages', 'downloaded_at')
    ) AS required(table_name, column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns c
        WHERE c.table_schema = 'public'
          AND c.table_name = required.table_name
          AND c.column_name = required.column_name
    );

    IF missing_columns <> 0 THEN
        RAISE EXCEPTION 'KAN-101 postcheck failed: % required columns missing', missing_columns;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_compliance_review_packages_mapping'
    ) THEN
        RAISE EXCEPTION 'KAN-101 postcheck failed: review package mapping index missing';
    END IF;

    SELECT COUNT(*) INTO violated_claims
    FROM compliance_review_packages
    WHERE compliance_claim IS DISTINCT FROM FALSE
       OR regulatory_claim IS DISTINCT FROM FALSE
       OR requires_auditor_review IS DISTINCT FROM TRUE
       OR certification IS DISTINCT FROM FALSE;

    IF violated_claims <> 0 THEN
        RAISE EXCEPTION 'KAN-101 postcheck failed: % review package rows violate no-claims constraints', violated_claims;
    END IF;

    RAISE NOTICE 'KAN-101 postcheck PASS: review package table, indexes, and no-claim constraints exist';
END $$;
