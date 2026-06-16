# KAN-126 Change Risk CAB Packet Manual Disposition Report

Date: 2026-06-16

## Summary

KAN-126 adds manual CAB disposition metadata to KAN-125 Change Risk CAB packets.

This is a human review layer over an immutable packet artifact. It does not approve deployments, block releases, mutate providers or repositories, mutate source evaluations, or make compliance/certification claims.

## Implemented

- Supabase migration/postcheck `v66`.
- Backend review fields on `change_risk_cab_packets`.
- Backend routes:
  - `GET /change-risk/cab-packets/{packet_id}/review`.
  - `PATCH /change-risk/cab-packets/{packet_id}/review`.
- Admin-only update and Admin/Auditor read.
- Safe text validation and secret-like value rejection.
- Admin audit actions:
  - `change_risk_cab_packet_review_viewed`.
  - `change_risk_cab_packet_review_updated`.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions and focused tests.
- Desktop CAB disposition UI with explicit manual-only warning.

## Local Validation

- Backend `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`.
- Backend `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`.
- Tauri `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`.
- Tauri `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`.
- Tauri `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Focused store tests: `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`45` passed).
- Full frontend tests: `pnpm --dir gitgov test` (`379` passed).
- Frontend build: `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused real Postgres backend test with `TEST_DATABASE_URL` mapped from ignored local `DATABASE_URL`: `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation` passed.
- Real Postgres migration/postcheck rollback for `v66` passed.

## Real Test Evidence

The focused backend test creates tenant-scoped Change Risk evaluations, updates source manual review states, creates CAB packets, reads and updates CAB disposition, rejects unsafe payloads, checks RBAC and tenant isolation, denies Agent keys, verifies audit rows, and asserts unchanged artifact hash plus unchanged Deployment Gate and Agent Governance counts.

## Production Status

PR `#440` merged to `main` as `b7bc9e81`.

Production migration `v66` and postcheck passed against the configured Supabase database.

Render deploy `dep-d8ofa167r5hc73c1nf5g` for commit `b7bc9e81` reached `live`.

Production smoke passed:

- `/health=ok`.
- Authenticated `/stats=200`.
- Created CAB packet `crcab_23d138be426a4967ae0895810e679a19`.
- Packet artifact hash stayed `sha256:d314caf9c2e41886cdfcbd5e56c841ea84f6329b0c00c7f6f7398a3dbe3b1d9a` across review updates.
- Review state moved through `pending_review`, `reviewed`, `accepted_risk`, and final `needs_mitigation`.
- Final review required follow-up and recorded safe owner `release-owner`.
- Secret-looking review note was rejected with HTTP `400`.
- Final no-claim flags stayed safe: `manual_cab_disposition_only=true`, `advisory_only=true`, `release_blocking=false`, `deployment_execution=false`, `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
- Review audit rows were present: `2` view events and `3` update events for the packet.
- Deployment Gate authorization count stayed `2`.
- Agent Governance evaluation count stayed `7`.
