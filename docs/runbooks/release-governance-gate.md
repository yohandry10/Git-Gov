# Release Governance Gate Runbook

Updated: 2026-05-01

Ticket: `KAN-47`

## Purpose

Use this gate when a customer explicitly wants GitGov release governance to affect a release decision.

Default use is report-only. Enforcement must be selected deliberately.

## Local Report-Only Check

```powershell
.\scripts\control-plane\validate_release_governance_gate.ps1 `
  -RepositoryFullName yohandry10/Git-Gov `
  -ReleaseId KAN-47 `
  -Environment production `
  -OrgName yohandry10 `
  -OutputPath out\release-governance-gate.json
```

## Local Enforcement Check

```powershell
.\scripts\control-plane\validate_release_governance_gate.ps1 `
  -RepositoryFullName yohandry10/Git-Gov `
  -ReleaseId KAN-47 `
  -Environment production `
  -OrgName yohandry10 `
  -Enforce `
  -OutputPath out\release-governance-gate-enforce.json
```

`-Enforce` fails only when the evaluator reports `blocking=true`.

Use `-FailOnWouldBlock` only when advisory/would-block states should fail too. Use `-RequirePolicySatisfied` only when every unsatisfied policy should fail.

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
status=recorded
policy_mode=record-only
blocking=false
would_block=false
```

That proves the gate can observe release governance without changing the customer's default release behavior.
