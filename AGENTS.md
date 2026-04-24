# Agent Operating Context

This repository is operated from `C:\Users\PC\Desktop\GitGov` on Windows PowerShell.

## Access

- GitHub CLI is installed at `C:\Users\PC\Tools\gh\bin\gh.exe`.
- `gh` is authenticated as `yohandry10` with admin access to `yohandry10/Git-Gov`.
- GitHub token scopes observed: `repo`, `workflow`, `read:org`, `gist`.
- Render API access is available via local ignored env files only. Do not commit or print token values.
- Local Render env key name: `RENDER_API_KEY`.
- Local GitGov API env key name: `GITGOV_API_KEY`.
- SonarCloud direct API access is not available unless `SONAR_TOKEN` is present locally.
- Jenkins direct API access is not available unless `JENKINS_SERVER_URL`, `JENKINS_USER`, and `JENKINS_API_TOKEN` are present locally.

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

## Render

- Primary backend service: `gitgov-api`
- Primary backend URL: `https://gitgov-api.onrender.com`
- Render service type: Docker web service.
- Render region: Oregon.
- Render service is reachable through the Render API using `RENDER_API_KEY` from ignored local env files.

## GitHub Actions Configuration

- Repository secret required by GitGov workflows: `GITGOV_API_KEY`.
- Repository variable required by GitGov workflows: `GITGOV_URL=https://gitgov-api.onrender.com`.
- Repository secret used by Sonar workflows when present: `SONAR_TOKEN`.
- Sonar variables:
  - `SONAR_HOST_URL=https://sonarcloud.io`
  - `SONAR_PROJECT_KEY=yohandry10_git-gov`
- The quality gate policy matrix workflow is optional at workflow level but its job is required by branch protection.
- The matrix workflow must run on both `pull_request` and `push` to `main`; otherwise PR merges can be blocked by a required check that never appears.
- Release Readiness Gate is advisory by default on `push`; use manual `workflow_dispatch` with `enforce_gate=true` when a release must be blocked by readiness score.

## External Service Credentials

- SonarCloud API access requires `SONAR_TOKEN` in local ignored env files. Keep `SONAR_HOST_URL=https://sonarcloud.io` and `SONAR_PROJECT_KEY=yohandry10_git-gov`.
- Jenkins read/build access requires `JENKINS_SERVER_URL`, `JENKINS_USER`, and `JENKINS_API_TOKEN`.
- Jenkins trigger-only access can use `JENKINS_JOB_NAME` and `JENKINS_BUILD_TRIGGER_TOKEN`, but that is not enough to inspect logs or build status.
- If Jenkins posts to GitGov, keep `JENKINS_WEBHOOK_SECRET` aligned with the Jenkins shared secret header expected by the backend.

## Verified State

- Render backend health endpoint passed on `https://gitgov-api.onrender.com/health`.
- GitGov Render backend has policy and Sonar-style pipeline evidence for `yohandry10/Git-Gov`.
- GitHub-hosted matrix validation passed on run `24877293195`.
- Job `Validate quality_gates warn/block matrix` passed on job `72836755674`.

## Safety Rules

- Never commit `.env`, `.env.local`, `.env.*.local`, `.mcp.json`, or files under `secrets/`.
- Never print API keys, Render tokens, GitHub tokens, Jenkins tokens, or Sonar tokens.
- Do not revert unrelated dirty files in the user's main worktree.
- Prefer `gh` for GitHub operations instead of browser steps.
- Prefer Render API for Render checks when `RENDER_API_KEY` is present.
