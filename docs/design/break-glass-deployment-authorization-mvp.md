# KAN-87 Break-glass Deployment Authorization MVP

Updated: 2026-06-14

## Summary

KAN-87 adds explicit break-glass authorization to Deployment Gates. KAN-88 hardens that path with approval routing: a deploy request can use break-glass only when a matching unexpired approval already exists.

When a deployment would be blocked by a customer-selected blocking release governance policy, an Admin caller can create a break-glass approval with a mandatory reason, approver, role, expiry, evidence packet hash, release id, repository, branch, target SHA, and environment. The later deploy authorization request references that approval, and GitGov records the original blocking policy result, the approval id/hash, exception reason, approver, expiry, request payload, policy checksum, and final `break_glass` decision.

## Approval Route

`POST /deployment-gates/break-glass-approvals` creates a pre-approval:

```json
{
  "release_id": "release-2026.06.14",
  "repository_full_name": "owner/repo",
  "branch": "main",
  "target_sha": "abcdef1234567890abcdef1234567890abcdef12",
  "environment": "production",
  "ticket_id": "KAN-88",
  "evidence_packet_hash": "64-hex-content-hash",
  "reason": "Production incident INC-2026-0614 requires immediate rollback while approval evidence is restored.",
  "approver": "incident.commander@example.com",
  "approver_role": "incident_commander",
  "expires_at": 1781413200000,
  "metadata": {
    "incident": "INC-2026-0614"
  }
}
```

`GET /deployment-gates/break-glass-approvals` lists approvals with org-scoped filters for approval id, repository, branch, target SHA, release id, environment, evidence hash, approver, and active-only status.

## Request Shape

`POST /deployment-gates/authorize` accepts optional `break_glass`:

```json
{
  "release_id": "release-2026.06.14",
  "repository_full_name": "owner/repo",
  "branch": "main",
  "target_sha": "abcdef1234567890abcdef1234567890abcdef12",
  "environment": "production",
  "deployer": "github-actions",
  "ticket_id": "KAN-87",
  "evidence_packet_hash": "64-hex-content-hash",
  "requested_by": "deploy-bot",
  "break_glass": {
    "requested": true,
    "approval_id": "dgbga_...",
    "reason": "Production incident INC-2026-0614 requires immediate rollback while approval evidence is restored.",
    "authorized_by": "incident.commander@example.com",
    "expires_at": 1781413200000
  }
}
```

## Decision Rules

- If the evaluated policy does not block, `break_glass` is rejected.
- If the evaluated policy blocks and `break_glass` references or matches a valid unexpired approval, the final decision is `break_glass`.
- If no matching approval exists, the deploy authorization request is rejected and no authorization history row is written.
- A matching approval must use the same release id, repository, branch, target SHA, environment, ticket id when supplied, and evidence packet hash.
- The break-glass approver must be separate from the deployer/requester.
- `approved=true`, because GitGov authorized the deployment exception.
- `blocking=true` and `would_block=true` remain true, because the underlying policy still blocked.
- `blocked_by` remains populated with the original governance blockers.
- `break_glass_reason`, `break_glass_authorized_by`, `break_glass_expires_at`, `break_glass_approval_id`, and `break_glass_approval_hash` are persisted and returned in history.

## Validation

- `break_glass.requested` must be `true` when the object is provided.
- `break_glass.reason` is required, trimmed, and must be at least 16 characters.
- `break_glass.approval_id` is optional only when exactly one active approval matches the deploy scope.
- `break_glass.authorized_by` is optional; when present, it must match the approval approver.
- `break_glass.expires_at` is optional, must be in the future, and cannot be more than 24 hours ahead.
- approval expiry is required and cannot be more than 24 hours ahead.
- Developer API keys still cannot authorize deployment gates.
- Scoped Admin keys still cannot write another tenant's deployment authorization history.
- Evidence packet binding and ticket matching rules remain unchanged.

## Desktop Visibility

`Governance > Releases` now distinguishes:

- `break-glass eligible`: the deploy was blocked and could have used an exception;
- `break-glass used`: the deploy was authorized by exception;
- `pre-approved`: the exception was backed by a persisted approval;
- exception reason, approver, approval id/hash, expiry, and original blockers.

## Non-Goals

- No automatic provider mutation.
- No automatic branch protection mutation.
- No long-lived standing bypass token.
- No break-glass for non-blocking/advisory/record-only evaluations.
- No hiding of original blockers when an exception is used.
