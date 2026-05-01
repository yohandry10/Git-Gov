# KAN-37 Formal Release Approval MVP

Updated: 2026-04-30

## Summary

KAN-37 adds the first persistent formal release approval model for enterprise adoption.

The goal is to turn "who approved this release, with what evidence and risk context" into an auditable GitGov record instead of a loose chat message, spreadsheet row, or manual note.

## Product Scope

This MVP adds a backend API and database table for append-only release approval decisions.

It covers:

- release identifier.
- repository, branch and target SHA.
- target environment.
- decision: `approved`, `rejected`, or `accepted-risk`.
- approver identity.
- optional Jira ticket ID.
- required evidence packet SHA-256 hash.
- optional evidence packet URI.
- structured evidence summary.
- risk severity.
- risk acceptance reason and expiration when risk is accepted.
- server-side approval hash.
- admin audit log entry.

## API

Authenticated admin routes:

```text
GET /enterprise/release-approvals
POST /enterprise/release-approvals
```

Both routes use the same organization scope rules as enterprise adoption profiles:

- org-scoped admin keys can operate only inside their organization.
- global admin keys must provide `org_name`.
- non-admin keys are rejected.

## Create Request

Required fields:

```json
{
  "org_name": "example-org",
  "release_id": "release-2026.04.30",
  "repository_full_name": "example-org/example-repo",
  "environment": "production",
  "decision": "approved",
  "approver": "release.manager@example.com",
  "evidence_packet_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}
```

Optional fields:

- `branch`
- `target_sha`
- `ticket_id`
- `evidence_packet_uri`
- `evidence_summary`
- `risk_severity`
- `risk_acceptance_reason`
- `expires_at`

`expires_at` uses Unix epoch milliseconds, matching other GitGov API timestamp conventions.

## Validation Rules

- `repository_full_name` must look like `owner/repo`.
- `target_sha`, when present, must be 7 to 64 hexadecimal characters.
- `decision` must be one of `approved`, `rejected`, or `accepted-risk`.
- `ticket_id`, when present, must look like `KAN-37`.
- `evidence_packet_hash` is required and must be a 64-character hexadecimal SHA-256 hash.
- `evidence_packet_uri`, when present, must be a relative API path or `http(s)` URL. Local files and custom schemes are rejected.
- `evidence_summary` must be a JSON object and is size-bounded.
- `risk_severity` must be `none`, `low`, `medium`, `high`, or `critical`.
- `approved` cannot be used for high or critical risk.
- `accepted-risk` requires a non-`none` risk severity, reason and future expiration.
- accepted-risk expiration cannot be more than 366 days in the future.

## Database

Migration:

```text
gitgov/gitgov-server/supabase/supabase_schema_v24.sql
```

Post-check:

```text
gitgov/gitgov-server/supabase/checks/v24_postcheck.sql
```

Table:

```text
enterprise_release_approvals
```

The table is append-only in this MVP. There is no update or delete endpoint. A changed decision should create a new approval record with its own hash and audit entry.

## Security Notes

- The API does not read or print provider secrets.
- The evidence packet hash is required so the approval can point to immutable evidence content.
- The admin audit entry stores only release metadata, decision, risk level, ticket ID and approval hash.
- Evidence URIs deliberately reject `file://` and custom protocol values.

## KAN-43 Dashboard Follow-Up

KAN-43 adds the first dashboard release approval wizard on top of this backend API.

The follow-up adds:

- recent approval list in the admin dashboard.
- create approval form with evidence hash, approver, decision, risk, expiration and explicit operator confirmation.
- Tauri client commands for the existing list/create backend routes.

The server-side validation and append-only data model remain owned by KAN-37.

## Remaining Non-Goals

- No multi-approver quorum engine.
- No cryptographic human signature.
- No automatic release gate enforcement from approval state.
- No Vercel AI SDK Copilot work.

Those are follow-ups after the formal approval record exists.
