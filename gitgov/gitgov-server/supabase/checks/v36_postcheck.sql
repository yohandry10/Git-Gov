-- KAN-87 postcheck: break-glass deployment authorization evidence is persisted.

WITH checks AS (
    SELECT
        'deployment_gate_authorizations.break_glass_columns' AS check_name,
        CASE WHEN COUNT(*) = 4 THEN 'PASS' ELSE 'FAIL' END AS status
    FROM information_schema.columns
    WHERE table_name = 'deployment_gate_authorizations'
      AND column_name IN (
          'break_glass_used',
          'break_glass_reason',
          'break_glass_authorized_by',
          'break_glass_expires_at'
      )
    UNION ALL
    SELECT
        'deployment_gate_authorizations.decision_break_glass',
        CASE WHEN pg_get_constraintdef(oid) LIKE '%break_glass%' THEN 'PASS' ELSE 'FAIL' END
    FROM pg_constraint
    WHERE conname = 'deployment_gate_authorizations_decision_check'
      AND conrelid = 'deployment_gate_authorizations'::regclass
    UNION ALL
    SELECT
        'deployment_gate_authorizations.break_glass_reason_constraint',
        CASE WHEN COUNT(*) = 1 THEN 'PASS' ELSE 'FAIL' END
    FROM pg_constraint
    WHERE conname = 'deployment_gate_break_glass_reason_check'
      AND conrelid = 'deployment_gate_authorizations'::regclass
)
SELECT * FROM checks ORDER BY check_name;
