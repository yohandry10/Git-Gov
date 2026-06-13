# KAN-46 Release Governance Evaluator MVP

Updated: 2026-05-01

## Summary

KAN-46 turns the KAN-45 release governance profile into an evaluated product result.

GitGov can now answer a direct question for a release:

```text
Given this org policy, repository, release, environment, and evidence packet hash, is the release governance policy satisfied?
```

The important default remains unchanged: `record-only` does not block anything. Blocking appears only when the customer has explicitly selected a blocking policy such as `approval-required` or `quorum-required`.

## Backend Endpoint

New admin endpoint:

```text
GET /enterprise/release-governance/evaluate
```

Query fields:

| Field | Required | Meaning |
| --- | --- | --- |
| `org_name` | For global admin keys | Organization scope. |
| `repository_full_name` | Yes | Repository in `owner/repo` form. |
| `release_id` | Yes | Customer release identifier. |
| `environment` | Yes | Environment being evaluated, for example `production`. |
| `evidence_packet_hash` | Yes | SHA-256 evidence packet hash to bind approval evidence to a specific packet. |

The current backend requires a known release-bound evidence packet hash and rejects requests whose packet binding does not match repository, release, branch, target SHA, or environment.

The endpoint is admin-only and is included in the stale-auth-cache sensitive route set.

## Response Semantics

The response returns:

- `status`: one of `recorded`, `advisory-warning`, `approved`, `would-block`, or `blocked`.
- `policy_satisfied`: whether the selected policy requirements are met.
- `blocking`: true only when the customer policy is blocking and requirements are not met.
- `would_block`: true when the selected mode would prevent release if used as a gate.
- `valid_approval_count` and `required_approval_count`.
- `policy`: the normalized release governance policy GitGov evaluated.
- `approvals`: matching approval records summarized for operator review.
- `issues`: plain-language reasons the release is not satisfied.
- `next_steps`: what the operator should do next.

Status behavior:

| Policy mode | Missing approval result | Blocks by default |
| --- | --- | --- |
| `record-only` | `recorded` | No |
| `advisory` | `advisory-warning` | No |
| `approval-required` | `blocked` when enforcement is `blocking` | Yes, only because customer selected it |
| `quorum-required` | `blocked` when quorum is incomplete | Yes, only because customer selected it |

## Quorum MVP

KAN-46 does not add a database migration.

To support role-based quorum without changing the existing approval table, GitGov reads the approver role from approval metadata:

```json
{
  "evidence_summary": {
    "approver_role": "security"
  }
}
```

The Desktop release approval flow now captures an optional approver role and stores it in `evidence_summary.approver_role`. The evaluator counts distinct approvers by role and compares them with the KAN-45 profile quorum rules.

## Desktop Flow

The release approval surface now includes:

- an `Approver role` field.
- an `Evaluate governance` action.
- a governance result panel showing policy mode, enforcement, valid approval count, required approval count, blocking/would-block flags, quorum rules, issues, and next steps.

After a new approval is created, the release approval surface refreshes the governance evaluation for the current release form.

## Enforcement Follow-Up

KAN-47 consumes this evaluator through an optional workflow gate. The KAN-47 gate remains manual/report-only by default and fails only when enforcement is explicitly requested and this evaluator returns `blocking=true`.

## Security Notes

- The evaluator reuses admin auth and org-scope rules.
- Global admin keys still require an explicit `org_name`.
- The endpoint reads adoption profile policy and release approval metadata only.
- It does not read provider tokens, `.env` values, Authorization headers, or raw customer secrets.
- Evidence packet matching uses hashes and binding metadata, not raw packet contents.
- `record-only` remains safe for default onboarding because it cannot create a blocking result.

## Non-Goals

- No customer workflow is mutated.
- No remote provider state is changed.
- No database migration is added.
- No release is blocked unless an explicit workflow or caller, such as the KAN-47 optional gate or KAN-83 deployment authorization API, treats `blocking=true` as a gate.
- No cryptographic human signature model is added.
- No default multi-approver requirement is introduced.
