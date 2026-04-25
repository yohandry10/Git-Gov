# Agent Operating Context

This repository is operated from `C:\Users\PC\Desktop\GitGov` on Windows PowerShell.

## Access

- GitHub CLI is installed at `C:\Users\PC\Tools\gh\bin\gh.exe`.
- `gh` is authenticated as `yohandry10` with admin access to `yohandry10/Git-Gov`.
- GitHub token scopes observed: `repo`, `workflow`, `read:org`, `gist`, `admin:repo_hook`.
- Render API access is available via local ignored env files only. Do not commit or print token values.
- Local Render env key name: `RENDER_API_KEY`.
- Local GitGov API env key name: `GITGOV_API_KEY`.
- Local SonarQube API access is available when `SONAR_HOST_URL`, `SONAR_TOKEN`, and `SONAR_PROJECT_KEY` are loaded from ignored local env files.
- Jenkins direct API access is available when `JENKINS_SERVER_URL`, `JENKINS_USER`, and `JENKINS_API_TOKEN` are loaded from ignored local env files.
- Jira Cloud API access is available when `JIRA_BASE_URL`, `JIRA_EMAIL`, `JIRA_API_TOKEN`, and `JIRA_PROJECT_KEY` are loaded from ignored local env files.
- Local ignored env files currently used by the agent:
  - `C:\Users\PC\Desktop\GitGov\gitgov\.env`
  - `C:\Users\PC\Desktop\GitGov\gitgov\gitgov-server\.env`
- Treat these local env files as the source of truth for provider tokens. Never print token values from them.

## Agent Capabilities

- GitHub:
  - Inspect repo state, branches, commits, PRs, checks, branch protection, and workflow runs through `gh`.
  - Create branches, push commits, open PRs, merge PRs when checks pass and user intent is clear.
  - Read GitHub Actions logs and rerun workflows when needed.
  - Manage repository Actions variables and secrets when explicitly requested; secret creation or updates are sensitive operations.
  - Manage repository webhooks through `gh` when explicitly requested; webhook secrets must come from ignored local env files only.
- Render:
  - Query service metadata, deployments, logs, and health through the Render API.
  - Verify the deployed backend at `https://gitgov-api.onrender.com`.
  - Trigger deploys or inspect deploy failures when `RENDER_API_KEY` is present locally.
- Local SonarQube:
  - Access local SonarQube at `http://localhost:9000`.
  - Authenticate by API using `SONAR_TOKEN` from ignored env files.
  - Query project quality gate, measures, issues, hotspots, and analysis status for `SONAR_PROJECT_KEY=yohandry10_git-gov`.
  - Use the browser session for UI-only operations when `@browser-use` is explicitly available.
- Jenkins:
  - Access local Jenkins at `http://localhost:8096`.
  - Authenticate by API as `admin` using `JENKINS_API_TOKEN` from ignored env files.
  - Inspect job metadata, build history, build logs, queue state, and build results.
  - Current job name: `gitgov-demo-pipeline`.
  - Trigger Jenkins builds by authenticated API when requested. Trigger-only `/build?token=...` needs `JENKINS_BUILD_TRIGGER_TOKEN` if that flow is required.
- Jira:
  - Access Jira Cloud at `https://yohandrychirinos1.atlassian.net`.
  - Authenticate by API using `JIRA_EMAIL` and `JIRA_API_TOKEN` from ignored env files.
  - Query project metadata, issue types, issues, and comments for `JIRA_PROJECT_KEY=KAN`.
  - Create Jira issues and comments by API when explicitly requested.
  - Native Jira webhook target is `https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10` once deployed.
  - Native Jira webhook authentication uses `X-Hub-Signature` HMAC with `JIRA_WEBHOOK_SECRET`; do not use `GITGOV_API_KEY` in the native Jira webhook URL.
  - Use Jira ticket IDs in branch names, commit messages, PR titles, and PR comments to generate GitGov traceability evidence.
- Local stack:
  - Use Docker Compose profiles `jenkins` and `sonar` to start Jenkins and SonarQube.
  - Validate local services before editing CI/CD configuration.

## GitHub Repository

- Repository: `yohandry10/Git-Gov`
- Default branch: `main`
- Branch protection is enabled for `main`.
- Required status checks currently include:
  - `Security Guard`
  - `Server Clippy + Check`
  - `Desktop Rust Clippy`
  - `Frontend Lint + Typecheck`
  - `Website Lint + Typecheck + Build`
  - `Validate quality_gates warn/block matrix`
