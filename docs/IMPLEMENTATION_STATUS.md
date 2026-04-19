# GitGov Implementation Status

Updated: 2026-04-19

## Completed

- Repository migration completed to `yohandry10/Git-Gov`.
- Hardcoded legacy references removed from CI/Jenkins paths.
- Chat audit persistence implemented in database migration `v21`:
  - `chat_query_events`
  - `chat_query_tool_calls`
- Conversational response contract enriched with:
  - `trace_id`
  - `confidence`
  - `sources`
  - `entities_detected`
  - `time_range_used`
  - `actions_recommended`
- Jira webhook ingestion now supports organization scoping:
  - Uses API key scope by default.
  - Accepts optional org hint in payload (`org_name`, `organization`, `org`, `tenant`).
- Non-blocking Sonar workflow added:
  - `.github/workflows/sonar-governance.yml`
  - Optional telemetry publish to `/integrations/jenkins`.
- Local SonarQube stack added to Docker Compose:
  - profile `sonar` with `sonarqube` + `sonarqube-db`
  - local endpoint `http://127.0.0.1:9000`
- Jenkins Sonar integration added (optional, non-blocking):
  - `Jenkinsfile` now includes stage `Sonar Scan (Optional)`.
  - Stage bootstraps `sonar-scanner` if missing, polls CE task and quality gate via Sonar API.
  - Telemetry publish now includes `quality_gate` stage and optional `sonar_dashboard` artifact.
  - Fallback credential supported: `sonar-token` (Jenkins Secret Text) when `SONAR_TOKEN` env is not present.
  - `SONAR_PROJECT_KEY` is auto-inferred from repo name when missing (example: `yohandry10_git-gov`).
  - Jenkins shell scripts hardened for `/bin/sh` compatibility and secret-safe execution (no token echo in logs).
  - Event payload contract aligned with backend (`artifacts` as string array).
- Dashboard Sonar visibility (SQ-03) added:
  - Sonar status badge per commit in recent commits table.
  - Sonar scan/pass/fail/unstable sample metrics in pipeline health widget.
- Quality gate enforcement surface (SQ-04 phase 1) added:
  - `quality_gates` enforcement level in policy contract (Desktop + Tauri model).
  - Policy editor and governance presets now expose `Off/Warn/Block` for Sonar quality gates.
  - Push governance pre-check now triggers when `quality_gates` is enabled.
- Quality gate policy evaluator (SQ-04 phase 2) added server-side:
  - `/policy/check` now includes `quality_gates` in enforcement level resolution.
  - Evaluates latest Sonar-correlated pipeline run by commit SHA.
  - Applies warn/block outcomes when quality gate status is not green.
- Quality gate signal/alert integration (SQ-06 phase 1) added:
  - `/policy/check` now persists a `noncompliance_signal` (`policy_violation`) when `quality_gate_green` fails.
  - Signal evidence includes repo, commit, job, status, enforcement, and is deduplicated (24h window).
  - Alert webhook now emits a dedicated `Quality Gate no verde` message when configured.
  - Validation runbook updated with signal/alert verification (`docs/QUALITY_GATE_POLICY_VALIDATION.md`).
  - Notification formatters now include unit tests (`notifications::tests`).
- Desktop policy-check payload now includes `commit` (HEAD SHA) for richer server-side evaluation.
- Jenkins policy-check stage hardened:
  - Parses JSON response from `/policy/check` (`allowed`, `advisory`, `warnings`, `enforcement_applied`).
  - Fails the build on non-advisory denies, or advisory denies when `GITGOV_STRICT=true`.
- Release readiness scoring (phase 1) added in dashboard:
  - Composite `0-100` score from Jenkins success rate + Jira coverage + Sonar pass rate.
  - Displays signal coverage (`n/3`) to indicate confidence when one source is missing.
- Executive risk outcomes telemetry (phase 1) added in dashboard:
  - `Risk Outcomes (operativo)` widget now exposes derived KPIs from existing signals:
    - trusted-path rate
    - blocked-push rate
    - traceability gap
    - pipeline failure rate (7d)
    - sonar failure rate (sample)
    - unresolved violations rate + critical count
  - Includes composite risk score (`0-100`) with explicit signal coverage (`n/5`).
  - Public docs surface added in website (`/docs/risk-outcomes`, EN/ES) with KPI formulas and operating bands.
- Export surface (`UX-01`) enabled in Control Plane dashboard:
  - `ExportPanel` is now mounted in `ServerDashboard` (admin view), enabling direct audit export and export history visibility from the main dashboard flow.
- Role UX/API alignment improvement:
  - `/chat/ask` now allows `Admin`, `Architect`, and `PM` roles (previously admin-only).
  - Dashboard renders `ConversationalChatPanel` for `Architect` and `PM` in non-admin view.
- Documentation/API contract drift (P0 docs pass) reduced:
  - `/policy/check` examples aligned to real payload keys (`repo`, `commit`) in EN/ES governance docs.
  - `docs/ARCHITECTURE.md` auth semantics aligned for `/signals`, `/violations/{id}/decisions`, and `/policy/check`.
  - `gitgov-server/README.md` export formats aligned to real support (`JSON/CSV`) and compliance path normalized.
  - `CONTRIBUTING.md` clone command generalized to `<owner>/<repo>`.
  - `gitgov-web` Control Plane docs (EN/ES) role table now reflects current access for `Architect` and `PM`.
