WITH checks AS (
    SELECT
        'deployment_gate_break_glass_approvals.table' AS check_name,
        EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_name = 'deployment_gate_break_glass_approvals'
        ) AS passed
    UNION ALL
    SELECT
        'deployment_gate_break_glass_approvals.constraints',
        EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'deployment_gate_break_glass_approvals_approval_id_check'
        )
        AND EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'deployment_gate_break_glass_approvals_approver_role_check'
        )
    UNION ALL
    SELECT
        'deployment_gate_authorizations.approval_link_columns',
        EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'deployment_gate_authorizations'
              AND column_name = 'break_glass_approval_id'
        )
        AND EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'deployment_gate_authorizations'
              AND column_name = 'break_glass_approval_hash'
        )
    UNION ALL
    SELECT
        'deployment_gate_authorizations.approval_link_constraint',
        EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'deployment_gate_break_glass_approval_link_check'
        )
)
SELECT
    check_name,
    CASE WHEN passed THEN 'PASS' ELSE 'FAIL' END AS status
FROM checks
ORDER BY check_name;
