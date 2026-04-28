# GitGov Current Context Handoff

Updated: 2026-04-28
Ticket: `KAN-22`

Read this file first when resuming work. It is the compact operational handoff for the current GitGov state.

## Exact Current Point

- Local workspace: `C:\Users\PC\Desktop\GitGov`.
- Expected branch before new work: `main`.
- Latest completed handoff baseline: `c1951c8 docs(KAN-22): refresh current context evidence`.
- Last merged PR: `#89` - `docs(KAN-22): refresh current context evidence`.
- Previous merged PR: `#88` - `docs(KAN-22): add current context handoff`.
- Treat commit/PR fields in this file as a validated handoff baseline, not an auto-updating source of truth; always run `git status --short --branch` and `git log -1 --oneline main` before new work.
- Worktree expectation before new work: clean and aligned with `origin/main`.
- Implementation-status backlog is closed. Remaining items are operational decisions, optional future enhancements, or evidence hygiene.
- Any future branch, commit, and PR title must include a Jira ticket ID such as `KAN-22`.

## Latest Verified GitHub Checks

Latest post-merge validation for handoff baseline commit `c1951c8` passed:

- `CI` - run `25048800803`
- `Release Readiness Gate` - run `25048800807`
- `Quality Gate Policy Matrix (Optional)` - run `25048800838`
- `Secret Scan` - run `25048800831`
- `SonarQube Governance (Non-Blocking)` - run `25048800812`
- `Public Naming Guard` - run `25048800822`
- `Governance Correlation Smoke (Optional)` - run `25048800832`
- `Desktop Updater Readiness (Optional)` - run `25048800795`

## Non-Negotiable Operating Decisions

### Sonar

- SonarCloud is not a valid path for this repository because the current GitHub repository/account is personal, not organizational.
- Do not ask again to use SonarCloud for this repo.
- Do not propose SonarCloud onboarding unless the repository is moved to a GitHub organization.
- Local SonarQube is the selected Sonar runtime.
- Local SonarQube URL: `http://localhost:9000`.
- Sonar project key: `yohandry10_git-gov`.
- GitHub-hosted Sonar scans should skip while `SONAR_HOST_URL=http://localhost:9000`, because hosted runners cannot reach the workstation.
- If GitHub Actions must run a real local Sonar scan, first add and validate a dedicated self-hosted runner using `docs/runbooks/local-sonar-self-hosted-runner.md`.

### Jenkins

- Jenkins authenticated API access is already configured and is the normal agent path.
- Jenkins URL: `http://localhost:8096`.
- Current Jenkins job: `gitgov-demo-pipeline`.
- Jenkins authenticated API access supports inspection, logs, queue state, build history, and authenticated build operations.
- `JENKINS_BUILD_TRIGGER_TOKEN` is only for unauthenticated/manual URL build starts:

```text
{JENKINS_SERVER_URL}/job/{JENKINS_JOB_NAME}/build?token={JENKINS_BUILD_TRIGGER_TOKEN}
```

- Do not ask for the trigger-only token unless the user explicitly wants that unauthenticated/manual URL flow.

### OpenAPI and SDKs

- OpenAPI is the machine-readable API description used by Swagger tools and generated SDKs.
- OpenAPI is not the API itself.
- Normal GitGov API work uses the real backend routes/API.
- `/api-docs` is intentionally a partial schema explorer.
- `docs/ARCHITECTURE.md` plus the backend `main.rs` route table are the operational route source of truth.
- Full OpenAPI annotation is optional product work. Implement it only if generated SDKs or Swagger contract tests become a real requirement.

### Documentation Memory

- After any major access/configuration/deployment/validation change, update `AGENTS.md` and the relevant `docs/` file before finalizing a PR.
- Keep this handoff file current when the project state changes materially.
- Never print or commit token values.

## Access and Tooling

### GitHub

- Repository: `yohandry10/Git-Gov`.
- Default branch: `main`.
- GitHub CLI path: `C:\Users\PC\Tools\gh\bin\gh.exe`.
- `gh` is authenticated as `yohandry10`.
- Branch protection is enabled on `main`.
- Required checks are strict and admin-enforced.
- Traceability policy is active:
  - Branch names must include Jira IDs, except protected/base branches.
  - PR titles must include Jira IDs.
  - Commit messages must include Jira IDs.
  - Local guard: `.\scripts\security\publication_guard.ps1`.

### Render

- Production backend service: `gitgov-api`.
- Production URL: `https://gitgov-api.onrender.com`.
- Service ID: `srv-d7lgtc77f7vs73b38uqg`.
- Render service type: Docker web service.
- Render branch: `main`.
- Render root directory: `gitgov/gitgov-server`.
- Render API access is available through ignored local env files as `RENDER_API_KEY`.

### Jira

- Base URL: `https://yohandrychirinos1.atlassian.net`.
- Project key: `KAN`.
- Project name: `GitGov`.
- Current native Jira webhook target:

```text
https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10
```