- Publication hardening guardrails added:
  - `.github/workflows/secret-scan.yml` now includes `Security Guard` steps that enforce restricted-doc exclusions on PR/push.
  - `.gitignore` now excludes `.claude/`, `CLAUDE.md`, `.kiro/`, `.trae/`, `.windsurf/`.
- Secret scanning widened and mandatory on CI surface:
  - `.github/workflows/secret-scan.yml` now runs on all push/PR branches plus manual dispatch.
  - Security permissions for findings publication are declared in workflow.
  - `Security Guard` now also blocks tracked `.env` files (except `.env.example`) and assistant-local artifacts (`.agents/`, `skills/`, `gitgov-video/`).
- CI coverage expanded for documentation website:
  - `.github/workflows/ci.yml` now includes `Website Lint + Typecheck + Build` for `gitgov-web`.
  - Uses `pnpm` lockfile with Node 20 and build validation to catch docs/web regressions before merge.
  - Job order hardened for clean runners (`build` before standalone `typecheck`) to ensure `.next/types` is present.
  - Job now explicitly clears `.next` cache before validation to avoid stale route-type artifacts.
- Jenkins SCM migration runbook documented:
  - `docs/DEPLOYMENT.md` now includes a step-by-step checklist to force jobs to the new repository URL and verify console output.
  - `scripts/jenkins/check_job_repo.ps1` validates Jenkins job SCM URL via `config.xml` and fails on legacy repo markers.
- Quality gate policy validation completed end-to-end (local stack):
  - Verified `quality_gates=warn` keeps advisory flow (`allowed=true`) on non-green Sonar.
  - Verified `quality_gates=block` denies (`allowed=false`) on non-green Sonar.
  - Verified `policy_violation` signal persistence for `quality_gate_green`.
  - Runbook aligned to real API contract (`PUT /policy/{repo_name}/override`, URL-encoded repo path, `offset` on `/signals`).
- Jenkins commit/pipeline correlation validated end-to-end (local stack):
  - Ingested client commit event with contract-correct fields (`repo_full_name`, `commit_sha`).
  - Verified `/integrations/jenkins/correlations` resolves pipeline metadata for matching commit SHA.
- Correlation smoke automation added:
  - New script `scripts/jenkins/validate_commit_pipeline_correlation.ps1`.
  - Validates `/events` ingest + `/integrations/jenkins/correlations` match for a commit SHA (optional pipeline injection for test bootstrap).
  - Deployment guide includes execution commands.
- Branch protection automation prepared:
  - `scripts/github/set_required_checks.ps1` applies required checks and PR protection to `main` via GitHub API.
  - `scripts/github/check_branch_protection.ps1` validates required checks currently configured on `main`.
  - `scripts/github/harden_repo_governance.ps1` orchestrates CI config check + branch protection apply/verify in one execution.
  - Scripts now accept `-GitHubToken` plus env fallbacks (`GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_PAT`) for non-interactive runs.
  - Live execution completed: branch protection applied and verified on `main` with required checks (`server-lint`, `desktop-lint`, `frontend-lint`, `website-lint`, `Security Guard`), strict checks enabled, admins enforced.
- `docs/DEPLOYMENT.md` now includes execution commands + verification checklist.
- Sonar CI rollout preflight automation prepared:
  - `scripts/github/check_ci_repo_config.ps1` audits required GitHub secrets/variables for Sonar + GitGov telemetry.
  - `scripts/github/bootstrap_ci_variables.ps1` bootstraps CI variables (`SONAR_PROJECT_KEY` required, optional `SONAR_HOST_URL` / `GITGOV_URL`).
  - `docs/DEPLOYMENT.md` now includes command + PASS/FAIL expectations for repo CI config.
- Legacy migration hardening added:
  - `Security Guard` in `.github/workflows/secret-scan.yml` blocks forbidden legacy-repo markers in tracked files.

## In Progress

- SonarCloud rollout for GitHub-hosted CI in environments without org constraints.
- Consolidating governance telemetry in dashboards and executive reporting.

## Next Technical Steps

1. Configure repository-level CI secrets/variables for Sonar and GitGov telemetry.
   - Current live status: `SONAR_PROJECT_KEY` and `SONAR_HOST_URL` configured.
   - Pending required secrets: `SONAR_TOKEN`, `GITGOV_API_KEY`.
2. Wire correlation smoke script into CI/manual release checklist to catch contract drift before deployment.
3. Validate the same `quality_gates=warn/block` matrix on GitHub-hosted CI once SonarCloud org onboarding is available (local/Jenkins validation already complete; runbook: `docs/QUALITY_GATE_POLICY_VALIDATION.md`).
4. Tune scoring weights/thresholds with production telemetry and define SLA bands per repo tier.

## Required GitHub Configuration (for Sonar workflow)

Secrets:

- `SONAR_TOKEN`
- `GITGOV_API_KEY`
- `GITGOV_JENKINS_SECRET` (optional)

Variables:

- `SONAR_PROJECT_KEY`
- `SONAR_HOST_URL` (optional, default `https://sonarcloud.io`)
- `GITGOV_URL` (optional if only scan is needed)
