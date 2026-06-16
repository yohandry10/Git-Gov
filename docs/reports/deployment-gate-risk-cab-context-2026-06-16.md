# KAN-128 Deployment Gate Risk & CAB Evidence Context Report

Date: 2026-06-16

Issue: `#450`

Branch: `product/KAN-128-deployment-gate-risk-cab-context`

## Summary

KAN-128 adds read-only Risk & CAB context to Deployment Gate History. It composes existing records
instead of creating a new table or mutating source evidence.

New backend route:

- `GET /deployment-gates/{deployment_gate_id}/risk-context`

Returned context:

- Deployment Gate authorization.
- Related Change Risk evaluations.
- Related CAB packets.
- Related CAB decision manifests.
- Latest risk level and review status.
- Triggered rule count.
- Explicit no-claim flags.

## Guardrails

- `advisory_only=true`
- `enforcement_used=false`
- `llm_used=false`
- `agent_governance_used=false`
- `compliance_claim=false`
- `certification=false`

The feature does not recalculate risk, approve deployments, block releases, execute deploys, mutate
providers/repos, mutate source evaluations, create CAB packets/manifests automatically, or create
legal/compliance/certification claims.

## Local Validation

Passed:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests)
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run` (`381` tests)
- `npm --prefix gitgov test -- --run src/test/useControlPlaneStore.test.ts` (`47` tests)
- `npm --prefix gitgov run build` with the pre-existing Vite large chunk warning
- Focused real Postgres backend test:
  `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation`
- `git diff --check`
- `scripts/security/publication_guard.ps1`

The backend test now covers the KAN-128 chain:

- Gate -> Change Risk evaluation -> CAB packet -> CAB disposition -> CAB decision manifest.
- Admin/Auditor read.
- Developer denial.
- Tenant isolation.
- Agent key denial.
- No mutation of Deployment Gates, Change Risk evaluations, CAB packets, or CAB manifests.
- No AI/Agent Governance dependency.
- No compliance/certification claims.

## Pending

- Full local validation sweep.
- PR checks.
- Merge.
- Render deploy.
- Production smoke against `https://gitgov-api.onrender.com`.
