# KAN-83 Deployment Authorization API MVP

Updated: 2026-06-14

## Summary

KAN-83 turns the existing release-governance evaluator into a CI/CD-facing deployment authorization API.

The endpoint is intentionally conservative:

- it requires a release-bound evidence packet hash;
- it reuses the existing `release_governance` policy semantics;
- it records every authorization attempt in Postgres;
- it blocks only when customer policy explicitly evaluates as blocking;
- it returns advisory warnings for setup gaps without silently failing deployments.

## Backend Contract

Routes:

```text
POST /deployment-gates/authorize
GET /deployment-gates/authorizations
POST /deployment-gates/break-glass-approvals
GET /deployment-gates/break-glass-approvals
```

Both routes are Admin-only and use the same org scoping model as enterprise release governance:

- scoped org API keys can omit `org_name`;
- global Admin/founder keys must pass `org_name`;
- cross-tenant access is rejected.

`POST /deployment-gates/authorize` request:

```json
{
  "org_name": "example-org",
  "release_id": "release-2026.06.13",
  "repository_full_name": "example-org/app",
  "branch": "main",
  "target_sha": "abcdef1234567890abcdef1234567890abcdef12",
  "environment": "production",
  "deployer": "github-actions",
  "ticket_id": "KAN-83",
  "evidence_packet_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "requested_by": "deploy-bot",
  "deployment_run_id": "gha-123456",
  "break_glass": {
    "requested": true,
    "approval_id": "dgbga_...",
    "reason": "Production incident INC-2026-0614 requires immediate rollback while approval evidence is restored.",
    "authorized_by": "incident.commander@example.com",
    "expires_at": 1781413200000
  },
  "metadata": {
    "workflow": "deploy-production"
  }
}
```

Required fields:

- `release_id`
- `repository_full_name`
- `branch`
- `target_sha`
- `environment`
- `deployer`
- `evidence_packet_hash`

The evidence packet hash must already exist for the organization in `release_evidence_packets`, and its binding must match release id, repository, branch, target SHA, and environment. This prevents a deployment caller from authorizing a different SHA with a stale or unrelated packet.

Response:

```json
{
  "authorization_id": "dga_...",
  "decision": "approved",
  "approved": true,
  "blocking": false,
  "would_block": false,
  "reason": "Deployment approved by current release governance policy.",
  "blocked_by": [],
  "warnings": [],
  "policy_checksum": "...",
  "break_glass_eligible": false,
  "break_glass_used": false,
  "evaluation": {
    "status": "recorded",
    "policy_satisfied": true
  },
  "details": {
    "contract_version": "deployment-gate-authorization.v1"
  }
}
```

Decision semantics:

| Evaluator result | API decision | `approved` | Meaning |
| --- | --- | --- | --- |
| `blocking=true` | `blocked` | `false` | Customer policy explicitly blocks this deployment. |
| `blocking=true` with valid pre-approved `break_glass` | `break_glass` | `true` | Deployment is authorized by audited exception while original blockers remain recorded. |
| `would_block=true` without blocking enforcement | `advisory` | `true` | Deployment may proceed, but GitGov records what would block under enforcement. |
| setup warnings only | `advisory` | `true` | Release policy allows deploy, but onboarding/setup evidence is incomplete. |
| no issues | `approved` | `true` | Current policy is satisfied or record-only. |

## Persistence

Migration:

```text
gitgov/gitgov-server/supabase/supabase_schema_v35.sql
```

Table:

```text
deployment_gate_authorizations
```

The table stores:

- stable `authorization_id`;
- org, release, repository, branch, target SHA, environment, deployer;
- evidence packet hash/URI;
- `decision`, `approved`, `blocking`, `would_block`;
- reason, warnings, blockers, policy checksum;
- break-glass eligibility, usage, reason, authorizer, and optional expiry;
- break-glass approval id/hash when an exception is used;
- full release-governance evaluation;
- compact details including first governed repo setup status;
- original normalized request payload;
- requester and timestamp.

`GET /deployment-gates/authorizations` lists persisted attempts with filters for authorization id, repository, branch, target SHA, release id, environment, decision, and deployer.

## Break-glass Approval Routing

KAN-88 adds explicit pre-approval before a deployment request can use break-glass.

`POST /deployment-gates/break-glass-approvals` persists an approval bound to:

- organization;
- release id;
- repository;
- branch;
- target SHA;
- environment;
- ticket id when present;
- evidence packet hash and URI;
- reason;
- approver and approver role;
- expiry no more than 24 hours ahead;
- approval hash and creator.

`POST /deployment-gates/authorize` accepts `break_glass` only when the evaluated policy is blocking and an active approval with the same binding exists. If `break_glass.approval_id` is sent, that exact approval must match. If it is omitted, GitGov still requires a matching active approval for the same evidence-bound deployment scope.

## Relationship To Existing Work

KAN-80 prepares the first governed repo and surfaces setup readiness.

KAN-46 evaluates release governance policy against approvals and evidence.

KAN-47 provides an optional workflow gate script.

KAN-83 is the stable product API CI/CD systems should call when they need a first-class deployment authorization decision plus audit history.

## Non-Goals

- No default hard blocking for record-only customers.
- No provider-token reads.
- No mutation of GitHub Actions, Jenkins, GitLab, Render, or Vercel.
- No bypass of evidence packet binding.
- No OPA/Rego execution in this slice.
- No cryptographic signing model.
- No Desktop UI history panel in this slice; the backend history API is ready for a later UI.
