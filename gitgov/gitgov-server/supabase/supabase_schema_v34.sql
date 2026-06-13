-- KAN-82: Explicit platform principals for founder/superadmin access.
-- Platform principals are outside tenant scope and are not GitHub identities.

CREATE TABLE IF NOT EXISTS platform_principals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL UNIQUE,
    principal_type TEXT NOT NULL DEFAULT 'platform_founder',
    status TEXT NOT NULL DEFAULT 'active',
    display_name TEXT,
    email TEXT,
    auth_method TEXT NOT NULL DEFAULT 'api_key',
    external_subject TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    last_authenticated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'platform_principals_type_check'
          AND conrelid = 'platform_principals'::regclass
    ) THEN
        ALTER TABLE platform_principals
            ADD CONSTRAINT platform_principals_type_check
            CHECK (principal_type IN ('platform_founder', 'platform_operator', 'platform_auditor'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'platform_principals_status_check'
          AND conrelid = 'platform_principals'::regclass
    ) THEN
        ALTER TABLE platform_principals
            ADD CONSTRAINT platform_principals_status_check
            CHECK (status IN ('active', 'disabled', 'break_glass'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'platform_principals_auth_method_check'
          AND conrelid = 'platform_principals'::regclass
    ) THEN
        ALTER TABLE platform_principals
            ADD CONSTRAINT platform_principals_auth_method_check
            CHECK (auth_method IN ('api_key', 'sso', 'oidc', 'break_glass'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_platform_principals_type_status
    ON platform_principals(principal_type, status);

CREATE INDEX IF NOT EXISTS idx_platform_principals_updated_at
    ON platform_principals(updated_at DESC);

CREATE OR REPLACE FUNCTION update_platform_principals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_platform_principals_updated_at ON platform_principals;
CREATE TRIGGER trg_platform_principals_updated_at
    BEFORE UPDATE ON platform_principals
    FOR EACH ROW
    EXECUTE FUNCTION update_platform_principals_updated_at();

INSERT INTO platform_principals (
    client_id,
    principal_type,
    status,
    display_name,
    auth_method,
    metadata
)
VALUES (
    'bootstrap-admin',
    'platform_founder',
    'active',
    'GitGov Platform Founder',
    'api_key',
    '{"source":"migration_v34","tenant_scope":"platform"}'::jsonb
)
ON CONFLICT (client_id) DO UPDATE SET
    principal_type = 'platform_founder',
    status = 'active',
    display_name = COALESCE(platform_principals.display_name, EXCLUDED.display_name),
    auth_method = 'api_key',
    metadata = COALESCE(platform_principals.metadata, '{}'::jsonb) || EXCLUDED.metadata;
