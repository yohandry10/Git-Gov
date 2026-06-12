# Render Policy Core Docker Context Hotfix

Date: 2026-06-12
Ticket: KAN-77

## What Failed

PR `#214` merged as `0acfd26 security(KAN-77): harden event capture and policy as code (#214)`.
GitHub checks passed, but the automatic Render deploy `dep-d8lsqf7avr4c73fsemlg` failed during
Docker build.

The Render build log showed:

```text
failed to load source for dependency `gitgov-policy-core`
Unable to update /policy-core
failed to read `/policy-core/Cargo.toml`
No such file or directory (os error 2)
```

## Root Cause

KAN-77 added a shared Rust crate at `gitgov/policy-core`.

The backend dependency is:

```toml
gitgov-policy-core = { path = "../policy-core" }
```

Render was still configured with root directory `gitgov/gitgov-server`, Docker context `.`, and
Dockerfile `./Dockerfile`. That context included only the server crate, so Docker could not copy or
compile the sibling `policy-core` crate.

The same stale context existed in local `docker-compose.yml`.

## Fix

PR `#215` merged as `e4bec3f fix(KAN-77): align Render Docker context for policy core (#215)`.

Changes:

- `gitgov/gitgov-server/Dockerfile` now expects Docker context `gitgov`.
- The Dockerfile copies both `gitgov-server` and `policy-core`.
- The backend is built with `cargo build --release --manifest-path gitgov-server/Cargo.toml`.
- `docker-compose.yml` now uses context `./gitgov` and Dockerfile `gitgov-server/Dockerfile`.
- `AGENTS.md`, `docs/CURRENT_CONTEXT.md`, and `docs/DEPLOYMENT.md` now document the production
  Render context contract.
- Render service `gitgov-api` was updated through the Render API:
  - root directory: `gitgov`
  - Docker context: `.`
  - Dockerfile path: `gitgov-server/Dockerfile`

## Validation

Local validation before PR `#215`:

- `docker compose config` showed build context `C:\Users\PC\Desktop\GitGov\gitgov` and Dockerfile
  `gitgov-server/Dockerfile`.
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml` passed and compiled
  `gitgov-policy-core`.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.

GitHub validation:

- All PR `#215` checks passed before merge.
- Post-merge checks for `e4bec3f` passed, including `CI`, `Release Readiness Gate`, `Secret Scan`,
  `Public Naming Guard`, `Quality Gate Policy Matrix (Optional)`, `Governance Correlation Smoke
  (Optional)`, `Desktop Updater Readiness (Optional)`, and `SonarQube Governance (Non-Blocking)`.

Production validation:

- Render deploy `dep-d8lsul8k1i2s73dk1ph0` for commit `e4bec3f` reached `live`.
- `GET https://gitgov-api.onrender.com/health` returned `status=ok`.
- Authenticated `GET https://gitgov-api.onrender.com/stats` returned HTTP `200`.

Follow-up production DB validation:

- PR `#216` initially exposed a production `GET /policy/yohandry10%2FGit-Gov` database error in
  the required `Validate quality_gates warn/block matrix` check.
- Root cause: production had not fully applied the KAN-77 `supabase_schema_v31.sql` policy source
  metadata migration.
- While applying `v31`, the migration also exposed an idempotency bug: PostgreSQL cannot
  `CREATE OR REPLACE FUNCTION get_policy_history(...)` when the OUT-parameter return row changes.
- `supabase_schema_v31.sql` was corrected to `DROP FUNCTION IF EXISTS get_policy_history(UUID,
  INTEGER)` before recreating the function.
- Production `v31` was applied successfully: all three `source_metadata` columns exist and
  `get_policy_history` exists once.
- Authenticated production `GET /policy/yohandry10%2FGit-Gov` returned HTTP `200`.
- Local rerun of `scripts/jenkins/validate_quality_gate_policy_matrix.ps1` against production
  passed for the same failing/green commits used by the PR check.

## Notes

The exact local Docker image build was not run before the hotfix because Docker Desktop was not
running (`dockerDesktopLinuxEngine` pipe missing). The production Render build is the authoritative
validation for this incident and completed successfully after the context fix.
