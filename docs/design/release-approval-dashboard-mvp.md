# KAN-43 Release Approval Dashboard MVP

Updated: 2026-05-01

## Summary

KAN-43 adds the first operator dashboard flow for formal enterprise release approvals.

KAN-37 created the backend approval record. KAN-43 makes that record usable from the GitGov dashboard so an admin can list recent approvals and create a formal release decision without calling the API manually.

## Scope

- Dashboard panel: `gitgov/src/components/control_plane/ReleaseApprovalPanel.tsx`.
- Store actions: `loadEnterpriseReleaseApprovals` and `createEnterpriseReleaseApproval`.
- Tauri commands:
  - `cmd_server_list_enterprise_release_approvals`.
  - `cmd_server_create_enterprise_release_approval`.
- Backend routes reused:
  - `GET /enterprise/release-approvals`.
  - `POST /enterprise/release-approvals`.

## User Flow

1. Admin opens the Control Plane dashboard.
2. GitGov loads recent release approvals for the selected organization and repository.
3. Admin fills release, repository, branch, environment, decision, approver, evidence hash, and optional ticket/SHA/URI.
4. For accepted risk, admin must provide risk severity, reason, and expiration.
5. Admin must confirm the evidence and decision before submit.
6. GitGov creates an append-only approval record and prepends it to the recent approvals list.

## Client Validation

The dashboard validates before submit:

- release is required.
- repository must look like `owner/repo`.
- environment is required.
- approver is required.
- evidence hash must be a 64-character SHA-256 hex value.
- target SHA, when present, must be 7 to 64 hex characters.
- ticket, when present, must look like `KAN-43`.
- evidence URI must be a relative API path or `http(s)` URL.
- approved decisions cannot carry high or critical risk.
- accepted risk requires non-`none` severity, reason, and 1 to 366 day expiration.
- operator confirmation is required.

Backend KAN-37 validation remains the source of truth. Client validation is an operator safety layer.

## Security Notes

- The dashboard does not read provider secrets or local env files.
- The Tauri commands forward only the configured GitGov API key as Bearer auth.
- Approval records store evidence hashes and metadata, not raw provider credentials.
- The UI requires explicit confirmation before creating a formal decision.
- The API remains admin-only.

## Configurable Governance Defaults

The KAN-43 dashboard creates and lists approval records. It does not silently turn release approvals into a blocking deployment gate.

Default behavior remains `record-only`:

- admins can create approval evidence.
- operators can review approval status.
- reports can include formal release decision evidence.
- pipelines are not failed by default because an approval is missing.
- multiple approvers are not required by default.

Future quorum and release gate enforcement must be customer-configurable opt-in behavior.

Examples:

- A customer can keep `record-only` mode for audit history.
- A customer can choose advisory mode to show warnings without blocking deploys.
- A customer can later choose blocking enforcement for production only.
- A customer can later choose quorum rules such as one engineering approval and one security approval.

Those choices must be explicit. GitGov should not surprise a customer by blocking releases just because the release approval dashboard exists.

See `docs/design/configurable-release-governance-defaults.md`.

## Non-Goals

- No default multi-approver quorum.
- No cryptographic human signature.
- No automatic release gate enforcement from approval state unless a future customer policy explicitly enables it.
- No remote customer repository mutation.
