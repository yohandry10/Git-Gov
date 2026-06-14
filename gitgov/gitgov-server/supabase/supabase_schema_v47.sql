-- GitGov Control Plane Schema v47 - Framework Pack Review Provenance

BEGIN;

ALTER TABLE compliance_framework_packs
    ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT,
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS review_notes_safe TEXT,
    ADD COLUMN IF NOT EXISTS rejected_reason_safe TEXT,
    ADD COLUMN IF NOT EXISTS review_updated_at TIMESTAMPTZ;

ALTER TABLE compliance_framework_packs
    DROP CONSTRAINT IF EXISTS compliance_framework_packs_review_status_check;

UPDATE compliance_framework_packs
SET review_status = CASE review_status
    WHEN 'customer_review_required' THEN 'needs_review'
    WHEN 'customer_reviewed' THEN 'reviewed'
    ELSE review_status
END;

ALTER TABLE compliance_framework_packs
    ALTER COLUMN review_status SET DEFAULT 'needs_review';

ALTER TABLE compliance_framework_packs
    ADD CONSTRAINT compliance_framework_packs_review_status_check
        CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected', 'archived'));

UPDATE compliance_framework_packs
SET archived_at = COALESCE(archived_at, NOW()),
    review_updated_at = COALESCE(review_updated_at, NOW())
WHERE review_status = 'archived';

UPDATE compliance_framework_packs
SET reviewed_at = COALESCE(reviewed_at, NOW()),
    review_updated_at = COALESCE(review_updated_at, NOW())
WHERE review_status = 'reviewed';

CREATE INDEX IF NOT EXISTS idx_compliance_framework_packs_org_review_status
    ON compliance_framework_packs(org_id, review_status, created_at DESC);

DO $$
BEGIN
    GRANT SELECT, INSERT, UPDATE ON compliance_framework_packs TO gitgov_server;
EXCEPTION
    WHEN undefined_object THEN
        NULL;
END $$;

COMMIT;