- Native Jira webhook name: `GitGov signed issue sync`.
- Native Jira webhook is signed with `JIRA_WEBHOOK_SECRET`.
- Use Jira ticket IDs in branches, commits, PR titles, and PR comments.

### Local Env Files

Tokens and secrets are in ignored local env files only:

- `C:\Users\PC\Desktop\GitGov\gitgov\.env`
- `C:\Users\PC\Desktop\GitGov\gitgov\gitgov-server\.env`

Never print values from these files. Treat them as source of truth for local access.

Expected local keys include:

- `GITGOV_API_KEY`
- `GITGOV_URL`
- `RENDER_API_KEY`
- `SONAR_HOST_URL`
- `SONAR_TOKEN`
- `SONAR_PROJECT_KEY`
- `JENKINS_SERVER_URL`
- `JENKINS_USER`
- `JENKINS_API_TOKEN`
- `JENKINS_JOB_NAME`
- `JIRA_BASE_URL`
- `JIRA_EMAIL`
- `JIRA_API_TOKEN`
- `JIRA_PROJECT_KEY`
- `JIRA_WEBHOOK_SECRET`
- `GITHUB_WEBHOOK_SECRET`

## Current Validation Commands

Run these from `C:\Users\PC\Desktop\GitGov`.

Publication and traceability guard:

```powershell
.\scripts\security\publication_guard.ps1
```

Provider access smoke test:

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

Jira traceability coverage:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50
```

Jenkins trigger-only dry run:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1
```

Use `-Trigger` only when a real unauthenticated/manual URL build launch is intended.

## Recent Ticket Chain

- `KAN-14`: refreshed local/production operational validation after Docker Desktop and Sonar/Jenkins profiles were up.
- `KAN-15`: added guard that `/api-docs` remains a partial schema explorer.
- `KAN-16`: added provider access validator; latest refresh on 2026-04-28 returned all checks `ok`, readiness `92/100`, pipeline success `98.81%`, Jira coverage `69.88%`, and Sonar pass `98.81%`.
- `KAN-17`: documented local Sonar self-hosted runner path without enabling it.
- `KAN-18`: documented Jenkins trigger-only token flow as optional and dry-run-first.
- `KAN-19`: added Jira traceability coverage validator; latest recorded coverage was `96.67%` (`58/60`) over 720h.
- `KAN-20`: closed implementation backlog semantics; remaining items are operational decisions.
- `KAN-21`: clarified SonarCloud, OpenAPI/SDK, and Jenkins trigger-only defaults.
- `KAN-22`: created this current-context handoff, refreshed it through PR `#89` with baseline commit `c1951c8`, and fixed PowerShell workflow splatting in risk-tier baseline and desktop updater readiness workflows after scheduled/optional job failures.

## Latest Workflow Fix Context

- `Risk Tier Baseline Calibration` scheduled run `24999681550` failed on 2026-04-27 because `.github/workflows/risk-tier-baseline-calibration.yml` used array splatting with `"-Param", value` pairs; PowerShell passed those positionally, so `-RepoFullName` reached the `Tier` parameter.
- `.github/workflows/desktop-updater-readiness.yml` used the same pattern and failed inside its optional job when `gitgov/src-tauri/tauri.conf.json` was bound to `TimeoutSeconds`.
- Use hashtable splatting for workflow PowerShell script blocks that call repository scripts with named parameters.
- Local validation for the fix generated a risk-tier baseline report with readiness `92/100`, composite risk `8/100`, and ran desktop updater readiness with endpoint probe skipped, returning the expected optional `WARN` state.
- Manual Risk Tier Baseline runs `25049577630` and `25049782826` on `main` confirmed the calibration step generated a report, then failed artifact upload because `report_path` was not visible to `actions/upload-artifact`; the workflow now uploads the deterministic report path directly.
- Final manual Risk Tier Baseline validation run `25049984199` passed on `main` commit `8e9b043` and uploaded artifact `risk-tier-baseline-25049984199` ID `6682824924`.

## Current Work Classification

No active implementation blocker remains in the documented status list.

Current work types are:

- Operational validation cadence.
- Evidence freshness.
- Optional product enhancements.
- Future implementation only when explicitly requested.

## Practical Next Steps

When resuming, do this first:

1. Run `git status --short --branch`.
2. Read `AGENTS.md` and this file.
3. If work changes code or docs, create/use a Jira ticket first.
4. Use a Jira-traceable branch such as `docs/KAN-22-current-context-handoff`.
5. Run `.\scripts\security\publication_guard.ps1` before commit.
6. Push, open PR, wait for required checks, merge only when green.
7. After merge, pull `main`, wait for post-merge checks, and comment the Jira ticket with evidence.

## Do Not Reopen Without New Product Decision

- SonarCloud for this personal repo.
- Jenkins trigger-only token for normal agent work.
- Full OpenAPI annotation as a blocker.
- Old EC2/Nginx/systemd deployment path; Render is current production.
- Non-traceable commits or PRs.
