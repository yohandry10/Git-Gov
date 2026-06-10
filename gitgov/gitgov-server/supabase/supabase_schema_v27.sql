-- v27: harden API key role integrity.
-- Keep the migration additive: NOT VALID enforces future writes without failing deploy
-- if a legacy environment has historical bad rows that need manual cleanup.
DO $$
BEGIN
    IF to_regclass('api_keys') IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'api_keys_role_check'
          AND conrelid = to_regclass('api_keys')
    ) THEN
        ALTER TABLE api_keys
            ADD CONSTRAINT api_keys_role_check
            CHECK (role IN ('Admin', 'Architect', 'Developer', 'PM'))
            NOT VALID;
    END IF;
END $$;
