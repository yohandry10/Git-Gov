-- KAN-90: Agent Governance Policy API MVP.
-- Stores deterministic "can this agent do this?" evaluations as append-only
-- governance evidence. Agents request/simulate; GitGov policy decides.

CREATE TABLE IF NOT EXISTS agent_governance_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (evaluation_id LIKE 'agv_%'),
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
