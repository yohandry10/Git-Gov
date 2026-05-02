-- v25: Enterprise onboarding checklist tracking.
-- Stores operator tracking metadata for guided onboarding checklist stages.

CREATE TABLE IF NOT EXISTS enterprise_onboarding_checklist_tracking (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    tracking JSONB NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_onboarding_checklist_tracking_updated_at
    ON enterprise_onboarding_checklist_tracking(updated_at DESC);
