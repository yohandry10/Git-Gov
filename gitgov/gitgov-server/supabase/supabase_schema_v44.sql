-- GitGov Control Plane Schema v44 - Evidence-to-Control Mapping
-- =====================================================================
-- KAN-100: Persist deterministic, non-regulatory control mappings over
-- KAN-99 Compliance Evidence Export artifacts.

CREATE TABLE IF NOT EXISTS compliance_control_frameworks (
    framework_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    is_regulatory BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (framework_id = lower(framework_id)),
    CHECK (is_regulatory = FALSE)
);

CREATE TABLE IF NOT EXISTS compliance_controls (
    id TEXT PRIMARY KEY,
    framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE CASCADE,
    control_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    required_evidence_types JSONB NOT NULL DEFAULT '[]'::jsonb,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (framework_id, control_id),
    CHECK (control_id ~ '^GG-RG-[0-9]{2}$')
);

CREATE TABLE IF NOT EXISTS compliance_evidence_mappings (
    mapping_id TEXT PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    evidence_export_id TEXT NOT NULL REFERENCES compliance_evidence_exports(export_id) ON DELETE RESTRICT,
    evidence_export_hash TEXT NOT NULL,
    framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE RESTRICT,
    framework_version TEXT NOT NULL,
    created_by_user_id TEXT NOT NULL,
    compliance_claim BOOLEAN NOT NULL DEFAULT FALSE,
    regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE,
    requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (mapping_id LIKE 'cem_%'),
    CHECK (evidence_export_hash ~ '^sha256:[a-f0-9]{64}$'),
    CHECK (compliance_claim = FALSE),
    CHECK (regulatory_claim = FALSE),
    CHECK (requires_auditor_review = TRUE)
);

CREATE TABLE IF NOT EXISTS compliance_evidence_mapping_items (
    id TEXT PRIMARY KEY,
    mapping_id TEXT NOT NULL REFERENCES compliance_evidence_mappings(mapping_id) ON DELETE CASCADE,
    control_id TEXT NOT NULL,
    control_title TEXT NOT NULL,
    status TEXT NOT NULL CHECK (
        status IN (
            'evidence_present',
            'partial',
            'missing',
            'not_applicable',
            'manual_review_required'
        )
    ),
    evidence_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    missing_evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    notes_safe TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (mapping_id, control_id)
);

CREATE INDEX IF NOT EXISTS idx_compliance_controls_framework_order
    ON compliance_controls(framework_id, sort_order, control_id);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_mappings_org_created
    ON compliance_evidence_mappings(org_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_mappings_export
    ON compliance_evidence_mappings(org_id, evidence_export_id);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_mapping_items_mapping
    ON compliance_evidence_mapping_items(mapping_id, control_id);

INSERT INTO compliance_control_frameworks (
    framework_id,
    name,
    version,
    description,
    is_regulatory,
    is_active
)
VALUES (
    'gitgov_release_governance_baseline_v1',
    'GitGov Release Governance Baseline',
    '1.0.0',
    'GitGov-owned, non-regulatory evidence baseline for reviewing release governance controls.',
    FALSE,
    TRUE
)
ON CONFLICT (framework_id) DO UPDATE SET
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    description = EXCLUDED.description,
    is_regulatory = FALSE,
    is_active = TRUE;

INSERT INTO compliance_controls (
    id,
    framework_id,
    control_id,
    title,
    description,
    required_evidence_types,
    sort_order
)
VALUES
    (
        'gg-rg-01',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-01',
        'Deployment gate decision recorded',
        'The release evidence contains a Deployment Gate decision.',
        '["deployment_gate.decision"]'::jsonb,
        10
    ),
    (
        'gg-rg-02',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-02',
        'Policy source and checksum recorded',
        'The release evidence records the policy source and checksum used for the gate decision.',
        '["policy.source","policy.checksum"]'::jsonb,
        20
    ),
    (
        'gg-rg-03',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-03',
        'Human approval evidence captured when required',
        'The release evidence shows required human release approvals when policy requires approval.',
        '["release_approval"]'::jsonb,
        30
    ),
    (
        'gg-rg-04',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-04',
        'CI/build evidence captured',
        'The release evidence references CI or build execution evidence.',
        '["ci_build_evidence"]'::jsonb,
        40
    ),
    (
        'gg-rg-05',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-05',
        'Code review or PR evidence captured',
        'The release evidence references code change and review evidence where available.',
        '["code_change_evidence","pr_review_evidence"]'::jsonb,
        50
    ),
    (
        'gg-rg-06',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-06',
        'Security or quality evidence captured',
        'The release evidence references security or quality gate evidence.',
        '["quality_gate_result"]'::jsonb,
        60
    ),
    (
        'gg-rg-07',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-07',
        'Deployment target and environment recorded',
        'The release evidence records repository, branch, target SHA, and environment.',
        '["deployment_target"]'::jsonb,
        70
    ),
    (
        'gg-rg-08',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-08',
        'Missing evidence and gaps are explicit',
        'The release evidence exposes missing evidence and gaps instead of hiding them.',
        '["missing_evidence"]'::jsonb,
        80
    ),
    (
        'gg-rg-09',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-09',
        'Audit trail exists',
        'The release evidence includes audit timestamps and redaction markers.',
        '["audit_trail"]'::jsonb,
        90
    ),
    (
        'gg-rg-10',
        'gitgov_release_governance_baseline_v1',
        'GG-RG-10',
        'Agent Governance not required for manual-first gate evidence',
        'The release evidence confirms Deployment Gates work without requiring Agent Governance.',
        '["deployment_gate.agent_governance_used"]'::jsonb,
        100
    )
ON CONFLICT (framework_id, control_id) DO UPDATE SET
    title = EXCLUDED.title,
    description = EXCLUDED.description,
    required_evidence_types = EXCLUDED.required_evidence_types,
    sort_order = EXCLUDED.sort_order;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gitgov_server') THEN
        GRANT SELECT ON compliance_control_frameworks TO gitgov_server;
        GRANT SELECT ON compliance_controls TO gitgov_server;
        GRANT SELECT, INSERT ON compliance_evidence_mappings TO gitgov_server;
        GRANT SELECT, INSERT ON compliance_evidence_mapping_items TO gitgov_server;
    END IF;
END $$;

COMMENT ON TABLE compliance_evidence_mappings IS
    'KAN-100 deterministic, non-regulatory evidence-to-control mappings over KAN-99 exports.';
