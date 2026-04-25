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
- `Security Guard` also enforces traceability hygiene:
  - Branch names must include a Jira-style ticket ID such as `KAN-4` except protected/base branches like `main`.
  - Pull request titles must include a Jira-style ticket ID such as `KAN-4`.
  - New commit messages in PRs/pushes must include a Jira-style ticket ID.
  - Local equivalent: `scripts/github/check_traceability_policy.ps1`.
  - Local preflight equivalent: `scripts/security/publication_guard.ps1`.
  - Local commit hook: `.githooks/commit-msg` when `core.hooksPath=.githooks`.

## GitHub Webhooks

- Primary GitHub webhook ID: `610772988`.
- Primary GitHub webhook URL: `https://gitgov-api.onrender.com/webhooks/github`.
- Render has `GITHUB_WEBHOOK_SECRET` configured for `gitgov-api`; do not print or commit the value.
- Configured events: `push`, `create`, `pull_request`, `pull_request_review`, `pull_request_review_comment`, `issue_comment`, `check_run`, `check_suite`, and `status`.
- GitHub webhook delivery has been validated with real repository events returning HTTP `200`.
- PR merge materialization is idempotent: duplicate `pull_request` deliveries for merged PRs should still repair `pull_request_merges` and ticket correlations.
- Production PR-title validation completed with Jira ticket `KAN-4`:
  - GitHub merged PR titles containing `KAN-4` were ingested through real webhook deliveries.
  - Webhook redelivery for a merged PR returned `processed=true`.
  - `pull_request_merges` reached at least `2` records in production validation.
  - Jira backfill scanned `2` merged PRs and created `2` PR-title correlations.
  - `commit_ticket_correlations.source` must remain `pr_title`; production DB constraints allow `branch_name`, `commit_message`, `pr_title`, and `manual`.
- Ticket coverage semantics include PR merge evidence:
  - `GET /integrations/jira/ticket-coverage` builds its commit universe from both `client_events(event_type='commit')` and `pull_request_merges`.
  - For PR merge evidence, coverage uses `merge_commit_sha` from payload first and falls back to `head_sha`.
  - This lets PR-title correlations count toward Jira coverage even when the merge commit was only observed through GitHub webhook evidence.

## Render

- Primary backend service: `gitgov-api`
- Primary backend URL: `https://gitgov-api.onrender.com`
- Render service ID: `srv-d7lgtc77f7vs73b38uqg`
- Render service type: Docker web service.
- Render region: Oregon.
- Render deploy branch: `main`.
- Render root directory: `gitgov/gitgov-server`.
- Render service is reachable through the Render API using `RENDER_API_KEY` from ignored local env files.
- `docs/DEPLOYMENT.md` treats Render as the current production route; the former EC2/Nginx/systemd material is retained as legacy/self-hosted guidance only.

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
- GitGov admin/manual Jira ingest endpoint remains `https://gitgov-api.onrender.com/integrations/jira` and requires Bearer admin auth plus `x-gitgov-jira-secret` when `JIRA_WEBHOOK_SECRET` is configured. Global admin keys also require an `org_name` payload hint such as `yohandry10`.
- Native Jira webhook setup should use Jira's webhook secret field with the same value as Render `JIRA_WEBHOOK_SECRET`.
- Native Jira webhook is configured in Jira Cloud:
  - Webhook name: `GitGov signed issue sync`
  - Webhook ID: `1`
  - Events: `jira:issue_created`, `jira:issue_updated`, `jira:issue_deleted`
  - JQL filter: `project = KAN`
  - Status: enabled and signed.
