DO $$
BEGIN
    IF to_regclass('compliance_framework_review_report_manifests') IS NULL THEN
        RAISE EXCEPTION 'KAN-110 postcheck failed: provenance manifests table missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE tablename = 'compliance_framework_review_report_manifests'
          AND indexname = 'idx_cfr_report_manifests_report_created'
    ) THEN
        RAISE EXCEPTION 'KAN-110 postcheck failed: report manifest history index missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE tablename = 'compliance_framework_review_report_manifests'
          AND indexname = 'idx_cfr_report_manifests_hash'
    ) THEN
        RAISE EXCEPTION 'KAN-110 postcheck failed: manifest hash uniqueness index missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_framework_review_report_manifests_payload_check'
    ) THEN
        RAISE EXCEPTION 'KAN-110 postcheck failed: no-claim payload constraint missing';
    END IF;

    RAISE NOTICE 'KAN-110 postcheck passed: reviewed report provenance manifests are installed.';
END
$$;
