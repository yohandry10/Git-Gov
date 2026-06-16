-- KAN-125: Change Risk CAB Review Packet.

CREATE TABLE IF NOT EXISTS change_risk_cab_packets (
    packet_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    filters_json JSONB NOT NULL,
    evaluation_ids_json JSONB NOT NULL,
    artifact_hash TEXT NOT NULL,
    artifact_json JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    downloaded_at TIMESTAMPTZ,
    download_count BIGINT NOT NULL DEFAULT 0,
    archived_at TIMESTAMPTZ,
    archived_by_user_id TEXT,
    CHECK (packet_id ~ '^crcab_[a-f0-9]{32}$'),
    CHECK (length(name) BETWEEN 1 AND 160),
    CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (status IN ('active', 'archived')),
    CHECK (jsonb_typeof(filters_json) = 'object'),
    CHECK (jsonb_typeof(evaluation_ids_json) = 'array'),
    CHECK (
        artifact_json ->> 'schema_version' = 'gitgov_change_risk_cab_packet.v1'
        AND COALESCE((artifact_json #>> '{claims,compliance_claim}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{claims,certification}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{claims,legal_attestation}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{claims,regulatory_claim}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{claims,compliance_score}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,advisory_only}')::boolean, false) = true
        AND COALESCE((artifact_json #>> '{audit_metadata,llm_used}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,agent_governance_used}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,agent_governance_required}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,enforcement}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,deployment_execution}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,provider_mutation}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,repository_mutation}')::boolean, true) = false
        AND COALESCE((artifact_json #>> '{audit_metadata,source_evaluations_mutated}')::boolean, true) = false
    )
);

ALTER TABLE change_risk_cab_packets
    ALTER COLUMN download_count TYPE BIGINT USING download_count::BIGINT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_change_risk_cab_packets_org_hash
    ON change_risk_cab_packets(org_id, artifact_hash);

CREATE INDEX IF NOT EXISTS idx_change_risk_cab_packets_org_created
    ON change_risk_cab_packets(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_risk_cab_packets_status
    ON change_risk_cab_packets(org_id, status, created_at DESC);
