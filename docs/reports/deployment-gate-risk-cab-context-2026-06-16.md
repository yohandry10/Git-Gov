# KAN-128 Deployment Gate Risk & CAB Evidence Context Report

Date: 2026-06-16

Issue: `#450`

PR: `#451`

Main commit: `27b2b5d5`

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

## Production Validation

- No production migration was required.
- Post-merge `main` CI and guard workflows passed.
- Render deploy `dep-d8oi2p99rddc73d37320` for `27b2b5d5` reached `live`.
- `/health` returned `ok`.
- Authenticated `/stats` returned HTTP `200`.
- Deployment Gate total remained `2`.
- Agent Governance evaluation total remained `7`.

Production smoke source:

- Deployment Gate: `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`
- Change Risk evaluation: `cra_b8408c9e4aa44989bd1146d5ff5d4c30`
- Risk level: `medium`
- Review status: `accepted_risk`
- Trace hash:
  `sha256:5c1d4c8504c0a52c42176f525b8ea9a35a5c1f2826cb5a18af99311dd47b5f46`
- CAB packet: `crcab_cf0af176f7674b16821d5cf61b5225b8`
- CAB packet hash:
  `sha256:78edf32af71d3d96872b08080dc4c009bac2a7b33fe61d078bf33a3eb4d2ad51`
- CAB review status: `needs_mitigation`
- CAB decision manifest: `crcabdm_8df5e6df7297acb8155730f48b5cc526`
- Manifest hash:
  `sha256:ea93d018394b141665b83698cadcb1a519aef602c82571bfcfb0a385fde1936f`

Production `GET /deployment-gates/{deployment_gate_id}/risk-context` returned:

- `context_evals=1`
- `context_packets=1`
- `context_manifests=1`
- `latest_risk_level=medium`
- `latest_review_status=accepted_risk`
- `advisory_only=true`
- `enforcement_used=false`
- `llm_used=false`
- `agent_governance_used=false`
- `compliance_claim=false`
- `certification=false`
- after manifest revoke, the same context returned manifest status `revoked`.
