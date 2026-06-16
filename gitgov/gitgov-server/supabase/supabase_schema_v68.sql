CREATE TABLE IF NOT EXISTS executive_governance_snapshots (
    snapshot_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    filters_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    artifact_hash TEXT NOT NULL,
    artifact_json JSONB NOT NULL,
    repository_count BIGINT NOT NULL DEFAULT 0 CHECK (repository_count >= 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    downloaded_at TIMESTAMPTZ,
    download_count BIGINT NOT NULL DEFAULT 0 CHECK (download_count >= 0),
    archived_at TIMESTAMPTZ,
    archived_by_user_id TEXT,
    CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK ((status = 'archived' AND archived_at IS NOT NULL) OR status = 'active')
);

CREATE INDEX IF NOT EXISTS idx_executive_governance_snapshots_org_created
    ON executive_governance_snapshots(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_executive_governance_snapshots_org_status
    ON executive_governance_snapshots(org_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_executive_governance_snapshots_org_hash
    ON executive_governance_snapshots(org_id, artifact_hash);
