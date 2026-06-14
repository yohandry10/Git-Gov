# KAN-88 Break-glass Approval Routing

Updated: 2026-06-14

## Summary

KAN-88 hardens Deployment Gates break-glass handling by separating approval from deployment execution.

Before this change, KAN-87 allowed an Admin caller to include `break_glass` directly in `POST /deployment-gates/authorize` when the evaluated release policy was blocking. That preserved blockers and audit history, but the approval was inline with the deploy request.

KAN-88 adds a prior approval route:

```text
POST /deployment-gates/break-glass-approvals
GET /deployment-gates/break-glass-approvals
```

`POST /deployment-gates/authorize` now accepts `break_glass` only when a valid, unexpired, evidence-bound approval exists for the same release id, repository, branch, target SHA, environment, ticket id, and evidence packet hash.

## Implemented

- Added `deployment_gate_break_glass_approvals`.
- Added `break_glass_approval_id` and `break_glass_approval_hash` to `deployment_gate_authorizations`.
- Added Supabase migration `gitgov/gitgov-server/supabase/supabase_schema_v37.sql`.
- Added postcheck `gitgov/gitgov-server/supabase/checks/v37_postcheck.sql`.
- Added backend models, DB persistence/listing, admin routes, and audit log entries.
- Added validation for required reason, approver, approver role, expiry, evidence binding, and org scope.
- Updated deployment authorization logic so missing/expired/mismatched approvals reject the request before authorization history is written.
- Updated Desktop history DTOs and `DeploymentGateHistoryPanel` to show `pre-approved`, approval id, and approval hash.

## Business Rules

- Break-glass remains invalid for non-blocking/advisory/record-only evaluations.
- Approval expiry is required and cannot be more than 24 hours ahead.
- The approval must be bound to the exact release evidence packet and deploy target.
- The approver must be separate from the deployer/requester.
- Original blockers remain recorded on a successful `decision=break_glass` authorization.
- Provider templates must not auto-create or auto-use break-glass.

## Validation

Local validation completed:

```text
cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check
cargo check --manifest-path gitgov/gitgov-server/Cargo.toml
cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings
TEST_DATABASE_URL=<temporary local postgres on 127.0.0.1:55433> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml deployment_gate -- --nocapture
TEST_DATABASE_URL=<temporary local postgres on 127.0.0.1:55433> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml
cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check
cargo check --manifest-path gitgov/src-tauri/Cargo.toml
cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path gitgov/src-tauri/Cargo.toml
npm --prefix gitgov test -- --run src/test/components/DeploymentGateHistoryPanel.test.tsx
npm --prefix gitgov run typecheck
npm --prefix gitgov run lint
npm --prefix gitgov test -- --run
npm --prefix gitgov run build
git diff --check
.\scripts\security\publication_guard.ps1
```

The temporary Postgres validation applied `supabase_schema.sql`, `supabase_schema_v28.sql`, `supabase_schema_v35.sql`, `supabase_schema_v36.sql`, `supabase_schema_v37.sql`, and `checks/v37_postcheck.sql`; the postcheck returned `PASS` for the new table, constraints, authorization link columns, and link constraint.

`npm --prefix gitgov run build` completed with the existing Vite large chunk warning.

Focused backend tests cover:

- valid pre-approved break-glass against a blocking policy;
- rejected break-glass without prior approval;
- rejected expired approval;
- rejected evidence binding mismatch;
- rejected break-glass when policy does not block;
- admin and org-scope enforcement;
- ticket mismatch protection;
- normal blocked/advisory deployment authorization history.

No local pre-PR validation gap remains.

## Production Validation

PR `#316` merged to `main` as `bd44db1` on 2026-06-14. Post-merge checks passed, including `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.

Production validation completed:

- Applied `supabase_schema_v37.sql`.
- `checks/v37_postcheck.sql` returned `PASS` for the new table, constraints, authorization link columns, and link constraint.
- Render deploy `dep-d8n324u8bjmc73en5qgg` for `bd44db1` reached `live`.
- `GET https://gitgov-api.onrender.com/health` returned `ok`.
- Authenticated `GET /stats` returned HTTP `200`.
- Anonymous `GET /deployment-gates/break-glass-approvals?org_name=yohandry10&limit=1` returned HTTP `401`.
- Authenticated `POST /deployment-gates/break-glass-approvals` created approval `dgbga_8be2e0b2a33741368ab211e7d4b5e77f` against existing release evidence.
- Authenticated `GET /deployment-gates/break-glass-approvals?org_name=yohandry10&approval_id=dgbga_8be2e0b2a33741368ab211e7d4b5e77f&active_only=true` returned `total=1`.
