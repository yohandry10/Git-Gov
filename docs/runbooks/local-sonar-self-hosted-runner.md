# Local SonarQube Self-Hosted Runner Runbook

Date: 2026-04-28

## Purpose

Use this runbook only if GitHub Actions needs to run a real SonarQube scan against the local SonarQube server at `http://localhost:9000`.

Current default behavior is intentional:

- `sonar-governance.yml` runs on GitHub-hosted runners.
- GitHub-hosted runners cannot reach the workstation-local SonarQube endpoint.
- The workflow skips local SonarQube scans instead of failing CI.
- Jenkins/local validation remains the supported Sonar path until a self-hosted runner is added.

Do not make the Sonar workflow blocking on a self-hosted runner until the runner is registered, online, and validated.

## Preconditions

- Local Docker Desktop is running.
- Local SonarQube is running at `http://localhost:9000`.
- Local Jenkins is running at `http://localhost:8096` if Jenkins evidence is part of the validation.
- Ignored local env files contain the required provider credentials.
- GitHub repository admin access is available.

Local validation command:

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

Expected result:

- top-level `ok` is `true`
- `sonarqube.ok` is `true`
- `jenkins.ok` is `true`
- `release-readiness.ok` is `true`

## Recommended Runner Model

Use a dedicated GitHub self-hosted runner for this repository.

Recommended labels:

- `self-hosted`
- `Windows`
- `X64`
- `gitgov-local-sonar`

Reasoning:

- The runner must be on the same workstation or network segment that can reach `http://localhost:9000`.
- A dedicated label avoids accidentally scheduling GitGov validation on unrelated self-hosted runners.
- Keeping the current GitHub-hosted path unchanged prevents branch protection from waiting on an offline local machine.

## GitHub Setup

1. Open the repository in GitHub.
2. Go to `Settings > Actions > Runners`.
3. Click `New self-hosted runner`.
4. Choose the runner OS that matches the workstation.
5. Follow GitHub's generated commands to download and configure the runner.
6. Add the custom label `gitgov-local-sonar` during or after runner registration.
7. Start the runner and confirm it appears as `Idle` in GitHub.

Do not paste runner registration tokens into tracked files or documentation.

## Repository Configuration

Required repository Actions configuration:

| Type | Name | Value |
|---|---|---|
| Variable | `SONAR_HOST_URL` | `http://localhost:9000` when the runner is on the same host as SonarQube |
| Variable | `SONAR_PROJECT_KEY` | `yohandry10_git-gov` |
| Secret | `SONAR_TOKEN` | Local SonarQube token |
| Variable | `GITGOV_URL` | `https://gitgov-api.onrender.com` |
| Secret | `GITGOV_API_KEY` | GitGov admin API key |

Optional telemetry secret:

| Type | Name | Purpose |
|---|---|---|
| Secret | `GITGOV_JENKINS_SECRET` | Adds the Jenkins shared-secret header when publishing Sonar telemetry through `/integrations/jenkins` |

## Activation Pattern

Keep the existing GitHub-hosted workflow path until the self-hosted runner is validated.

Safe activation sequence:

1. Register the runner.
2. Validate local providers:

   ```powershell
   .\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
   ```

3. Trigger a manual Sonar validation branch or draft PR that uses `runs-on: [self-hosted, gitgov-local-sonar]`.
4. Confirm the Sonar scan reaches `http://localhost:9000`.
5. Confirm telemetry publishes to GitGov without printing secrets.
6. Only then decide whether the self-hosted Sonar check should become required.

## Workflow Change Template

Use this template only after the runner is online.

```yaml
jobs:
  sonar:
    name: Sonar Scan + Quality Gate
    runs-on: [self-hosted, gitgov-local-sonar]
```

Do not apply this to the default required workflow while the runner is offline or experimental.

## Rollback

If the local runner goes offline or blocks PRs:

1. Revert the workflow `runs-on` change back to `ubuntu-latest`.
2. Keep `continue-on-error: true` for Sonar governance unless branch protection is intentionally updated.
3. Re-run the PR checks.
4. Validate provider access locally before attempting another activation.

## Acceptance Criteria

- GitHub shows the runner online with label `gitgov-local-sonar`.
- The runner can reach `http://localhost:9000/api/system/status`.
- `validate_provider_access.ps1 -IncludeReleaseReadiness` returns `ok: true`.
- Sonar workflow telemetry reaches GitGov only through configured secrets and variables.
- Branch protection is not changed until the self-hosted path has at least one successful validation run.
