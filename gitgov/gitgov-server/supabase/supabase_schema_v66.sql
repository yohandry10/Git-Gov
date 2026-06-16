-- KAN-126: Change Risk CAB Packet Manual Disposition.

ALTER TABLE change_risk_cab_packets
    ADD COLUMN IF NOT EXISTS review_status TEXT NOT NULL DEFAULT 'pending_review',
    ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT,
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS review_notes_safe TEXT,
    ADD COLUMN IF NOT EXISTS mitigation_notes_safe TEXT,
    ADD COLUMN IF NOT EXISTS decision_reason_safe TEXT,
    ADD COLUMN IF NOT EXISTS follow_up_required BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS follow_up_owner_safe TEXT,
    ADD COLUMN IF NOT EXISTS review_updated_at TIMESTAMPTZ;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND conname = 'change_risk_cab_packets_review_status_check'
    ) THEN
        ALTER TABLE change_risk_cab_packets
            ADD CONSTRAINT change_risk_cab_packets_review_status_check
            CHECK (
                review_status IN (
                    'pending_review',
                    'reviewed',
                    'accepted_risk',
                    'needs_mitigation',
                    'returned_to_owner',
                    'rejected'
                )
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND conname = 'change_risk_cab_packets_review_notes_len_check'
    ) THEN
        ALTER TABLE change_risk_cab_packets
            ADD CONSTRAINT change_risk_cab_packets_review_notes_len_check
            CHECK (
                COALESCE(length(review_notes_safe), 0) <= 1000
                AND COALESCE(length(mitigation_notes_safe), 0) <= 1000
                AND COALESCE(length(decision_reason_safe), 0) <= 1000
                AND COALESCE(length(follow_up_owner_safe), 0) <= 1000
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.change_risk_cab_packets'::regclass
          AND conname = 'change_risk_cab_packets_follow_up_check'
    ) THEN
        ALTER TABLE change_risk_cab_packets
            ADD CONSTRAINT change_risk_cab_packets_follow_up_check
            CHECK (
                review_status <> 'needs_mitigation'
                OR (follow_up_required = TRUE AND mitigation_notes_safe IS NOT NULL)
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_change_risk_cab_packets_review
    ON change_risk_cab_packets(org_id, review_status, COALESCE(review_updated_at, created_at) DESC);
