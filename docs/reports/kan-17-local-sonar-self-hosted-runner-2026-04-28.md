# KAN-17 Local Sonar Self-Hosted Runner Runbook

Date: 2026-04-28

## Purpose

Close the operational documentation gap for using local SonarQube from GitHub Actions.

## Result

Added `docs/runbooks/local-sonar-self-hosted-runner.md`.

The runbook covers:

- why GitHub-hosted runners skip local SonarQube by design
- preconditions for local Sonar/Jenkins validation
- recommended self-hosted runner labels
- GitHub runner setup path
- repository variables/secrets needed for Sonar telemetry
- safe activation sequence
- workflow `runs-on` template for a validated runner
- rollback steps if the runner blocks PRs

## Decision

No workflow behavior was changed in this ticket.

Reason:

- The repository does not currently have a validated dedicated self-hosted runner.
- Changing the required Sonar workflow to `runs-on: [self-hosted, gitgov-local-sonar]` before the runner exists could leave required checks queued and block PRs.
- The current GitHub-hosted/non-blocking behavior is still correct until the runner is operational.

## Validation

Local documentation and publication checks:

```powershell
git diff --check
.\scripts\security\publication_guard.ps1
```

Provider access validation remains:

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

## Remaining Action

Only if GitHub Actions must run real Sonar scans against local SonarQube:

1. Register a dedicated self-hosted runner.
2. Add label `gitgov-local-sonar`.
3. Validate access from the runner host.
4. Run a non-required/manual validation first.
5. Update workflow/branch protection only after successful validation.
