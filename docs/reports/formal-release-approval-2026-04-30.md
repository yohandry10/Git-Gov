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

## Remaining Work

KAN-37 intentionally does not include:

- dashboard approval wizard.
- multi-approver release quorum.
- external signing workflow.
- release gate enforcement based on approval state.
- AI SDK Copilot.

The next major product feature remains Vercel AI SDK Copilot after KAN-37 is merged and validated.
