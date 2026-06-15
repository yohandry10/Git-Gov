-- KAN-109: Framework Review Report Auditor assignments and safe comments.

CREATE TABLE IF NOT EXISTS compliance_framework_review_report_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
    auditor_client_id TEXT NOT NULL,
    assignment_status TEXT NOT NULL DEFAULT 'active',
    assigned_by_user_id TEXT NOT NULL,
    assignment_notes_safe TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT compliance_framework_review_report_assignments_status_check
        CHECK (assignment_status IN ('active', 'revoked')),
    CONSTRAINT compliance_framework_review_report_assignments_auditor_check
        CHECK (
            auditor_client_id <> ''
            AND length(auditor_client_id) <= 128
            AND auditor_client_id !~* '(bearer |ghp_|glpat-|sk-)'
        ),
    CONSTRAINT compliance_framework_review_report_assignments_note_check
        CHECK (
            assignment_notes_safe IS NULL
            OR (
                length(assignment_notes_safe) <= 1000
                AND assignment_notes_safe !~* '(<script|</|<iframe|bearer |ghp_|glpat-|sk-)'
            )
        ),
    UNIQUE (org_id, report_id, auditor_client_id)
);

CREATE INDEX IF NOT EXISTS idx_cfr_report_assignments_report_status
    ON compliance_framework_review_report_assignments(org_id, report_id, assignment_status);

CREATE INDEX IF NOT EXISTS idx_cfr_report_assignments_auditor_status
    ON compliance_framework_review_report_assignments(org_id, auditor_client_id, assignment_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS compliance_framework_review_report_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
    commenter_client_id TEXT NOT NULL,
    comment_body_safe TEXT NOT NULL,
    review_status_suggestion TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT compliance_framework_review_report_comments_body_check
        CHECK (
            comment_body_safe <> ''
            AND length(comment_body_safe) <= 2000
            AND comment_body_safe !~* '(<script|</|<iframe|bearer |ghp_|glpat-|sk-)'
        ),
    CONSTRAINT compliance_framework_review_report_comments_suggestion_check
        CHECK (
            review_status_suggestion IS NULL
            OR review_status_suggestion IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')
        )
);

CREATE INDEX IF NOT EXISTS idx_cfr_report_comments_report_created
    ON compliance_framework_review_report_comments(org_id, report_id, created_at ASC);

COMMENT ON TABLE compliance_framework_review_report_assignments IS
    'KAN-109 tenant Auditor assignments for existing Framework Review Reports. Assignments route collaboration only and do not change report artifacts or compliance claims.';

COMMENT ON TABLE compliance_framework_review_report_comments IS
    'KAN-109 safe multi-reviewer comments for Framework Review Reports. Comments are metadata only and do not change report artifacts or compliance claims.';
