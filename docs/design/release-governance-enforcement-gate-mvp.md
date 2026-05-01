# KAN-47 Release Governance Enforcement Gate MVP

Updated: 2026-05-01

## Summary

KAN-47 adds the first optional release governance gate on top of the KAN-46 evaluator.

The rule remains explicit:

- `record-only` stays non-blocking.
- GitGov does not fail customer releases by default.
- A workflow can fail only when the operator/customer explicitly runs the gate with enforcement enabled and the KAN-46 evaluator reports `blocking=true`.

## Components

- Local/CI script: `scripts/control-plane/validate_release_governance_gate.ps1`.
- GitGov repo workflow: `.github/workflows/release-governance-gate.yml`.
- Enterprise CLI template support: `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- Dashboard workflow pack support: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Runbook: `docs/runbooks/release-governance-gate.md`.

## Gate Behavior

The script calls:

```text
GET /enterprise/release-governance/evaluate
```

It records a sanitized JSON report with:

- evaluated release id.
- repository.
- environment.
- policy mode and enforcement.
- `policy_satisfied`.
- `blocking`.
- `would_block`.
- valid and required approval counts.
- issues and next steps.

Exit behavior:

| Mode | Fails on `blocking=true` | Fails on `would_block=true` | Default |
| --- | --- | --- | --- |
| report-only | No | No | Yes |
| `-Enforce` | Yes | No | No |
| `-FailOnWouldBlock` | No | Yes | No |
| `-RequirePolicySatisfied` | Fails whenever policy is not satisfied | Fails whenever policy is not satisfied | No |

The GitHub workflow is `workflow_dispatch` only. It does not run on push, pull request, or schedule by default.

## Enterprise Template Behavior

Workflow template packs include `release-governance-gate.yml` only when:

- the customer enabled the `formal-approval` module, and
- `release_governance.mode` is not `record-only`.

Generated defaults follow the customer-selected policy:

- `advisory`: generated gate remains report/advisory by default.
- `approval-required`: generated gate defaults to enforcement.
- `quorum-required`: generated gate defaults to enforcement.

This keeps onboarding safe while making customer-selected enforcement portable.

## Secret Safety

- The script reads secrets from ignored env files or process environment only.
- The report never stores API key values.
- The script and workflow do not print Authorization headers.
- Generated customer templates contain variable and secret names only.
- Approval identity details are omitted from the script report by default.

## Non-Goals

- No database migration.
- No new backend endpoint.
- No default release blocking.
- No remote customer repository mutation.
- No cryptographic signing or first-class approver-role table.
