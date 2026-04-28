# Jenkins Trigger-Only Token Flow Runbook

Date: 2026-04-28

## Purpose

Use this runbook only when the unauthenticated/manual Jenkins URL flow is required:

```text
{JENKINS_SERVER_URL}/job/{JENKINS_JOB_NAME}/build?token={JENKINS_BUILD_TRIGGER_TOKEN}
```

For normal agent work, prefer authenticated Jenkins API access through:

- `JENKINS_SERVER_URL`
- `JENKINS_USER`
- `JENKINS_API_TOKEN`
- `JENKINS_JOB_NAME`

This authenticated API path is already the configured operating path for the agent. It is sufficient for inspection, logs, queue state, build history, and authenticated build operations.

The trigger-only token can start builds, but it cannot inspect logs, queue state, or build results. API access remains required for verification.

## Secret Handling

- Store the trigger token only in ignored local env files as `JENKINS_BUILD_TRIGGER_TOKEN`.
- Do not commit or print the token.
- Do not paste the token into GitHub Actions variables.
- If the flow is not needed, leave `JENKINS_BUILD_TRIGGER_TOKEN` unset.

## Validation Script

Dry-run inspection:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1
```

Strict dry-run, requiring the token to be present and aligned:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1 -RequireTriggerToken
```

Actually trigger a build:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1 -RequireTriggerToken -Trigger
```

Do not use `-Trigger` unless a build is expected.

## Expected Dry-Run Output

The script reports:

- Jenkins API inspection status
- last observed build number/result
- whether `JENKINS_BUILD_TRIGGER_TOKEN` is loaded
- whether Jenkins job config exposes an `authToken`
- whether the loaded token matches the job config when Jenkins exposes it
- a redacted trigger URL with `token=***`

The script never prints the trigger token value.

## Operating Rules

- Use authenticated Jenkins API for diagnostics and build verification.
- Use trigger-only URL only for manual/unauthenticated launch scenarios.
- If a build is triggered through the URL flow, immediately verify status through Jenkins API or `validate_provider_access.ps1`.
- Keep `JENKINS_WEBHOOK_SECRET` separate from `JENKINS_BUILD_TRIGGER_TOKEN`; the webhook secret authenticates Jenkins-to-GitGov telemetry, not Jenkins build starts.

## Rollback

If the trigger token is suspected leaked:

1. Rotate the token in Jenkins job configuration.
2. Update ignored local env files.
3. Re-run strict dry-run validation.
4. Do not reuse the old URL.
