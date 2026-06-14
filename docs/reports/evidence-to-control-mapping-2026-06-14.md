# Evidence-to-Control Mapping Validation Report

Date: 2026-06-14

Ticket: `KAN-100`

Issue: GitHub `#352`

Branch: `feature/KAN-100-evidence-control-mapping`

## Product Decision

GPT/product review and local repo analysis selected `KAN-100: Evidence-to-Control Mapping MVP`.

The slice intentionally avoids official regulatory mapping. It maps KAN-99 evidence exports to GitGov's own release governance baseline and always marks the output as review material, not a compliance claim.

Mandatory response flags:

```text
compliance_claim=false
regulatory_claim=false
requires_auditor_review=true
```

## Implementation

Backend:

- `GET /compliance/control-frameworks`
- `GET /compliance/control-frameworks/{framework_id}`
- `POST /compliance/evidence-mappings`
- `GET /compliance/evidence-mappings/{mapping_id}`

Persistence:

- `gitgov/gitgov-server/supabase/supabase_schema_v44.sql`
- `gitgov/gitgov-server/supabase/supabase_schema_v44_postcheck.sql`

Tests:

- `gitgov/gitgov-server/src/integration_tests/compliance_evidence_mappings.rs`

Documentation:

- `docs/design/evidence-to-control-mapping-mvp.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`
- `docs/ARCHITECTURE.md`

## Local Validation

Temporary Postgres:

```text
container: gitgov-kan100-pg
host: 127.0.0.1:55443
database: gitgov
```

Commands executed:

```powershell
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:55443/gitgov'; cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_evidence_mappings -- --nocapture
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml sensitive_admin_path_detection_matches_expected_routes -- --nocapture
$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:55443/gitgov'; cargo test --manifest-path gitgov\gitgov-server\Cargo.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo check`: passed.
- `cargo clippy -D warnings`: passed.
- Focused Evidence-to-Control Mapping tests: `2` passed.
- Sensitive admin route test: `1` passed.
- Full backend tests: `300` passed.

Migration validation:

- `supabase_schema_v44.sql` applied to temporary Postgres.
- `supabase_schema_v44_postcheck.sql` returned:

```text
KAN-100 postcheck PASS: framework, controls, mapping tables, and indexes exist
```

## Real Test Coverage

The main integration test creates a real KAN-99 evidence export through the API, then maps it through KAN-100. Assertions verify:

- mapping id starts with `cem_`;
- source export id and hash are preserved;
- framework id/version are correct;
- 10 controls are returned;
- `compliance_claim=false`;
- `regulatory_claim=false`;
- `requires_auditor_review=true`;
- missing Sonar evidence maps to `GG-RG-06=missing`;
- PR review evidence gap maps to `GG-RG-05=partial`;
- mapping response does not leak secret-like fixture payloads;
- no new Agent Governance evaluations are created.

The second integration test verifies Admin-only access, tenant isolation, framework allow-listing, and framework catalog APIs.
