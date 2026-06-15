-- KAN-108: Tenant-scoped Auditor role for compliance evidence review.

DO $$
BEGIN
    IF to_regclass('api_keys') IS NOT NULL THEN
        ALTER TABLE api_keys DROP CONSTRAINT IF EXISTS api_keys_role_check;
        ALTER TABLE api_keys
            ADD CONSTRAINT api_keys_role_check
            CHECK (role IN ('Admin', 'Auditor', 'Architect', 'Developer', 'PM'))
            NOT VALID;
    END IF;

    IF to_regclass('org_users') IS NOT NULL THEN
        ALTER TABLE org_users DROP CONSTRAINT IF EXISTS org_users_role_check;
        ALTER TABLE org_users
            ADD CONSTRAINT org_users_role_check
            CHECK (role IN ('Admin', 'Auditor', 'Architect', 'Developer', 'PM'))
            NOT VALID;
    END IF;

    IF to_regclass('org_invitations') IS NOT NULL THEN
        ALTER TABLE org_invitations DROP CONSTRAINT IF EXISTS org_invitations_role_check;
        ALTER TABLE org_invitations
            ADD CONSTRAINT org_invitations_role_check
            CHECK (role IN ('Admin', 'Auditor', 'Architect', 'Developer', 'PM'))
            NOT VALID;
    END IF;
END
$$;

COMMENT ON CONSTRAINT api_keys_role_check ON api_keys IS
    'KAN-108 allows tenant-scoped Auditor API keys for read/review-only compliance evidence workflows.';
