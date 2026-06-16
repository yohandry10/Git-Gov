-- KAN-127 postcheck: Change Risk CAB Decision Manifest.

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_cab_decision_manifests'
    ) THEN
        RAISE EXCEPTION 'change_risk_cab_decision_manifests table is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'change_risk_cab_decision_manifests'
          AND column_name = 'download_count'
          AND data_type = 'bigint'
    ) THEN
        RAISE EXCEPTION 'change_risk_cab_decision_manifests.download_count must be bigint';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_decision_manifests'::regclass
          AND conname = 'change_risk_cab_decision_manifests_manifest_json_check'
    ) THEN
        RAISE EXCEPTION 'change_risk_cab_decision_manifests no-claim manifest constraint is missing';
    END IF;
END $$;
