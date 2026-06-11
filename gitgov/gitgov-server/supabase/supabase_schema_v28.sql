-- v28: Bind release approvals to durable evidence packet snapshots.
--
-- A release approval must reference an evidence packet hash that was generated
-- for the same org/repo/release/environment/branch/target SHA. This table stores
-- the immutable packet snapshot and its binding context so governance evaluation
-- does not trust a client-supplied SHA-256 string.

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
