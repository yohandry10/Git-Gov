-- KAN-80: Persist the First Governed Repo Setup run per organization.
-- The setup is intentionally one active row per org. Re-running setup updates
-- the same run_id so readiness and evidence remain idempotent.

CREATE TABLE IF NOT EXISTS enterprise_first_governed_repo_setups (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    run_id UUID NOT NULL DEFAULT gen_random_uuid(),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'ready', 'blocked', 'completed')),
    goal TEXT NOT NULL DEFAULT 'govern_release'
        CHECK (goal IN (
            'govern_release',
            'generate_audit_evidence',
            'standardize_workflows',
            'assess_governance_gaps'
        )),
    repository_full_name TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'main',
    selected_providers JSONB NOT NULL DEFAULT '["github"]'::jsonb,
    selected_modules JSONB NOT NULL DEFAULT '["traceability","release-readiness","evidence-packets"]'::jsonb,
    policy_preset TEXT NOT NULL DEFAULT 'moderate'
        CHECK (policy_preset IN ('audit-only', 'moderate', 'strict')),
    baseline JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_enterprise_first_governed_repo_setups_repo
    ON enterprise_first_governed_repo_setups(repository_full_name);

CREATE INDEX IF NOT EXISTS idx_enterprise_first_governed_repo_setups_status
    ON enterprise_first_governed_repo_setups(status);

CREATE INDEX IF NOT EXISTS idx_enterprise_first_governed_repo_setups_gate_readiness
    ON enterprise_first_governed_repo_setups((baseline ->> 'gate_readiness'));

CREATE OR REPLACE FUNCTION update_enterprise_first_governed_repo_setups_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_enterprise_first_governed_repo_setups_updated_at
    ON enterprise_first_governed_repo_setups;

CREATE TRIGGER trg_enterprise_first_governed_repo_setups_updated_at
    BEFORE UPDATE ON enterprise_first_governed_repo_setups
    FOR EACH ROW
    EXECUTE FUNCTION update_enterprise_first_governed_repo_setups_updated_at();
