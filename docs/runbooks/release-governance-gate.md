# Release Governance Gate Runbook

Updated: 2026-06-13

Tickets: `KAN-47`, `KAN-49`, `KAN-84`

## Purpose

Use this gate when a customer explicitly wants GitGov release governance to affect a release decision.

Default use is report-only. Enforcement must be selected deliberately. Since `KAN-84`, the local script
and generated workflow templates call the Deployment Gates authorization API and persist an authorization
history record. They no longer call only the lower-level release-governance evaluator.

## Local Report-Only Check

```powershell
.\scripts\control-plane\validate_release_governance_gate.ps1 `
  -RepositoryFullName yohandry10/Git-Gov `
  -Branch main `
  -TargetSha abcdef1234567890abcdef1234567890abcdef12 `
  -ReleaseId KAN-47 `
  -Environment production `
  -Deployer gitgov-validator `
  -TicketId KAN-47 `
  -EvidencePacketHash 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef `
  -OrgName yohandry10 `
  -OutputPath out\release-governance-gate.json
```

## Local Enforcement Check

```powershell
.\scripts\control-plane\validate_release_governance_gate.ps1 `
  -RepositoryFullName yohandry10/Git-Gov `
  -Branch main `
  -TargetSha abcdef1234567890abcdef1234567890abcdef12 `
  -ReleaseId KAN-47 `
  -Environment production `
  -Deployer gitgov-validator `
  -TicketId KAN-47 `
  -EvidencePacketHash 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef `
  -OrgName yohandry10 `
  -Enforce `
  -OutputPath out\release-governance-gate-enforce.json
```

`-Enforce` fails only when the authorization response reports `blocking=true`.

Use `-FailOnWouldBlock` only when advisory/would-block states should fail too. Use `-RequirePolicySatisfied` only when every unsatisfied policy should fail.

`-EvidencePacketHash` must be the SHA-256 hash of a release-bound GitGov evidence packet whose
repository, branch, target SHA, release id, and environment match the request. This is deliberate:
Deployment Gates should never authorize one commit with evidence generated for another commit.

## GitHub Workflow

Workflow:

```text
.github/workflows/release-governance-gate.yml
```

It is manual only:

```text
workflow_dispatch
```

Required configuration:

- GitHub Actions variable: `GITGOV_URL`.
- GitHub Actions secret: `GITGOV_API_KEY`.

Do not paste secret values into workflow YAML.

## Expected Safe Default

When the organization profile is still `record-only`, the gate should return a passing report with:

```text
status=passed
authorization.decision=approved or advisory
policy_mode=record-only
blocking=false
would_block=false
```

That proves the gate can observe release governance without changing the customer's default release behavior.

## Artifact Monitor

KAN-49 adds a separate monitor for the evidence artifact produced by the gate:

```text
.github/workflows/release-governance-gate-artifact-monitor.yml
```

This monitor is manual only in the GitGov repository. It checks the latest successful `release-governance-gate.yml` run for a fresh artifact named like:

```text
release-governance-gate-*
```

Default accepted age:

```text
720 hours
```

Run it after at least one successful release governance gate run. If it fails with missing or expired artifact, rerun the Release Governance Gate to create fresh evidence.

Enterprise workflow packs include this monitor only when:

- the customer enabled `formal-approval`;
- the release governance policy is non-`record-only` through the base policy or an environment override; and
- the customer selected `artifact-monitoring`.

It is not generated for default `record-only` release approval evidence.
