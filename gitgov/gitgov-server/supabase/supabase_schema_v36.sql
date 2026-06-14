-- KAN-87: Break-glass deployment authorization evidence.
-- Extends deployment_gate_authorizations without rewriting existing history.

ALTER TABLE deployment_gate_authorizations
    ADD COLUMN IF NOT EXISTS break_glass_used BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS break_glass_reason TEXT,
    ADD COLUMN IF NOT EXISTS break_glass_authorized_by TEXT,
    ADD COLUMN IF NOT EXISTS break_glass_expires_at TIMESTAMPTZ;

ALTER TABLE deployment_gate_authorizations
    DROP CONSTRAINT IF EXISTS deployment_gate_authorizations_decision_check;

ALTER TABLE deployment_gate_authorizations
    ADD CONSTRAINT deployment_gate_authorizations_decision_check
    CHECK (decision IN ('approved', 'advisory', 'blocked', 'break_glass'));

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'deployment_gate_break_glass_reason_check'
          AND conrelid = 'deployment_gate_authorizations'::regclass
    ) THEN
        ALTER TABLE deployment_gate_authorizations
            ADD CONSTRAINT deployment_gate_break_glass_reason_check
            CHECK (
                (break_glass_used = FALSE AND break_glass_reason IS NULL)
                OR
                (break_glass_used = TRUE AND break_glass_reason IS NOT NULL AND length(trim(break_glass_reason)) >= 16)
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_break_glass
    ON deployment_gate_authorizations(org_id, break_glass_used, created_at DESC)
    WHERE break_glass_used = TRUE;
