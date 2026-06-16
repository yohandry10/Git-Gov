DO $$
BEGIN
    IF to_regclass('public.executive_governance_snapshots') IS NULL THEN
        RAISE EXCEPTION 'executive_governance_snapshots table is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND indexname = 'idx_executive_governance_snapshots_org_created'
    ) THEN
        RAISE EXCEPTION 'idx_executive_governance_snapshots_org_created is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.executive_governance_snapshots'::regclass
          AND pg_get_constraintdef(oid) LIKE '%artifact_hash%'
    ) THEN
        RAISE EXCEPTION 'executive_governance_snapshots artifact hash check is missing';
    END IF;
END $$;
