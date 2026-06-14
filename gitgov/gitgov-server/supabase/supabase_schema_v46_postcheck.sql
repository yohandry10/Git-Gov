-- GitGov Control Plane Schema v46 postcheck

DO $$
DECLARE
    missing_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO missing_count
    FROM (
        VALUES
            ('compliance_framework_packs', 'pack_hash'),
            ('compliance_framework_packs', 'official_regulatory_mapping'),
            ('compliance_framework_packs', 'requires_auditor_review'),
            ('compliance_control_frameworks', 'org_id'),
            ('compliance_control_frameworks', 'owner_type'),
            ('compliance_control_frameworks', 'source'),
            ('compliance_control_frameworks', 'framework_pack_id'),
            ('compliance_control_frameworks', 'pack_hash')
    ) AS expected(table_name, column_name)
    WHERE NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = expected.table_name
          AND column_name = expected.column_name
    );

    IF missing_count > 0 THEN
        RAISE EXCEPTION 'v46 postcheck failed: missing required columns';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM compliance_framework_packs
        WHERE compliance_claim
           OR regulatory_claim
           OR gitgov_certifies
           OR NOT requires_auditor_review
           OR official_regulatory_mapping
    ) THEN
        RAISE EXCEPTION 'v46 postcheck failed: customer framework pack claim flags are unsafe';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM compliance_control_frameworks
        WHERE is_regulatory
           OR official_regulatory_mapping
           OR owner_type NOT IN ('gitgov', 'customer')
           OR source NOT IN ('gitgov_owned', 'customer_provided')
    ) THEN
        RAISE EXCEPTION 'v46 postcheck failed: control framework provenance flags are unsafe';
    END IF;
END $$;
