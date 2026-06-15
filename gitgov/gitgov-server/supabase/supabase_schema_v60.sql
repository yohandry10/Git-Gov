-- KAN-118: Saved manual Period Compliance Report profiles.

CREATE TABLE IF NOT EXISTS compliance_period_report_profiles (
    profile_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    created_by_user_id TEXT NOT NULL,
    updated_by_user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    period_type TEXT NOT NULL DEFAULT 'monthly'
        CHECK (period_type IN ('monthly', 'quarterly', 'annual', 'custom')),
    framework_id TEXT,
    framework_owner_type TEXT CHECK (
        framework_owner_type IS NULL
        OR framework_owner_type IN ('gitgov_managed', 'customer_provided')
    ),
    include_pdf BOOLEAN NOT NULL DEFAULT TRUE,
    include_manifest BOOLEAN NOT NULL DEFAULT TRUE,
    retention_days INTEGER NOT NULL DEFAULT 2555 CHECK (retention_days BETWEEN 30 AND 3650),
    filters JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    run_count INTEGER NOT NULL DEFAULT 0 CHECK (run_count >= 0),
    last_run_at TIMESTAMPTZ,
    last_period_report_id TEXT REFERENCES compliance_period_reports(period_report_id) ON DELETE SET NULL,
    last_pdf_export_id TEXT,
    last_manifest_id TEXT,
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (profile_id LIKE 'cprprof_%'),
    CHECK (char_length(name) BETWEEN 1 AND 120),
    CHECK (jsonb_typeof(filters) = 'object'),
    CHECK (
        filters::text !~* '(<script|</|<iframe|bearer |ghp_|glpat-|sk-)'
        AND char_length(filters::text) <= 4000
    ),
    CHECK (
        status = 'archived'
        OR archived_at IS NULL
    ),
    CHECK (
        status <> 'archived'
        OR archived_at IS NOT NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_cpr_profiles_org_status_updated
    ON compliance_period_report_profiles(org_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_cpr_profiles_org_framework
    ON compliance_period_report_profiles(org_id, framework_id, updated_at DESC);

COMMENT ON TABLE compliance_period_report_profiles IS
    'KAN-118 saved manual Period Compliance Report profiles. Profiles are reusable operator templates for manual run-now generation, not schedulers or certification claims.';

COMMENT ON COLUMN compliance_period_report_profiles.include_pdf IS
    'When true, manual profile run also materializes a Period Compliance Report PDF using existing KAN-114 logic.';

COMMENT ON COLUMN compliance_period_report_profiles.include_manifest IS
    'When true, manual profile run also materializes a Period Compliance Report provenance manifest using existing KAN-116 logic.';