- Render has `JIRA_WEBHOOK_SECRET` configured for `gitgov-api`; do not print or commit the value.
- Domain SLO targets in `ops/slo/domain-slo-targets.json` must include `org_name=yohandry10`; leaving it blank causes SLO validation to read unscoped telemetry and overstate traceability gap.
- Run `scripts/control-plane/validate_domain_slo_target_config.ps1 -RequireOrgName -RequireRepoFullName -RequireBranch` after editing SLO targets; CI enforces the same static guardrail before live validation.
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
- Release Readiness Gate also runs daily by schedule at `10:17 UTC`; scheduled runs refresh Jira/PR correlations before scoring and enforce the standard readiness target.
- Push/manual Release Readiness Gate runs remain advisory unless `enforce_gate=true`; a failed pre-score Jira correlation refresh is only blocking when the gate is enforced.
- Manual Release Readiness Gate runs default to a 720h lookback window and can disable the pre-score Jira correlation refresh with `refresh_jira_correlations=false` if an operator needs a pure read-only check.
- First-party GitHub Actions were upgraded for Node 24 action-runtime compatibility:
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
  - `actions/upload-artifact@v7`
  - `pnpm/action-setup@v5`
  - `node-version: 20` remains the project build runtime where configured; it is not the internal runtime of `actions/*`.

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
- API contract drift reconciliation ticket `KAN-8` records that `docs/ARCHITECTURE.md` is aligned with the real backend routes for `/jobs/{job_id}/retry`, `/compliance/{org_name}`, and `/violations/{violation_id}/decisions`; the remaining contract debt is optional OpenAPI path completeness, not route-table drift. `docs/ENTERPRISE_READINESS_DECISION.md` is ignored internal audit memory and must not be force-added.
- `.env.example` placeholder policy ticket `KAN-9` hardens publication safety: real `.env` files remain blocked, `.env.example` remains trackable, and both local `publication_guard.ps1` plus GitHub `Security Guard` validate that sensitive keys in `.env.example` contain placeholder-only values.
- Implementation summary ticket `KAN-10` consolidates the latest closed points and remaining backlog in `docs/IMPLEMENTATION_STATUS.md` and `docs/reports/implementation-progress-summary-2026-04-25.md`.
- API key diagnosis ticket `KAN-11` corrected the manual ingest finding: local ignored `GITGOV_API_KEY` is present and validates against production `/stats` with HTTP `200`. Manual `/integrations/jira` calls also require `x-gitgov-jira-secret` and `org_name` when production `JIRA_WEBHOOK_SECRET` is configured.
- Current remaining blockers/gaps after `KAN-7`/`KAN-8`/`KAN-9`/`KAN-11`: Sonar remains local unless a self-hosted runner is added; Jenkins trigger-only token is only needed for unauthenticated build URLs; OpenAPI path completeness is optional unless generated SDK/contract testing is required; traceability coverage stays an operational discipline.

## Verified State

- Render backend health endpoint passed on `https://gitgov-api.onrender.com/health`.
- Local ignored `GITGOV_API_KEY` was validated against production `https://gitgov-api.onrender.com/stats` with HTTP `200`; Render does not need a `GITGOV_API_KEY` env var for current DB-backed admin auth, though setting it there can be used as bootstrap consistency.
- Manual Jira ingest to production was validated with Bearer `GITGOV_API_KEY`, `x-gitgov-jira-secret`, and `org_name=yohandry10`; the previous `401` diagnosis was a missing Jira secret header, not a bad GitGov API key.
- Deployment documentation drift was cleaned so Render is documented as current production, GitHub/Jira webhooks are documented as already configured, and domain/`certbot` work is marked as optional for self-hosted/custom-domain migrations.
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
- GitHub PR merge delivery validation with `KAN-4` titles was completed in production:
  - PR merge evidence was materialized in `pull_request_merges`.
  - PR-title correlations were created for merge/head SHAs using `source=pr_title`.
  - Latest validated GitHub redelivery returned HTTP `200` and `processed=true`.