- Admin enforcement is enabled.
- Required status checks are strict.

## GitHub Webhooks

- Primary GitHub webhook ID: `610772988`.
- Primary GitHub webhook URL: `https://gitgov-api.onrender.com/webhooks/github`.
- Render has `GITHUB_WEBHOOK_SECRET` configured for `gitgov-api`; do not print or commit the value.
- Configured events: `push`, `create`, `pull_request`, `pull_request_review`, `pull_request_review_comment`, `issue_comment`, `check_run`, `check_suite`, and `status`.
- GitHub webhook delivery has been validated with real repository events returning HTTP `200`.
- PR merge materialization is idempotent: duplicate `pull_request` deliveries for merged PRs should still repair `pull_request_merges` and ticket correlations.

## Render

- Primary backend service: `gitgov-api`
- Primary backend URL: `https://gitgov-api.onrender.com`
- Render service ID: `srv-d7lgtc77f7vs73b38uqg`
- Render service type: Docker web service.
- Render region: Oregon.
- Render deploy branch: `main`.
- Render root directory: `gitgov/gitgov-server`.
- Render service is reachable through the Render API using `RENDER_API_KEY` from ignored local env files.

## Jira

- Jira base URL: `https://yohandrychirinos1.atlassian.net`
- Jira project name: `GitGov`
- Jira project key: `KAN`
- Jira project ID: `10000`
- Available issue types observed by API: `Epic`, `Subtask`, `Tarea`, `Historia`.
- Jira credentials are stored only in ignored local env files:
  - `JIRA_BASE_URL`
  - `JIRA_EMAIL`
  - `JIRA_API_TOKEN`
  - `JIRA_PROJECT_KEY`
- GitGov native Jira webhook endpoint after deployment: `https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10`.
- GitGov admin/manual Jira ingest endpoint remains `https://gitgov-api.onrender.com/integrations/jira` and requires Bearer admin auth.
- Native Jira webhook setup should use Jira's webhook secret field with the same value as Render `JIRA_WEBHOOK_SECRET`.
- Native Jira webhook is configured in Jira Cloud:
  - Webhook name: `GitGov signed issue sync`
  - Webhook ID: `1`
  - Events: `jira:issue_created`, `jira:issue_updated`, `jira:issue_deleted`
  - JQL filter: `project = KAN`
  - Status: enabled and signed.
- Render has `JIRA_WEBHOOK_SECRET` configured for `gitgov-api`; do not print or commit the value.
- Traceability validation tickets created by API:
  - `KAN-4` - Validate GitGov traceability through PR titles
  - `KAN-5` - Validate GitGov traceability through PR comments
  - `KAN-6` - Validate GitGov release readiness evidence

## GitHub Actions Configuration

- Repository secret required by GitGov workflows: `GITGOV_API_KEY`.
- Repository variable required by GitGov workflows: `GITGOV_URL=https://gitgov-api.onrender.com`.
- SonarCloud is not used for this repository because the GitHub account is personal, not organizational.
- Local SonarQube is the selected Sonar runtime. Repository variable `SONAR_HOST_URL=http://localhost:9000`; GitHub-hosted Sonar scan must skip unless a self-hosted runner can reach that host.
- Sonar variable for local/runtime use: `SONAR_PROJECT_KEY=yohandry10_git-gov`.
- GitHub Actions `SONAR_TOKEN` is optional while SonarQube remains local; do not force it for GitHub-hosted runners because the scan is expected to skip when `SONAR_HOST_URL` is localhost.
- The quality gate policy matrix workflow is optional at workflow level but its job is required by branch protection.
- The matrix workflow must run on both `pull_request` and `push` to `main`; otherwise PR merges can be blocked by a required check that never appears.
- Release Readiness Gate is advisory by default on `push`; use manual `workflow_dispatch` with `enforce_gate=true` when a release must be blocked by readiness score.

## External Service Credentials

