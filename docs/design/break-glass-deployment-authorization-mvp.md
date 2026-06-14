# KAN-87 Break-glass Deployment Authorization MVP

Updated: 2026-06-14

## Summary

KAN-87 adds explicit break-glass authorization to Deployment Gates.

When a deployment would be blocked by a customer-selected blocking release governance policy, an Admin caller can submit a break-glass request with a mandatory reason. GitGov records the original blocking policy result, the exception reason, the authorizing actor, optional expiry, request payload, policy checksum, and the final `break_glass` decision.

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
    "reason": "Production incident INC-2026-0614 requires immediate rollback while approval evidence is restored.",
    "authorized_by": "incident.commander@example.com",
    "expires_at": 1781413200000
  }
}
```

## Decision Rules

- If the evaluated policy does not block, `break_glass` is rejected.
- If the evaluated policy blocks and `break_glass` is valid, the final decision is `break_glass`.
- `approved=true`, because GitGov authorized the deployment exception.
- `blocking=true` and `would_block=true` remain true, because the underlying policy still blocked.
- `blocked_by` remains populated with the original governance blockers.
- `break_glass_reason`, `break_glass_authorized_by`, and `break_glass_expires_at` are persisted and returned in history.

## Validation

- `break_glass.requested` must be `true` when the object is provided.
- `break_glass.reason` is required, trimmed, and must be at least 16 characters.
- `break_glass.authorized_by` is optional; when missing, GitGov records the authenticated API client id.
- `break_glass.expires_at` is optional, must be in the future, and cannot be more than 24 hours ahead.
- Developer API keys still cannot authorize deployment gates.
- Scoped Admin keys still cannot write another tenant's deployment authorization history.
- Evidence packet binding and ticket matching rules remain unchanged.

## Desktop Visibility

`Governance > Releases` now distinguishes:

- `break-glass eligible`: the deploy was blocked and could have used an exception;
- `break-glass used`: the deploy was authorized by exception;
- exception reason, authorizing actor, expiry, and original blockers.

## Non-Goals

- No automatic provider mutation.
- No automatic branch protection mutation.
- No long-lived standing bypass token.
- No break-glass for non-blocking/advisory/record-only evaluations.
- No hiding of original blockers when an exception is used.
