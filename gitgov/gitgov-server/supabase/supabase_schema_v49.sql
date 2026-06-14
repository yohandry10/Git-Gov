-- KAN-106: framework review report inventory history query support.

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_created
    ON compliance_framework_review_reports(org_id, framework_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_mapping
    ON compliance_framework_review_reports(org_id, framework_id, mapping_id);

CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_package
    ON compliance_framework_review_reports(org_id, framework_id, review_package_id);

