-- KAN-117: Manual review/sign-off metadata for Period Compliance Reports.

ALTER TABLE compliance_period_reports
    ADD COLUMN IF NOT EXISTS review_status TEXT NOT NULL DEFAULT 'needs_review';

ALTER TABLE compliance_period_reports
    ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT;

ALTER TABLE compliance_period_reports
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;

ALTER TABLE compliance_period_reports
    ADD COLUMN IF NOT EXISTS review_notes_safe TEXT;

ALTER TABLE compliance_period_reports
    DROP CONSTRAINT IF EXISTS compliance_period_reports_review_status_check;

ALTER TABLE compliance_period_reports
    ADD CONSTRAINT compliance_period_reports_review_status_check
    CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected'));

ALTER TABLE compliance_period_reports
    DROP CONSTRAINT IF EXISTS compliance_period_reports_review_notes_safe_check;

ALTER TABLE compliance_period_reports
    ADD CONSTRAINT compliance_period_reports_review_notes_safe_check
    CHECK (
        review_notes_safe IS NULL
        OR (
            char_length(review_notes_safe) <= 1000
            AND review_notes_safe !~* '(<script|</|<iframe|bearer |ghp_|glpat-|sk-)'
        )
    );

ALTER TABLE compliance_period_reports
    DROP CONSTRAINT IF EXISTS compliance_period_reports_terminal_review_note_check;

ALTER TABLE compliance_period_reports
    ADD CONSTRAINT compliance_period_reports_terminal_review_note_check
    CHECK (
        review_status NOT IN ('needs_changes', 'rejected')
        OR review_notes_safe IS NOT NULL
    );

CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_org_review_status
    ON compliance_period_reports(org_id, review_status, created_at DESC);

ALTER TABLE compliance_period_report_access_log
    DROP CONSTRAINT IF EXISTS compliance_period_report_access_log_action_check;

ALTER TABLE compliance_period_report_access_log
    ADD CONSTRAINT compliance_period_report_access_log_action_check
    CHECK (
        action IN (
            'viewed',
            'downloaded_json',
            'downloaded_pdf',
            'archived',
            'retention_updated',
            'manifest_created',
            'manifest_downloaded',
            'review_updated'
        )
    );

ALTER TABLE compliance_period_report_access_log
    DROP CONSTRAINT IF EXISTS compliance_period_report_access_log_artifact_type_check;

ALTER TABLE compliance_period_report_access_log
    ADD CONSTRAINT compliance_period_report_access_log_artifact_type_check
    CHECK (artifact_type IN ('metadata', 'json', 'pdf', 'retention', 'manifest', 'review'));

COMMENT ON COLUMN compliance_period_reports.review_status IS
    'KAN-117 manual review status for Period Compliance Reports. This is an auditable human workflow state, not a compliance certification or legal attestation.';

COMMENT ON COLUMN compliance_period_reports.review_notes_safe IS
    'KAN-117 plain-text safe review notes for Period Compliance Reports. Tokens, HTML/script tags, and secret-looking values are rejected before storage.';