- Local SonarQube API access is configured through ignored env files. Current local values use `SONAR_HOST_URL=http://localhost:9000` and `SONAR_PROJECT_KEY=yohandry10_git-gov`.
- Current local SonarQube token name: `gitgov-local`.
- Current local SonarQube token expires on May 22, 2026.
- Jenkins read/build access is configured through ignored env files with `JENKINS_SERVER_URL=http://localhost:8096`, `JENKINS_USER=admin`, `JENKINS_API_TOKEN`, and `JENKINS_JOB_NAME=gitgov-demo-pipeline`.
- Current Jenkins API token name: `codex-local`.
- Jenkins trigger-only access can use `JENKINS_JOB_NAME` and `JENKINS_BUILD_TRIGGER_TOKEN`, but that is not enough to inspect logs or build status.
- If Jenkins posts to GitGov, keep `JENKINS_WEBHOOK_SECRET` aligned with the Jenkins shared secret header expected by the backend.
- Jira Cloud API access is configured through ignored env files with `JIRA_BASE_URL`, `JIRA_EMAIL`, `JIRA_API_TOKEN`, and `JIRA_PROJECT_KEY`.
- GitHub webhook authentication is configured through ignored env files with `GITHUB_WEBHOOK_SECRET`; keep it aligned with Render and the GitHub repository webhook.
- Native Jira webhooks require `JIRA_WEBHOOK_SECRET` on Render and the same webhook secret in Jira Cloud.
- Current native Jira webhook name is `GitGov signed issue sync`; it is signed with `JIRA_WEBHOOK_SECRET` and targets `https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10`.

## Verified State

- Render backend health endpoint passed on `https://gitgov-api.onrender.com/health`.
- GitGov Render backend has policy and Sonar-style pipeline evidence for `yohandry10/Git-Gov`; last observed correlation sample contained 12 Sonar/Jenkins evidence items for `main`.
- GitHub-hosted matrix validation passed on run `24877293195`.
- Job `Validate quality_gates warn/block matrix` passed on job `72836755674`.
- Local SonarQube API token validation passed with `SONAR_TOKEN`; project `yohandry10_git-gov` quality gate was `OK`.
- Local Jenkins API validation passed through `/whoAmI/api/json`; authenticated user is `admin`.
- Local Jenkins job API validation passed for `gitgov-demo-pipeline`; last observed build was `#30`, result `SUCCESS`, not building.
- Jira Cloud API validation passed through `/rest/api/3/myself` and `/rest/api/3/project/KAN`.
- Jira project `KAN` is reachable and accepts issue type `Tarea`.
- Jira issues `KAN-4`, `KAN-5`, and `KAN-6` were created for GitGov traceability validation.
- Jira issues `KAN-4`, `KAN-5`, and `KAN-6` were ingested into GitGov through `POST /integrations/jira`.
- Native signed Jira webhook delivery was validated end-to-end by updating `KAN-6` in Jira Cloud and observing `last_ingest_at` advance in GitGov.
- GitGov Jira correlation was validated with `KAN-6`; the main commit from `docs(KAN-6): document Jira API access (#21)` produced one commit-ticket correlation.
- Jira ticket coverage for `yohandry10/Git-Gov` over the 720h validation window was last observed at `1/25` commits with tickets (`4.0%`) after additional GitHub-hosted merge commits were ingested.
- Repo/branch-scoped readiness validation for `yohandry10/Git-Gov` on `main` produced standard readiness `69/100` against target `75`, composite risk `29/100`, signal coverage `3/3`; current blocker is Jira traceability coverage, not Sonar or Jenkins evidence.
- GitHub repository webhook ID `610772988` is active and delivered real `pull_request`, `push`, `issue_comment`, `check_run`, `check_suite`, and `status` events to Render with HTTP `200`.
- GitHub webhook ingestion includes `pull_request_review_comment` and PR-linked `issue_comment`; these events are stored as first-class evidence and can create commit-ticket correlations from ticket IDs in comment/title text.
- GitHub merged PR title ingestion creates commit-ticket correlations for the merge commit SHA when the PR title contains a ticket ID, so future `main` merge commits can count toward Jira ticket coverage.
- `POST /integrations/jira/correlate` also scans recent merged PR titles as a backfill path for historical ticket coverage.

## Safety Rules

- Never commit `.env`, `.env.local`, `.env.*.local`, `.mcp.json`, or files under `secrets/`.
- Never print API keys, Render tokens, GitHub tokens, Jenkins tokens, or Sonar tokens.
- Never paste provider tokens into GitHub Actions variables; use GitHub Actions secrets for sensitive values.
- Do not revert unrelated dirty files in the user's main worktree.
- Prefer `gh` for GitHub operations instead of browser steps.
- Prefer Render API for Render checks when `RENDER_API_KEY` is present.
- Prefer Jenkins API for Jenkins checks when `JENKINS_API_TOKEN` is present.
- Prefer SonarQube API for Sonar checks when `SONAR_TOKEN` is present.
- Prefer Jira API for Jira checks when `JIRA_API_TOKEN` is present.
