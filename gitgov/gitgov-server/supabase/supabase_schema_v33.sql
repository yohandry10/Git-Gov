-- KAN-81: Platform Superadmin tenant catalog foundation.
-- GitGov Internal remains a normal tenant; platform principals live outside
-- tenant scope (api_keys.org_id IS NULL) and administer this catalog.

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS tenant_type TEXT NOT NULL DEFAULT 'customer';

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS lifecycle_status TEXT NOT NULL DEFAULT 'active';

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS provisioning_source TEXT NOT NULL DEFAULT 'legacy';

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS provisioned_by TEXT;

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS platform_metadata JSONB NOT NULL DEFAULT '{}';

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS suspended_at TIMESTAMPTZ;

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;

ALTER TABLE orgs
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'orgs_tenant_type_check'
          AND conrelid = 'orgs'::regclass
    ) THEN
        ALTER TABLE orgs
            ADD CONSTRAINT orgs_tenant_type_check
            CHECK (tenant_type IN ('customer', 'internal', 'sandbox'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'orgs_lifecycle_status_check'
          AND conrelid = 'orgs'::regclass
    ) THEN
        ALTER TABLE orgs
            ADD CONSTRAINT orgs_lifecycle_status_check
            CHECK (lifecycle_status IN ('trial', 'active', 'suspended', 'archived', 'deleted'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'orgs_provisioning_source_check'
          AND conrelid = 'orgs'::regclass
    ) THEN
        ALTER TABLE orgs
            ADD CONSTRAINT orgs_provisioning_source_check
            CHECK (provisioning_source IN ('legacy', 'github_webhook', 'platform_founder', 'migration'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_orgs_tenant_type ON orgs(tenant_type);
CREATE INDEX IF NOT EXISTS idx_orgs_lifecycle_status ON orgs(lifecycle_status);
CREATE INDEX IF NOT EXISTS idx_orgs_provisioning_source ON orgs(provisioning_source);
