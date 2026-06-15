use crate::auth;
pub(super) use crate::db::Database;
pub(super) use crate::handlers::PolicyCheckBlockingScope;
use crate::handlers::{self, AppState, ConversationalRuntime};
pub(super) use axum::http::StatusCode;
use axum::{
    body::Body,
    http::Request,
    middleware,
    routing::{get, patch, post, put},
    Router,
};
use sha2::Digest;
use sqlx::PgPool;
pub(super) use sqlx::Row;
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
pub(super) use std::sync::Arc;
use std::sync::Mutex;
pub(super) use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tower::ServiceExt;

/// Try to connect to the test database and set up an isolated schema.
/// Returns None if TEST_DATABASE_URL is not set or connection fails (test will be skipped).
/// Returns (pool_with_schema, schema_name, admin_pool_for_teardown).
pub(super) async fn try_setup() -> Option<(PgPool, String, PgPool)> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;

    // Admin pool: used to create/drop schema only.
    let admin_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await
        .ok()?;

    let mut extension_conn = admin_pool
        .acquire()
        .await
        .expect("acquire extension setup connection");
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(9_021_001_i64)
        .execute(&mut *extension_conn)
        .await
        .expect("lock shared test database extensions");
    for extension in ["uuid-ossp", "pgcrypto"] {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = $1)")
                .bind(extension)
                .fetch_one(&mut *extension_conn)
                .await
                .expect("check shared test database extension");
        if !exists {
            let ddl = format!(r#"CREATE EXTENSION "{}""#, extension);
            sqlx::query(&ddl)
                .execute(&mut *extension_conn)
                .await
                .expect("create shared test database extension");
        }
    }
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(9_021_001_i64)
        .execute(&mut *extension_conn)
        .await
        .expect("unlock shared test database extensions");
    drop(extension_conn);

    let schema = format!("test_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

    // Create schema using admin pool.
    sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema))
        .execute(&admin_pool)
        .await
        .expect("create test schema");

    // Build a pool where EVERY connection sets search_path to the test schema.
    let schema_for_hook = schema.clone();
    let test_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(move |conn, _meta| {
            let s = schema_for_hook.clone();
            Box::pin(async move {
                sqlx::query(&format!("SET search_path TO \"{}\"", s))
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect test pool with schema");

    // Apply minimal DDL needed for the Golden Path tests.
    let ddl = r#"
        CREATE TABLE IF NOT EXISTS orgs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            github_id BIGINT UNIQUE,
            login TEXT UNIQUE NOT NULL,
            name TEXT,
            avatar_url TEXT,
            tenant_type TEXT NOT NULL DEFAULT 'customer'
                CHECK (tenant_type IN ('customer', 'internal', 'sandbox')),
            lifecycle_status TEXT NOT NULL DEFAULT 'active'
                CHECK (lifecycle_status IN ('trial', 'active', 'suspended', 'archived', 'deleted')),
            provisioning_source TEXT NOT NULL DEFAULT 'legacy'
                CHECK (provisioning_source IN ('legacy', 'github_webhook', 'platform_founder', 'migration')),
            provisioned_by TEXT,
            platform_metadata JSONB NOT NULL DEFAULT '{}',
            suspended_at TIMESTAMPTZ,
            archived_at TIMESTAMPTZ,
            deleted_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_adoption_profiles (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            profile JSONB NOT NULL,
            updated_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_onboarding_checklist_tracking (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            tracking JSONB NOT NULL,
            updated_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_first_governed_repo_setups (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            run_id UUID NOT NULL DEFAULT gen_random_uuid(),
            status TEXT NOT NULL DEFAULT 'draft'
                CHECK (status IN ('draft', 'ready', 'blocked', 'completed')),
            goal TEXT NOT NULL DEFAULT 'govern_release'
                CHECK (goal IN (
                    'govern_release',
                    'generate_audit_evidence',
                    'standardize_workflows',
                    'assess_governance_gaps'
                )),
            repository_full_name TEXT NOT NULL,
            default_branch TEXT NOT NULL DEFAULT 'main',
            selected_providers JSONB NOT NULL DEFAULT '["github"]'::jsonb,
            selected_modules JSONB NOT NULL DEFAULT '["traceability","release-readiness","evidence-packets"]'::jsonb,
            policy_preset TEXT NOT NULL DEFAULT 'moderate'
                CHECK (policy_preset IN ('audit-only', 'moderate', 'strict')),
            baseline JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_by TEXT NOT NULL,
            updated_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            completed_at TIMESTAMPTZ
        );

        CREATE TABLE IF NOT EXISTS enterprise_release_approvals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            release_id TEXT NOT NULL,
            repository_full_name TEXT NOT NULL,
            branch TEXT,
            target_sha TEXT,
            environment TEXT NOT NULL,
            decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'accepted-risk')),
            approver TEXT NOT NULL,
            ticket_id TEXT,
            evidence_packet_hash TEXT,
            evidence_packet_uri TEXT,
            evidence_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
            risk_severity TEXT NOT NULL DEFAULT 'none' CHECK (risk_severity IN ('none', 'low', 'medium', 'high', 'critical')),
            risk_acceptance_reason TEXT,
            expires_at TIMESTAMPTZ,
            approval_hash TEXT NOT NULL UNIQUE,
            created_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_enterprise_release_approvals_binding
            ON enterprise_release_approvals(
                org_id,
                repository_full_name,
                release_id,
                environment,
                branch,
                target_sha,
                evidence_packet_hash
            );

        CREATE TABLE IF NOT EXISTS deployment_gate_authorizations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            authorization_id TEXT NOT NULL UNIQUE,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            release_id TEXT NOT NULL,
            repository_full_name TEXT NOT NULL,
            branch TEXT NOT NULL,
            target_sha TEXT NOT NULL,
            environment TEXT NOT NULL,
            deployer TEXT NOT NULL,
            ticket_id TEXT,
            evidence_packet_hash TEXT NOT NULL,
            evidence_packet_uri TEXT,
            decision TEXT NOT NULL CHECK (decision IN ('approved', 'advisory', 'blocked', 'break_glass')),
            approved BOOLEAN NOT NULL,
            blocking BOOLEAN NOT NULL,
            would_block BOOLEAN NOT NULL,
            reason TEXT NOT NULL,
            blocked_by JSONB NOT NULL DEFAULT '[]'::jsonb,
            warnings JSONB NOT NULL DEFAULT '[]'::jsonb,
            policy_checksum TEXT NOT NULL,
            break_glass_eligible BOOLEAN NOT NULL DEFAULT FALSE,
            break_glass_used BOOLEAN NOT NULL DEFAULT FALSE,
            break_glass_reason TEXT,
            break_glass_authorized_by TEXT,
            break_glass_expires_at TIMESTAMPTZ,
            break_glass_approval_id TEXT,
            break_glass_approval_hash TEXT,
            evaluation JSONB NOT NULL,
            details JSONB NOT NULL DEFAULT '{}'::jsonb,
            request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
            requested_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_org_created
            ON deployment_gate_authorizations(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_deployment_gate_authorizations_scope
            ON deployment_gate_authorizations(
                org_id,
                repository_full_name,
                branch,
                environment,
                created_at DESC
            );

        CREATE TABLE IF NOT EXISTS compliance_evidence_exports (
            export_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            created_by_user_id TEXT NOT NULL,
            scope TEXT NOT NULL CHECK (scope IN ('deployment_gate')),
            deployment_gate_id TEXT,
            release_id TEXT,
            status TEXT NOT NULL CHECK (status IN ('completed', 'failed')),
            format TEXT NOT NULL CHECK (format IN ('json')),
            artifact_hash TEXT NOT NULL,
            policy_checksum TEXT,
            gate_decision TEXT,
            payload_json_redacted JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ,
            error_message_safe TEXT,
            CONSTRAINT compliance_evidence_exports_deployment_gate_required
                CHECK (scope <> 'deployment_gate' OR deployment_gate_id IS NOT NULL),
            CONSTRAINT compliance_evidence_exports_hash_shape
                CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$')
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_evidence_exports_org_created
            ON compliance_evidence_exports(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_evidence_exports_deployment_gate
            ON compliance_evidence_exports(org_id, deployment_gate_id);

        CREATE TABLE IF NOT EXISTS compliance_control_frameworks (
            framework_id TEXT PRIMARY KEY,
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT NOT NULL,
            is_regulatory BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            owner_type TEXT NOT NULL DEFAULT 'gitgov' CHECK (owner_type IN ('gitgov', 'customer')),
            owner_name TEXT,
            source TEXT NOT NULL DEFAULT 'gitgov_owned' CHECK (source IN ('gitgov_owned', 'customer_provided')),
            is_gitgov_owned BOOLEAN NOT NULL DEFAULT TRUE,
            official_regulatory_mapping BOOLEAN NOT NULL DEFAULT FALSE CHECK (official_regulatory_mapping = FALSE),
            framework_pack_id TEXT,
            pack_hash TEXT,
            created_by_user_id TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (framework_id = lower(framework_id)),
            CHECK (is_regulatory = FALSE)
        );

        CREATE TABLE IF NOT EXISTS compliance_framework_packs (
            id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            framework_id TEXT NOT NULL,
            framework_name TEXT NOT NULL,
            framework_version TEXT NOT NULL,
            description TEXT NOT NULL,
            owner_type TEXT NOT NULL DEFAULT 'customer' CHECK (owner_type IN ('customer')),
            owner_name TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'customer_provided' CHECK (source = 'customer_provided'),
            review_status TEXT NOT NULL DEFAULT 'needs_review'
                CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected', 'archived')),
            schema_version TEXT NOT NULL,
            pack_hash TEXT NOT NULL,
            raw_pack_redacted JSONB NOT NULL,
            control_count INTEGER NOT NULL CHECK (control_count BETWEEN 1 AND 50),
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
            gitgov_certifies BOOLEAN NOT NULL DEFAULT FALSE CHECK (gitgov_certifies = FALSE),
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
            official_regulatory_mapping BOOLEAN NOT NULL DEFAULT FALSE CHECK (official_regulatory_mapping = FALSE),
            created_by_user_id TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            reviewed_by_user_id TEXT,
            reviewed_at TIMESTAMPTZ,
            review_notes_safe TEXT,
            rejected_reason_safe TEXT,
            review_updated_at TIMESTAMPTZ,
            archived_at TIMESTAMPTZ,
            CHECK (id LIKE 'cfp_%'),
            CHECK (framework_id = lower(framework_id)),
            CHECK (framework_id LIKE 'customer_%'),
            CHECK (pack_hash ~ '^sha256:[a-f0-9]{64}$')
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
            CHECK (control_id ~ '^[A-Z0-9][A-Z0-9_.:-]{0,63}$')
        );

        ALTER TABLE compliance_control_frameworks
            ADD CONSTRAINT compliance_control_frameworks_framework_pack_fk
                FOREIGN KEY (framework_pack_id) REFERENCES compliance_framework_packs(id) ON DELETE SET NULL;

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

        CREATE TABLE IF NOT EXISTS compliance_review_packages (
            review_package_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            created_by_user_id TEXT NOT NULL,
            mapping_id TEXT NOT NULL REFERENCES compliance_evidence_mappings(mapping_id) ON DELETE RESTRICT,
            evidence_export_id TEXT NOT NULL REFERENCES compliance_evidence_exports(export_id) ON DELETE RESTRICT,
            evidence_export_hash TEXT NOT NULL,
            mapping_hash TEXT NOT NULL,
            framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE RESTRICT,
            framework_version TEXT NOT NULL,
            format TEXT NOT NULL CHECK (format IN ('json')),
            artifact_hash TEXT NOT NULL,
            payload_json_redacted JSONB NOT NULL,
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE,
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE,
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE,
            certification BOOLEAN NOT NULL DEFAULT FALSE,
            review_status TEXT NOT NULL DEFAULT 'needs_review'
                CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')),
            reviewed_by_user_id TEXT,
            reviewed_at TIMESTAMPTZ,
            review_notes_safe TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            downloaded_at TIMESTAMPTZ,
            error_message_safe TEXT,
            CHECK (review_package_id LIKE 'crp_%'),
            CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (evidence_export_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (mapping_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (compliance_claim = FALSE),
            CHECK (regulatory_claim = FALSE),
            CHECK (requires_auditor_review = TRUE),
            CHECK (certification = FALSE)
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_review_packages_org_created
            ON compliance_review_packages(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_review_packages_mapping
            ON compliance_review_packages(org_id, mapping_id);

        CREATE TABLE IF NOT EXISTS compliance_framework_review_reports (
            report_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            created_by_user_id TEXT NOT NULL,
            mapping_id TEXT NOT NULL REFERENCES compliance_evidence_mappings(mapping_id) ON DELETE RESTRICT,
            review_package_id TEXT NOT NULL REFERENCES compliance_review_packages(review_package_id) ON DELETE RESTRICT,
            evidence_export_id TEXT NOT NULL REFERENCES compliance_evidence_exports(export_id) ON DELETE RESTRICT,
            evidence_export_hash TEXT NOT NULL,
            mapping_hash TEXT NOT NULL,
            review_package_hash TEXT NOT NULL,
            framework_id TEXT NOT NULL REFERENCES compliance_control_frameworks(framework_id) ON DELETE RESTRICT,
            framework_version TEXT NOT NULL,
            framework_owner_type TEXT NOT NULL CHECK (framework_owner_type IN ('gitgov', 'customer')),
            framework_review_status TEXT CHECK (
                framework_review_status IS NULL
                OR framework_review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected', 'archived')
            ),
            pack_hash TEXT,
            format TEXT NOT NULL CHECK (format IN ('json')),
            artifact_hash TEXT NOT NULL,
            payload_json_redacted JSONB NOT NULL,
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE,
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE,
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE,
            certification BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            downloaded_at TIMESTAMPTZ,
            error_message_safe TEXT,
            CHECK (report_id LIKE 'frr_%'),
            CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (evidence_export_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (mapping_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (review_package_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (pack_hash IS NULL OR pack_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (compliance_claim = FALSE),
            CHECK (regulatory_claim = FALSE),
            CHECK (requires_auditor_review = TRUE),
            CHECK (certification = FALSE)
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_org_created
            ON compliance_framework_review_reports(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_mapping
            ON compliance_framework_review_reports(org_id, mapping_id);

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_package
            ON compliance_framework_review_reports(org_id, review_package_id);

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_created
            ON compliance_framework_review_reports(org_id, framework_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_mapping
            ON compliance_framework_review_reports(org_id, framework_id, mapping_id);

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_framework_package
            ON compliance_framework_review_reports(org_id, framework_id, review_package_id);

        ALTER TABLE compliance_framework_review_reports
            ADD COLUMN IF NOT EXISTS review_status TEXT NOT NULL DEFAULT 'needs_review'
                CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')),
            ADD COLUMN IF NOT EXISTS reviewed_by_user_id TEXT,
            ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
            ADD COLUMN IF NOT EXISTS review_notes_safe TEXT;

        CREATE INDEX IF NOT EXISTS idx_compliance_framework_review_reports_review_status
            ON compliance_framework_review_reports(org_id, review_status, created_at DESC);

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
            review_status_suggestion TEXT CHECK (
                review_status_suggestion IS NULL
                OR review_status_suggestion IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')
            ),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE INDEX IF NOT EXISTS idx_cfr_report_comments_report_created
            ON compliance_framework_review_report_comments(org_id, report_id, created_at ASC);

        CREATE TABLE IF NOT EXISTS compliance_framework_review_report_manifests (
            manifest_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
            generated_by_user_id TEXT NOT NULL,
            manifest_hash TEXT NOT NULL,
            previous_manifest_hash TEXT,
            signature_algorithm TEXT NOT NULL DEFAULT 'sha256-provenance-manifest-v1',
            payload_json_redacted JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (manifest_id ~ '^frrm_[0-9a-f]{32}$'),
            CHECK (manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
            CHECK (previous_manifest_hash IS NULL OR previous_manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
            CHECK (signature_algorithm = 'sha256-provenance-manifest-v1')
        );

        CREATE INDEX IF NOT EXISTS idx_cfr_report_manifests_report_created
            ON compliance_framework_review_report_manifests(org_id, report_id, created_at DESC);

        CREATE UNIQUE INDEX IF NOT EXISTS idx_cfr_report_manifests_hash
            ON compliance_framework_review_report_manifests(org_id, manifest_hash);

        CREATE TABLE IF NOT EXISTS compliance_framework_review_report_pdf_exports (
            pdf_export_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            report_id TEXT NOT NULL REFERENCES compliance_framework_review_reports(report_id) ON DELETE CASCADE,
            manifest_id TEXT NOT NULL REFERENCES compliance_framework_review_report_manifests(manifest_id) ON DELETE RESTRICT,
            created_by_user_id TEXT NOT NULL,
            source_report_hash TEXT NOT NULL,
            manifest_hash TEXT NOT NULL,
            pdf_artifact_hash TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'application/pdf',
            page_count INTEGER NOT NULL DEFAULT 1 CHECK (page_count BETWEEN 1 AND 200),
            pdf_bytes BYTEA NOT NULL,
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
            certification BOOLEAN NOT NULL DEFAULT FALSE CHECK (certification = FALSE),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            downloaded_at TIMESTAMPTZ,
            CHECK (pdf_export_id LIKE 'frrpdf_%'),
            CHECK (source_report_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (manifest_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (pdf_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (content_type = 'application/pdf')
        );

        CREATE INDEX IF NOT EXISTS idx_cfr_report_pdf_exports_report_created
            ON compliance_framework_review_report_pdf_exports(org_id, report_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_cfr_report_pdf_exports_manifest
            ON compliance_framework_review_report_pdf_exports(org_id, manifest_id);

        CREATE TABLE IF NOT EXISTS compliance_period_reports (
            period_report_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            created_by_user_id TEXT NOT NULL,
            framework_id TEXT,
            date_range_start TIMESTAMPTZ NOT NULL,
            date_range_end TIMESTAMPTZ NOT NULL,
            report_count INTEGER NOT NULL CHECK (report_count > 0),
            source_report_ids JSONB NOT NULL,
            format TEXT NOT NULL DEFAULT 'json',
            status TEXT NOT NULL DEFAULT 'generated',
            artifact_hash TEXT NOT NULL,
            payload_json_redacted JSONB NOT NULL,
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
            certification BOOLEAN NOT NULL DEFAULT FALSE CHECK (certification = FALSE),
            review_status TEXT NOT NULL DEFAULT 'needs_review'
                CHECK (review_status IN ('needs_review', 'reviewed', 'needs_changes', 'rejected')),
            reviewed_by_user_id TEXT,
            reviewed_at TIMESTAMPTZ,
            review_notes_safe TEXT CHECK (
                review_notes_safe IS NULL
                OR (
                    char_length(review_notes_safe) <= 1000
                    AND review_notes_safe !~* '(<script|</|<iframe|bearer |ghp_|glpat-|sk-)'
                )
            ),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            retention_status TEXT NOT NULL DEFAULT 'active'
                CHECK (retention_status IN ('active', 'archived', 'retention_expired')),
            retention_until TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 years'),
            download_count INTEGER NOT NULL DEFAULT 0 CHECK (download_count >= 0),
            last_downloaded_at TIMESTAMPTZ,
            archived_at TIMESTAMPTZ,
            downloaded_at TIMESTAMPTZ,
            error_message_safe TEXT,
            CHECK (
                (retention_status = 'archived' AND archived_at IS NOT NULL)
                OR retention_status <> 'archived'
            ),
            CHECK (
                review_status NOT IN ('needs_changes', 'rejected')
                OR review_notes_safe IS NOT NULL
            ),
            CHECK (period_report_id LIKE 'cpr_%'),
            CHECK (date_range_end > date_range_start),
            CHECK (jsonb_typeof(source_report_ids) = 'array'),
            CHECK (jsonb_array_length(source_report_ids) = report_count),
            CHECK (format = 'json'),
            CHECK (status IN ('generated', 'failed')),
            CHECK (artifact_hash ~ '^sha256:[a-f0-9]{64}$')
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_org_created
            ON compliance_period_reports(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_framework_created
            ON compliance_period_reports(org_id, framework_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_date_range
            ON compliance_period_reports(org_id, date_range_start, date_range_end);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_artifact_hash
            ON compliance_period_reports(org_id, artifact_hash);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_retention
            ON compliance_period_reports(org_id, retention_status, retention_until);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_reports_org_review_status
            ON compliance_period_reports(org_id, review_status, created_at DESC);

        CREATE TABLE IF NOT EXISTS compliance_period_report_access_log (
            access_log_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
            actor_client_id TEXT NOT NULL,
            action TEXT NOT NULL CHECK (
                action IN ('viewed', 'downloaded_json', 'downloaded_pdf', 'archived', 'retention_updated', 'manifest_created', 'manifest_downloaded', 'review_updated')
            ),
            artifact_type TEXT NOT NULL CHECK (artifact_type IN ('metadata', 'json', 'pdf', 'retention', 'manifest', 'review')),
            artifact_id TEXT,
            artifact_hash TEXT CHECK (artifact_hash IS NULL OR artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (access_log_id LIKE 'cprlog_%'),
            CHECK (jsonb_typeof(metadata) = 'object')
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_period_report_access_log_report_created
            ON compliance_period_report_access_log(org_id, period_report_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_report_access_log_actor_created
            ON compliance_period_report_access_log(org_id, actor_client_id, created_at DESC);

        CREATE TABLE IF NOT EXISTS compliance_period_report_pdf_exports (
            pdf_export_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
            created_by_user_id TEXT NOT NULL,
            source_period_report_hash TEXT NOT NULL,
            pdf_artifact_hash TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'application/pdf',
            page_count INTEGER NOT NULL DEFAULT 1 CHECK (page_count BETWEEN 1 AND 200),
            pdf_bytes BYTEA NOT NULL,
            compliance_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (compliance_claim = FALSE),
            regulatory_claim BOOLEAN NOT NULL DEFAULT FALSE CHECK (regulatory_claim = FALSE),
            requires_auditor_review BOOLEAN NOT NULL DEFAULT TRUE CHECK (requires_auditor_review = TRUE),
            certification BOOLEAN NOT NULL DEFAULT FALSE CHECK (certification = FALSE),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            downloaded_at TIMESTAMPTZ,
            CHECK (pdf_export_id LIKE 'cprpdf_%'),
            CHECK (source_period_report_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (pdf_artifact_hash ~ '^sha256:[a-f0-9]{64}$'),
            CHECK (content_type = 'application/pdf')
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_period_report_pdf_exports_report_created
            ON compliance_period_report_pdf_exports(org_id, period_report_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_compliance_period_report_pdf_exports_hash
            ON compliance_period_report_pdf_exports(org_id, pdf_artifact_hash);

        CREATE TABLE IF NOT EXISTS compliance_period_report_manifests (
            manifest_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            period_report_id TEXT NOT NULL REFERENCES compliance_period_reports(period_report_id) ON DELETE CASCADE,
            generated_by_user_id TEXT NOT NULL,
            manifest_hash TEXT NOT NULL,
            previous_manifest_hash TEXT,
            signature_algorithm TEXT NOT NULL DEFAULT 'sha256-period-report-provenance-manifest-v1',
            payload_json_redacted JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (manifest_id ~ '^cprm_[0-9a-f]{32}$'),
            CHECK (manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
            CHECK (previous_manifest_hash IS NULL OR previous_manifest_hash ~ '^sha256:[0-9a-f]{64}$'),
            CHECK (signature_algorithm = 'sha256-period-report-provenance-manifest-v1'),
            CHECK (
                payload_json_redacted ? 'schema_version'
                AND payload_json_redacted ? 'hash_chain'
                AND payload_json_redacted ? 'claims'
                AND payload_json_redacted ? 'audit_metadata'
                AND COALESCE((payload_json_redacted #>> '{claims,compliance_claim}')::boolean, true) = false
                AND COALESCE((payload_json_redacted #>> '{claims,regulatory_claim}')::boolean, true) = false
                AND COALESCE((payload_json_redacted #>> '{claims,certification}')::boolean, true) = false
                AND COALESCE((payload_json_redacted #>> '{audit_metadata,agent_governance_required}')::boolean, true) = false
                AND COALESCE((payload_json_redacted #>> '{audit_metadata,source_period_report_artifact_mutated}')::boolean, true) = false
            )
        );

        CREATE INDEX IF NOT EXISTS idx_compliance_period_report_manifests_report_created
            ON compliance_period_report_manifests(org_id, period_report_id, created_at DESC);

        CREATE UNIQUE INDEX IF NOT EXISTS idx_compliance_period_report_manifests_hash
            ON compliance_period_report_manifests(org_id, manifest_hash);

        INSERT INTO compliance_control_frameworks (
            framework_id,
            name,
            version,
            description,
            is_regulatory,
            is_active,
            owner_type,
            owner_name,
            source,
            is_gitgov_owned,
            official_regulatory_mapping
        )
        VALUES (
            'gitgov_release_governance_baseline_v1',
            'GitGov Release Governance Baseline',
            '1.0.0',
            'GitGov-owned, non-regulatory evidence baseline for reviewing release governance controls.',
            FALSE,
            TRUE,
            'gitgov',
            'GitGov',
            'gitgov_owned',
            TRUE,
            FALSE
        )
        ON CONFLICT (framework_id) DO UPDATE SET
            name = EXCLUDED.name,
            version = EXCLUDED.version,
            description = EXCLUDED.description,
            is_regulatory = FALSE,
            is_active = TRUE,
            owner_type = 'gitgov',
            owner_name = 'GitGov',
            source = 'gitgov_owned',
            is_gitgov_owned = TRUE,
            official_regulatory_mapping = FALSE;

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
            ('gg-rg-01', 'gitgov_release_governance_baseline_v1', 'GG-RG-01', 'Deployment gate decision recorded', 'The release evidence contains a Deployment Gate decision.', '["deployment_gate.decision"]'::jsonb, 10),
            ('gg-rg-02', 'gitgov_release_governance_baseline_v1', 'GG-RG-02', 'Policy source and checksum recorded', 'The release evidence records the policy source and checksum used for the gate decision.', '["policy.source","policy.checksum"]'::jsonb, 20),
            ('gg-rg-03', 'gitgov_release_governance_baseline_v1', 'GG-RG-03', 'Human approval evidence captured when required', 'The release evidence shows required human release approvals when policy requires approval.', '["release_approval"]'::jsonb, 30),
            ('gg-rg-04', 'gitgov_release_governance_baseline_v1', 'GG-RG-04', 'CI/build evidence captured', 'The release evidence references CI or build execution evidence.', '["ci_build_evidence"]'::jsonb, 40),
            ('gg-rg-05', 'gitgov_release_governance_baseline_v1', 'GG-RG-05', 'Code review or PR evidence captured', 'The release evidence references code change and review evidence where available.', '["code_change_evidence","pr_review_evidence"]'::jsonb, 50),
            ('gg-rg-06', 'gitgov_release_governance_baseline_v1', 'GG-RG-06', 'Security or quality evidence captured', 'The release evidence references security or quality gate evidence.', '["quality_gate_result"]'::jsonb, 60),
            ('gg-rg-07', 'gitgov_release_governance_baseline_v1', 'GG-RG-07', 'Deployment target and environment recorded', 'The release evidence records repository, branch, target SHA, and environment.', '["deployment_target"]'::jsonb, 70),
            ('gg-rg-08', 'gitgov_release_governance_baseline_v1', 'GG-RG-08', 'Missing evidence and gaps are explicit', 'The release evidence exposes missing evidence and gaps instead of hiding them.', '["missing_evidence"]'::jsonb, 80),
            ('gg-rg-09', 'gitgov_release_governance_baseline_v1', 'GG-RG-09', 'Audit trail exists', 'The release evidence includes audit timestamps and redaction markers.', '["audit_trail"]'::jsonb, 90),
            ('gg-rg-10', 'gitgov_release_governance_baseline_v1', 'GG-RG-10', 'Agent Governance not required for manual-first gate evidence', 'The release evidence confirms Deployment Gates work without requiring Agent Governance.', '["deployment_gate.agent_governance_used"]'::jsonb, 100)
        ON CONFLICT (framework_id, control_id) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            required_evidence_types = EXCLUDED.required_evidence_types,
            sort_order = EXCLUDED.sort_order;

        CREATE TABLE IF NOT EXISTS deployment_gate_break_glass_approvals (
            approval_id TEXT PRIMARY KEY,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            release_id TEXT NOT NULL,
            repository_full_name TEXT NOT NULL,
            branch TEXT NOT NULL,
            target_sha TEXT NOT NULL,
            environment TEXT NOT NULL,
            ticket_id TEXT,
            evidence_packet_hash TEXT NOT NULL,
            evidence_packet_uri TEXT,
            reason TEXT NOT NULL,
            approver TEXT NOT NULL,
            approver_role TEXT NOT NULL DEFAULT 'incident_commander'
                CHECK (approver_role IN ('incident_commander', 'security', 'release_manager', 'platform_admin')),
            expires_at TIMESTAMPTZ NOT NULL,
            approval_hash TEXT NOT NULL UNIQUE,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            CHECK (approval_id LIKE 'dgbga_%'),
            CHECK (length(trim(reason)) >= 16)
        );

        CREATE INDEX IF NOT EXISTS idx_deployment_gate_break_glass_approvals_scope
            ON deployment_gate_break_glass_approvals(
                org_id,
                repository_full_name,
                branch,
                environment,
                target_sha,
                evidence_packet_hash,
                expires_at DESC
            );

        CREATE TABLE IF NOT EXISTS agent_governance_settings (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            mode TEXT NOT NULL DEFAULT 'manual_only'
                CHECK (mode IN ('manual_only', 'opt_in_enabled')),
            payload_mode TEXT NOT NULL DEFAULT 'minimized'
                CHECK (payload_mode IN ('minimized')),
            reason TEXT,
            updated_by TEXT NOT NULL DEFAULT 'system',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            CHECK (
                (enabled = FALSE AND mode = 'manual_only')
                OR
                (enabled = TRUE AND mode = 'opt_in_enabled')
            )
        );

        CREATE INDEX IF NOT EXISTS idx_agent_governance_settings_enabled
            ON agent_governance_settings(enabled, updated_at DESC);

        CREATE TABLE IF NOT EXISTS agent_governance_agent_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            key_id TEXT NOT NULL UNIQUE,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            token_hash TEXT NOT NULL UNIQUE,
            token_prefix TEXT NOT NULL DEFAULT 'ggag_',
            token_last4 TEXT NOT NULL,
            token_preview TEXT NOT NULL,
            display_name TEXT NOT NULL,
            description TEXT,
            environment TEXT,
            scopes JSONB NOT NULL DEFAULT '["agent_governance:evaluate"]'::jsonb,
            allowed_actions JSONB NOT NULL DEFAULT '["commit","push","open_pr","merge_pr","deploy"]'::jsonb,
            expires_at TIMESTAMPTZ,
            last_used_at TIMESTAMPTZ,
            revoked_at TIMESTAMPTZ,
            rotated_at TIMESTAMPTZ,
            rotated_from_key_id TEXT,
            replaced_by_key_id TEXT,
            rotation_reason TEXT,
            created_by TEXT NOT NULL,
            revoked_by TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (key_id LIKE 'agk_%'),
            CHECK (token_prefix = 'ggag_'),
            CHECK (jsonb_typeof(scopes) = 'array'),
            CHECK (jsonb_typeof(allowed_actions) = 'array')
        );

        CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_org_created
            ON agent_governance_agent_keys(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_active
            ON agent_governance_agent_keys(org_id, revoked_at, expires_at);

        CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_rotation_from
            ON agent_governance_agent_keys(org_id, rotated_from_key_id)
            WHERE rotated_from_key_id IS NOT NULL;

        CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_replaced_by
            ON agent_governance_agent_keys(org_id, replaced_by_key_id)
            WHERE replaced_by_key_id IS NOT NULL;

        CREATE INDEX IF NOT EXISTS idx_agent_governance_agent_keys_expiry
            ON agent_governance_agent_keys(org_id, expires_at)
            WHERE revoked_at IS NULL AND expires_at IS NOT NULL;

        CREATE TABLE IF NOT EXISTS agent_governance_evaluations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            evaluation_id TEXT NOT NULL UNIQUE,
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            agent_id TEXT NOT NULL,
            agent_type TEXT NOT NULL DEFAULT 'unknown',
            actor TEXT NOT NULL,
            action TEXT NOT NULL CHECK (action IN ('commit', 'push', 'open_pr', 'merge_pr', 'change_policy', 'deploy')),
            repository_full_name TEXT NOT NULL,
            branch TEXT,
            target_sha TEXT,
            environment TEXT,
            ticket_id TEXT,
            operation_id TEXT,
            decision TEXT NOT NULL CHECK (decision IN ('allowed', 'requires_approval', 'blocked')),
            allowed BOOLEAN NOT NULL,
            requires_approval BOOLEAN NOT NULL,
            reason TEXT NOT NULL,
            reasons JSONB NOT NULL DEFAULT '[]'::jsonb,
            required_evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
            policy_id TEXT NOT NULL,
            policy_checksum TEXT NOT NULL,
            evaluation JSONB NOT NULL DEFAULT '{}'::jsonb,
            request_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            principal_type TEXT,
            agent_key_id TEXT,
            agent_display_name TEXT,
            attribution_id TEXT,
            correlation_id TEXT,
            parent_correlation_id TEXT,
            session_id TEXT,
            tool_name TEXT,
            tool_version TEXT,
            agent_name TEXT,
            external_run_id TEXT,
            consumer_type TEXT DEFAULT 'agent_governance',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            CHECK (evaluation_id LIKE 'agv_%'),
            CHECK (attribution_id IS NULL OR attribution_id LIKE 'attr_%'),
            CHECK (correlation_id IS NULL OR length(correlation_id) BETWEEN 1 AND 128),
            CHECK (consumer_type IS NULL OR consumer_type IN ('agent_governance', 'agent_dry_run')),
            CHECK (
                (decision = 'allowed' AND allowed = TRUE AND requires_approval = FALSE)
                OR
                (decision = 'requires_approval' AND allowed = FALSE AND requires_approval = TRUE)
                OR
                (decision = 'blocked' AND allowed = FALSE AND requires_approval = FALSE)
            )
        );

        CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_org_created
            ON agent_governance_evaluations(org_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_scope
            ON agent_governance_evaluations(
                org_id,
                repository_full_name,
                action,
                decision,
                created_at DESC
            );

        CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_agent
            ON agent_governance_evaluations(org_id, agent_id, created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_correlation
            ON agent_governance_evaluations(org_id, correlation_id, created_at DESC)
            WHERE correlation_id IS NOT NULL;

        CREATE INDEX IF NOT EXISTS idx_agent_governance_evaluations_session
            ON agent_governance_evaluations(org_id, session_id, created_at DESC)
            WHERE session_id IS NOT NULL;

        CREATE TABLE IF NOT EXISTS release_evidence_packets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            ticket_id TEXT NOT NULL,
            release_id TEXT NOT NULL,
            repository_full_name TEXT NOT NULL,
            branch TEXT NOT NULL,
            target_sha TEXT NOT NULL,
            environment TEXT NOT NULL,
            evidence_packet_hash TEXT NOT NULL,
            evidence_packet_uri TEXT NOT NULL,
            packet JSONB NOT NULL,
            generated_by TEXT NOT NULL,
            generated_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE (org_id, evidence_packet_hash)
        );

        CREATE INDEX IF NOT EXISTS idx_release_evidence_packets_binding
            ON release_evidence_packets(
                org_id,
                repository_full_name,
                release_id,
                environment,
                branch,
                target_sha,
                evidence_packet_hash
            );

        CREATE TABLE IF NOT EXISTS repos (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            github_id BIGINT UNIQUE,
            full_name TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            private BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            key_hash TEXT UNIQUE NOT NULL,
            client_id TEXT NOT NULL,
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            role TEXT NOT NULL DEFAULT 'Developer',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            last_used TIMESTAMPTZ,
            is_active BOOLEAN DEFAULT TRUE
        );

        CREATE TABLE IF NOT EXISTS platform_principals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id TEXT NOT NULL UNIQUE,
            principal_type TEXT NOT NULL DEFAULT 'platform_founder'
                CHECK (principal_type IN ('platform_founder', 'platform_operator', 'platform_auditor')),
            status TEXT NOT NULL DEFAULT 'active'
                CHECK (status IN ('active', 'disabled', 'break_glass')),
            display_name TEXT,
            email TEXT,
            auth_method TEXT NOT NULL DEFAULT 'api_key'
                CHECK (auth_method IN ('api_key', 'sso', 'oidc', 'break_glass')),
            external_subject TEXT,
            metadata JSONB NOT NULL DEFAULT '{}',
            last_authenticated_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS client_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            event_uuid TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            user_login TEXT NOT NULL,
            user_name TEXT,
            branch TEXT,
            commit_sha TEXT,
            files JSONB DEFAULT '[]',
            status TEXT NOT NULL,
            reason TEXT,
            metadata JSONB DEFAULT '{}',
            client_version TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            synced_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS github_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            delivery_id TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            actor_login TEXT,
            actor_id BIGINT,
            ref_name TEXT,
            ref_type TEXT,
            before_sha TEXT,
            after_sha TEXT,
            commit_shas JSONB DEFAULT '[]',
            commits_count INTEGER DEFAULT 0,
            payload JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            processed_at TIMESTAMPTZ
        );

        CREATE TABLE IF NOT EXISTS violations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            github_event_id UUID REFERENCES github_events(id),
            client_event_id UUID REFERENCES client_events(id),
            violation_type TEXT NOT NULL,
            severity TEXT DEFAULT 'warning',
            confidence_level TEXT DEFAULT 'pending',
            reason TEXT,
            user_login TEXT,
            branch TEXT,
            commit_sha TEXT,
            details JSONB DEFAULT '{}',
            correlated_github_event_id UUID REFERENCES github_events(id),
            correlated_client_event_id UUID REFERENCES client_events(id),
            resolved BOOLEAN DEFAULT FALSE,
            resolved_at TIMESTAMPTZ,
            resolved_by TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policies (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE UNIQUE,
            config JSONB NOT NULL,
            checksum TEXT NOT NULL,
            source_metadata JSONB NOT NULL DEFAULT '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb,
            override_actor TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS webhook_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            delivery_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            signature TEXT,
            payload JSONB NOT NULL,
            payload_sha256 TEXT,
            processed BOOLEAN DEFAULT FALSE,
            error TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            processed_at TIMESTAMPTZ
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_webhook_events_payload_sha256
            ON webhook_events (payload_sha256);

        CREATE TABLE IF NOT EXISTS pipeline_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            pipeline_id TEXT NOT NULL,
            job_name TEXT NOT NULL,
            status TEXT NOT NULL,
            branch TEXT,
            commit_sha TEXT,
            repo_full_name TEXT,
            duration_ms BIGINT,
            triggered_by TEXT,
            stages JSONB DEFAULT '[]',
            artifacts JSONB DEFAULT '[]',
            payload JSONB DEFAULT '{}',
            ingested_at TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_pipeline_events_dedupe
            ON pipeline_events (pipeline_id, job_name, (COALESCE(commit_sha, '')), ingested_at);

        CREATE TABLE IF NOT EXISTS project_tickets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            ticket_id TEXT NOT NULL,
            project_key TEXT NOT NULL,
            ticket_url TEXT,
            title TEXT,
            status TEXT,
            assignee TEXT,
            reporter TEXT,
            ticket_type TEXT,
            priority TEXT,
            labels JSONB DEFAULT '[]',
            related_commits TEXT[] NOT NULL DEFAULT '{}',
            related_prs TEXT[] NOT NULL DEFAULT '{}',
            related_branches TEXT[] NOT NULL DEFAULT '{}',
            raw_payload JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_project_tickets_org_ticket
            ON project_tickets(org_id, ticket_id);

        CREATE TABLE IF NOT EXISTS commit_ticket_correlations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            commit_sha TEXT NOT NULL,
            ticket_id TEXT NOT NULL,
            correlation_source TEXT NOT NULL,
            confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_commit_ticket_unique_org
            ON commit_ticket_correlations (
                COALESCE(org_id, '00000000-0000-0000-0000-000000000000'::uuid),
                commit_sha,
                ticket_id
            );

        CREATE TABLE IF NOT EXISTS export_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            requested_by TEXT NOT NULL,
            format TEXT NOT NULL,
            filters JSONB DEFAULT '{}',
            event_count INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS cli_commands (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id),
            user_login TEXT NOT NULL,
            command TEXT NOT NULL,
            origin TEXT NOT NULL DEFAULT 'manual_input',
            branch TEXT NOT NULL DEFAULT '',
            repo_name TEXT,
            exit_code INTEGER,
            duration_ms BIGINT,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT cli_commands_origin_check
                CHECK (origin IN ('button_click', 'manual_input'))
        );
        CREATE INDEX IF NOT EXISTS idx_cli_commands_org_created
            ON cli_commands(org_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_cli_commands_user_created
            ON cli_commands(user_login, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_cli_commands_repo
            ON cli_commands(repo_name, created_at DESC);

        CREATE TABLE IF NOT EXISTS governance_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            event_type TEXT NOT NULL,
            actor TEXT,
            repo TEXT,
            branch TEXT,
            details JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS pr_merges (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            repo TEXT NOT NULL,
            pr_number INTEGER NOT NULL,
            title TEXT,
            author TEXT,
            merged_by TEXT,
            base_branch TEXT,
            head_branch TEXT,
            commit_sha TEXT,
            reviewers JSONB DEFAULT '[]',
            approved_by JSONB DEFAULT '[]',
            review_count INTEGER DEFAULT 0,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
            url TEXT,
            merged_at TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS pull_request_merges (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id),
            repo_id UUID REFERENCES repos(id),
            delivery_id TEXT NOT NULL UNIQUE,
            pr_number INT NOT NULL,
            pr_title TEXT,
            author_login TEXT,
            merged_by_login TEXT,
            head_sha TEXT,
            base_branch TEXT,
            payload JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS admin_audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            actor_client_id TEXT NOT NULL,
            action TEXT NOT NULL,
            target_type TEXT,
            target_id TEXT,
            metadata JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS client_sessions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id TEXT NOT NULL,
            org_id UUID,
            app_version TEXT,
            os TEXT,
            hostname TEXT,
            last_seen TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(client_id)
        );

        CREATE TABLE IF NOT EXISTS identity_aliases (
            canonical_login TEXT NOT NULL,
            alias_login TEXT NOT NULL,
            org_id UUID REFERENCES orgs(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (canonical_login, alias_login),
            UNIQUE (alias_login)
        );

        CREATE TABLE IF NOT EXISTS noncompliance_signals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            signal_type TEXT NOT NULL,
            severity TEXT DEFAULT 'medium',
            status TEXT DEFAULT 'open',
            description TEXT,
            evidence JSONB DEFAULT '{}',
            user_login TEXT,
            repo TEXT,
            branch TEXT,
            commit_sha TEXT,
            detected_at TIMESTAMPTZ DEFAULT NOW(),
            reviewed_at TIMESTAMPTZ,
            reviewed_by TEXT,
            resolution TEXT,
            violation_id UUID,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS violation_decisions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            violation_id UUID NOT NULL,
            actor TEXT NOT NULL,
            decision TEXT NOT NULL,
            reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_history (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            repo_id TEXT NOT NULL,
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            config JSONB,
            checksum TEXT,
            source_metadata JSONB NOT NULL DEFAULT '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb,
            changed_by TEXT,
            change_type TEXT,
            previous_checksum TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_drift_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            user_login TEXT NOT NULL,
            action TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            result TEXT NOT NULL,
            before_checksum TEXT,
            after_checksum TEXT,
            duration_ms BIGINT,
            metadata JSONB DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_change_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            repo_name TEXT NOT NULL,
            requested_by TEXT NOT NULL,
            requested_config JSONB NOT NULL,
            requested_checksum TEXT NOT NULL,
            source_metadata JSONB NOT NULL DEFAULT '{"source_mode":"control-plane-managed","reviewers":[],"drift_status":"unknown"}'::jsonb,
            reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_change_request_decisions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            request_id UUID UNIQUE REFERENCES policy_change_requests(id) ON DELETE CASCADE,
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            decision TEXT NOT NULL,
            decided_by TEXT NOT NULL,
            note TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            job_type TEXT NOT NULL,
            payload JSONB DEFAULT '{}',
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER DEFAULT 0,
            max_attempts INTEGER DEFAULT 3,
            error TEXT,
            worker_id TEXT,
            locked_at TIMESTAMPTZ,
            completed_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS org_users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL,
            login TEXT NOT NULL,
            display_name TEXT,
            email TEXT,
            role TEXT NOT NULL DEFAULT 'Developer',
            status TEXT NOT NULL DEFAULT 'active',
            created_by TEXT,
            updated_by TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(org_id, login)
        );

        CREATE TABLE IF NOT EXISTS org_invitations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL,
            invite_email TEXT,
            invite_login TEXT,
            role TEXT NOT NULL DEFAULT 'Developer',
            token_hash TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            invited_by TEXT NOT NULL,
            accepted_by TEXT,
            expires_at TIMESTAMPTZ NOT NULL,
            accepted_at TIMESTAMPTZ,
            revoked_by TEXT,
            revoked_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS feature_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_login TEXT NOT NULL,
            org_id TEXT,
            title TEXT NOT NULL,
            description TEXT,
            category TEXT DEFAULT 'general',
            priority TEXT DEFAULT 'normal',
            status TEXT DEFAULT 'open',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Indexes for performance
        CREATE INDEX IF NOT EXISTS idx_client_events_uuid ON client_events(event_uuid);
        CREATE INDEX IF NOT EXISTS idx_client_events_created ON client_events(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_client_events_type ON client_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_client_events_user ON client_events(user_login);
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);

        -- Minimal stats function mirroring the production get_audit_stats JSON shape.
        CREATE OR REPLACE FUNCTION get_audit_stats(p_org_id UUID DEFAULT NULL)
        RETURNS JSON AS $func$
            SELECT json_build_object(
                'github_events', json_build_object(
                    'total', (SELECT COUNT(*) FROM github_events WHERE p_org_id IS NULL OR org_id = p_org_id),
                    'today', (SELECT COUNT(*) FROM github_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= date_trunc('day', NOW())),
                    'pushes_today', 0,
                    'by_type', '{}'::json
                ),
                'client_events', json_build_object(
                    'total', (SELECT COUNT(*) FROM client_events WHERE p_org_id IS NULL OR org_id = p_org_id),
                    'today', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= date_trunc('day', NOW())),
                    'blocked_today', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND status = 'blocked' AND created_at >= date_trunc('day', NOW())),
                    'desktop_pushes_today', (SELECT COUNT(*) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND event_type = 'successful_push' AND created_at >= date_trunc('day', NOW())),
                    'by_type', '{}'::json,
                    'by_status', '{}'::json
                ),
                'violations', json_build_object(
                    'total', (SELECT COUNT(*) FROM violations WHERE p_org_id IS NULL OR org_id = p_org_id),
                    'unresolved', (SELECT COUNT(*) FROM violations WHERE (p_org_id IS NULL OR org_id = p_org_id) AND resolved = FALSE),
                    'critical', (SELECT COUNT(*) FROM violations WHERE (p_org_id IS NULL OR org_id = p_org_id) AND severity = 'critical')
                ),
                'active_devs_week', (SELECT COUNT(DISTINCT user_login) FROM client_events WHERE (p_org_id IS NULL OR org_id = p_org_id) AND created_at >= NOW() - INTERVAL '7 days'),
                'active_repos', (SELECT COUNT(*) FROM repos WHERE p_org_id IS NULL OR org_id = p_org_id)
            );
        $func$ LANGUAGE sql STABLE;
    "#;

    sqlx::raw_sql(ddl)
        .execute(&test_pool)
        .await
        .expect("apply test DDL");

    Some((test_pool, schema, admin_pool))
}

/// Drop the test schema after the test. Uses admin_pool (no search_path override).
pub(super) async fn teardown(admin_pool: &PgPool, schema: &str) {
    let _ = sqlx::query(&format!("DROP SCHEMA \"{}\" CASCADE", schema))
        .execute(admin_pool)
        .await;
}

/// Insert a test API key into the database. Returns the raw key.
pub(super) async fn insert_test_api_key(pool: &PgPool, client_id: &str, role: &str) -> String {
    let raw_key = format!("test-key-{}", uuid::Uuid::new_v4());
    let hash = format!("{:x}", sha2::Sha256::digest(raw_key.as_bytes()));
    sqlx::query(
        "INSERT INTO api_keys (key_hash, client_id, role, is_active) VALUES ($1, $2, $3, true)",
    )
    .bind(&hash)
    .bind(client_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("insert test API key");
    raw_key
}

pub(super) async fn insert_platform_founder_principal(pool: &PgPool, client_id: &str) -> String {
    let row = sqlx::query(
        r#"
        INSERT INTO platform_principals (
            client_id,
            principal_type,
            status,
            display_name,
            auth_method,
            metadata
        )
        VALUES (
            $1,
            'platform_founder',
            'active',
            'Test Platform Founder',
            'api_key',
            '{"source":"integration_test"}'::jsonb
        )
        ON CONFLICT (client_id) DO UPDATE SET
            principal_type = 'platform_founder',
            status = 'active',
            auth_method = 'api_key'
        RETURNING id::text
        "#,
    )
    .bind(client_id)
    .fetch_one(pool)
    .await
    .expect("insert platform founder principal");

    row.get("id")
}

pub(super) async fn insert_test_api_key_for_org(
    pool: &PgPool,
    client_id: &str,
    role: &str,
    org_id: &str,
) -> String {
    let raw_key = format!("test-key-{}", uuid::Uuid::new_v4());
    let hash = format!("{:x}", sha2::Sha256::digest(raw_key.as_bytes()));
    sqlx::query(
        "INSERT INTO api_keys (key_hash, client_id, role, org_id, is_active) VALUES ($1, $2, $3, $4::uuid, true)",
    )
    .bind(&hash)
    .bind(client_id)
    .bind(role)
    .bind(org_id)
    .execute(pool)
    .await
    .expect("insert org-scoped test API key");
    raw_key
}

pub(super) async fn insert_test_org(pool: &PgPool, login: &str) -> String {
    let org_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind(login)
        .bind(format!("{} Org", login))
        .execute(pool)
        .await
        .expect("insert test org");
    org_id
}

/// Insert a minimal org + repo for policy endpoints.
pub(super) async fn insert_test_repo(pool: &PgPool, full_name: &str) -> (String, String) {
    let org_id = uuid::Uuid::new_v4().to_string();
    let repo_id = uuid::Uuid::new_v4().to_string();
    let org_login = full_name
        .split('/')
        .next()
        .map(str::trim)
        .filter(|owner| !owner.is_empty())
        .unwrap_or("org")
        .to_string();
    let repo_name = full_name.split('/').nth(1).unwrap_or("repo").to_string();

    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind(&org_login)
        .bind("Test Org")
        .execute(pool)
        .await
        .expect("insert test org");

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name, private) VALUES ($1::uuid, $2::uuid, $3, $4, false)",
    )
    .bind(&repo_id)
    .bind(&org_id)
    .bind(full_name)
    .bind(&repo_name)
    .execute(pool)
    .await
    .expect("insert test repo");

    (org_id, repo_id)
}

pub(super) async fn insert_test_policy(pool: &PgPool, repo_id: &str, config: serde_json::Value) {
    sqlx::query(
        r#"
        INSERT INTO policies (id, org_id, repo_id, config, checksum, override_actor)
        SELECT
            gen_random_uuid(),
            r.org_id,
            r.id,
            $2::jsonb,
            $3,
            'integration-test'
        FROM repos r
        WHERE r.id = $1::uuid
        ON CONFLICT (repo_id) DO UPDATE
        SET config = EXCLUDED.config,
            checksum = EXCLUDED.checksum,
            updated_at = NOW()
        "#,
    )
    .bind(repo_id)
    .bind(config)
    .bind(format!("checksum-{}", uuid::Uuid::new_v4()))
    .execute(pool)
    .await
    .expect("insert test policy");
}

/// Build a minimal Router with auth middleware for integration testing.
pub(super) fn build_test_app_with_options(
    db: Arc<Database>,
    alert_webhook_url: Option<String>,
    drift_alert_webhook_urls: Vec<String>,
    policy_check_block_scopes: Vec<PolicyCheckBlockingScope>,
) -> Router {
    let state = AppState {
        db: Arc::clone(&db),
        github_webhook_secret: None,
        github_personal_access_token: None,
        jenkins_webhook_secret: None,
        jira_webhook_secret: None,
        start_time: Instant::now(),
        worker_id: "test-worker".to_string(),
        http_client: reqwest::Client::new(),
        alert_webhook_url,
        drift_alert_webhook_urls,
        strict_actor_match: false,
        reject_synthetic_logins: false,
        events_max_batch: 1000,
        llm_api_key: None,
        llm_model: "test".to_string(),
        feature_request_webhook_url: None,
        conversational_runtime: Arc::new(Mutex::new(ConversationalRuntime::default())),
        chat_llm_semaphore: Arc::new(Semaphore::new(1)),
        chat_llm_queue_timeout_ms: 500,
        chat_llm_timeout_ms: 9000,
        stats_cache_ttl: Duration::from_millis(100),
        stats_cache: Arc::new(Mutex::new(HashMap::new())),
        org_lookup_cache_ttl: Duration::from_millis(0),
        org_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_lookup_cache_ttl: Duration::from_millis(0),
        repo_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_upsert_min_interval: Duration::from_millis(0),
        repo_upsert_last_attempt: Arc::new(Mutex::new(HashMap::new())),
        cache_invalidation_min_interval: Duration::from_millis(0),
        stats_cache_invalidation_min_interval: Duration::from_millis(0),
        logs_cache_invalidation_min_interval: Duration::from_millis(0),
        stats_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        stats_cache_refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
        logs_cache_ttl: Duration::from_millis(100),
        logs_cache_stale_on_error: Duration::from_millis(1000),
        logs_reject_offset_pagination: false,
        outbox_server_lease_enabled: false,
        outbox_server_lease_ttl_ms: 2000,
        outbox_lease_telemetry: Arc::new(Mutex::new(handlers::OutboxLeaseTelemetry::default())),
        logs_cache: Arc::new(Mutex::new(HashMap::new())),
        logs_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        client_session_upsert_min_interval: Duration::from_millis(0),
        client_session_last_upsert: Arc::new(Mutex::new(HashMap::new())),
        sse_tx: tokio::sync::broadcast::channel::<handlers::SseNotification>(64).0,
        sse_max_connections: Arc::new(Semaphore::new(50)),
        sse_distributed_enabled: false,
        sse_distributed_channel: "test_sse".to_string(),
        policy_check_block_scopes,
    };

    let auth_routes = Router::new()
        .route("/events", post(handlers::ingest_client_events))
        .route("/logs", get(handlers::get_logs))
        .route("/stats", get(handlers::get_stats))
        .route("/stats/daily", get(handlers::get_daily_activity))
        .route("/dashboard", get(handlers::get_dashboard))
        .route(
            "/compliance/{org_name}",
            get(handlers::get_compliance_dashboard),
        )
        .route(
            "/signals/detect/{org_name}",
            post(handlers::trigger_detection),
        )
        .route("/me", get(handlers::get_me))
        .route("/orgs", get(handlers::list_orgs).post(handlers::create_org))
        .route("/orgs/{login}", get(handlers::get_org))
        .route(
            "/platform/tenants",
            get(handlers::list_platform_tenants).post(handlers::provision_platform_tenant_endpoint),
        )
        .route(
            "/platform/tenants/{login}/lifecycle",
            patch(handlers::update_platform_tenant_lifecycle),
        )
        .route(
            "/integrations/jenkins",
            post(handlers::ingest_jenkins_pipeline_event),
        )
        .route(
            "/integrations/jenkins/status",
            get(handlers::get_jenkins_integration_status),
        )
        .route(
            "/integrations/jenkins/correlations",
            get(handlers::get_jenkins_commit_correlations),
        )
        .route(
            "/integrations/correlations/v2",
            get(handlers::get_correlation_v2),
        )
        .route(
            "/integrations/jira/tickets/{ticket_id}",
            get(handlers::get_jira_ticket_detail),
        )
        .route(
            "/integrations/jira/status",
            get(handlers::get_jira_integration_status),
        )
        .route(
            "/integrations/jira/correlate",
            post(handlers::correlate_jira_tickets),
        )
        .route(
            "/integrations/jira/ticket-coverage",
            get(handlers::get_jira_ticket_coverage),
        )
        .route(
            "/evidence/packets/tickets/{ticket_id}",
            get(handlers::get_ticket_evidence_packet),
        )
        .route("/pr-merges", get(handlers::list_pr_merges))
        .route(
            "/cli/commands",
            post(handlers::ingest_cli_command).get(handlers::list_cli_commands),
        )
        .route(
            "/api-keys",
            get(handlers::list_api_keys).post(handlers::create_api_key),
        )
        .route("/api-keys/{id}/revoke", post(handlers::revoke_api_key))
        .route("/export", post(handlers::export_events))
        .route(
            "/compliance/control-frameworks",
            get(handlers::list_compliance_control_frameworks),
        )
        .route(
            "/compliance/control-frameworks/{framework_id}",
            get(handlers::get_compliance_control_framework),
        )
        .route(
            "/compliance/framework-packs",
            get(handlers::list_compliance_framework_packs),
        )
        .route(
            "/compliance/framework-packs/import",
            post(handlers::import_compliance_framework_pack),
        )
        .route(
            "/compliance/framework-packs/diff",
            get(handlers::diff_compliance_framework_packs),
        )
        .route(
            "/compliance/framework-packs/{framework_pack_id}",
            get(handlers::get_compliance_framework_pack),
        )
        .route(
            "/compliance/framework-packs/{framework_pack_id}/review",
            patch(handlers::review_compliance_framework_pack),
        )
        .route(
            "/compliance/evidence-exports",
            post(handlers::create_compliance_evidence_export),
        )
        .route(
            "/compliance/evidence-exports/{export_id}",
            get(handlers::get_compliance_evidence_export),
        )
        .route(
            "/compliance/evidence-exports/{export_id}/download",
            get(handlers::download_compliance_evidence_export),
        )
        .route(
            "/compliance/evidence-mappings",
            post(handlers::create_compliance_evidence_mapping),
        )
        .route(
            "/compliance/evidence-mappings/{mapping_id}",
            get(handlers::get_compliance_evidence_mapping),
        )
        .route(
            "/compliance/review-packages",
            post(handlers::create_compliance_review_package),
        )
        .route(
            "/compliance/review-packages/{review_package_id}",
            get(handlers::get_compliance_review_package),
        )
        .route(
            "/compliance/review-packages/{review_package_id}/download",
            get(handlers::download_compliance_review_package),
        )
        .route(
            "/compliance/framework-review-reports",
            get(handlers::list_compliance_framework_review_reports)
                .post(handlers::create_compliance_framework_review_report),
        )
        .route(
            "/compliance/framework-review-reports/assigned-to-me",
            get(handlers::list_assigned_compliance_framework_review_reports),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}",
            get(handlers::get_compliance_framework_review_report),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/assignments",
            get(handlers::list_compliance_framework_review_report_assignments)
                .put(handlers::upsert_compliance_framework_review_report_assignments),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/comments",
            get(handlers::list_compliance_framework_review_report_comments)
                .post(handlers::create_compliance_framework_review_report_comment),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/review",
            patch(handlers::review_compliance_framework_review_report),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/download",
            get(handlers::download_compliance_framework_review_report),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/pdf-export",
            get(handlers::get_compliance_framework_review_report_pdf_export)
                .post(handlers::create_compliance_framework_review_report_pdf_export),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/pdf-export/download",
            get(handlers::download_compliance_framework_review_report_pdf_export),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/provenance-manifests",
            post(handlers::create_compliance_framework_review_report_provenance_manifest),
        )
        .route(
            "/compliance/framework-review-reports/{report_id}/provenance-manifests/{manifest_id}",
            get(handlers::download_compliance_framework_review_report_provenance_manifest),
        )
        .route(
            "/compliance/period-reports",
            get(handlers::list_compliance_period_reports)
                .post(handlers::create_compliance_period_report),
        )
        .route(
            "/compliance/period-reports/{period_report_id}",
            get(handlers::get_compliance_period_report),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/retention",
            patch(handlers::update_compliance_period_report_retention),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/review",
            get(handlers::get_compliance_period_report_review)
                .patch(handlers::review_compliance_period_report),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/access-log",
            get(handlers::list_compliance_period_report_access_log),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/pdf-export",
            get(handlers::get_compliance_period_report_pdf_export)
                .post(handlers::create_compliance_period_report_pdf_export),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/pdf-export/download",
            get(handlers::download_compliance_period_report_pdf_export),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/provenance-manifests",
            post(handlers::create_compliance_period_report_provenance_manifest),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/provenance-manifests/{manifest_id}",
            get(handlers::download_compliance_period_report_provenance_manifest),
        )
        .route(
            "/compliance/period-reports/{period_report_id}/download",
            get(handlers::download_compliance_period_report),
        )
        .route(
            "/enterprise/adoption-profile",
            get(handlers::get_enterprise_adoption_profile)
                .put(handlers::upsert_enterprise_adoption_profile),
        )
        .route(
            "/enterprise/onboarding-checklist-tracking",
            get(handlers::get_enterprise_onboarding_checklist_tracking)
                .put(handlers::upsert_enterprise_onboarding_checklist_tracking),
        )
        .route(
            "/enterprise/first-governed-repo-setup",
            get(handlers::get_first_governed_repo_setup)
                .put(handlers::upsert_first_governed_repo_setup),
        )
        .route(
            "/enterprise/release-approvals",
            get(handlers::list_enterprise_release_approvals)
                .post(handlers::create_enterprise_release_approval),
        )
        .route(
            "/enterprise/release-governance/evaluate",
            get(handlers::evaluate_enterprise_release_governance),
        )
        .route(
            "/deployment-gates/authorize",
            post(handlers::authorize_deployment_gate),
        )
        .route(
            "/deployment-gates/authorizations",
            get(handlers::list_deployment_gate_authorizations),
        )
        .route(
            "/deployment-gates/break-glass-approvals",
            get(handlers::list_deployment_gate_break_glass_approvals)
                .post(handlers::create_deployment_gate_break_glass_approval),
        )
        .route(
            "/agent-governance/evaluate",
            post(handlers::evaluate_agent_governance),
        )
        .route(
            "/agent-governance/dry-run",
            post(handlers::dry_run_agent_governance),
        )
        .route(
            "/agent-governance/context",
            get(handlers::get_agent_governance_context),
        )
        .route(
            "/agent-governance/settings",
            get(handlers::get_agent_governance_settings)
                .put(handlers::upsert_agent_governance_settings),
        )
        .route(
            "/agent-governance/evaluations",
            get(handlers::list_agent_governance_evaluations),
        )
        .route(
            "/agent-governance/agent-keys",
            get(handlers::list_agent_governance_agent_keys)
                .post(handlers::create_agent_governance_agent_key),
        )
        .route(
            "/agent-governance/agent-keys/{key_id}",
            axum::routing::delete(handlers::revoke_agent_governance_agent_key),
        )
        .route(
            "/agent-governance/agent-keys/{key_id}/rotate",
            post(handlers::rotate_agent_governance_agent_key),
        )
        .route("/policy/{repo_name}", get(handlers::get_policy))
        .route(
            "/policy/{repo_name}/override",
            put(handlers::override_policy),
        )
        .route("/policy/check", post(handlers::policy_check))
        .route(
            "/policy/{repo_name}/requests",
            post(handlers::create_policy_change_request).get(handlers::list_policy_change_requests),
        )
        .route(
            "/policy/requests/{request_id}/approve",
            post(handlers::approve_policy_change_request),
        )
        .route(
            "/policy/requests/{request_id}/reject",
            post(handlers::reject_policy_change_request),
        )
        .route(
            "/policy/drift-events",
            post(handlers::ingest_policy_drift_event).get(handlers::list_policy_drift_events),
        )
        .layer(middleware::from_fn_with_state(
            Arc::clone(&db),
            auth::auth_middleware,
        ));

    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/detailed", get(handlers::detailed_health))
        .route(
            "/org-invitations/accept",
            post(handlers::accept_org_invitation),
        )
        .merge(auth_routes)
        .with_state(Arc::new(state))
}

pub(super) fn build_test_app_with_alerts(
    db: Arc<Database>,
    alert_webhook_url: Option<String>,
    drift_alert_webhook_urls: Vec<String>,
) -> Router {
    build_test_app_with_options(db, alert_webhook_url, drift_alert_webhook_urls, vec![])
}

pub(super) fn build_test_app_with_policy_check_scopes(
    db: Arc<Database>,
    policy_check_block_scopes: Vec<PolicyCheckBlockingScope>,
) -> Router {
    build_test_app_with_options(db, None, vec![], policy_check_block_scopes)
}

pub(super) fn build_test_app(db: Arc<Database>) -> Router {
    build_test_app_with_options(db, None, vec![], vec![])
}

/// Helper: make a JSON request to the test app.
pub(super) async fn json_request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
    api_key: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    builder = match method {
        "GET" => builder.method("GET"),
        "POST" => builder.method("POST"),
        _ => builder.method(method),
    };
    if let Some(key) = api_key {
        builder = builder.header("Authorization", format!("Bearer {}", key));
    }
    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }
    let req_body = body
        .map(|b| Body::from(b.to_string()))
        .unwrap_or(Body::empty());
    let request = builder.body(req_body).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), 1_000_000)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    (status, body_str)
}

pub(super) async fn spawn_webhook_probe() -> (
    String,
    tokio::sync::oneshot::Receiver<String>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind webhook probe listener");
    let addr = listener.local_addr().expect("listener local addr");
    let (body_tx, body_rx) = tokio::sync::oneshot::channel::<String>();
    let task = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut socket, _) = listener.accept().await.expect("accept webhook connection");
        let mut buf = vec![0u8; 16 * 1024];
        let read = socket.read(&mut buf).await.expect("read webhook request");
        let req = String::from_utf8_lossy(&buf[..read]).to_string();
        let body = req.split("\r\n\r\n").nth(1).unwrap_or_default().to_string();
        let _ = body_tx.send(body);
        let _ = socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .await;
    });

    (format!("http://{}", addr), body_rx, task)
}

/// Macro to reduce boilerplate: skip test if DB unavailable.
macro_rules! setup_or_skip {
    () => {
        match try_setup().await {
            Some(result) => result,
            None => {
                eprintln!("SKIPPED: TEST_DATABASE_URL not set or unreachable");
                return;
            }
        }
    };
}

// ========================================================================
// TESTS
// ========================================================================