- GitHub webhook ingestion includes `pull_request_review_comment` and PR-linked `issue_comment`; these events are stored as first-class evidence and can create commit-ticket correlations from ticket IDs in comment/title text.
- GitHub webhook extraction contract tests cover `check_run`, `check_suite`, `status`, and `pull_request_review_comment`; run `cargo test github_webhook_tests` from `gitgov/gitgov-server` after changing webhook evidence parsing.
- Post-merge validation for GitHub webhook extraction contract tests passed on `main` commit `946fac3`: CI run `24927816238`, Quality Gate Policy Matrix run `24927816230`, and Release Readiness Gate run `24927816225`.
- Admin dashboard GitHub reporting includes an executive evidence coverage summary in `EventBreakdownGrid`: PR lifecycle, reviews, PR comments, and checks/status are collapsed to `n/4` coverage with missing signal labels.
- Admin dashboard GitHub reporting includes a local trend widget `GitHubEvidenceTrendWidget`; it stores operator-captured evidence snapshots in browser `localStorage` under `gitgov.dashboard.github_evidence_trend` and does not query GitHub Actions or provider tokens.
- Post-merge validation for the executive GitHub evidence dashboard summary passed on `main` commit `01d275c`: CI run `24938441269`, Quality Gate Policy Matrix run `24938441278`, and Release Readiness Gate run `24938441273`.
- Post-merge validation for the dashboard GitHub evidence trend widget passed on `main` commit `74a51a5`: CI run `24940280762`, Quality Gate Policy Matrix run `24940280775`, and Release Readiness Gate run `24940280751`.
- Admin dashboard audit exports package the same GitHub executive evidence summary into downloaded JSON under `executive_summary.github_evidence`; raw export records remain under `data`.
- Post-merge validation for GitHub evidence export packaging passed on `main` commit `458c048`: CI run `24938795096`, Quality Gate Policy Matrix run `24938795085`, and Release Readiness Gate run `24938795100`.
- `scripts/control-plane/generate_github_evidence_report.ps1` generates a standalone Markdown executive report from `/stats.github_events.by_type` or an offline stats JSON fixture. Use `-StatsJsonPath` for token-free validation and `-GitGovUrl`/`-ApiKey` for live Control Plane reporting.
- `.github/workflows/github-evidence-report.yml` runs the GitHub evidence executive report generator manually or weekly on Monday 13:23 UTC, uploads the Markdown artifact, and skips cleanly when `GITGOV_URL` or `GITGOV_API_KEY` is missing.
- Manual GitHub evidence report workflow validation passed on run `24939329055` for `main` commit `3935c21`; artifact `github-evidence-executive-report` was uploaded successfully.
- `scripts/control-plane/validate_github_evidence_report_artifact.ps1` validates operational freshness of the GitHub evidence report artifact by querying GitHub Actions for the latest successful `github-evidence-report.yml` run, confirming artifact `github-evidence-executive-report` exists, is not expired, and is within the configured max age.
- `.github/workflows/github-evidence-artifact-monitor.yml` runs the artifact freshness monitor manually or weekly on Tuesday 14:07 UTC and uploads `github-evidence-artifact-monitor` JSON evidence.
- Local validation of the artifact monitor passed against workflow run `24939329055`; artifact `6642253304` was fresh and not expired.
- First GitHub-hosted validation of the artifact monitor passed on workflow run `24939815276`; artifact `github-evidence-artifact-monitor` ID `6642391452` uploaded successfully and was not expired.
- `scripts/control-plane/generate_github_evidence_trend_report.ps1` generates Markdown/JSON trend history by downloading recent `github-evidence-executive-report` artifacts from successful `github-evidence-report.yml` runs and parsing status, coverage, and missing signal fields.
- `.github/workflows/github-evidence-trend-report.yml` runs the trend report manually or weekly on Tuesday 14:17 UTC and uploads artifact `github-evidence-trend-report`.
- Operational use of the GitHub evidence dashboard, Markdown report, artifact freshness monitor, and trend report is documented in `docs/runbooks/github-evidence-operations.md`.
- Post-merge validation for the GitHub evidence operations runbook passed on `main` commit `7577f90`: CI run `24940874607`, Quality Gate Policy Matrix run `24940874602`, Release Readiness Gate run `24940874616`, Secret Scan run `24940874599`, SonarQube Governance run `24940874600`, Public Naming Guard run `24940874603`, Governance Correlation Smoke run `24940874611`, and Desktop Updater Readiness run `24940874597`.
- GitHub evidence operational adoption baseline completed on 2026-04-25:
  - Executive report workflow run `24941348198` succeeded on `main` commit `65613b0`; artifact `github-evidence-executive-report` ID `6642829154`.
  - Artifact monitor workflow run `24941358185` succeeded; artifact `github-evidence-artifact-monitor` ID `6642831722`.
  - Trend workflow run `24941363195` succeeded; artifact `github-evidence-trend-report` ID `6642833188`.
  - Local artifact freshness monitor returned `PASS`; latest artifact was not expired and age was `0.02h`.
  - Local trend parsed `2` reports and still observed `Sin evidencia` / `0/4 signals`; Jira follow-up `KAN-7` tracks the data-quality/scope investigation.
