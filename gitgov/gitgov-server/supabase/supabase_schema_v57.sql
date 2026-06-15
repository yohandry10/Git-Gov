ALTER TABLE compliance_period_reports
    ADD COLUMN IF NOT EXISTS retention_status TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS retention_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS download_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_downloaded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;

UPDATE compliance_period_reports
SET retention_until = COALESCE(retention_until, created_at + INTERVAL '7 years'),
    last_downloaded_at = COALESCE(last_downloaded_at, downloaded_at),
    download_count = GREATEST(download_count, CASE WHEN downloaded_at IS NULL THEN 0 ELSE 1 END)
WHERE retention_until IS NULL
   OR (last_downloaded_at IS NULL AND downloaded_at IS NOT NULL)
   OR download_count < CASE WHEN downloaded_at IS NULL THEN 0 ELSE 1 END;

ALTER TABLE compliance_period_reports
    ALTER COLUMN retention_until SET NOT NULL;

ALTER TABLE compliance_period_reports
    ALTER COLUMN retention_until SET DEFAULT (NOW() + INTERVAL '7 years');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_retention_status_check'
    ) THEN
        ALTER TABLE compliance_period_reports
            ADD CONSTRAINT compliance_period_reports_retention_status_check
            CHECK (retention_status IN ('active', 'archived', 'retention_expired'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_download_count_check'
    ) THEN
        ALTER TABLE compliance_period_reports
            ADD CONSTRAINT compliance_period_reports_download_count_check
            CHECK (download_count >= 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'compliance_period_reports_archived_status_check'
    ) THEN
        ALTER TABLE compliance_period_reports
            ADD CONSTRAINT compliance_period_reports_archived_status_check
            CHECK (
                (retention_status = 'archived' AND archived_at IS NOT NULL)
                OR retention_status <> 'archived'
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_retention
    ON compliance_period_reports(org_id, retention_status, retention_until);

CREATE TABLE IF NOT EXISTS compliance_period_report_access_log (
    access_log_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
    actor_client_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (
        action IN ('viewed', 'downloaded_json', 'downloaded_pdf', 'archived', 'retention_updated')
    ),
    artifact_type TEXT NOT NULL CHECK (artifact_type IN ('metadata', 'json', 'pdf', 'retention')),
    artifact_id TEXT,
    artifact_hash TEXT CHECK (artifact_hash IS NULL OR artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (access_log_id LIKE 'cprlog_%'),
    CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_access_log_report_created
    ON compliance_period_report_access_log(org_id, period_report_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_period_report_access_log_actor_created
    ON compliance_period_report_access_log(org_id, actor_client_id, created_at DESC);

COMMENT ON TABLE compliance_period_report_access_log IS
    'KAN-115 append-only custody log for Period Compliance Report views, JSON/PDF downloads, archiving, and retention updates.';
