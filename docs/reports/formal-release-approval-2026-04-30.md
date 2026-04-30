# KAN-37 Formal Release Approval MVP

Updated: 2026-04-30

## Summary

KAN-37 adds the first formal enterprise release approval API.

This closes the biggest remaining enterprise self-service gap before starting the Vercel AI SDK Copilot: GitGov can now store a release approval decision with approver, risk, expiration and evidence packet hash.

## Changes

- Added `GET /enterprise/release-approvals`.
- Added `POST /enterprise/release-approvals`.
- Added release approval request, response and list models.
- Added append-only `enterprise_release_approvals` database table.
- Added Supabase migration `supabase_schema_v24.sql`.
- Added Supabase post-check `v24_postcheck.sql`.
- Added admin-only org-scope validation using the same rule as enterprise adoption profiles.
- Marked release approval routes as sensitive admin paths so stale admin auth cache is not accepted for this operation.
- Added validation for release IDs, repositories, SHAs, Jira IDs, evidence hashes, evidence URIs, decisions, risk acceptance and expiration.
- Added admin audit logging for created release approvals.
- Added backend unit tests for the approval validation contract.

## API Contract

Authenticated admin routes:

```text
GET /enterprise/release-approvals
POST /enterprise/release-approvals
```

Create decisions:

- `approved`
- `rejected`
- `accepted-risk`

Risk severities:

- `none`
- `low`
- `medium`
- `high`
- `critical`

## PR

- PR: `#125` - `product(KAN-37): add formal release approvals`.
- Merge commit: `d7ae92e`.
- Jira final comment: `10196`.

## Local Validation

Backend validation tests:

```powershell
cd gitgov\gitgov-server
cargo test enterprise_release_approval_validation
```

Result:

- `5` tests passed.
- `0` failed.

Full backend test suite:

```powershell
cd gitgov\gitgov-server
cargo test
```

Result:

- `178` tests passed.
- `0` failed.

Backend compile:

```powershell
cd gitgov\gitgov-server
cargo check
```

Result:

- passed.

Backend lint:

```powershell
cd gitgov\gitgov-server
cargo clippy -- -D warnings
```

Result:

- passed.

Repository checks:

```powershell
git diff --check
.\scripts\security\publication_guard.ps1
```

Result:

- both passed.

## Post-Merge Validation

Post-merge `main` checks passed for commit `d7ae92e`:

- `CI` run `25193460879`.
- `Release Readiness Gate` run `25193460902`.
- `Quality Gate Policy Matrix (Optional)` run `25193460904`.
- `Secret Scan` run `25193460915`.
- `Public Naming Guard` run `25193460892`.
- `SonarQube Governance (Non-Blocking)` run `25193460922`.
- `Governance Correlation Smoke (Optional)` run `25193460903`.
- `Desktop Updater Readiness (Optional)` run `25193460881`.

## Production Migration

Production `v24` was applied on 2026-04-30 using ignored local `DATABASE_URL` without printing credentials.

Postcheck passed:

- `enterprise_release_approvals.table_exists` - `PASS`.
- `enterprise_release_approvals.primary_key` - `PASS`.
- `enterprise_release_approvals.decision_check` - `PASS`.
- `enterprise_release_approvals.org_created_index` - `PASS`.
- `enterprise_release_approvals.repo_release_index` - `PASS`.

Revalidation commands for new environments:

```powershell
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/supabase_schema_v24.sql
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/checks/v24_postcheck.sql
```

Do not print the database URL or credentials.

## Production Endpoint Validation

Render deploy `dep-d7ptsvhoagis738cj88g` for commit `d7ae92e` reached `live`.

Production smoke results:

- `GET /health` returned `200`.
- Anonymous `GET /enterprise/release-approvals?org_name=yohandry10` returned `401`.
- Authenticated `GET /enterprise/release-approvals` for `release_id=KAN-37-runtime-smoke` returned `total=0` before creation.
- Authenticated `GET /evidence/packets/tickets/KAN-37` returned `found=true`.
- Authenticated `POST /enterprise/release-approvals` created `KAN-37-runtime-smoke` with decision `approved`.
- The returned `approval_hash` was 64 hex characters.
- Authenticated follow-up list for `KAN-37-runtime-smoke` returned `total=1`.

## Remaining Work

KAN-37 intentionally does not include:

- dashboard approval wizard.
- multi-approver release quorum.
- external signing workflow.
- release gate enforcement based on approval state.
- AI SDK Copilot.

The next major product feature remains Vercel AI SDK Copilot after KAN-37 is merged and validated.