- `KAN-7` GitHub evidence stats root cause:
  - `github-evidence-report.yml` reads `GET /stats.github_events.by_type`.
  - Existing `supabase_schema_v18.sql` and `supabase_schema_v19.sql` optimized `get_audit_stats` but returned zeroed GitHub stats.
  - Migration `supabase_schema_v22.sql` restores real `github_events` totals, daily counts, `by_type`, and `active_repos` while preserving v19 violation decision semantics.
  - Postcheck file: `gitgov/gitgov-server/supabase/checks/v22_postcheck.sql`.
  - Production DB migration v22 was applied with `psql`; `v22_postcheck.sql` passed.
  - Post-migration `/stats.github_events.by_type` showed real evidence counts, including `pull_request=75`, `issue_comment=93`, `check_run=1937`, `check_suite=599`, and `status=148`.
  - Local live executive report generated `Parcial` / `3/4 signals`; remaining missing signal is `Reviews` because the current sample lacks `pull_request_review` events.
  - GitHub-hosted validation passed: report run `24942000355` artifact `6643010178`, artifact monitor run `24942008460` artifact `6643012934`, trend run `24942016196` artifact `6643015713`.
  - To close the remaining `Reviews` signal, create/use a Jira-traceable PR and have a reviewer submit a GitHub PR review event; review comments alone count under `PR comments`, not `Reviews`.
  - PR `#71` created a real `pull_request_review` event; `/stats.github_events.by_type.pull_request_review` reached `1`.
  - Local live report after PR `#71` review validation generated `Completo` / `4/4 signals` with evidence file `docs/reports/github-evidence-executive-report-prod-review-v22-2026-04-25.md`.
  - GitHub-hosted post-review validation passed after PR `#71` merged on `main` commit `0a7a230`: report run `24942351831` generated `Completo` / `4/4 signals`, monitor run `24942357291` returned `PASS`, and trend run `24942362269` reported latest coverage `4/4 signals`.
  - Latest GitHub evidence report artifact ID after review validation: `6643110541`; it is the current cloud evidence reference for complete PR lifecycle, reviews, PR comments, and checks/status coverage.
- Admin dashboard Risk Outcomes now includes informational `Time-to-Evidence` and `MTTR pipeline` metrics from Jenkins commit-pipeline correlations:
  - `Time-to-Evidence` is commit timestamp to correlated pipeline ingestion timestamp.
  - `MTTR pipeline` is recoverable non-green Jenkins pipeline event to next successful run for the same job.
  - Duplicate pipeline evidence is ignored before calculating samples.
  - These metrics render `N/A` with insufficient evidence and are not part of composite risk/readiness scoring until tier-specific SLOs are calibrated.
  - Local validation passed with `npm test -- --run src/test/components/dashboard-helpers.test.ts`, full `npm test -- --run`, `npm run typecheck`, `npm run lint`, `git diff --check`, and `.\scripts\security\publication_guard.ps1`.
  - Post-merge GitHub-hosted validation passed on `main` commit `adb5399`: CI `24941724773`, Quality Gate Policy Matrix `24941724754`, Release Readiness Gate `24941724756`, Secret Scan `24941724779`, SonarQube Governance `24941724778`, Public Naming Guard `24941724766`, Governance Correlation Smoke `24941724751`, and Desktop Updater Readiness `24941724750`.
- Local live validation of the trend generator parsed workflow run `24939329055` and produced a 1-report trend with latest coverage `0/4 signals`; this reflects the existing `/stats.github_events.by_type` visibility note, not a secret/config leak.
- First GitHub-hosted validation of the trend workflow passed on run `24940027811` for `main` commit `a58ae81`; artifact `github-evidence-trend-report` ID `6642453325` uploaded successfully and was not expired.
- Post-merge validation for the trend workflow rollout passed on `main` commit `a58ae81`: CI run `24940024455`, Quality Gate Policy Matrix run `24940024458`, and Release Readiness Gate run `24940024457`.
- GitHub merged PR title ingestion creates commit-ticket correlations for the merge commit SHA when the PR title contains a ticket ID, so future `main` merge commits can count toward Jira ticket coverage.
- `POST /integrations/jira/correlate` also scans recent merged PR titles as a backfill path for historical ticket coverage.
- Last production PR-title backfill validation for `KAN-4` observed:
  - `scanned_prs=2`
  - `correlations_created=2`
  - four `KAN-4` correlation rows across validated merge/head SHAs
