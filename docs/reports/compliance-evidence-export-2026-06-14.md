# Compliance Evidence Export Validation Report

Date: 2026-06-14

Ticket: `KAN-99`

Issue: GitHub `#349`

Branch: `feature/KAN-99-compliance-evidence-export`

## Product Decision

After `KAN-98`, GPT/product review and local repo analysis selected `KAN-99 Compliance Evidence Export v1` as the next roadmap slice. The decision is to pause MCP/agent expansion and package existing Deployment Gate evidence for audit review.

This slice remains manual-first:

- source is an existing Deployment Gate authorization;
- output is JSON only;
- artifact states `compliance_claim=false`;
- artifact states `framework_mapping=false`;
- Deployment Gate artifacts preserve `agent_governance_used=false`;
- no Agent Governance evaluation row is created.

## Implementation

Backend:

- `POST /compliance/evidence-exports`
- `GET /compliance/evidence-exports/{export_id}`
- `GET /compliance/evidence-exports/{export_id}/download`

Persistence:

- `gitgov/gitgov-server/supabase/supabase_schema_v43.sql`
- `gitgov/gitgov-server/supabase/supabase_schema_v43_postcheck.sql`

Tests:

- `gitgov/gitgov-server/src/integration_tests/compliance_evidence_exports.rs`

Documentation:

- `docs/design/compliance-evidence-export-v1.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`
- `docs/ARCHITECTURE.md`

## Local Validation

Temporary Postgres:

```text
container: gitgov-kan99-pg
host: 127.0.0.1:55442
database: gitgov
```

Commands executed:

```powershell
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:55442/gitgov'; cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_evidence_exports -- --nocapture
$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:55442/gitgov'; cargo test --manifest-path gitgov\gitgov-server\Cargo.toml
git diff --check
```

Results:

- `cargo check`: passed.
- `cargo fmt --check`: passed.
- `cargo clippy -D warnings`: passed.
- Focused Compliance Evidence Export tests: `2` passed.
- Full backend tests: `298` passed.
- `git diff --check`: passed.

Migration validation:

- `supabase_schema_v43.sql` applied to the temporary Postgres database.
- `supabase_schema_v43_postcheck.sql` returned:

```text
KAN-99 postcheck PASS: compliance evidence exports table, columns, and indexes exist
```

## Risk Notes

- This is not a regulatory mapper yet. It creates verified evidence packaging that future framework mapping can consume.
- This is not a PDF/auditor portal yet. JSON is the canonical artifact for `KAN-99`.
- This is not an agent feature. It must stay usable for regulated customers that do not allow agents.
