-- GitGov Control Plane Schema v46 - Customer-Owned Framework Pack Import

BEGIN;

CREATE TABLE IF NOT EXISTS compliance_framework_packs (
    id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    framework_id TEXT NOT NULL,
    framework_name TEXT NOT NULL,
    framework_version TEXT NOT NULL,
    description TEXT NOT NULL,
    owner_type TEXT NOT NULL DEFAULT 'customer' CHECK (owner_type IN ('customer')),
    owner_name TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'customer_provided' CHECK (source = 'customer_provided'),
    review_status TEXT NOT NULL DEFAULT 'customer_review_required'
        CHECK (review_status IN ('customer_review_required', 'customer_reviewed', 'archived')),
    schema_version TEXT NOT NULL,
    pack_hash TEXT NOT NULL,
    raw_pack_redacted JSONB NOT NULL,
    control_count INTEGER NOT NULL CHECK (control_count BETWEEN 1 AND 50),
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
    regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
    gitgov_certifies BOOLEAN NOT NULL DEFAULT FALSE CHECK (gitgov_certifies = FALSE),
    requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
    official_regulatory_mapping BOOLEAN NOT NULL DEFAULT FALSE CHECK (official_regulatory_mapping = FALSE),
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    CHECK (id LIKE 'cfp_%'),
    CHECK (framework_id = lower(framework_id)),
    CHECK (framework_id LIKE 'customer_%'),
    CHECK (pack_hash ~ '^sha256:[a-f0-9]{64}$')
);

ALTER TABLE compliance_control_frameworks
    ADD COLUMN IF NOT EXISTS org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS owner_type TEXT NOT NULL DEFAULT 'gitgov',
    ADD COLUMN IF NOT EXISTS owner_name TEXT,
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'gitgov_owned',
    ADD COLUMN IF NOT EXISTS is_gitgov_owned BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS official_regulatory_mapping BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS framework_pack_id TEXT,
    ADD COLUMN IF NOT EXISTS pack_hash TEXT,
    ADD COLUMN IF NOT EXISTS created_by_user_id TEXT;

ALTER TABLE compliance_control_frameworks
    DROP CONSTRAINT IF EXISTS compliance_control_frameworks_owner_type_check,
    DROP CONSTRAINT IF EXISTS compliance_control_frameworks_source_check,
    DROP CONSTRAINT IF EXISTS compliance_control_frameworks_no_official_regulatory_mapping,
    DROP CONSTRAINT IF EXISTS compliance_control_frameworks_pack_hash_shape;

ALTER TABLE compliance_control_frameworks
    ADD CONSTRAINT compliance_control_frameworks_owner_type_check
        CHECK (owner_type IN ('gitgov', 'customer')),
    ADD CONSTRAINT compliance_control_frameworks_source_check
        CHECK (source IN ('gitgov_owned', 'customer_provided')),
    ADD CONSTRAINT compliance_control_frameworks_no_official_regulatory_mapping
        CHECK (official_regulatory_mapping = FALSE),
    ADD CONSTRAINT compliance_control_frameworks_pack_hash_shape
        CHECK (pack_hash IS NULL OR pack_hash ~ '^sha256:[a-f0-9]{64}$');

ALTER TABLE compliance_control_frameworks
    DROP CONSTRAINT IF EXISTS compliance_control_frameworks_framework_pack_fk;

ALTER TABLE compliance_control_frameworks
    ADD CONSTRAINT compliance_control_frameworks_framework_pack_fk
        FOREIGN KEY (framework_pack_id) REFERENCES compliance_framework_packs(id) ON DELETE SET NULL;

ALTER TABLE compliance_controls
    DROP CONSTRAINT IF EXISTS compliance_controls_control_id_check;

ALTER TABLE compliance_controls
    ADD CONSTRAINT compliance_controls_control_id_check
        CHECK (control_id ~ '^[A-Z0-9][A-Z0-9_.:-]{0,63}$');

UPDATE compliance_control_frameworks
SET owner_type = 'gitgov',
    owner_name = 'GitGov',
    source = 'gitgov_owned',
    is_gitgov_owned = TRUE,
    official_regulatory_mapping = FALSE
WHERE framework_id = 'gitgov_release_governance_baseline_v1';

CREATE INDEX IF NOT EXISTS idx_compliance_framework_packs_org_created
    ON compliance_framework_packs(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_packs_org_framework
    ON compliance_framework_packs(org_id, framework_id);

CREATE INDEX IF NOT EXISTS idx_compliance_control_frameworks_org_active
    ON compliance_control_frameworks(org_id, is_active, framework_id);

DO $$
BEGIN
    GRANT SELECT, INSERT, UPDATE ON compliance_framework_packs TO gitgov_server;
    GRANT SELECT, INSERT, UPDATE ON compliance_control_frameworks TO gitgov_server;
    GRANT SELECT, INSERT, UPDATE, DELETE ON compliance_controls TO gitgov_server;
EXCEPTION
    WHEN undefined_object THEN
        NULL;
END $$;

COMMIT;