- Coverage/readiness query semantics were updated after the `33.33%` observation: ticket coverage now includes materialized PR merge commits in addition to desktop/client commit events. After deploying this change, re-run `/integrations/jira/correlate` and `/integrations/jira/ticket-coverage` for `yohandry10/Git-Gov` on `main` to confirm production readiness movement.
- Production validation after deploy `0494648` completed:
  - Render deploy for `fix(KAN-4): count PR merges in ticket coverage (#35)` reached `live`.
  - `/health` returned `ok`.
  - Jira correlation backfill scanned `4` PRs and created `0` new rows because existing correlations were already present.
  - `/integrations/jira/ticket-coverage?repo_full_name=yohandry10%2FGit-Gov&branch=main&hours=720` returned `total_commits=30`, `commits_with_ticket=5`, `coverage_percentage=16.67`.
  - `validate_release_readiness_gate.ps1` passed for `yohandry10/Git-Gov` on `main`: readiness `77/100` vs target `75`, signal coverage `3/3`, pipeline success `96.77%`, Sonar pass `96.77%`, Jira coverage `16.67%`.
- Production validation after traceability guard rollout completed:
  - Jira correlation backfill scanned `8` PRs and created `0` new rows because relevant rows already existed.
  - Ticket coverage for `yohandry10/Git-Gov`, branch `main`, 720h returned `total_commits=34`, `commits_with_ticket=9`, `coverage_percentage=26.47`.
  - Release readiness passed with readiness `79/100` vs target `75`, signal coverage `3/3`, pipeline success `97.14%`, Sonar pass `97.14%`, Jira coverage `26.47%`.
  - This confirms the branch/PR/commit Jira-ID guardrail is improving coverage through the PR-title merge evidence path.
- Scheduled release readiness monitoring is configured in `.github/workflows/release-readiness-gate.yml`:
  - Runs daily at `10:17 UTC`.
  - Refreshes Jira correlations through `POST /integrations/jira/correlate` before scoring.
  - Uploads both `release-readiness-gate-<run_id>.json` and `jira-correlation-refresh-<run_id>.json`.
  - Fails scheduled runs when readiness is below the configured standard target.
- First GitHub-hosted validation after scheduling passed on run `24927045053`:
  - Event: `push` on `main` for commit `a94114c`.
  - Jira correlation refresh artifact was generated.
  - Readiness was `81/100` against target `75`.
  - Signal coverage was `3/3`.
- GitHub Actions Node 24 compatibility upgrade is documented at `docs/reports/github-actions-node24-upgrade-2026-04-25.md`.
- First GitHub-hosted validation after the full Node 24 action-runtime upgrade passed:
  - `main` commit `3f4c601`.
  - CI run `24927274092` passed with `actions/checkout@v6`, `actions/setup-node@v6`, and `pnpm/action-setup@v5`.
  - The previous Node.js 20 action-runtime annotation was not present in the CI run output.
  - Release Readiness Gate run `24927274091` passed with readiness `82/100`, target `75`, and signal coverage `3/3`.
- Production tier/SLO calibration completed on 2026-04-25 after refreshing Jira PR correlations:
  - Jira backfill scanned `14` merged PRs and created `0` new correlations.
  - Tier baseline reports were generated under `docs/reports/risk-tier-baseline-prod-2026-04-25/`.
  - Critical: readiness `96/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Standard: readiness `95/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Internal: readiness `96/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Domain SLO validation reports were generated under `docs/reports/domain-slo-validation-prod-2026-04-25/`.
  - Domains `core-platform`, `standard-services`, and `internal-tools` all passed after scoping targets to `org_name=yohandry10`.
- Domain SLO target config validation is enforced by `scripts/control-plane/validate_domain_slo_target_config.ps1` in CI and `.github/workflows/domain-slo-validation.yml`, requiring explicit `org_name`, `repo_full_name`, and `branch` scope.
- Post-merge validation for SLO target config guardrail passed on `main` commit `f0a3470`: CI run `24927603357`, Quality Gate Policy Matrix run `24927603365`, and Release Readiness Gate run `24927603352`.

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
- After any major access/configuration/deployment/validation change, update `AGENTS.md` and the relevant document under `docs/` before finalizing the PR. This repository relies on docs as persistent agent memory.
- Future branches, commits, and PR titles should include Jira ticket IDs to preserve GitGov ticket coverage and release readiness.
