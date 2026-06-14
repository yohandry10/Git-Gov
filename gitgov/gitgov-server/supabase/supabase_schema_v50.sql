-- KAN-107: Admin-reviewed workflow metadata for Framework Review Reports.

ALTER TABLE compliance_framework_review_reports
    ADD COLUMN IF NOT EXISTS review_status TEXT NOT NULL DEFAULT 'needs_review'
        CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')),
    ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT,
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS review_notes_safe TEXT;

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_review_status
    ON compliance_framework_review_reports(org_id, review_status, created_at DESC);

COMMENT ON COLUMN compliance_framework_review_reports.review_status IS
    'KAN-107 manual review workflow status for the framework review report metadata. Does not change artifact hash or create compliance claims.';

