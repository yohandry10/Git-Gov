# GitGov Operations Access Runbook

Updated: 2026-04-24

## Purpose

This runbook documents the operational access surfaces used to validate GitGov end-to-end without exposing secrets.

It covers:

- GitHub repository and Actions operations.
- Render backend deployment checks.
- Local SonarQube checks.
- Local Jenkins checks.
- Where credentials live locally.
- What the agent can safely do with each access surface.

## Credential Storage

Operational credentials are stored only in ignored local `.env` files:

- `gitgov/.env`
- `gitgov/gitgov-server/.env`

Never commit or print these values.

Tracked documentation may name variables and services, but must not include token values.

## GitHub

Repository:

- Owner/repo: `yohandry10/Git-Gov`
- Default branch: `main`
- GitHub CLI path: `C:\Users\PC\Tools\gh\bin\gh.exe`
- Authenticated user: `yohandry10`

Validated capabilities:

- Inspect branches, commits, PRs, collaborators, contributors, workflow runs, and branch protection.
- Read GitHub Actions logs.
- Push branches and create PRs.
- Configure branch protection and required checks when explicitly requested.
- Configure Actions secrets/variables when explicitly requested.

Current branch protection on `main`:

- Strict status checks enabled.
- Admin enforcement enabled.
- Required checks:
  - `Security Guard`
  - `Server Clippy + Check`
  - `Desktop Rust Clippy`
  - `Frontend Lint + Typecheck`
  - `Website Lint + Typecheck + Build`
  - `Validate quality_gates warn/block matrix`

GitHub Actions repository configuration:

- Secret: `GITGOV_API_KEY`
- Variable: `GITGOV_URL=https://gitgov-api.onrender.com`
- Variable: `SONAR_HOST_URL=http://localhost:9000`
- Variable: `SONAR_PROJECT_KEY=yohandry10_git-gov`
- SonarCloud is not used for this repository because the connected GitHub account is personal, not organizational.
- GitHub-hosted Sonar scan remains non-blocking/skipped when `SONAR_HOST_URL` points to local SonarQube, because hosted runners cannot reach the workstation.

## Render

Backend service:

- Service name: `gitgov-api`
- Service ID: `srv-d7lgtc77f7vs73b38uqg`
- URL: `https://gitgov-api.onrender.com`
- Branch: `main`
- Root directory: `gitgov/gitgov-server`
- Runtime: Docker web service
- Region: Oregon

Local credential:

- `RENDER_API_KEY`

Validated capabilities:

- Query service metadata through Render API.
- Inspect deploy state and logs.
- Verify deployed backend health.
- Trigger or monitor deploys when explicitly requested.

## Local SonarQube

Local endpoint:

- `http://localhost:9000`

Docker Compose profile:

```powershell
docker compose --profile sonar up -d sonarqube-db sonarqube
```

Local env variables:

- `SONAR_HOST_URL=http://localhost:9000`
- `SONAR_TOKEN`
- `SONAR_PROJECT_KEY=yohandry10_git-gov`

Current local token metadata:

- Token name: `gitgov-local`
- Expiration: May 22, 2026

Validated capabilities:

- Authenticate to SonarQube API with local token.
- Query authentication status.
- Query projects, quality gates, measures, issues, hotspots, and analysis status.
- Use UI navigation through `@browser-use` when needed for UI-only flows.

Operational decision:

- SonarQube local is the supported Sonar runtime for this repo.
- Do not default workflows or docs to `https://sonarcloud.io`.
- `SONAR_HOST_URL=http://localhost:9000` is valid for local tooling and as an explicit signal that GitHub-hosted scan must skip unless a self-hosted runner is used.

## Local Jenkins

Local endpoint:

- `http://localhost:8096`

Docker Compose profile:

```powershell
docker compose --profile jenkins up -d jenkins
```

Local env variables:

- `JENKINS_SERVER_URL=http://localhost:8096`
- `JENKINS_USER=admin`
- `JENKINS_API_TOKEN`
- `JENKINS_JOB_NAME=gitgov-demo-pipeline`

Current local token metadata:

- Token name: `codex-local`

Validated capabilities:

- Authenticate to Jenkins API as `admin`.
- Query `/whoAmI/api/json`.
- Inspect job metadata.
- Inspect build history, status, and logs.
- Inspect queue state.
- Trigger authenticated builds when explicitly requested.

Last validated job state:

- Job: `gitgov-demo-pipeline`
- Buildable: `true`
- Last observed build: `#30`
- Last observed result: `SUCCESS`
- Last observed building state: `false`

Trigger-only URL flow:

```text
${JENKINS_SERVER_URL}/job/${JENKINS_JOB_NAME}/build?token=${JENKINS_BUILD_TRIGGER_TOKEN}
```

This flow is separate from `JENKINS_API_TOKEN`. Use it only when the job is configured with a build trigger token.

## Local Stack Commands

Start Jenkins and SonarQube:

```powershell
docker compose --profile jenkins --profile sonar up -d jenkins sonarqube-db sonarqube
```

Check containers:

```powershell
docker compose ps
```

Validate SonarQube token without printing it:

```powershell
$envFile = "gitgov/gitgov-server/.env"
$vars = @{}
Get-Content $envFile | ForEach-Object {
  if ($_ -match "^([^#=]+)=(.*)$") { $vars[$matches[1]] = $matches[2] }
}
$basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes($vars["SONAR_TOKEN"] + ":"))
Invoke-RestMethod -Uri "$($vars["SONAR_HOST_URL"])/api/authentication/validate" -Headers @{ Authorization = "Basic $basic" }
```

Validate Jenkins token without printing it:

```powershell
$envFile = "gitgov/gitgov-server/.env"
$vars = @{}
Get-Content $envFile | ForEach-Object {
  if ($_ -match "^([^#=]+)=(.*)$") { $vars[$matches[1]] = $matches[2] }
}
$basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes($vars["JENKINS_USER"] + ":" + $vars["JENKINS_API_TOKEN"]))
Invoke-RestMethod -Uri "$($vars["JENKINS_SERVER_URL"])/whoAmI/api/json" -Headers @{ Authorization = "Basic $basic" }
```

## Safety Rules

- Do not commit `.env`, `.env.local`, `.env.*.local`, `.mcp.json`, or files under `secrets/`.
- Do not print token values.
- Use GitHub Actions secrets for sensitive values.
- Use GitHub Actions variables only for non-sensitive values.
- Confirm before creating, rotating, deleting, or transmitting credentials.
- Confirm before changing repo visibility, branch protection, or cloud sharing/access settings.
- Prefer API/CLI checks over browser clicks when local credentials are present.
