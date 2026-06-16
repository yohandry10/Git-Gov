-- KAN-127: Change Risk CAB Decision Manifest.

CREATE TABLE IF NOT EXISTS change_risk_cab_decision_manifests (
    manifest_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    cab_packet_id TEXT NOT NULL REFERENCES change_risk_cab_packets(packet_id) ON DELETE CASCADE,
    cab_packet_hash TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    manifest_json JSONB NOT NULL,
    review_status_snapshot TEXT NOT NULL,
    reviewed_by_user_id TEXT,
    reviewed_at TIMESTAMPTZ,
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    download_count BIGINT NOT NULL DEFAULT 0,
    downloaded_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    revoked_at TIMESTAMPTZ,
    revoked_by_user_id TEXT,
    CONSTRAINT change_risk_cab_decision_manifests_id_check
        CHECK (manifest_id ~ '^crcabdm_[a-f0-9]{32}$'),
    CONSTRAINT change_risk_cab_decision_manifests_packet_id_check
        CHECK (cab_packet_id ~ '^crcab_[a-f0-9]{32}$'),
    CONSTRAINT change_risk_cab_decision_manifests_packet_hash_check
        CHECK (cab_packet_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT change_risk_cab_decision_manifests_manifest_hash_check
        CHECK (manifest_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT change_risk_cab_decision_manifests_review_status_check
        CHECK (
        review_status_snapshot IN (
            'reviewed',
            'accepted_risk',
            'needs_mitigation',
            'returned_to_owner',
            'rejected'
        )
    ),
    CONSTRAINT change_risk_cab_decision_manifests_status_check
        CHECK (status IN ('active', 'revoked')),
    CONSTRAINT change_risk_cab_decision_manifests_revoke_check
        CHECK (
        status = 'active'
        OR (revoked_at IS NOT NULL AND revoked_by_user_id IS NOT NULL)
    ),
    CONSTRAINT change_risk_cab_decision_manifests_manifest_json_check
        CHECK (
        manifest_json ->> 'schema_version' = 'gitgov_change_risk_cab_decision_manifest.v1'
        AND COALESCE((manifest_json #>> '{claims,advisory_only}')::boolean, false) = true
        AND COALESCE((manifest_json #>> '{claims,llm_used}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{claims,agent_governance_used}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{claims,compliance_claim}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{claims,certification}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,enforcement}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,release_blocking}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,deployment_execution}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,provider_mutation}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,repository_mutation}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,source_cab_packet_mutated}')::boolean, true) = false
        AND COALESCE((manifest_json #>> '{audit_metadata,source_evaluations_mutated}')::boolean, true) = false
    )
);

ALTER TABLE change_risk_cab_decision_manifests
    ALTER COLUMN download_count TYPE BIGINT USING download_count::BIGINT;

CREATE INDEX IF NOT EXISTS idx_change_risk_cab_decision_manifests_packet
    ON change_risk_cab_decision_manifests(org_id, cab_packet_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_risk_cab_decision_manifests_status
    ON change_risk_cab_decision_manifests(org_id, status, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_change_risk_cab_decision_manifests_org_hash
    ON change_risk_cab_decision_manifests(org_id, manifest_hash);
