DO $$
DECLARE
    invalid_constraints integer;
BEGIN
    SELECT COUNT(*) INTO invalid_constraints
    FROM (
        VALUES
            ('api_keys', 'api_keys_role_check'),
            ('org_users', 'org_users_role_check'),
            ('org_invitations', 'org_invitations_role_check')
    ) AS expected(table_name, constraint_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        WHERE t.relname = expected.table_name
          AND c.conname = expected.constraint_name
          AND pg_get_constraintdef(c.oid) LIKE '%Auditor%'
    );

    IF invalid_constraints > 0 THEN
        RAISE EXCEPTION 'KAN-108 postcheck failed: % role constraints do not include Auditor', invalid_constraints;
    END IF;

    RAISE NOTICE 'KAN-108 postcheck passed: tenant Auditor role constraints are installed.';
END
$$;
