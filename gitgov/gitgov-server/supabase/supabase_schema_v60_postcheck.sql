-- KAN-118 postcheck: Saved manual Period Compliance Report profiles.

DO $$
DECLARE
    table_exists BOOLEAN;
    column_count INTEGER;
    constraint_count INTEGER;
    index_count INTEGER;
BEGIN
    SELECT to_regclass('public.compliance_period_report_profiles') IS NOT NULL
    INTO table_exists;

    IF NOT table_exists THEN
        RAISE EXCEPTION 'KAN-118 postcheck failed: compliance_period_report_profiles table missing';
    END IF;

    SELECT COUNT(*)
    INTO column_count
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'compliance_period_report_profiles'
      AND column_name IN (
          'profile_id',
          'org_id',
          'name',
          'period_type',
          'framework_id',
          'framework_owner_type',
          'include_pdf',
          'include_manifest',
          'retention_days',
          'filters',
          'status',
          'run_count',
          'last_run_at',
          'last_period_report_id',
          'last_pdf_export_id',
          'last_manifest_id',
          'archived_at'
      );

    IF column_count <> 17 THEN
        RAISE EXCEPTION 'KAN-118 postcheck failed: expected profile columns missing';
    END IF;

    SELECT COUNT(*)
    INTO constraint_count
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    JOIN pg_namespace n ON n.oid = t.relnamespace
    WHERE n.nspname = 'public'
      AND t.relname = 'compliance_period_report_profiles'
      AND pg_get_constraintdef(c.oid) LIKE ANY (ARRAY[
          '%period_type IN%',
          '%status IN%',
          '%retention_days%',
          '%profile_id%',
          '%filters%'
      ]);

    IF constraint_count < 5 THEN
        RAISE EXCEPTION 'KAN-118 postcheck failed: profile constraints missing';
    END IF;

    SELECT COUNT(*)
    INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename = 'compliance_period_report_profiles'
      AND indexname IN (
          'idx_cpr_profiles_org_status_updated',
          'idx_cpr_profiles_org_framework'
      );

    IF index_count <> 2 THEN
        RAISE EXCEPTION 'KAN-118 postcheck failed: profile indexes missing';
    END IF;

    RAISE NOTICE 'KAN-118 postcheck passed: saved manual Period Compliance Report profiles are installed.';
END $$;
