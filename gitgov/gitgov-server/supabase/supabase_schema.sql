-- GitGov Control Plane Schema for Supabase
-- Enterprise-grade audit system with append-only guarantees
-- Run this in the Supabase SQL Editor

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- ORGANIZATIONS
-- ============================================================================
CREATE TABLE IF NOT EXISTS orgs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    github_id BIGINT UNIQUE,
    login TEXT UNIQUE NOT NULL,
    name TEXT,
    avatar_url TEXT,
    tenant_type TEXT NOT NULL DEFAULT 'customer'
        CHECK (tenant_type IN ('customer', 'internal', 'sandbox')),
    lifecycle_status TEXT NOT NULL DEFAULT 'active'
        CHECK (lifecycle_status IN ('trial', 'active', 'suspended', 'archived', 'deleted')),
    provisioning_source TEXT NOT NULL DEFAULT 'legacy'
        CHECK (provisioning_source IN ('legacy', 'github_webhook', 'platform_founder', 'migration')),
    provisioned_by TEXT,
    platform_metadata JSONB NOT NULL DEFAULT '{}',
    suspended_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orgs_tenant_type ON orgs(tenant_type);
CREATE INDEX IF NOT EXISTS idx_orgs_lifecycle_status ON orgs(lifecycle_status);
CREATE INDEX IF NOT EXISTS idx_orgs_provisioning_source ON orgs(provisioning_source);

CREATE TABLE IF NOT EXISTS enterprise_adoption_profiles (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    profile JSONB NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_adoption_profiles_updated_at
    ON enterprise_adoption_profiles(updated_at DESC);

CREATE TABLE IF NOT EXISTS enterprise_onboarding_checklist_tracking (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    tracking JSONB NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_onboarding_checklist_tracking_updated_at
    ON enterprise_onboarding_checklist_tracking(updated_at DESC);

CREATE TABLE IF NOT EXISTS enterprise_release_approvals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT,
    target_sha TEXT,
    environment TEXT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'accepted-risk')),
    approver TEXT NOT NULL,
    ticket_id TEXT,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT,
    evidence_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    risk_severity TEXT NOT NULL DEFAULT 'none' CHECK (risk_severity IN ('none', 'low', 'medium', 'high', 'critical')),
    risk_acceptance_reason TEXT,
    expires_at TIMESTAMPTZ,
    approval_hash TEXT NOT NULL UNIQUE,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_org_created
    ON enterprise_release_approvals(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_repo_release
    ON enterprise_release_approvals(org_id, repository_full_name, release_id);

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_expiry
    ON enterprise_release_approvals(expires_at)
    WHERE expires_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_binding
    ON enterprise_release_approvals(
        org_id,
        repository_full_name,
        release_id,
        environment,
        branch,
        target_sha,
        evidence_packet_hash
    );

CREATE TABLE IF NOT EXISTS release_evidence_packets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    ticket_id TEXT NOT NULL,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    target_sha TEXT NOT NULL,
    environment TEXT NOT NULL,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT NOT NULL,
    packet JSONB NOT NULL,
    generated_by TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (org_id, evidence_packet_hash)
);

CREATE INDEX IF NOT EXISTS idx_release_evidence_packets_binding
    ON release_evidence_packets(
        org_id,
        repository_full_name,
        release_id,
        environment,
        branch,
        target_sha,
        evidence_packet_hash
    );

