# KAN-101 Control Mapping Review Package

Date: 2026-06-14

## Summary

KAN-101 implements a JSON-only Control Mapping Review Package on top of KAN-99 Compliance Evidence Exports and KAN-100 Evidence-to-Control mappings.

The package is for customer/auditor review only. It is not a compliance certification, not an official regulatory mapping, and not an Agent Governance feature.

## Implemented

- Backend routes:
  - `POST /compliance/review-packages`
  - `GET /compliance/review-packages/{review_package_id}`
  - `GET /compliance/review-packages/{review_package_id}/download`
- Supabase migration `supabase_schema_v45.sql`
- Postcheck `supabase_schema_v45_postcheck.sql`
- Persisted table `compliance_review_packages`
- Artifact schema `gitgov_control_review_package.v1`
- Deterministic `mapping_hash`
- Idempotent `crp_...` package ids for the same org/mapping/schema/hash
- Admin audit action `compliance_review_package.created`
- Real Postgres integration tests for full KAN-99/KAN-100/KAN-101 flow, hash verification, permissions, validation, tenant isolation, redaction, and no Agent Governance side effects

## Explicitly Not Implemented

- SOC 2, ISO, NIST, PCI, SBS, or LGPD official pack
- Customer-provided framework import
- PDF/DOCX export
- Compliance score or badge
- Auditor signature/comment workflow
- OPA/Rego execution
- Policy mutation
- Provider mutation
- MCP, chatbot, BYOM, or required agents

## Local Validation

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `supabase_schema_v45.sql` plus `supabase_schema_v45_postcheck.sql` against temporary PostgreSQL 16 on `127.0.0.1:55444`
- focused `compliance_review_packages` integration tests: `2` passed
- full backend test suite: `302` passed
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

The migration postcheck was validated against a real PostgreSQL schema containing the direct KAN-101 dependencies: `orgs`, the KAN-99 `compliance_evidence_exports` table, KAN-100 `supabase_schema_v44.sql`, and KAN-101 `supabase_schema_v45.sql`.

Production validation should apply v45, wait for Render deploy, and smoke:

- `/health`
- authenticated `/stats`
- use a real KAN-99 export and KAN-100 mapping
- create review package
- read metadata
- download artifact
- recalculate hash
- verify no claims and `certification=false`
- verify Agent Governance evaluations do not increase
