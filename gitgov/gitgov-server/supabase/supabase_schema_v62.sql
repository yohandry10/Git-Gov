-- KAN-121: Change Risk Assessment Advisory MVP.
-- Persists deterministic, manual-first risk assessments for change/deployment
-- candidates. This table is advisory-only and explicitly cannot represent AI,
-- agent, compliance, certification, or deployment-enforcement decisions.

CREATE TABLE IF NOT EXISTS change_risk_evaluations (
    evaluation_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    repository_full_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    environment TEXT NOT NULL,
    change_id TEXT,
    deployment_gate_id TEXT,
    release_id TEXT,
    commit_sha TEXT,
    evidence_packet_hash TEXT,
    risk_level TEXT NOT NULL CHECK (risk_level IN ('low', 'medium', 'high', 'unknown')),
    risk_reasons JSONB NOT NULL DEFAULT '[]'::jsonb,
    missing_evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    blocking_gaps JSONB NOT NULL DEFAULT '[]'::jsonb,
    recommended_manual_actions JSONB NOT NULL DEFAULT '[]'::jsonb,
    advisory_only BOOLEAN NOT NULL DEFAULT TRUE,
    llm_used BOOLEAN NOT NULL DEFAULT FALSE,
    agent_governance_used BOOLEAN NOT NULL DEFAULT FALSE,
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE,
    certification BOOLEAN NOT NULL DEFAULT FALSE,
    evaluation JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (evaluation_id LIKE 'cra_%'),
    CHECK (advisory_only = TRUE),
    CHECK (llm_used = FALSE),
    CHECK (agent_governance_used = FALSE),
    CHECK (compliance_claim = FALSE),
    CHECK (certification = FALSE)
);

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_org_created
    ON change_risk_evaluations(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_scope
    ON change_risk_evaluations(
        org_id,
        repository_full_name,
        branch,
        environment,
        created_at DESC
    );

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_gate
    ON change_risk_evaluations(org_id, deployment_gate_id, created_at DESC)
    WHERE deployment_gate_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_release
    ON change_risk_evaluations(org_id, release_id, environment, created_at DESC)
    WHERE release_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_commit
    ON change_risk_evaluations(org_id, commit_sha, created_at DESC)
    WHERE commit_sha IS NOT NULL;
