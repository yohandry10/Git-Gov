-- KAN-123: Change Risk manual review and mitigation notes.

ALTER TABLE change_risk_evaluations
    ADD COLUMN IF NOT EXISTS review_status TEXT NOT NULL DEFAULT 'needs_review',
    ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT,
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS review_notes_safe TEXT,
    ADD COLUMN IF NOT EXISTS mitigation_notes_safe TEXT,
    ADD COLUMN IF NOT EXISTS decision_reason_safe TEXT,
    ADD COLUMN IF NOT EXISTS review_updated_at TIMESTAMPTZ;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_review_status_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_review_status_check
            CHECK (review_status IN ('needs_review', 'reviewed', 'accepted_risk', 'needs_mitigation', 'rejected'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'change_risk_evaluations_review_notes_safe_len_check'
          AND conrelid = 'public.change_risk_evaluations'::regclass
    ) THEN
        ALTER TABLE change_risk_evaluations
            ADD CONSTRAINT change_risk_evaluations_review_notes_safe_len_check
            CHECK (
                length(coalesce(review_notes_safe, '')) <= 1000
                AND length(coalesce(mitigation_notes_safe, '')) <= 1000
                AND length(coalesce(decision_reason_safe, '')) <= 1000
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_change_risk_evaluations_review
    ON change_risk_evaluations(org_id, review_status, COALESCE(review_updated_at, created_at) DESC);
