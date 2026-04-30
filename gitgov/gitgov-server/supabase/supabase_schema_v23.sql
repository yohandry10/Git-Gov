-- v23: Enterprise adoption profile persistence.
-- Stores one validated, secret-free onboarding profile per organization.

CREATE TABLE IF NOT EXISTS enterprise_adoption_profiles (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    profile JSONB NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_adoption_profiles_updated_at
    ON enterprise_adoption_profiles(updated_at DESC);
