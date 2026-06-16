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

Not deployed yet in this branch. Apply `supabase_schema_v66.sql` plus `supabase_schema_v66_postcheck.sql` before production smoke after merge.
