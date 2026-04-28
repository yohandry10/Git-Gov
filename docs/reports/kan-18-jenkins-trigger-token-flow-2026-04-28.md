# KAN-18 Jenkins Trigger-Only Token Flow

Date: 2026-04-28

## Purpose

Close the documentation and validation gap for the optional Jenkins `/build?token=...` flow.

## Result

Added:

- `scripts/jenkins/validate_trigger_token_flow.ps1`
- `docs/runbooks/jenkins-trigger-token-flow.md`

## Behavior

The validator:

- loads ignored local env files by default
- authenticates to Jenkins with `JENKINS_USER` and `JENKINS_API_TOKEN` for safe inspection
- checks the configured job and last build metadata
- checks whether `JENKINS_BUILD_TRIGGER_TOKEN` is loaded
- inspects Jenkins job config for an `authToken` node when available
- emits only redacted trigger URL output
- does not trigger a build unless `-Trigger` is explicitly passed

## Decision

Authenticated Jenkins API remains the default path for agent work. The trigger-only token remains optional and should only be used for manual or unauthenticated build-start scenarios.

## Validation

Dry-run:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1
```

Latest dry-run result:

- `ok=true`
- Jenkins API inspection passed for `gitgov-demo-pipeline`
- last build `#30`, result `SUCCESS`, not building
- `JENKINS_BUILD_TRIGGER_TOKEN` was not loaded
- trigger URL was emitted only in redacted form with `token=***`
- no build was triggered

Strict dry-run:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1 -RequireTriggerToken
```

Triggering a real build is intentionally explicit:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1 -RequireTriggerToken -Trigger
```

## Remaining Work

No platform code is required unless trigger-only builds become part of a formal CI/CD path. If that happens, add scheduled/manual validation evidence and keep trigger launch separate from authenticated log/status inspection.
