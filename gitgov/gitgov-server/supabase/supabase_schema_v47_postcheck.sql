-- GitGov Control Plane Schema v47 postcheck - Framework Pack Review Provenance

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'compliance_framework_packs'
          AND column_name IN (
              'reviewed_by_user_id',
              'reviewed_at',
              'review_notes_safe',
              'rejected_reason_safe',
              'review_updated_at'
          )
        GROUP BY table_name
        HAVING COUNT(*) = 5
    ) THEN
        RAISE EXCEPTION 'v47 postcheck failed: missing framework pack review columns';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM compliance_framework_packs
        WHERE review_status IN ('customer_review_required', 'customer_reviewed')
    ) THEN
        RAISE EXCEPTION 'v47 postcheck failed: legacy framework pack review statuses remain';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM compliance_framework_packs
        WHERE review_status NOT IN ('needs_review', 'reviewed', 'needs_changes', 'rejected', 'archived')
    ) THEN
        RAISE EXCEPTION 'v47 postcheck failed: unexpected framework pack review status';
    END IF;
END $$;