CREATE TABLE IF NOT EXISTS deployment_gate_authorizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    authorization_id TEXT NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    release_id TEXT NOT NULL,
    repository_full_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    target_sha TEXT NOT NULL,
    environment TEXT NOT NULL,
    deployer TEXT NOT NULL,
    ticket_id TEXT,
    evidence_packet_hash TEXT NOT NULL,
    evidence_packet_uri TEXT,
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'advisory', 'blocked')),
    approved BOOLEAN NOT NULL,
    blocking BOOLEAN NOT NULL,
    would_block BOOLEAN NOT NULL,
    reason TEXT NOT NULL,
    blocked_by JSONB NOT NULL DEFAULT '[]'::jsonb,
    warnings JSONB NOT NULL DEFAULT '[]'::jsonb,
    policy_checksum TEXT NOT NULL,
    break_glass_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    evaluation JSONB NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    requested_by TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_org_created
    ON deployment_gate_authorizations(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_scope
    ON deployment_gate_authorizations(
        org_id,
        repository_full_name,
        branch,
        environment,
        created_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_release
    ON deployment_gate_authorizations(org_id, release_id, environment, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_target
    ON deployment_gate_authorizations(org_id, target_sha, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_governance_settings (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mode TEXT NOT NULL DEFAULT 'manual_only'
        CHECK (mode IN ('manual_only', 'opt_in_enabled')),
    payload_mode TEXT NOT NULL DEFAULT 'minimized'
        CHECK (payload_mode IN ('minimized')),
    reason TEXT,
    updated_by TEXT NOT NULL DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (enabled = FALSE AND mode = 'manual_only')
        OR
        (enabled = TRUE AND mode = 'opt_in_enabled')
    )
);

CREATE INDEX IF NOT EXISTS idx_agent_governance_settings_enabled
    ON agent_governance_settings(enabled, updated_at DESC);

CREATE TABLE IF NOT EXISTS agent_governance_agent_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_id TEXT NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    token_prefix TEXT NOT NULL DEFAULT 'ggag_',
    token_last4 TEXT NOT NULL,
    token_preview TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    environment TEXT,
    scopes JSONB NOT NULL DEFAULT '["agent_governance:evaluate"]'::jsonb,
    allowed_actions JSONB NOT NULL DEFAULT '["commit","push","open_pr","merge_pr","deploy"]'::jsonb,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_by TEXT NOT NULL,
    revoked_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (key_id LIKE 'agk_%'),
    CHECK (token_prefix = 'ggag_'),
    CHECK (jsonb_typeof(scopes) = 'array'),
    CHECK (jsonb_typeof(allowed_actions) = 'array')
);

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_org_created
    ON agent_governance_agent_keys(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_active
    ON agent_governance_agent_keys(org_id, revoked_at, expires_at);

CREATE TABLE IF NOT EXISTS agent_governance_evaluations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    evaluation_id TEXT NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    agent_type TEXT NOT NULL DEFAULT 'unknown',
    actor TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('commit', 'push', 'open_pr', 'merge_pr', 'change_policy', 'deploy')),
    repository_full_name TEXT NOT NULL,
    branch TEXT,
    target_sha TEXT,
    environment TEXT,
    ticket_id TEXT,
    operation_id TEXT,
    decision TEXT NOT NULL CHECK (decision IN ('allowed', 'requires_approval', 'blocked')),
    allowed BOOLEAN NOT NULL,
    requires_approval BOOLEAN NOT NULL,
    reason TEXT NOT NULL,
    reasons JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    policy_id TEXT NOT NULL,
    policy_checksum TEXT NOT NULL,
    evaluation JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    principal_type TEXT,
    agent_key_id TEXT,
    agent_display_name TEXT,
    attribution_id TEXT,
    correlation_id TEXT,
    parent_correlation_id TEXT,
    session_id TEXT,
    tool_name TEXT,
    tool_version TEXT,
    agent_name TEXT,
    external_run_id TEXT,
    consumer_type TEXT DEFAULT 'agent_governance',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (evaluation_id LIKE 'agv_%'),
    CHECK (attribution_id IS NULL OR attribution_id LIKE 'attr_%'),
    CHECK (correlation_id IS NULL OR length(correlation_id) BETWEEN 1 AND 128),
    CHECK (consumer_type IS NULL OR consumer_type IN ('agent_governance', 'agent_dry_run')),
    CHECK (
        (decision = 'allowed' AND allowed = TRUE AND requires_approval = FALSE)
        OR
        (decision = 'requires_approval' AND allowed = FALSE AND requires_approval = TRUE)
        OR
        (decision = 'blocked' AND allowed = FALSE AND requires_approval = FALSE)
    )
);

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_org_created
    ON agent_governance_evaluations(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_scope
    ON agent_governance_evaluations(
        org_id,
        repository_full_name,
        action,
        decision,
        created_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_agent
    ON agent_governance_evaluations(org_id, agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_correlation
    ON agent_governance_evaluations(org_id, correlation_id, created_at DESC)
    WHERE correlation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_session
    ON agent_governance_evaluations(org_id, session_id, created_at DESC)
    WHERE session_id IS NOT NULL;

-- ============================================================================
-- REPOSITORIES
-- ============================================================================
CREATE TABLE IF NOT EXISTS repos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    github_id BIGINT UNIQUE,
    full_name TEXT UNIQUE NOT NULL,  -- e.g., "org/repo"
    name TEXT NOT NULL,
    private BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_repos_org ON repos(org_id);
CREATE INDEX IF NOT EXISTS idx_repos_full_name ON repos(full_name);

-- ============================================================================
-- MEMBERS (Organization members with roles)
-- ============================================================================
CREATE TABLE IF NOT EXISTS members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    github_login TEXT NOT NULL,
    github_id BIGINT,
    role TEXT NOT NULL DEFAULT 'Developer',  -- Admin, Architect, Developer, PM
    groups JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(org_id, github_login)
);

CREATE INDEX IF NOT EXISTS idx_members_org ON members(org_id);
CREATE INDEX IF NOT EXISTS idx_members_login ON members(github_login);

-- ============================================================================
-- GITHUB EVENTS (Source of Truth - from webhooks)
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    delivery_id TEXT UNIQUE NOT NULL,  -- X-GitHub-Delivery header (idempotency)
    event_type TEXT NOT NULL,          -- push, create, delete, etc.
    actor_login TEXT,
    actor_id BIGINT,
    ref_name TEXT,                     -- branch or tag name
    ref_type TEXT,                     -- branch, tag
    before_sha TEXT,
    after_sha TEXT,
    commit_shas JSONB DEFAULT '[]',
    commits_count INTEGER DEFAULT 0,
    payload JSONB NOT NULL,            -- Raw webhook payload
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

-- Indexes for github_events
CREATE INDEX IF NOT EXISTS idx_github_events_delivery ON github_events(delivery_id);
CREATE INDEX IF NOT EXISTS idx_github_events_org_repo ON github_events(org_id, repo_id);
CREATE INDEX IF NOT EXISTS idx_github_events_created ON github_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_github_events_type ON github_events(event_type);
CREATE INDEX IF NOT EXISTS idx_github_events_actor ON github_events(actor_login);

-- ============================================================================
-- CLIENT EVENTS (Telemetry from Desktop App)
-- ============================================================================
CREATE TABLE IF NOT EXISTS client_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    event_uuid TEXT UNIQUE NOT NULL,   -- Client-generated UUID (idempotency)
    event_type TEXT NOT NULL,          -- attempt_push, successful_push, blocked_push, push_failed, governance_blocked_push, cli_command, commit, etc.
    user_login TEXT NOT NULL,
    user_name TEXT,
    branch TEXT,
    commit_sha TEXT,
    files JSONB DEFAULT '[]',
    status TEXT NOT NULL,              -- success, blocked, failed
    reason TEXT,
    metadata JSONB DEFAULT '{}',       -- Additional context
    client_version TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    synced_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for client_events
CREATE INDEX IF NOT EXISTS idx_client_events_uuid ON client_events(event_uuid);
CREATE INDEX IF NOT EXISTS idx_client_events_org_repo ON client_events(org_id, repo_id);
CREATE INDEX IF NOT EXISTS idx_client_events_created ON client_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_client_events_type ON client_events(event_type);
CREATE INDEX IF NOT EXISTS idx_client_events_user ON client_events(user_login);
CREATE INDEX IF NOT EXISTS idx_client_events_status ON client_events(status);

-- ============================================================================
-- VIOLATIONS (Policy violations - append-only with confidence scoring)
-- ============================================================================
CREATE TABLE IF NOT EXISTS violations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    github_event_id UUID REFERENCES github_events(id),
    client_event_id UUID REFERENCES client_events(id),
    
    violation_type TEXT NOT NULL,      -- unauthorized_push, branch_protection, naming_violation, path_permission_violation
    severity TEXT DEFAULT 'warning',   -- info, warning, critical
    
    -- Confidence scoring (non-binary detection)
    confidence_level TEXT DEFAULT 'pending',  -- 'high', 'low', 'pending'
    reason TEXT,                        -- 'direct_push_no_client_event', 'missing_telemetry_outbox_pending', etc.
    
    user_login TEXT,
    branch TEXT,
    commit_sha TEXT,
    details JSONB DEFAULT '{}',
    
    -- Correlation fields
    correlated_github_event_id UUID REFERENCES github_events(id),
    correlated_client_event_id UUID REFERENCES client_events(id),
    
    -- Review/confirmation fields
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_violations_org_repo ON violations(org_id, repo_id);
CREATE INDEX IF NOT EXISTS idx_violations_created ON violations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_violations_type ON violations(violation_type);
CREATE INDEX IF NOT EXISTS idx_violations_resolved ON violations(resolved);
CREATE INDEX IF NOT EXISTS idx_violations_confidence ON violations(confidence_level);
CREATE INDEX IF NOT EXISTS idx_violations_reason ON violations(reason);

-- ============================================================================
-- VIOLATION DECISIONS (Resolution/status transitions - append-only)
-- ============================================================================
CREATE TABLE IF NOT EXISTS violation_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    violation_id UUID NOT NULL REFERENCES violations(id) ON DELETE CASCADE,
    decision_type TEXT NOT NULL CHECK (decision_type IN (
        'acknowledged',
        'false_positive',
        'resolved',
        'escalated',
        'dismissed',
        'wont_fix'
    )),
    decided_by TEXT NOT NULL,
    decided_at TIMESTAMPTZ DEFAULT NOW(),
    notes TEXT,
    evidence JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT violation_decisions_once_per_type UNIQUE (violation_id, decision_type)
);

CREATE INDEX IF NOT EXISTS idx_violation_decisions_violation_id
    ON violation_decisions(violation_id);
CREATE INDEX IF NOT EXISTS idx_violation_decisions_decided_by
    ON violation_decisions(decided_by);
CREATE INDEX IF NOT EXISTS idx_violation_decisions_decided_at
    ON violation_decisions(decided_at DESC);

-- ============================================================================
-- POLICIES (Configurable per repo - can be updated)
-- ============================================================================
CREATE TABLE IF NOT EXISTS policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE UNIQUE,
    config JSONB NOT NULL,             -- gitgov.toml content as JSON
    checksum TEXT NOT NULL,
    source_metadata JSONB NOT NULL DEFAULT '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb,
    override_actor TEXT,               -- Client ID of user who made the last change
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policies_repo ON policies(repo_id);
CREATE INDEX IF NOT EXISTS idx_policies_source_mode
    ON policies ((source_metadata ->> 'source_mode'));

-- ============================================================================
-- API KEYS (For desktop client authentication)
-- ============================================================================
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_hash TEXT UNIQUE NOT NULL,
    client_id TEXT NOT NULL,
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'Developer',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    CONSTRAINT api_keys_role_check CHECK (role IN ('Admin', 'Architect', 'Developer', 'PM'))
);

CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_client ON api_keys(client_id);

-- ============================================================================
-- PLATFORM PRINCIPALS (Superadmin identities outside tenant scope)
-- ============================================================================
CREATE TABLE IF NOT EXISTS platform_principals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT platform_principals_type_check CHECK (principal_type IN ('platform_founder', 'platform_operator', 'platform_auditor')),
    CONSTRAINT platform_principals_status_check CHECK (status IN ('active', 'disabled', 'break_glass')),
    CONSTRAINT platform_principals_auth_method_check CHECK (auth_method IN ('api_key', 'sso', 'oidc', 'break_glass'))
);

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
    '{"source":"schema_bootstrap","tenant_scope":"platform"}'::jsonb
)
ON CONFLICT (client_id) DO UPDATE SET
    principal_type = 'platform_founder',
    status = 'active',
    display_name = COALESCE(platform_principals.display_name, EXCLUDED.display_name),
    auth_method = 'api_key',
    metadata = COALESCE(platform_principals.metadata, '{}'::jsonb) || EXCLUDED.metadata;

