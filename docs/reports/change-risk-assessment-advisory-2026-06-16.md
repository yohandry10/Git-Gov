# KAN-121 Change Risk Assessment Advisory Validation

Date: 2026-06-16

## Summary

KAN-121 implements a deterministic, manual-first Change Risk Assessment Advisory MVP. It persists
advisory evaluations for change/release candidates and Deployment Gate authorizations, exposes them
through backend APIs, and surfaces them in Desktop Governance > Releases.

The feature is deliberately advisory only. It does not approve, block, certify, deploy, mutate
providers, mutate repositories, use AI, or depend on Agent Governance.

## Implemented

- Supabase migration/postcheck `v62`.
- Backend persistence and routes:
  - `POST /change-risk/evaluations`.
  - `GET /change-risk/evaluations`.
  - `GET /change-risk/evaluations/{evaluation_id}`.
- Deterministic evaluator for:
  - complete approved gate context.
  - production environment risk uplift.
  - missing deployment gate/evidence/release context.
  - blocked or approval-required gates.
  - break-glass use.
  - advisory/would-block gate warnings.
- Tauri DTOs, client methods, commands, and invoke registration.
- Desktop store state/actions and `ChangeRiskPanel` under Governance > Releases.
- Public design documentation.

## Safety Guarantees

Every persisted record is constrained to:

- `advisory_only=true`.
- `llm_used=false`.
- `agent_governance_used=false`.
- `compliance_claim=false`.
- `certification=false`.

The backend tests also assert that Agent Governance scoped keys cannot create or read this evidence
and that no `agent_governance_evaluations` rows are created by KAN-121.

## Local Validation

Passed:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml -- --check`.
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`.
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`.
- Focused real Postgres backend tests:
  - `change_risk_evaluates_gate_context_without_ai_agents_or_claims`.
  - `change_risk_is_tenant_scoped_and_handles_missing_context_advisory`.
- Full backend real Postgres suite (`313` passed).
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml -- --check`.
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`.
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed).
- `pnpm --dir gitgov typecheck`.
- `pnpm --dir gitgov lint`.
- `pnpm --dir gitgov test -- useControlPlaneStore` (`41` passed).
- `pnpm --dir gitgov test` (`375` passed).
- `pnpm --dir gitgov build`.
- `v62` migration and postcheck in a real Postgres transaction with rollback:
  - table check `PASS`.
  - no-claim constraints check `PASS`.
  - index check `PASS`.

Known warning:

- Vite still reports the existing large base chunk warning after build. This warning predates KAN-121
  and does not block the advisory feature.

## Production Validation

Completed after merge:

1. PR `#422` merged to `main` as `eb66480`.
2. Production migration `gitgov/gitgov-server/supabase/supabase_schema_v62.sql` applied.
3. Production postcheck `gitgov/gitgov-server/supabase/checks/v62_postcheck.sql` returned:
   - `change_risk_evaluations.table = PASS`.
   - `change_risk_evaluations.no_claim_constraints = PASS`.
   - `change_risk_evaluations.indexes = PASS`.
4. Render deploy `dep-d8oanqmq1p3s73fc8u7g` for `eb66480` reached `live`.
5. Production smoke:
   - `/health=ok`.
   - authenticated `/stats=200`.
   - `GET /change-risk/evaluations?org_name=yohandry10&limit=1` succeeded.
   - `POST /change-risk/evaluations` created advisory record
     `cra_9d53d9cd29a7439aa0485607edeae64e`.
   - The record used repo `yohandry10/Git-Gov`, branch `main`, environment `production`,
     change/release `KAN-121`, and commit `eb66480`.
   - Response returned `risk_level=medium`, `advisory_only=true`, `llm_used=false`,
     `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
   - Missing evidence was `deployment_gate_authorization, release_evidence_packet`, which is the
     expected advisory result for this smoke because it intentionally did not bind an existing
     Deployment Gate authorization.
   - Fetch by evaluation ID returned the same record.
