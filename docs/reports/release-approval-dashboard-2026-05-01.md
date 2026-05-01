# KAN-43 Release Approval Dashboard MVP

Updated: 2026-05-01

## Summary

KAN-43 implements the first dashboard wizard for formal release approvals.

This closes the main KAN-37 non-goal around dashboard usability: admins can now create and review formal release decisions from the Control Plane UI instead of using the backend API directly.

## Traceability

- Jira issue: `KAN-43 - Dashboard release approval wizard MVP`.
- Implementation branch: `product/KAN-43-release-approval-dashboard`.
- Implementation PR: `#140 - product(KAN-43): add release approval dashboard`.
- Design: `docs/design/release-approval-dashboard-mvp.md`.

## Implementation

- Added `ReleaseApprovalPanel` to the admin dashboard.
- Added Zustand types and actions for:
  - listing enterprise release approvals;
  - creating enterprise release approvals.
- Added Tauri client structs, methods, and commands for:
  - `GET /enterprise/release-approvals`;
  - `POST /enterprise/release-approvals`.
- Registered the new Tauri commands in `src-tauri/src/lib.rs`.
- Added store regression tests for list/create behavior.

## UX Behavior

- The panel shows recent release approvals with decision, release, environment, repository, approver, risk, creation time, approval hash, and expiration when present.
- The form supports `approved`, `rejected`, and `accepted-risk`.
- The form can copy the current Evidence Packet hash and URI through an explicit `Use current packet` action.
- The submit button remains disabled until required fields and operator confirmation are valid.

## Validation

Frontend:

- `npm test -- --run src/test/useControlPlaneStore.test.ts`: `21` tests passed.
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm test -- --run`: `25` test files passed, `280` tests passed.
- `npm run build`: passed with the existing Vite large chunk warning.

Tauri:

- `cargo fmt`: passed.
- `cargo check`: passed.
- `cargo clippy -- -D warnings`: passed.
- `cargo test`: `23` tests passed.

## Security Notes

- No provider secrets, local env files, Authorization headers, or raw approval evidence payloads are printed by the UI.
- The dashboard sends only approval metadata and evidence hashes to the existing admin-only backend route.
- Client-side validation mirrors the important KAN-37 server rules but does not replace server validation.

## Residual Work

- Multi-approver quorum and signatures remain future enterprise governance work.
- Release gate enforcement from approval state remains future work.
- A dedicated approval history drill-down can be added later if customers need more than the compact recent list.