-- ============================================================================
-- WEBHOOK EVENTS (Raw incoming webhooks for debugging)
-- ============================================================================
CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    delivery_id TEXT,
    event_type TEXT NOT NULL,
    signature TEXT,
    payload JSONB NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_webhook_events_delivery ON webhook_events(delivery_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_created ON webhook_events(created_at DESC);

-- ============================================================================
-- APPEND-ONLY TRIGGER FUNCTION (Prevents UPDATE and DELETE)
-- ============================================================================
CREATE OR REPLACE FUNCTION prevent_update_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Table % is append-only. UPDATE and DELETE operations are not allowed.', TG_TABLE_NAME;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Apply append-only triggers to audit tables
DROP TRIGGER IF EXISTS github_events_append_only ON github_events;
CREATE TRIGGER github_events_append_only
    BEFORE UPDATE OR DELETE ON github_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

DROP TRIGGER IF EXISTS client_events_append_only ON client_events;
CREATE TRIGGER client_events_append_only
    BEFORE UPDATE OR DELETE ON client_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

DROP TRIGGER IF EXISTS violations_append_only ON violations;
DROP TRIGGER IF EXISTS violations_limited_update ON violations;
DROP TRIGGER IF EXISTS violations_no_delete ON violations;
DROP FUNCTION IF EXISTS violations_limited_update();
CREATE TRIGGER violations_append_only
    BEFORE UPDATE OR DELETE ON violations
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

DROP TRIGGER IF EXISTS violation_decisions_append_only ON violation_decisions;
CREATE TRIGGER violation_decisions_append_only
    BEFORE UPDATE OR DELETE ON violation_decisions
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- Helper to register decisions without mutating violations table.
CREATE OR REPLACE FUNCTION add_violation_decision(
    p_violation_id UUID,
    p_decision_type TEXT,
    p_decided_by TEXT,
    p_notes TEXT DEFAULT NULL,
    p_evidence JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    decision_id UUID;
BEGIN
    IF p_decision_type NOT IN ('acknowledged', 'false_positive', 'resolved', 'escalated', 'dismissed', 'wont_fix') THEN
        RAISE EXCEPTION 'Invalid decision_type: %. Must be one of: acknowledged, false_positive, resolved, escalated, dismissed, wont_fix', p_decision_type;
    END IF;

    INSERT INTO violation_decisions (
        violation_id, decision_type, decided_by, notes, evidence
    ) VALUES (
        p_violation_id, p_decision_type, p_decided_by, p_notes, COALESCE(p_evidence, '{}'::jsonb)
    )
    ON CONFLICT (violation_id, decision_type) DO NOTHING
    RETURNING id INTO decision_id;

    IF decision_id IS NULL THEN
        SELECT id INTO decision_id
        FROM violation_decisions
        WHERE violation_id = p_violation_id
          AND decision_type = p_decision_type
        LIMIT 1;
    END IF;

    RETURN decision_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Avoid 42P16 when an older view definition has a different column layout.
DROP VIEW IF EXISTS violation_current_status;
CREATE OR REPLACE VIEW violation_current_status
WITH (security_invoker = true) AS
SELECT
    v.id AS violation_id,
    v.org_id,
    v.repo_id,
    v.severity,
    v.violation_type,
    v.user_login,
    v.branch,
    v.commit_sha,
    v.created_at AS violation_created_at,
    vd.decision_type AS current_status,
    vd.decided_by,
    vd.decided_at,
    vd.notes AS decision_notes,
    CASE
        WHEN vd.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix') THEN true
        ELSE false
    END AS is_closed
FROM violations v
LEFT JOIN LATERAL (
    SELECT decision_type, decided_by, decided_at, notes
    FROM violation_decisions
    WHERE violation_id = v.id
    ORDER BY decided_at DESC, created_at DESC
    LIMIT 1
) vd ON true;

-- ============================================================================
-- ROW LEVEL SECURITY (RLS)
-- ============================================================================

-- Enable RLS on all tables
ALTER TABLE orgs ENABLE ROW LEVEL SECURITY;
ALTER TABLE repos ENABLE ROW LEVEL SECURITY;
ALTER TABLE members ENABLE ROW LEVEL SECURITY;
ALTER TABLE github_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE client_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE violations ENABLE ROW LEVEL SECURITY;
ALTER TABLE violation_decisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_principals ENABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_events ENABLE ROW LEVEL SECURITY;

-- Service role bypasses RLS (for server-side operations)
-- This is handled by Supabase automatically when using service_role key

-- Policies for authenticated users (using Supabase auth)
-- Admins can see everything
CREATE POLICY "Admins can see all orgs" ON orgs
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members 
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

CREATE POLICY "Admins can see all repos" ON repos
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members 
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

CREATE POLICY "Admins can see all github_events" ON github_events
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members 
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

CREATE POLICY "Admins can see all client_events" ON client_events
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members 
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

CREATE POLICY "Admins can see all violations" ON violations
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members 
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

CREATE POLICY "Admins can see all violation_decisions" ON violation_decisions
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

-- Users can see their own client events
CREATE POLICY "Users see own client_events" ON client_events
    FOR SELECT USING (user_login = auth.jwt() ->> 'user_name');

-- All authenticated users can read policies
CREATE POLICY "Authenticated read policies" ON policies
    FOR SELECT USING (auth.role() = 'authenticated');

-- ============================================================================
-- UTILITY FUNCTIONS
-- ============================================================================

-- Get or create org by GitHub login
CREATE OR REPLACE FUNCTION upsert_org(
    p_github_id BIGINT,
    p_login TEXT,
    p_name TEXT DEFAULT NULL,
    p_avatar_url TEXT DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    v_org_id UUID;
BEGIN
    INSERT INTO orgs (github_id, login, name, avatar_url)
    VALUES (p_github_id, p_login, p_name, p_avatar_url)
    ON CONFLICT (github_id) DO UPDATE SET
        name = COALESCE(p_name, orgs.name),
        avatar_url = COALESCE(p_avatar_url, orgs.avatar_url),
        updated_at = NOW()
    RETURNING id INTO v_org_id;
    
    RETURN v_org_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get or create repo by full_name
CREATE OR REPLACE FUNCTION upsert_repo(
    p_org_id UUID,
    p_github_id BIGINT,
    p_full_name TEXT,
    p_name TEXT,
    p_private BOOLEAN DEFAULT FALSE
) RETURNS UUID AS $$
DECLARE
    v_repo_id UUID;
BEGIN
    INSERT INTO repos (org_id, github_id, full_name, name, private)
    VALUES (p_org_id, p_github_id, p_full_name, p_name, p_private)
    ON CONFLICT (full_name) DO UPDATE SET
        name = p_name,
        private = p_private,
        updated_at = NOW()
    RETURNING id INTO v_repo_id;
    
    RETURN v_repo_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get or create member
CREATE OR REPLACE FUNCTION upsert_member(
    p_org_id UUID,
    p_github_login TEXT,
    p_github_id BIGINT DEFAULT NULL,
    p_role TEXT DEFAULT 'Developer'
) RETURNS UUID AS $$
DECLARE
    v_member_id UUID;
BEGIN
    INSERT INTO members (org_id, github_login, github_id, role)
    VALUES (p_org_id, p_github_login, p_github_id, p_role)
    ON CONFLICT (org_id, github_login) DO UPDATE SET
        github_id = COALESCE(p_github_id, members.github_id),
        role = p_role,
        updated_at = NOW()
    RETURNING id INTO v_member_id;
    
    RETURN v_member_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get audit stats (combines github_events and client_events)
CREATE OR REPLACE FUNCTION get_audit_stats(p_org_id UUID DEFAULT NULL)
RETURNS JSON AS $$
DECLARE
    result JSON;
BEGIN
    SELECT json_build_object(
        'github_events', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id)),
                'today', (SELECT COUNT(*) FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= DATE_TRUNC('day', NOW())),
                'pushes_today', (SELECT COUNT(*) FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND event_type = 'push' AND created_at >= DATE_TRUNC('day', NOW())),
                'by_type', COALESCE((SELECT json_object_agg(event_type, cnt) FROM (SELECT event_type, COUNT(*) as cnt FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id) GROUP BY event_type) t), '{}'::json)
            )
        ),
        'client_events', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id)),
                'today', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= DATE_TRUNC('day', NOW())),
                'blocked_today', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND status = 'blocked' AND created_at >= DATE_TRUNC('day', NOW())),
                'by_type', COALESCE((SELECT json_object_agg(event_type, cnt) FROM (SELECT event_type, COUNT(*) as cnt FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) GROUP BY event_type) t), '{}'::json),
                'by_status', COALESCE((SELECT json_object_agg(status, cnt) FROM (SELECT status, COUNT(*) as cnt FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) GROUP BY status) t), '{}'::json)
            )
        ),
        'violations', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM violations WHERE (p_org_id IS NULL OR org_id = p_org_id)),
                'unresolved', (
                    SELECT COUNT(*)
                    FROM violations v
                    LEFT JOIN LATERAL (
                        SELECT decision_type
                        FROM violation_decisions vd
                        WHERE vd.violation_id = v.id
                        ORDER BY vd.decided_at DESC, vd.created_at DESC
                        LIMIT 1
                    ) latest_decision ON true
                    WHERE (p_org_id IS NULL OR v.org_id = p_org_id)
                      AND NOT (
                          COALESCE(v.resolved, FALSE)
                          OR COALESCE(
                              latest_decision.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix'),
                              FALSE
                          )
                      )
                ),
                'critical', (
                    SELECT COUNT(*)
                    FROM violations v
                    LEFT JOIN LATERAL (
                        SELECT decision_type
                        FROM violation_decisions vd
                        WHERE vd.violation_id = v.id
                        ORDER BY vd.decided_at DESC, vd.created_at DESC
                        LIMIT 1
                    ) latest_decision ON true
                    WHERE (p_org_id IS NULL OR v.org_id = p_org_id)
                      AND v.severity = 'critical'
                      AND NOT (
                          COALESCE(v.resolved, FALSE)
                          OR COALESCE(
                              latest_decision.decision_type IN ('resolved', 'false_positive', 'dismissed', 'wont_fix'),
                              FALSE
                          )
                      )
                )
            )
        ),
        'active_devs_week', (SELECT COUNT(DISTINCT user_login) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= NOW() - INTERVAL '7 days'),
        'active_repos', (SELECT COUNT(DISTINCT repo_id) FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= NOW() - INTERVAL '7 days')
    ) INTO result;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get combined event log (for dashboard)
CREATE OR REPLACE FUNCTION get_combined_events(
    p_limit INTEGER DEFAULT 100,
    p_offset INTEGER DEFAULT 0,
    p_org_id UUID DEFAULT NULL,
    p_repo_id UUID DEFAULT NULL,
    p_source TEXT DEFAULT NULL,  -- 'github', 'client', or NULL for both
    p_event_type TEXT DEFAULT NULL,
    p_user_login TEXT DEFAULT NULL
) RETURNS TABLE (
    id TEXT,
    source TEXT,
    event_type TEXT,
    created_at TIMESTAMPTZ,
    user_login TEXT,
    repo_name TEXT,
    branch TEXT,
    status TEXT,
    details JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        g.id::TEXT,
        'github'::TEXT as source,
        g.event_type,
        g.created_at,
        g.actor_login as user_login,
        r.full_name as repo_name,
        g.ref_name as branch,
        NULL::TEXT as status,
        jsonb_build_object(
            'commits_count', g.commits_count,
            'after_sha', g.after_sha
        ) as details
    FROM github_events g
    LEFT JOIN repos r ON g.repo_id = r.id
    WHERE (p_org_id IS NULL OR g.org_id = p_org_id)
      AND (p_repo_id IS NULL OR g.repo_id = p_repo_id)
      AND (p_event_type IS NULL OR g.event_type = p_event_type)
      AND (p_source IS NULL OR p_source = 'github')
      AND (p_user_login IS NULL OR g.actor_login = p_user_login)

    UNION ALL

    SELECT
        c.id::TEXT,
        'client'::TEXT as source,
        c.event_type,
        c.created_at,
        c.user_login,
        r.full_name as repo_name,
        c.branch,
        c.status,
        jsonb_build_object(
            'reason', c.reason,
            'files', c.files
        ) as details
    FROM client_events c
    LEFT JOIN repos r ON c.repo_id = r.id
    WHERE (p_org_id IS NULL OR c.org_id = p_org_id)
      AND (p_repo_id IS NULL OR c.repo_id = p_repo_id)
      AND (p_event_type IS NULL OR c.event_type = p_event_type)
      AND (p_source IS NULL OR p_source = 'client')
      AND (p_user_login IS NULL OR c.user_login = p_user_login)

    ORDER BY created_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Cleanup old webhook events (keep last 30 days)
CREATE OR REPLACE FUNCTION cleanup_old_webhook_events(days_to_keep INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM webhook_events 
    WHERE created_at < NOW() - (days_to_keep || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================================
-- POLICY HISTORY (Historial de cambios de políticas)
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    config JSONB NOT NULL,
    checksum TEXT NOT NULL,
    source_metadata JSONB NOT NULL DEFAULT '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb,
    changed_by TEXT NOT NULL,
    change_type TEXT NOT NULL DEFAULT 'update',  -- 'create', 'update', 'delete'
    previous_checksum TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_history_repo ON policy_history(repo_id);
CREATE INDEX IF NOT EXISTS idx_policy_history_created ON policy_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_history_changed_by ON policy_history(changed_by);
CREATE INDEX IF NOT EXISTS idx_policy_history_source_mode
    ON policy_history ((source_metadata ->> 'source_mode'));

-- Trigger to auto-populate policy_history
DROP TRIGGER IF EXISTS policy_history_trigger ON policies;
CREATE OR REPLACE FUNCTION record_policy_change()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO policy_history (
        repo_id,
        config,
        checksum,
        source_metadata,
        changed_by,
        change_type,
        previous_checksum
    )
    VALUES (
        NEW.repo_id,
        NEW.config,
        NEW.checksum,
        NEW.source_metadata,
        COALESCE(NEW.override_actor, 'system'),
        CASE WHEN TG_OP = 'INSERT' THEN 'create' ELSE 'update' END,
        CASE WHEN TG_OP = 'UPDATE' THEN OLD.checksum ELSE NULL END
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE TRIGGER policy_history_trigger
    AFTER INSERT OR UPDATE ON policies
    FOR EACH ROW EXECUTE FUNCTION record_policy_change();

-- ============================================================================
-- POLICY DRIFT EVENTS (Client-side policy drift audit trail - append-only)
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_drift_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    user_login TEXT NOT NULL,
    action TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    result TEXT NOT NULL,
    before_checksum TEXT,
    after_checksum TEXT,
    duration_ms BIGINT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_drift_events_org_created
    ON policy_drift_events(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_repo_created
    ON policy_drift_events(repo_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_user_created
    ON policy_drift_events(user_login, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_action
    ON policy_drift_events(action);
CREATE INDEX IF NOT EXISTS idx_policy_drift_events_result
    ON policy_drift_events(result);

DROP TRIGGER IF EXISTS policy_drift_events_append_only ON policy_drift_events;
CREATE TRIGGER policy_drift_events_append_only
    BEFORE UPDATE OR DELETE ON policy_drift_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

ALTER TABLE policy_drift_events ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Admins can see all policy_drift_events" ON policy_drift_events
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM members
            WHERE members.github_login = auth.jwt() ->> 'user_name'
            AND members.role = 'Admin'
        )
    );

-- ============================================================================
-- CORRELATION CONFIG (Configuración de correlación y bypass detection)
-- ============================================================================

CREATE TABLE IF NOT EXISTS correlation_config (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE UNIQUE,
    correlation_window_minutes INTEGER DEFAULT 15,
    bypass_tolerance_minutes INTEGER DEFAULT 30,
    clock_skew_seconds INTEGER DEFAULT 60,
    auto_create_violations BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default config for each org
CREATE OR REPLACE FUNCTION create_default_correlation_config()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO correlation_config (org_id) VALUES (NEW.id) ON CONFLICT DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

DROP TRIGGER IF EXISTS org_correlation_config_trigger ON orgs;
CREATE TRIGGER org_correlation_config_trigger
    AFTER INSERT ON orgs
    FOR EACH ROW EXECUTE FUNCTION create_default_correlation_config();

-- ============================================================================
-- NONCOMPLIANCE SIGNALS (Señales de noncompliance - NO binario)
-- ============================================================================

CREATE TABLE IF NOT EXISTS noncompliance_signals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    github_event_id UUID REFERENCES github_events(id),
    client_event_id UUID REFERENCES client_events(id),
    
    signal_type TEXT NOT NULL,  -- 'untrusted_path', 'missing_telemetry', 'policy_violation', 'correlation_mismatch'
    confidence TEXT NOT NULL,   -- 'high', 'medium', 'low'
    
    actor_login TEXT NOT NULL,
    branch TEXT,
    commit_sha TEXT,
    
    evidence JSONB NOT NULL DEFAULT '{}',
    context JSONB DEFAULT '{}',
    
    status TEXT DEFAULT 'pending',  -- 'pending', 'investigating', 'confirmed', 'dismissed'
    investigated_by TEXT,
    investigated_at TIMESTAMPTZ,
    investigation_notes TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_signals_org_repo ON noncompliance_signals(org_id, repo_id);
CREATE INDEX IF NOT EXISTS idx_signals_created ON noncompliance_signals(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_signals_type ON noncompliance_signals(signal_type);
CREATE INDEX IF NOT EXISTS idx_signals_confidence ON noncompliance_signals(confidence);
CREATE INDEX IF NOT EXISTS idx_signals_status ON noncompliance_signals(status);
CREATE INDEX IF NOT EXISTS idx_signals_actor ON noncompliance_signals(actor_login);

-- Append-only trigger for noncompliance_signals - NO UPDATE OR DELETE
DROP TRIGGER IF EXISTS signals_append_only ON noncompliance_signals;
DROP TRIGGER IF EXISTS signals_no_update ON noncompliance_signals;
CREATE TRIGGER signals_append_only
    BEFORE UPDATE OR DELETE ON noncompliance_signals
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- ============================================================================
-- SIGNAL DECISIONS (Workflow decisions on signals - APPEND ONLY)
-- ============================================================================

CREATE TABLE IF NOT EXISTS signal_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    signal_id UUID REFERENCES noncompliance_signals(id) ON DELETE CASCADE,
    decision TEXT NOT NULL,           -- 'confirmed', 'dismissed', 'investigating'
    decided_by TEXT NOT NULL,
    severity TEXT,                    -- Only for 'confirmed' decisions
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_decisions_signal ON signal_decisions(signal_id);
CREATE INDEX IF NOT EXISTS idx_decisions_created ON signal_decisions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_decisions_by ON signal_decisions(decided_by);

-- Append-only trigger for signal_decisions
DROP TRIGGER IF EXISTS decisions_append_only ON signal_decisions;
CREATE TRIGGER decisions_append_only
    BEFORE UPDATE OR DELETE ON signal_decisions
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- ============================================================================
-- EXPORT LOGS (Registro de exports para auditoría)
-- ============================================================================

CREATE TABLE IF NOT EXISTS export_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    exported_by TEXT NOT NULL,
    export_type TEXT NOT NULL,  -- 'pdf', 'excel', 'json', 'csv'
    date_range_start TIMESTAMPTZ,
    date_range_end TIMESTAMPTZ,
    filters JSONB DEFAULT '{}',
    record_count INTEGER,
    content_hash TEXT,  -- SHA256 del contenido exportado
    file_path TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_exports_org ON export_logs(org_id);
CREATE INDEX IF NOT EXISTS idx_exports_created ON export_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_exports_by ON export_logs(exported_by);

-- ============================================================================
-- GOVERNANCE EVENTS (Audit log streaming from GitHub)
-- ============================================================================

CREATE TABLE IF NOT EXISTS governance_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    delivery_id TEXT UNIQUE NOT NULL,
    event_type TEXT NOT NULL,          -- branch_protection_changed, ruleset_modified, permission_changed, etc.
    actor_login TEXT,
    target TEXT,                        -- What resource was affected: branch name, ruleset, user
    old_value JSONB,
    new_value JSONB,
    payload JSONB NOT NULL,            -- Raw event payload
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_governance_org_repo ON governance_events(org_id, repo_id);
CREATE INDEX IF NOT EXISTS idx_governance_created ON governance_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_governance_type ON governance_events(event_type);
CREATE INDEX IF NOT EXISTS idx_governance_actor ON governance_events(actor_login);
CREATE INDEX IF NOT EXISTS idx_governance_delivery ON governance_events(delivery_id);

-- Append-only trigger for governance_events
DROP TRIGGER IF EXISTS governance_events_append_only ON governance_events;
CREATE TRIGGER governance_events_append_only
    BEFORE UPDATE OR DELETE ON governance_events
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- ============================================================================
-- JOB QUEUE (For async processing with backpressure)
-- ============================================================================

CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
    job_type TEXT NOT NULL,              -- 'detect_signals', 'correlate_events', etc.
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed'
    priority INTEGER DEFAULT 0,
    payload JSONB DEFAULT '{}',
    
    -- Locking for job processing
    locked_at TIMESTAMPTZ,
    locked_by TEXT,                      -- Worker ID
    
    -- Retry handling
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    last_error TEXT,
    next_run_at TIMESTAMPTZ DEFAULT NOW(),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_jobs_status_next_run ON jobs(status, next_run_at) WHERE status IN ('pending', 'running');
CREATE INDEX IF NOT EXISTS idx_jobs_org_type ON jobs(org_id, job_type);
CREATE INDEX IF NOT EXISTS idx_jobs_locked ON jobs(locked_at) WHERE locked_at IS NOT NULL;

-- Unique constraint: only one job of each type per org at a time
CREATE UNIQUE INDEX IF NOT EXISTS idx_jobs_unique_pending 
    ON jobs(org_id, job_type) 
    WHERE status IN ('pending', 'running');

-- ============================================================================
-- ORG PROCESSING STATE (Cursor for incremental processing)
-- ============================================================================

CREATE TABLE IF NOT EXISTS org_processing_state (
    org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
    
    -- Cursor for github_events processing
    last_processed_event_id UUID,
    last_processed_created_at TIMESTAMPTZ,
    
    -- Processing metadata
    last_run_at TIMESTAMPTZ,
    events_processed INTEGER DEFAULT 0,
    signals_created INTEGER DEFAULT 0,
    
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- OPTIMIZED INDEXES FOR INCREMENTAL PROCESSING
-- ============================================================================

-- Composite index for incremental cursor-based queries
DROP INDEX IF EXISTS idx_github_events_org_type_created;
CREATE INDEX idx_github_events_org_type_created 
    ON github_events(org_id, event_type, created_at, id);

-- Index for signal deduplication check
DROP INDEX IF EXISTS idx_signals_github_event;
CREATE INDEX idx_signals_github_event 
    ON noncompliance_signals(github_event_id);

-- Index for client event correlation
DROP INDEX IF EXISTS idx_client_events_org_user_commit;
CREATE INDEX idx_client_events_org_user_commit 
    ON client_events(org_id, user_login, commit_sha, created_at);

-- ============================================================================
-- SIGNAL DECISIONS (Workflow decisions on signals - APPEND ONLY)
-- ============================================================================

CREATE TABLE IF NOT EXISTS signal_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    signal_id UUID REFERENCES noncompliance_signals(id) ON DELETE CASCADE,
    decision TEXT NOT NULL,              -- 'confirmed', 'dismissed', 'investigating'
    decided_by TEXT NOT NULL,
    severity TEXT,                       -- Only for 'confirmed' decisions
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_decisions_signal ON signal_decisions(signal_id);
CREATE INDEX IF NOT EXISTS idx_decisions_created ON signal_decisions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_decisions_by ON signal_decisions(decided_by);

-- Append-only trigger for signal_decisions
DROP TRIGGER IF EXISTS decisions_append_only ON signal_decisions;
CREATE TRIGGER decisions_append_only
    BEFORE UPDATE OR DELETE ON signal_decisions
    FOR EACH ROW EXECUTE FUNCTION prevent_update_delete();

-- ============================================================================
-- CORRELATION FUNCTIONS
-- ============================================================================

-- Detect noncompliance signals from uncorrelated events (INCREMENTAL with cursor)
CREATE OR REPLACE FUNCTION detect_noncompliance_signals(
    p_org_id UUID,
    p_window_minutes INTEGER DEFAULT 15,
    p_tolerance_minutes INTEGER DEFAULT 30
) RETURNS INTEGER AS $$
DECLARE
    signal_count INTEGER := 0;
    rec RECORD;
    config RECORD;
    cursor_id UUID := NULL;
    cursor_at TIMESTAMPTZ := NULL;
    processed_count INTEGER := 0;
    last_event_id UUID;
    last_event_at TIMESTAMPTZ;
BEGIN
    -- Get org config
    SELECT * INTO config FROM correlation_config WHERE org_id = p_org_id;
    IF NOT FOUND THEN
        config := ROW(p_org_id, p_window_minutes, p_tolerance_minutes, 60, FALSE, NOW(), NOW());
    END IF;

    -- Get cursor from processing state
    SELECT last_processed_event_id, last_processed_created_at 
    INTO cursor_id, cursor_at
    FROM org_processing_state 
    WHERE org_id = p_org_id;
    
    -- If no cursor, process from configured window
    IF cursor_at IS NULL THEN
        cursor_at := NOW() - (config.correlation_window_minutes || ' minutes')::INTERVAL;
    END IF;

    -- Find github push events AFTER cursor (incremental processing)
    -- Uses composite index: idx_github_events_org_type_created
    FOR rec IN 
        SELECT ge.id, ge.actor_login, ge.repo_id, ge.ref_name, ge.after_sha, ge.created_at
        FROM github_events ge
        WHERE ge.org_id = p_org_id
          AND ge.event_type = 'push'
          AND (cursor_id IS NULL OR (ge.created_at, ge.id) > (cursor_at, cursor_id))
          AND NOT EXISTS (
              SELECT 1 FROM client_events ce
              WHERE ce.org_id = p_org_id
                AND ce.user_login = ge.actor_login
                AND ce.commit_sha = ge.after_sha
                AND ce.created_at BETWEEN 
                    ge.created_at - (config.clock_skew_seconds || ' seconds')::INTERVAL AND
                    ge.created_at + (config.clock_skew_seconds || ' seconds')::INTERVAL
          )
          AND NOT EXISTS (
              SELECT 1 FROM noncompliance_signals ns
              WHERE ns.github_event_id = ge.id
          )
        ORDER BY ge.created_at ASC, ge.id ASC
        LIMIT 100
    LOOP
        -- Track last processed event for cursor update
        last_event_id := rec.id;
        last_event_at := rec.created_at;
        processed_count := processed_count + 1;
        
        -- Check if there are pending client events (outbox not flushed)
        DECLARE
            pending_events INTEGER;
        BEGIN
            SELECT COUNT(*) INTO pending_events
            FROM client_events ce
            WHERE ce.org_id = p_org_id
              AND ce.user_login = rec.actor_login
              AND ce.created_at >= NOW() - (config.bypass_tolerance_minutes || ' minutes')::INTERVAL;

            -- Determine confidence based on pending events and timing
            IF pending_events > 0 THEN
                INSERT INTO noncompliance_signals (
                    org_id, repo_id, github_event_id, signal_type, confidence,
                    actor_login, branch, commit_sha, evidence
                ) VALUES (
                    p_org_id, rec.repo_id, rec.id, 'missing_telemetry', 'low',
                    rec.actor_login, rec.ref_name, rec.after_sha,
                    jsonb_build_object(
                        'github_event_time', rec.created_at,
                        'pending_client_events', pending_events,
                        'note', 'Client events exist but no matching push correlation'
                    )
                );
            ELSE
                INSERT INTO noncompliance_signals (
                    org_id, repo_id, github_event_id, signal_type, confidence,
                    actor_login, branch, commit_sha, evidence
                ) VALUES (
                    p_org_id, rec.repo_id, rec.id, 'untrusted_path', 'high',
                    rec.actor_login, rec.ref_name, rec.after_sha,
                    jsonb_build_object(
                        'github_event_time', rec.created_at,
                        'pending_client_events', 0,
                        'note', 'Push detected without GitGov client event'
                    )
                );
            END IF;
            
            signal_count := signal_count + 1;
        END;
    END LOOP;
    
    -- Update cursor in processing state
    IF last_event_id IS NOT NULL THEN
        INSERT INTO org_processing_state (org_id, last_processed_event_id, last_processed_created_at, last_run_at, events_processed, signals_created)
        VALUES (p_org_id, last_event_id, last_event_at, NOW(), processed_count, signal_count)
        ON CONFLICT (org_id) DO UPDATE SET
            last_processed_event_id = EXCLUDED.last_processed_event_id,
            last_processed_created_at = EXCLUDED.last_processed_created_at,
            last_run_at = NOW(),
            events_processed = org_processing_state.events_processed + processed_count,
            signals_created = org_processing_state.signals_created + signal_count,
            updated_at = NOW();
    END IF;
    
    RETURN signal_count;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Optional: Automatic trigger for correlation detection
-- Uncomment to enable DB-level automatic detection on every push event
-- Note: The control plane server also calls detect_noncompliance_signals after webhook processing
-- Using both the trigger AND the server call may result in duplicate detection attempts (harmless but inefficient)

-- DROP TRIGGER IF EXISTS auto_detect_signals ON github_events;
-- CREATE OR REPLACE FUNCTION trigger_detect_signals()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     IF NEW.event_type = 'push' AND NEW.org_id IS NOT NULL THEN
--         PERFORM detect_noncompliance_signals(NEW.org_id);
--     END IF;
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql SECURITY DEFINER;
-- CREATE TRIGGER auto_detect_signals
--     AFTER INSERT ON github_events
--     FOR EACH ROW EXECUTE FUNCTION trigger_detect_signals();

-- Correlate client and github events
CREATE OR REPLACE FUNCTION correlate_events(
    p_org_id UUID,
    p_window_minutes INTEGER DEFAULT 15
) RETURNS TABLE (
    github_event_id UUID,
    client_event_id UUID,
    correlation_score FLOAT,
    matched_on TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ge.id as github_event_id,
        ce.id as client_event_id,
        CASE 
            WHEN ge.after_sha = ce.commit_sha AND ge.actor_login = ce.user_login THEN 1.0
            WHEN ge.after_sha = ce.commit_sha THEN 0.8
            WHEN ge.actor_login = ce.user_login AND ge.ref_name = ce.branch THEN 0.6
            ELSE 0.3
        END as correlation_score,
        CASE
            WHEN ge.after_sha = ce.commit_sha AND ge.actor_login = ce.user_login THEN 'commit_sha+actor'
            WHEN ge.after_sha = ce.commit_sha THEN 'commit_sha'
            WHEN ge.actor_login = ce.user_login AND ge.ref_name = ce.branch THEN 'actor+branch'
            ELSE 'timestamp'
        END as matched_on
    FROM github_events ge
    JOIN client_events ce ON ce.org_id = ge.org_id
    WHERE ge.org_id = p_org_id
      AND ge.event_type = 'push'
      AND ce.created_at BETWEEN 
          ge.created_at - (p_window_minutes || ' minutes')::INTERVAL AND
          ge.created_at + (p_window_minutes || ' minutes')::INTERVAL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get compliance dashboard data
CREATE OR REPLACE FUNCTION get_compliance_dashboard(p_org_id UUID)
RETURNS JSON AS $$
DECLARE
    result JSON;
BEGIN
    SELECT json_build_object(
        'signals', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id),
                'pending', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id AND status = 'pending'),
                'high_confidence', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id AND confidence = 'high'),
                'by_type', COALESCE((SELECT json_object_agg(signal_type, cnt) FROM (SELECT signal_type, COUNT(*) as cnt FROM noncompliance_signals WHERE org_id = p_org_id GROUP BY signal_type) t), '{}'::json)
            )
        ),
        'correlation', (
            SELECT json_build_object(
                'github_pushes_24h', (SELECT COUNT(*) FROM github_events WHERE org_id = p_org_id AND event_type = 'push' AND created_at >= NOW() - INTERVAL '24 hours'),
                'client_pushes_24h', (SELECT COUNT(*) FROM client_events WHERE org_id = p_org_id AND event_type IN ('successful_push', 'attempt_push', 'blocked_push', 'governance_blocked_push', 'governance_warned_push', 'push_failed') AND created_at >= NOW() - INTERVAL '24 hours'),
                'correlation_rate', (
                    SELECT CASE 
                        WHEN COUNT(*) > 0 THEN 
                            (SELECT COUNT(*) FROM github_events ge WHERE ge.org_id = p_org_id AND ge.event_type = 'push' AND ge.created_at >= NOW() - INTERVAL '24 hours' AND EXISTS (
                                SELECT 1 FROM client_events ce WHERE ce.org_id = p_org_id AND ce.commit_sha = ge.after_sha
                            ))::FLOAT / COUNT(*)
                        ELSE 1.0
                    END
                    FROM github_events WHERE org_id = p_org_id AND event_type = 'push' AND created_at >= NOW() - INTERVAL '24 hours'
                )
            )
        ),
        'policy', (
            SELECT json_build_object(
                'repos_with_policy', (SELECT COUNT(*) FROM policies p JOIN repos r ON p.repo_id = r.id WHERE r.org_id = p_org_id),
                'total_repos', (SELECT COUNT(*) FROM repos WHERE org_id = p_org_id),
                'recent_changes', (SELECT COUNT(*) FROM policy_history ph JOIN repos r ON ph.repo_id = r.id WHERE r.org_id = p_org_id AND ph.created_at >= NOW() - INTERVAL '7 days')
            )
        ),
        'exports', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM export_logs WHERE org_id = p_org_id),
                'last_7_days', (SELECT COUNT(*) FROM export_logs WHERE org_id = p_org_id AND created_at >= NOW() - INTERVAL '7 days')
            )
        )
    ) INTO result;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Get policy history for a repo
CREATE OR REPLACE FUNCTION get_policy_history(
    p_repo_id UUID,
    p_limit INTEGER DEFAULT 50
) RETURNS TABLE (
    id TEXT,
    config JSONB,
    checksum TEXT,
    source_metadata JSONB,
    changed_by TEXT,
    change_type TEXT,
    previous_checksum TEXT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ph.id::TEXT,
        ph.config,
        ph.checksum,
        ph.source_metadata,
        ph.changed_by,
        ph.change_type,
        ph.previous_checksum,
        ph.created_at
    FROM policy_history ph
    WHERE ph.repo_id = p_repo_id
    ORDER BY ph.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================================
-- INITIAL DATA (Optional - for testing)
-- ============================================================================

-- Create a default org (uncomment and modify as needed)
-- INSERT INTO orgs (login, name) VALUES ('your-org', 'Your Organization');

-- Create a default admin member (uncomment and modify as needed)
-- INSERT INTO members (org_id, github_login, role) 
-- VALUES ((SELECT id FROM orgs WHERE login = 'your-org'), 'your-github-username', 'Admin');

-- ============================================================================
-- GRANTS (Service role needs full access)
-- ============================================================================

-- Grant necessary permissions to postgres user (service role)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO postgres;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT, INSERT ON violation_decisions TO gitgov_server;
        GRANT SELECT, INSERT ON policy_drift_events TO gitgov_server;
        GRANT SELECT ON violation_current_status TO gitgov_server;
        GRANT EXECUTE ON FUNCTION add_violation_decision(UUID, TEXT, TEXT, TEXT, JSONB) TO gitgov_server;
    END IF;
END;
$$;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE github_events IS 'Source of truth - events from GitHub webhooks. Append-only.';
COMMENT ON TABLE client_events IS 'Telemetry from desktop clients. Append-only.';
COMMENT ON TABLE violations IS 'Policy violations detected. Strict append-only. Resolution status tracked in violation_decisions.';
COMMENT ON TABLE violation_decisions IS 'Audit trail of resolution/status decisions for violations. Append-only.';
COMMENT ON TABLE policy_drift_events IS 'Client-side policy drift events. Append-only.';
COMMENT ON FUNCTION add_violation_decision(UUID, TEXT, TEXT, TEXT, JSONB) IS 'Insert-only helper for recording violation decisions.';
COMMENT ON VIEW violation_current_status IS 'Latest decision/status per violation derived from append-only decision history.';
COMMENT ON TABLE orgs IS 'Organizations using GitGov. Can be updated.';
COMMENT ON TABLE repos IS 'Repositories being tracked. Can be updated.';
COMMENT ON TABLE members IS 'Organization members with roles. Can be updated.';
COMMENT ON TABLE policies IS 'Per-repo policies (gitgov.toml content). Can be updated.';
