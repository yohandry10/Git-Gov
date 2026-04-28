# GitGov Implementation Status

Updated: 2026-04-28

## Current Execution Summary - 2026-04-25

This section consolidates the latest completed implementation/documentation points so the remaining backlog is explicit.

### Closed Points

| Ticket | Area | Result | Evidence |
|---|---|---|---|
| `KAN-7` | GitHub evidence reporting | Closed the report visibility gap from `0/4` to `4/4` signals. Applied `supabase_schema_v22.sql`, validated `pull_request_review` ingestion, and confirmed GitHub-hosted report/monitor/trend artifacts. | PR `#71`, PR `#72`, report run `24942351831`, monitor run `24942357291`, trend run `24942362269`, `docs/reports/github-evidence-executive-report-prod-review-v22-2026-04-25.md` |
| `KAN-8` | API contract documentation | Reconciled route-table drift. `docs/ARCHITECTURE.md` documents `/jobs/{job_id}/retry`, `/compliance/{org_name}`, and only `/violations/{violation_id}/decisions`; migration chain now includes `v22`. | PR `#73`, main commit `7e0cc4b`, `docs/reports/api-contract-drift-reconciliation-2026-04-25.md` |
| `KAN-9` | Publication security | Hardened `.env.example` policy. Real `.env` files remain blocked; `.env.example` stays trackable; local and GitHub guards reject non-placeholder values for sensitive keys. | PR `#74`, main commit `83240bb`, `docs/reports/env-example-placeholder-policy-2026-04-25.md` |
| `KAN-11` | GitGov API key diagnosis | Corrected the manual Jira ingest diagnosis. The ignored local `GITGOV_API_KEY` authenticates successfully against production; manual Jira ingest also requires `x-gitgov-jira-secret` and `org_name` when production `JIRA_WEBHOOK_SECRET` is configured. | Production `/stats` returned HTTP `200`; manual `/integrations/jira` accepted `KAN-8`; `docs/reports/gitgov-api-key-diagnosis-2026-04-25.md` |
| `KAN-12` | Website publication and traceability recovery | Recreated the local web changes under a traceable Jira branch/commit/PR flow. The invalid local-only commit `f2bdb24` (`dle`) was not pushed; the valid publication landed on `main` through PR `#77`. | PR `#77`, main commit `a0a4174`, CI run `24974947818`, Release Readiness run `24974947816`, `docs/reports/kan-12-web-publication-2026-04-28.md` |
| `KAN-13` | Documentation publication governance | Clarified when docs must use placeholders and when real repo/service identifiers may remain for agent operating memory or historical validation evidence. | `docs/PUBLICATION_POLICY.md`, `docs/reports/kan-13-publication-governance-2026-04-28.md` |
| `KAN-14` | Operational validation refresh | Refreshed local and production validation after starting Docker Desktop and the Sonar/Jenkins Compose profiles. | Render `/health` `ok`, production `/stats` HTTP `200`, local backend `/health` on port `3001`, Sonar `UP` / quality gate `OK`, Jenkins build `#30` `SUCCESS`, readiness `91/100`; `docs/reports/kan-14-operational-validation-2026-04-28.md` |
| `KAN-15` | OpenAPI partial-contract guard | Added a regression test that preserves the `/api-docs` partial schema-explorer disclaimer and keeps `docs/ARCHITECTURE.md` plus the `main.rs` route table as the operational contract source. | `gitgov/gitgov-server/src/openapi.rs`, `docs/reports/kan-15-openapi-partial-contract-guard-2026-04-28.md` |
| `KAN-16` | Provider access validation | Added a single secret-safe PowerShell smoke test for GitGov production/local health, SonarQube, Jenkins, Jira, and optional release readiness using ignored env files. | `scripts/control-plane/validate_provider_access.ps1`; latest validation all checks `ok`, readiness `91/100` |

### Current Remaining Work

1. `GITGOV_API_KEY` production admin access is usable from ignored local env files.
   - `https://gitgov-api.onrender.com/stats` returned HTTP `200` with the local key.
   - The previous manual `/integrations/jira` `401` was caused by missing Jira shared-secret handling, not by a bad GitGov API key.
   - Manual Jira ingest requires `Authorization: Bearer <GITGOV_API_KEY>`, `x-gitgov-jira-secret: <JIRA_WEBHOOK_SECRET>`, and an `org_name` payload hint such as `yohandry10`.
2. Sonar remains intentionally local.
   - SonarCloud is not applicable for the current personal GitHub account.
   - GitHub-hosted runners cannot reach `localhost:9000`; keep GitHub Sonar scan optional/non-blocking unless a self-hosted runner is added.
   - Latest local validation on 2026-04-28: SonarQube `UP`, project `yohandry10_git-gov`, quality gate `OK`.
3. Jenkins trigger-only URL flow is still optional and separate from Jenkins API access.
   - API inspection/build access works through `JENKINS_API_TOKEN`.
   - The unauthenticated/manual trigger URL requires `JENKINS_BUILD_TRIGGER_TOKEN` only if that flow is needed.
   - Latest local validation on 2026-04-28: job `gitgov-demo-pipeline`, last build `#30`, result `SUCCESS`, not building.
4. OpenAPI is still partial by design.
   - `/api-docs` is a schema explorer, not the full operational route contract.
   - Implement `#[utoipa::path]` coverage only if generated SDKs or Swagger-based contract tests become a requirement.
   - `KAN-15` added a unit guard so this partial-scope claim cannot be removed silently.
5. Traceability coverage remains an operating discipline.
   - Platform guardrails are active.
   - Continue using Jira IDs in branch names, PR titles, commit messages, and PR comments to keep readiness/ticket coverage healthy.
6. Documentation governance cleanup is now policy-defined.
   - Public examples/templates should use placeholders.
   - Agent operating memory and historical evidence snapshots may keep real repo/service identifiers when needed for validation scope.
   - Restricted forensic/strategy docs remain ignored and blocked by publication guard.

## Completed

- Repository migration completed to `<owner>/<repo>`.
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
- Bot trace redaction hardening (VS-13 phase 1):
  - `question` / `answer_preview` and tool payload fields are sanitized before persistence.
  - Trace payload sanitizer now redacts sensitive keys and nested token/email-like values.
  - `conversation_key` is persisted as SHA-256 hash (`conv_sha256:*`) in trace evidence.
- Jira webhook ingestion now supports organization scoping:
  - Uses API key scope by default.
  - Accepts optional org hint in payload (`org_name`, `organization`, `org`, `tenant`).
  - For global admin keys, `org_name` hint is now required (strict tenant scoping).
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
  - Fallback credential supported: `gitgov-token` (Jenkins Secret Text) when `SONAR_TOKEN` env is not present.
  - `SONAR_PROJECT_KEY` is auto-inferred from repo name when missing (example: `<owner>_<repo>`).
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
- Governed quality-gate exception flow (SQ-09 phase 1) added:
  - `PUT /policy/{repo}/override` now supports governed payload with `quality_gate_exception` (`reason`, `ticket_id`, `approved_by`, `expires_at`).
  - Quality gate enforcement downgrade (`block->warn/off`, `warn->off`) is rejected unless an active exception is provided.
  - Legacy override payload compatibility preserved (existing exception retained when clients send plain `GitGovConfig`).
  - `/policy/check` now recognizes active exception, marks violation as `enforcement=override`, and allows with warning while exception remains active.
  - Integration tests added:
    - `policy_override_rejects_quality_gate_downgrade_without_exception`
    - `policy_override_accepts_governed_exception_for_quality_gate_downgrade`
- Desktop policy-check payload now includes `commit` (HEAD SHA) for richer server-side evaluation.
- Jenkins policy-check stage hardened:
  - Parses JSON response from `/policy/check` (`allowed`, `advisory`, `warnings`, `enforcement_applied`).
  - Fails the build on non-advisory denies, or advisory denies when `GITGOV_STRICT=true`.
- Release readiness scoring (phase 1) added in dashboard:
  - Composite `0-100` score from Jenkins success rate + Jira coverage + Sonar pass rate.
  - Displays signal coverage (`n/3`) to indicate confidence when one source is missing.
- Release readiness gate (SQ-10 phase 2) added for CI/ops:
  - New script `scripts/jenkins/validate_release_readiness_gate.ps1` evaluates readiness by `repo+branch+tier` and exits non-zero when below target.
  - Supports strict signal coverage mode (`-FailOnMissingSignals`) and custom thresholds (`-MinReadiness`).
  - GitHub Actions workflow `.github/workflows/release-readiness-gate.yml` added (push `main` + manual dispatch), with explicit skip when `GITGOV_URL`/`GITGOV_API_KEY` are missing.
  - The workflow also runs daily at `10:17 UTC`, refreshes Jira/PR correlations before scoring, and enforces the standard readiness target on scheduled runs.
  - Push/manual runs remain advisory unless `enforce_gate=true`; failed Jira refresh only blocks enforced runs.
  - Manual runs default to a 720h lookback window and expose `refresh_jira_correlations` to control whether `/integrations/jira/correlate` runs before scoring.
  - Produces JSON artifact with score, signal coverage, and fail reasons per run.
  - Produces an additional Jira correlation refresh JSON artifact when pre-score refresh is enabled.
  - First GitHub-hosted validation after scheduling passed on run `24927045053` for commit `a94114c`: Jira refresh artifact generated, readiness `81/100`, target `75`, signal coverage `3/3`.
  - Jenkins pipeline integration added in `Jenkinsfile` as `Release Readiness Gate (Optional)`:
    - Controlled by env flags (`GITGOV_RELEASE_GATE_*`).
    - Emits `release_readiness` stage telemetry with score/target/coverage/reasons.
    - Honors `GITGOV_STRICT` for block vs warn behavior.
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
- Tier-aware scoring + SLA profiles added in dashboard:
  - New risk/readiness scoring model centralizes weights, bands, and thresholds by repo tier (`Critical`, `Standard`, `Internal`).
  - Admin dashboard now includes tier selector with persisted profile.
  - `Pipeline Health` and `Risk Outcomes` now apply tier-specific readiness/risk bands and SLA thresholds.
  - Risk outcomes docs (EN/ES) now include baseline SLA targets by tier.
- Weekly calibration automation added for tier baselines:
  - `scripts/control-plane/calibrate_risk_tier_baseline.ps1` computes release readiness + composite risk + KPI snapshot by tier from live Control Plane APIs.
  - Exports markdown evidence to `docs/reports/risk-tier-baseline-<timestamp>.md`.
  - Local baseline execution evidence captured for all tier profiles:
    - `docs/reports/risk-tier-baseline-local-2026-04-20.md` (standard)
    - `docs/reports/risk-tier-baseline-local-critical-2026-04-20.md` (critical)
    - `docs/reports/risk-tier-baseline-local-internal-2026-04-20.md` (internal)
  - Deployment runbook updated with execution command and expected output.
  - GitHub Actions scheduler/manual trigger added: `.github/workflows/risk-tier-baseline-calibration.yml` (weekly Monday 12:00 UTC, skips cleanly when `GITGOV_URL`/`GITGOV_API_KEY` are missing).
- Domain SLO lock/validation automation added:
  - `ops/slo/domain-slo-targets.json` defines per-domain tier + explicit SLO targets.
  - Production targets are scoped to `org_name=yohandry10`; unscoped validation overstates traceability gap because it reads broader telemetry.
  - `scripts/control-plane/validate_domain_slo_target_config.ps1` statically validates the lock file and requires org/repo/branch scope in CI.
  - `scripts/control-plane/validate_domain_slo_targets.ps1` validates each domain against locked targets using live Control Plane telemetry.
  - GitHub Actions scheduler/manual trigger added: `.github/workflows/domain-slo-validation.yml` (weekly Monday 12:45 UTC + manual dispatch).
  - Local evidence generated at `docs/reports/domain-slo-validation-local-2026-04-20/domain-slo-summary.md`.
  - Production evidence generated on 2026-04-25 at `docs/reports/domain-slo-validation-prod-2026-04-25/domain-slo-summary.md`; all three domains passed with traceability gap `11.8%`.
- Export surface (`UX-01`) enabled in Control Plane dashboard:
  - `ExportPanel` is now mounted in `ServerDashboard` (admin view), enabling direct audit export and export history visibility from the main dashboard flow.
- Role UX/API alignment improvement:
  - `/chat/ask` now allows `Admin`, `Architect`, and `PM` roles (previously admin-only).
  - Dashboard renders `ConversationalChatPanel` for `Architect` and `PM` in non-admin view.
- Authorization semantics normalized for admin gates:
  - `require_admin` now returns explicit `403 FORBIDDEN` (instead of `401`) when API key is valid but role is insufficient.
  - Added auth regression test to lock expected forbidden behavior.
- Public endpoint rate-limiting hardening applied:
  - Added explicit limiter for `POST /webhooks/github` (`GITGOV_RATE_LIMIT_GITHUB_WEBHOOK_PER_MIN`, default `240`).
  - Added explicit limiter for invitation public endpoints (`GET /org-invitations/preview/{token}`, `POST /org-invitations/accept`) via `GITGOV_RATE_LIMIT_ORG_INVITATION_PER_MIN` (default `90`).
- Jira ingest org scoping hardened:
  - `POST /integrations/jira` now enforces strict org scope resolution for global admin keys (requires `org_name` hint), preventing `project_tickets.org_id = NULL` ingestion paths.
  - Error contract for this path is now explicit: `org_name is required for global admin keys`.
- Jenkins ingest org scoping hardened:
  - `POST /integrations/jenkins` now enforces API-key org scope during ingestion.
  - Scoped admin keys cannot ingest pipeline events into a different org; unresolved repo scope now falls back to the key scope for scoped keys.
- OpenAPI/Swagger claim adjusted to reflect real scope:
  - `/api-docs` is now described as a schema explorer (partial), preventing mismatch with full operational route coverage.
  - OpenAPI info description now points to `docs/ARCHITECTURE.md` + `main.rs` route table as source of truth until full path annotation rollout.
- API contract drift reconciliation completed under `KAN-8`:
  - `docs/ARCHITECTURE.md` already documents the real backend routes for job retry, compliance, and violation decisions.
  - `docs/ARCHITECTURE.md` schema migration chain now includes `supabase_schema_v22.sql`.
  - The local ignored internal audit memory (`docs/ENTERPRISE_READINESS_DECISION.md`) was reconciled but remains intentionally untracked by `.gitignore`.
  - Evidence report: `docs/reports/api-contract-drift-reconciliation-2026-04-25.md`.
- `.env.example` publication policy hardened under `KAN-9`:
  - `.gitignore` already allows `.env.example` while blocking real `.env` files.
  - Local publication guard and GitHub `Security Guard` now fail when sensitive keys in tracked `.env.example` files contain non-placeholder values.
  - Existing `gitgov/.env.example` and `gitgov/gitgov-server/.env.example` passed the placeholder-only validation.
  - Evidence report: `docs/reports/env-example-placeholder-policy-2026-04-25.md`.
- Conversational bot quality/risk deterministic queries added:
  - `detect_query` now classifies quality gate health questions and release-readiness gate health questions.
  - `detect_query` now also classifies repo-ranking questions (`top repos con quality gate no verde`).
  - `/chat/ask` now returns scoped summaries for:
    - quality gate outcomes (`green/non-green`, affected repos/commits, policy-violation signals)
    - ranked Jira tickets linked to commits with non-green quality gates.
    - ranked Jira tickets deployed/released with non-green quality gates (risk after release).
    - ranked developers/equipos with highest non-green quality-gate volume.
    - release-readiness gate outcomes (`pass/warn/fail/other`, affected repos/commits)
    - ranked repositories with highest non-green quality-gate volume in a selected window.
    - ranked branches with highest non-green quality-gate volume in a selected window.
    - ranked repositories with highest release-readiness `FAIL` volume in a selected window.
    - ranked branches with highest release-readiness `FAIL` volume in a selected window.
  - Backed by new DB aggregations over `pipeline_events` + `noncompliance_signals` with window support (`24h/7d/30d` via query intent).
  - Classification regression tests updated and passing.
- Documentation/API contract drift (P0 docs pass) reduced:
  - `/policy/check` examples aligned to real payload keys (`repo`, `commit`) in EN/ES governance docs.
  - `docs/ARCHITECTURE.md` auth semantics aligned for `/signals`, `/violations/{id}/decisions`, and `/policy/check`.
  - `gitgov-server/README.md` export formats aligned to real support (`JSON/CSV`) and compliance path normalized.
  - `CONTRIBUTING.md` clone command generalized to `<owner>/<repo>`.
  - Deployment and validation runbooks now use neutral placeholders (`<owner>/<repo>`, `<owner>_<repo>`) instead of personal repository identifiers.
  - `gitgov-web` Control Plane docs (EN/ES) role table now reflects current access for `Architect` and `PM`.
- Desktop UI/infra hardcoded-repo coupling reduced:
  - Login/download repo link now supports `VITE_PUBLIC_REPO_URL`.
  - Desktop updater fallback now derives from `VITE_PUBLIC_REPO_URL` (or explicit `VITE_DESKTOP_DOWNLOAD_FALLBACK_URL`).
  - UI placeholder examples use generic values (no personal usernames/repo names).
- Publication hardening guardrails added:
  - `.github/workflows/secret-scan.yml` now includes `Security Guard` steps that enforce restricted-doc exclusions on PR/push.
  - `.gitignore` now excludes local assistant/editor scratch artifacts to avoid accidental publication.
  - Local equivalent guard added: `scripts/security/publication_guard.ps1` for pre-push validation (`restricted/env/legacy` checks).
  - Neutral naming guard added in CI + local guardrails: branch/PR/commit metadata now fail validation if they include internal tooling markers.
- Secret scanning widened and mandatory on CI surface:
  - `.github/workflows/secret-scan.yml` now runs on all push/PR branches plus manual dispatch.
  - Security permissions for findings publication are declared in workflow.
  - `Security Guard` now also blocks tracked `.env` files (except `.env.example`) and local automation/work artifacts (`.agents/`, `skills/`, generated media folders).
- CI coverage expanded for documentation website:
  - `.github/workflows/ci.yml` now includes `Website Lint + Typecheck + Build` for `gitgov-web`.
  - `.github/workflows/ci.yml` now includes `Workflow Lint` (`rhysd/actionlint`) to catch invalid GitHub Actions syntax before merge.
  - Uses `pnpm` lockfile with Node 20 and build validation to catch docs/web regressions before merge.
  - Job order hardened for clean runners (`build` before standalone `typecheck`) to ensure `.next/types` is present.
  - Job now explicitly clears `.next` cache before validation to avoid stale route-type artifacts.
  - Added explicit `pnpm/action-setup` bootstrap before `actions/setup-node` cache resolution (prevents `pnpm` missing executable failures on hosted runners).
  - First-party GitHub Actions are upgraded for Node 24 action-runtime compatibility:
    - `actions/checkout@v6`
    - `actions/setup-node@v6`
    - `actions/upload-artifact@v7`
    - `pnpm/action-setup@v5`
  - `node-version: 20` remains the application build runtime where configured.
  - First GitHub-hosted validation after the full upgrade passed on `main` commit `3f4c601`: CI run `24927274092` passed without the previous Node.js 20 action-runtime annotation, and Release Readiness Gate run `24927274091` passed with readiness `82/100`, target `75`, signal coverage `3/3`.
- Jenkins SCM migration runbook documented:
  - `docs/DEPLOYMENT.md` now includes a step-by-step checklist to force jobs to the new repository URL and verify console output.
  - `scripts/jenkins/check_job_repo.ps1` validates Jenkins job SCM URL via `config.xml` and fails on legacy repo markers.
- Quality gate policy validation completed end-to-end (local stack):
  - Verified `quality_gates=warn` keeps advisory flow (`allowed=true`) on non-green Sonar.
  - Verified `quality_gates=block` denies (`allowed=false`) on non-green Sonar.
  - Verified `policy_violation` signal persistence for `quality_gate_green`.
  - Runbook aligned to real API contract (`PUT /policy/{repo_name}/override`, URL-encoded repo path, `offset` on `/signals`).
  - Added automated matrix validator script:
    - `scripts/jenkins/validate_quality_gate_policy_matrix.ps1` toggles `quality_gates=warn/block`, validates failing+green commits, and restores original policy.
  - Added automatic SHA resolver for cloud runs:
    - `scripts/jenkins/resolve_quality_gate_matrix_commits.ps1` (correlations-first + signal fallback).
  - Added GitHub Actions optional matrix workflow:
    - `.github/workflows/quality-gate-policy-matrix.yml` (`push/main` + `workflow_dispatch`, auto-skip without config).
  - Evidence reports:
    - `docs/reports/quality-gate-policy-matrix-local-2026-04-20.md` (baseline)
    - `docs/reports/quality-gate-policy-matrix-auto-local-2026-04-20.md` (baseline)
    - `docs/reports/quality-gate-policy-matrix-auto-local-2026-04-23.md` (latest)
- Jenkins commit/pipeline correlation validated end-to-end (local stack):
  - Ingested client commit event with contract-correct fields (`repo_full_name`, `commit_sha`).
  - Verified `/integrations/jenkins/correlations` resolves pipeline metadata for matching commit SHA.
- Correlation smoke automation added:
  - New script `scripts/jenkins/validate_commit_pipeline_correlation.ps1`.
  - Validates `/events` ingest + `/integrations/jenkins/correlations` match for a commit SHA (optional pipeline injection for test bootstrap).
  - Supports optional `JENKINS_WEBHOOK_SECRET` via `-JenkinsSecret` when backend enforcement is enabled.
  - Wired into GitHub Actions via `.github/workflows/governance-correlation-smoke.yml` (push/main + manual dispatch, non-blocking, auto-skip when config is missing).
  - Deployment guide includes execution commands.
- Branch protection automation prepared:
  - `scripts/github/set_required_checks.ps1` applies required checks and PR protection to `main` via GitHub API.
  - `scripts/github/check_branch_protection.ps1` validates required checks currently configured on `main`.
  - `scripts/github/harden_repo_governance.ps1` orchestrates CI config check + branch protection apply/verify in one execution.
  - `scripts/github/harden_repo_governance.ps1` now supports `-BestEffort` to continue diagnostics when a fine-grained token lacks admin/actions-read permissions.
  - Scripts now accept `-GitHubToken` plus env fallbacks (`GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_PAT`, `GITHUB_PERSONAL_ACCESS_TOKEN`) for non-interactive runs.
  - If env token is not set, scripts auto-resolve `GITHUB_PERSONAL_ACCESS_TOKEN` from `gitgov/gitgov-server/.env`.
  - API failures now surface `accepted_permissions` hints from GitHub headers (faster token permission diagnosis).
  - `scripts/github/check_token_permissions.ps1` now supports machine-readable mode (`-EmitJson`) and optional non-failing diagnostics (`-NoFailOnForbidden`, `-Quiet`) for automation pipelines.
  - `harden_repo_governance.ps1` now runs token-permission preflight (`check_token_permissions.ps1`) before CI/protection steps.
  - `scripts/github/create_or_print_pr.ps1` added to automate PR creation and fallback to compare URL when token lacks `pull_requests` permissions.
  - GitHub/Jenkins helper scripts now avoid hardcoded personal repo defaults; `owner/repo` are auto-resolved from `GITHUB_REPOSITORY` or `git remote origin` when omitted.
- Live execution completed: branch protection applied and verified on `main` with required checks (`Server Clippy + Check`, `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`, `Security Guard`), strict checks enabled, admins enforced.
- `docs/DEPLOYMENT.md` now includes execution commands + verification checklist.
- Sonar CI rollout preflight automation prepared:
  - `scripts/github/check_ci_repo_config.ps1` audits required GitHub secrets/variables for Sonar + GitGov telemetry.
  - `scripts/github/bootstrap_ci_variables.ps1` bootstraps CI variables (`SONAR_PROJECT_KEY` required, optional `SONAR_HOST_URL` / `GITGOV_URL`).
  - `docs/DEPLOYMENT.md` now includes command + PASS/FAIL expectations for repo CI config.
  - Preflight mode control added:
    - `-AllowMissingSonar` (Sonar config optional for personal-account rollout).
    - `-RequireGitGovTelemetry` (enforces `GITGOV_API_KEY` + `GITGOV_URL`).
    - `-NoFailOnForbidden` (best-effort mode when fine-grained token cannot read Actions secrets/variables; reports `UNKNOWN` instead of failing).
  - `scripts/github/harden_repo_governance.ps1` forwards CI preflight flags for end-to-end governance runs (`AllowMissingSonar`, `RequireGitGovTelemetry`, and best-effort `NoFailOnForbidden`).
- Cloud CI preflight evidence captured:
  - `docs/reports/github-ci-preflight-2026-04-20.md` includes current PAT-scope diagnostic and required permission hints to close strict GitHub-hosted validation.
- Quality gate matrix revalidated end-to-end (local stack, latest):
  - `docs/reports/quality-gate-policy-matrix-auto-local-2026-04-23.md`
  - `docs/reports/quality-gate-matrix-commit-resolution-auto-local-2026-04-23.json`
  - Result: `PASS` (`warn` allows with violation; `block` denies non-green and allows green).
- Historical GitHub-hosted matrix attempts captured:
  - Earlier runs skipped while repo Actions config was incomplete.
  - This is superseded by the 2026-04-24 completed matrix validation on `main`.
- Public infra preflight automation added:
  - `scripts/deploy/validate_public_infra.ps1` validates domain DNS, TLS certificate, health endpoint, authenticated stats, and webhook/integration route reachability.
  - Local dry-run evidence generated at `docs/reports/public-infra-validation-local-2026-04-20.md` (expected `WARN` on non-HTTPS localhost).
- Enterprise readiness bundle automation added:
  - `scripts/deploy/run_enterprise_readiness_bundle.ps1` orchestrates infra, updater, quality-gate matrix, tier baseline, and GitHub cloud prechecks in one run.
  - Evidence bundles generated at:
    - `docs/reports/readiness-bundle-2026-04-20T075942Z/`
    - `docs/reports/readiness-bundle-2026-04-20T183000Z/`
  - Optional weekly/manual workflow added: `.github/workflows/enterprise-readiness-bundle.yml`.
- Desktop updater readiness automation added:
  - `scripts/deploy/validate_desktop_updater_readiness.ps1` validates `plugins.updater` config, endpoint syntax, and live `latest.json` reachability/manifest shape.
  - Local evidence generated at `docs/reports/desktop-updater-readiness-local-2026-04-20.md` (current warning: updater endpoint returns `404` for `latest.json` and requires publish step).
- Desktop updater release helpers implemented:
  - `scripts/release/desktop-updater/New-TauriUpdaterManifest.ps1`
  - `scripts/release/desktop-updater/Publish-DesktopUpdateAws.ps1`
  - `scripts/release/desktop-updater/New-TauriUpdaterConfigSnippet.ps1`
  - Optional cloud readiness workflow added: `.github/workflows/desktop-updater-readiness.yml` (push/main + manual dispatch, artifact report per run).
- Desktop updater phase 3 enforcement completed:
  - Runtime policy evaluator now enforces `min_supported_version` and `force_update` metadata from updater manifest (`latest.json`).
  - App-level mandatory update gate blocks normal navigation until update action/manual fallback.
  - Manifest helper script now supports critical-policy keys (`min_supported_version`, `force_update`, `force_update_reason`, `critical_update`).
  - Updater readiness validator now checks policy metadata shape and warns on missing/invalid enforcement fields.
- Legacy migration hardening added:
  - `Security Guard` in `.github/workflows/secret-scan.yml` blocks forbidden legacy-repo markers in tracked files.
- Public naming hardening added:
  - `.github/workflows/public-naming-guard.yml` enforces branch/commit naming policy and blocks internal-assistant markers (for public history hygiene).
  - `scripts/github/check_public_naming_policy.ps1` performs deterministic validation for branch name and commit subjects.
- CI lint stability hardening:
  - Refactored `gitgov-server` DB insert APIs to typed input structs to satisfy `clippy -D warnings` (removed `too_many_arguments` failures).
  - Local validation completed: `cargo clippy -- -D warnings` and `cargo test` (150 passed).
- GitHub-hosted quality-gate matrix validation completed:
  - `quality_gates=warn/block` matrix passed on GitHub-hosted CI after repository Actions config was aligned.
  - Required branch protection check `Validate quality_gates warn/block matrix` is present on `main`.
  - Follow-up output fix merged through PR `#6`.
  - Matrix branch PR `#5` merged into `main`.
- GitHub Actions repository configuration completed for GitGov telemetry:
  - `GITGOV_API_KEY` configured as a repository secret.
  - `GITGOV_URL=https://gitgov-api.onrender.com` configured as a repository variable.
  - SonarCloud is intentionally not the target because the current GitHub account is personal, not organizational.
  - SonarQube local is the selected Sonar runtime; GitHub-hosted Sonar scan remains optional/non-blocking unless a runner can reach the configured SonarQube host.
- Render backend deployment completed:
  - Backend service `gitgov-api` is deployed from `main`.
  - Public URL: `https://gitgov-api.onrender.com`.
  - Root directory: `gitgov/gitgov-server`.
  - Deployment guide drift was cleaned so Render is the documented production route; EC2/Nginx/systemd remains only as legacy/self-hosted guidance.
  - The old domain/`certbot`/webhook pending list was replaced with the actual state: Render HTTPS active, GitHub webhook configured, and native Jira webhook configured.
- Local operational access configured:
  - SonarQube local API token created and validated.
  - Jenkins local API token created and validated as `admin`.
  - Jenkins job `gitgov-demo-pipeline` API metadata validated.
  - Runbook added: `docs/OPERATIONS_ACCESS.md`.
- Jira Cloud operational access configured:
  - Jira API credentials are stored in ignored local env files.
  - Project `KAN` (`GitGov`, project ID `10000`) was validated by API.
  - Traceability validation tickets `KAN-4`, `KAN-5`, and `KAN-6` were created by API.
  - Native signed Jira webhook `GitGov signed issue sync` was configured for `jira:issue_created`, `jira:issue_updated`, and `jira:issue_deleted` with JQL `project = KAN`.
  - End-to-end Jira webhook delivery was validated by updating `KAN-6` and observing GitGov ingest advance.
- GitHub webhook operational access configured:
  - Repository webhook ID `610772988` targets `https://gitgov-api.onrender.com/webhooks/github`.
  - Events include push/create, PR lifecycle, PR reviews, PR review comments, PR-linked issue comments, check runs/suites, and commit statuses.
  - Webhook authentication is HMAC-based through `GITHUB_WEBHOOK_SECRET` configured on Render and in the GitHub webhook.
- GitHub PR-title traceability validation completed:
  - PR titles containing `KAN-4` are ingested from real GitHub webhook deliveries and can create `commit_ticket_correlations` rows with `source=pr_title`.
  - PR merge materialization is idempotent, so duplicate or redelivered `pull_request` events can repair missing `pull_request_merges` records.
  - GitHub org upsert now resolves existing organizations by `login` before inserting/updating by `github_id`, preventing production webhook failures on existing org rows.
- GitHub webhook evidence extraction contract tests added:
  - `github_webhook_tests` now cover `check_run`, `check_suite`, `status`, and `pull_request_review_comment` extraction without requiring database or provider credentials.
  - Validates branch/SHA/status metadata extraction and PR review comment SHA fallback behavior.
  - Post-merge validation on `main` for commit `946fac3` passed: CI run `24927816238`, Quality Gate Policy Matrix run `24927816230`, and Release Readiness Gate run `24927816225`.
- Executive GitHub evidence dashboard summary added:
  - `EventBreakdownGrid` now shows executive evidence coverage (`n/4`), status (`Completo`, `Parcial`, `Sin evidencia`), and missing signal families for PR lifecycle, reviews, PR comments, and checks/status.
  - `GitHubEvidenceTrendWidget` lets operators capture local dashboard snapshots and view coverage delta/history without requiring GitHub Actions token access from the frontend.
  - Trend snapshots are stored in browser `localStorage` under `gitgov.dashboard.github_evidence_trend`; GitHub Actions artifact trend reporting remains the cloud evidence path.
  - `buildGitHubEvidenceSummary` has Vitest coverage for complete, partial, and empty signal sets.
  - Post-merge validation on `main` for commit `01d275c` passed: CI run `24938441269`, Quality Gate Policy Matrix run `24938441278`, and Release Readiness Gate run `24938441273`.
  - Post-merge validation for the local trend widget on `main` commit `74a51a5` passed: CI run `24940280762`, Quality Gate Policy Matrix run `24940280775`, and Release Readiness Gate run `24940280751`.
  - PR-title correlation source names were aligned with the production DB constraint; valid sources remain `branch_name`, `commit_message`, `pr_title`, and `manual`.
  - Production validation after deploy observed real webhook delivery HTTP `200`, `processed=true`, at least `2` `pull_request_merges` records, and a Jira backfill run with `scanned_prs=2` and `correlations_created=2`.
  - Direct validation found `KAN-4` PR-title correlations across validated merge/head SHAs.
- Ticket coverage now counts PR merge evidence:
  - `/integrations/jira/ticket-coverage` no longer builds its denominator only from `client_events`.
  - Coverage now unions client commit events with materialized `pull_request_merges`.
  - For PR merges, it uses `merge_commit_sha` from payload first and falls back to `head_sha`.
  - PR-title correlations can therefore affect Jira ticket coverage even when the merge commit arrived only from a GitHub webhook.
  - Regression test added: `ticket_coverage_counts_pr_merge_commit_without_client_event`.
- Render production deployment context documented:
  - Service `gitgov-api` deploys from `main` with root directory `gitgov/gitgov-server`.
  - Render API access is available through ignored env key `RENDER_API_KEY`.
  - Production deploys were validated after the GitHub webhook and PR-title correlation fixes.
- Production validation after ticket coverage deploy:
  - Render deployed commit `0494648` for PR `#35`.
  - Health check passed on `https://gitgov-api.onrender.com/health`.
  - Jira backfill scanned `4` merged PRs and created `0` new correlations because relevant rows already existed.
  - Ticket coverage for `yohandry10/Git-Gov`, branch `main`, 720h returned `30` total commits, `5` with tickets, and `16.67%` coverage.
  - Release readiness gate passed with readiness `77/100` against standard target `75`, signal coverage `3/3`, pipeline success `96.77%`, and Sonar pass `96.77%`.
- Traceability guardrail added:
  - `Security Guard` now requires Jira-style ticket IDs in branch names, PR titles, and new commit messages.
  - Local helper added at `scripts/github/check_traceability_policy.ps1`.
  - `scripts/security/publication_guard.ps1` now invokes the traceability helper for branch + HEAD commit preflight.
  - `.githooks/commit-msg` now enforces Jira ticket IDs before local CLI commits when hooks are enabled.
  - PR template, contributing guide, and publication policy now document ticket-ID requirements.
  - This protects the `pull_request_merges` + PR-title coverage path from regressing as new work lands.
- Production validation after traceability guard rollout:
  - Jira backfill scanned `8` merged PRs and created `0` new correlations because existing rows were already present.
  - Ticket coverage for `yohandry10/Git-Gov`, branch `main`, 720h increased to `34` total commits, `9` with tickets, and `26.47%` coverage.
  - Release readiness gate passed with readiness `79/100` against standard target `75`, signal coverage `3/3`, pipeline success `97.14%`, and Sonar pass `97.14%`.
- Production tier/SLO calibration after Node 24 workflow hardening:
  - Jira PR-title backfill scanned `14` merged PRs and created `0` new correlations.
  - Tier baseline evidence generated under `docs/reports/risk-tier-baseline-prod-2026-04-25/`.
  - Critical profile: readiness `96/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Standard profile: readiness `95/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Internal profile: readiness `96/100`, composite risk `4/100`, traceability gap `11.8%`, no SLA breaches.
  - Domain SLO evidence generated under `docs/reports/domain-slo-validation-prod-2026-04-25/`.
  - `core-platform`, `standard-services`, and `internal-tools` passed SLO validation after targets were scoped to `org_name=yohandry10`.
- Domain SLO target config guardrail:
  - Added static validation script `scripts/control-plane/validate_domain_slo_target_config.ps1`.
  - CI `Workflow Lint` and `.github/workflows/domain-slo-validation.yml` now fail early if `ops/slo/domain-slo-targets.json` is malformed or lacks required `org_name`, `repo_full_name`, or `branch` scope.
  - Post-merge validation on `main` for commit `f0a3470` passed: CI run `24927603357`, Quality Gate Policy Matrix run `24927603365`, and Release Readiness Gate run `24927603352`.
- Executive GitHub evidence export packaging:
  - Dashboard audit exports now download a JSON package with `executive_summary.github_evidence` plus raw export records under `data`.
  - The export package reuses the dashboard `n/4` GitHub evidence model for PR lifecycle, reviews, PR comments, and checks/status.
  - Unit coverage validates the package shape and executive summary classification.
  - Post-merge validation on `main` for commit `458c048` passed: CI run `24938795096`, Quality Gate Policy Matrix run `24938795085`, and Release Readiness Gate run `24938795100`.
- Executive GitHub evidence report artifact generation:
  - Added `scripts/control-plane/generate_github_evidence_report.ps1`.
  - The script generates a standalone Markdown report from live `/stats` or an offline stats JSON fixture.
  - Reported signal model matches the dashboard/export package: PR lifecycle, reviews, PR comments, and checks/status.
  - Offline fixture validation passed without requiring provider tokens.
  - Added `.github/workflows/github-evidence-report.yml` for manual and weekly artifact generation.
  - The workflow uploads the generated Markdown report as `github-evidence-executive-report` and skips cleanly when `GITGOV_URL` or `GITGOV_API_KEY` is missing.
  - Manual workflow validation passed on run `24939329055` for `main` commit `3935c21`; artifact `github-evidence-executive-report` was uploaded successfully.
- Executive GitHub evidence report artifact monitoring:
  - Added `scripts/control-plane/validate_github_evidence_report_artifact.ps1`.
  - The script queries GitHub Actions for the latest successful `github-evidence-report.yml` run and validates artifact freshness without reading provider secrets.
  - Added `.github/workflows/github-evidence-artifact-monitor.yml` for manual and Tuesday 14:07 UTC freshness checks.
  - Local live validation passed against report workflow run `24939329055`; artifact `6642253304` existed, was not expired, and was within the 192h freshness window.
  - First GitHub-hosted validation passed on run `24939815276`; artifact `github-evidence-artifact-monitor` ID `6642391452` uploaded successfully and was not expired.
- Executive GitHub evidence trend reporting:
  - Added `scripts/control-plane/generate_github_evidence_trend_report.ps1`.
  - The script downloads recent non-expired `github-evidence-executive-report` artifacts from successful `github-evidence-report.yml` runs and parses status, coverage, and missing signal fields.
  - Added `.github/workflows/github-evidence-trend-report.yml` for manual and Tuesday 14:17 UTC trend generation.
  - Local live validation parsed workflow run `24939329055` and produced Markdown/JSON trend outputs with one report point.
  - First GitHub-hosted validation passed on run `24940027811` for `main` commit `a58ae81`; artifact `github-evidence-trend-report` ID `6642453325` uploaded successfully and was not expired.
  - Post-merge validation passed on `main` commit `a58ae81`: CI run `24940024455`, Quality Gate Policy Matrix run `24940024458`, and Release Readiness Gate run `24940024457`.
  - GitHub evidence stats scope fix:
    - Added migration `gitgov/gitgov-server/supabase/supabase_schema_v22.sql`.
    - Restores real `github_events` totals, daily counts, `by_type`, and `active_repos` in `get_audit_stats`.
    - Keeps v19 violation decision semantics.
  - Added postcheck `gitgov/gitgov-server/supabase/checks/v22_postcheck.sql`.
  - Production DB migration was applied and `v22_postcheck.sql` passed.
  - Initial live report validation returned `Parcial` / `3/4 signals`; the previous `0/4 signals` stats visibility gap is closed.
  - Initial GitHub-hosted validation passed: report run `24942000355`, artifact monitor run `24942008460`, trend run `24942016196`.
  - Post-review GitHub-hosted validation passed after PR `#71` merged on `main` commit `0a7a230`: report run `24942351831` generated `Completo` / `4/4 signals`, monitor run `24942357291` returned `PASS`, and trend run `24942362269` reported latest coverage `4/4 signals`.
  - Report evidence: `docs/reports/github-evidence-stats-scope-fix-2026-04-25.md`.

## Current Operating State

- Consolidating governance telemetry in dashboards and executive reporting.
  - GitHub evidence now has an executive coverage summary in the admin dashboard, local dashboard trend snapshots, exported audit JSON package, standalone Markdown report generator, optional GitHub Actions artifact workflow, artifact freshness monitor, and multi-run artifact trend report.
  - Operational adoption baseline completed on 2026-04-25: manual report, artifact monitor, and trend workflows passed; local monitor/trend scripts passed; evidence captured in `docs/reports/github-evidence-operational-adoption-2026-04-25.md`.
  - No implementation gap remains for the GitHub evidence operating path; recurring work is weekly operation through `docs/runbooks/github-evidence-operations.md`.
  - `KAN-7` stats visibility gap is closed: `supabase_schema_v22.sql` was applied in production and report/trend artifacts no longer show `0/4`.
  - Review signal validation procedure is documented in `docs/runbooks/github-evidence-operations.md`.
  - `pull_request_review` evidence was validated through PR `#71`; `/stats.github_events.by_type.pull_request_review` reached `1`.
  - Live report `docs/reports/github-evidence-executive-report-prod-review-v22-2026-04-25.md` now shows `Completo` / `4/4 signals`.
  - GitHub-hosted report/monitor/trend validation now shows `Completo` / `4/4 signals`: report run `24942351831`, monitor run `24942357291`, trend run `24942362269`.
  - Last GitHub-hosted validation for the export-packaged executive GitHub evidence summary passed on `main` commit `458c048` in CI run `24938795096`.
- Sonar token rotation remains an operational decision. The selected Sonar runtime is local SonarQube, not SonarCloud.
- Jenkins trigger-only URL flow still requires `JENKINS_BUILD_TRIGGER_TOKEN` if unauthenticated/manual trigger URLs are needed.
- Local `GITGOV_API_KEY` is valid for production admin auth. Manual `/integrations/jira` calls must include `x-gitgov-jira-secret` and `org_name`; the previous `401` was not a key rotation/sync issue.
- Website publication recovery completed in `KAN-12`: the prior local-only non-traceable commit was discarded from active branches, the web diff was recommitted as `web(KAN-12): publish marketing updates`, and both PR checks plus post-merge checks passed on `main`.
- Documentation publication governance clarified in `KAN-13`: real repo/service identifiers are allowed only for agent operating memory, historical evidence snapshots, or security-safe validation scope; examples, templates, and reusable public guides must use placeholders.

## Website Feature Claims Alignment

This section is the source of truth for `gitgov-web` `/features`.
If a capability is described on the marketing site, it must be represented here as one of:
- `Implemented`
- `Implemented with scope limits`
- `Not implemented yet`

If a website claim is not reflected here, treat it as unverified and do not publish it as a product capability.

### 1. Workstation Capture

- `Implemented`
- What is real:
  - Desktop captures Git activity locally and emits audit events from workstation commands.
  - Local offline queue persists to `outbox.jsonl`.
  - Retry behavior uses exponential backoff and fail-open connectivity semantics.
- Source files:
  - `gitgov/src-tauri/src/commands/git_commands.rs`
  - `gitgov/src-tauri/src/commands/branch_commands.rs`
  - `gitgov/src-tauri/src/outbox/queue.rs`
  - `gitgov/src-tauri/src/audit/db.rs`
- Safe website wording:
  - workstation capture
  - local evidence logging
  - offline queue / retry
  - append-only evidence flow
- Avoid overstating:
  - do not imply code content inspection
  - do not imply every workstation action is blocked; enforcement is specific to configured rules and command flows

### 2. Governance Engine

- `Implemented with scope limits`
- What is real:
  - Policy model exposes `Off / Warn / Block`.
  - Desktop push flow performs governance pre-check against Control Plane.
  - Branch naming / protected-branch rules are enforced in desktop command flows.
  - Server-side policy evaluation includes branches, commits, pull requests, traceability, and quality gates.
  - Governed quality-gate exceptions are implemented.
- Source files:
  - `gitgov/src-tauri/src/models/branch_rule.rs`
  - `gitgov/src-tauri/src/commands/git_commands.rs`
  - `gitgov/src-tauri/src/commands/branch_commands.rs`
  - `gitgov/gitgov-server/src/handlers/client_ingest_dashboard.rs`
  - `gitgov/gitgov-server/src/handlers/policy_admin.rs`
- Safe website wording:
  - policy-aware workflows
  - push governance pre-check
  - configurable enforcement modes
  - governed exceptions for quality gates
- Avoid overstating:
  - do not say GitGov blocks "all non-compliant code" generically
  - current strongest blocking surface is around push / branch / policy-check flows, not arbitrary editing activity

### 3. Integrations and Evidence Correlation

- `Implemented with scope limits`
- What is real:
  - Jenkins pipeline ingestion exists.
  - Commit-to-pipeline correlation exists.
  - Jira ingestion, correlation, ticket coverage, and ticket detail endpoints exist.
  - Native Jira webhooks can use `POST /webhooks/jira?org_name=<org>` with `X-Hub-Signature` HMAC validation against `JIRA_WEBHOOK_SECRET`.
  - GitHub webhook ingestion exists for `push`, `create`, all `pull_request` actions, all `pull_request_review` actions, `pull_request_review_comment`, PR-linked `issue_comment`, `check_run`, `check_suite`, and `status` events.
  - Merged PR records can enrich approvers through GitHub reviews API when `GITHUB_PERSONAL_ACCESS_TOKEN` is configured.
  - PR lifecycle, review activity, PR comment activity, and CI status-check activity are stored as first-class evidence in `github_events` (`event_type=pull_request|pull_request_review|pull_request_review_comment|issue_comment|check_run|check_suite|status`) with contextual metadata.
  - PR comment bodies/titles that contain ticket IDs can create commit-ticket correlations against the PR/comment SHA, improving traceability evidence without synthetic data.
  - Merged PR titles that contain ticket IDs can create commit-ticket correlations for the GitHub merge commit SHA, so ticket coverage can apply to `main` merge commits when PR titles include `KAN-*` or equivalent ticket IDs.
  - `POST /integrations/jira/correlate` includes a PR-title backfill pass for recent merged PRs, allowing existing `main` merge commits to be correlated when PR titles contain ticket IDs.
  - GitHub repository webhook delivery is configured for PR, review, comment, status, and push events against the Render backend.
  - Duplicate GitHub `pull_request` deliveries for merged PRs now continue through PR merge materialization and title-ticket correlation, allowing webhook redelivery to repair missing `pull_request_merges` evidence.
  - GitHub organization upsert now resolves existing org rows by `login` before inserting by `github_id`, preventing webhook ingestion failures when an org was previously created without a GitHub ID.
  - PR-title ticket correlations now use the existing `pr_title` correlation source, matching the production `commit_ticket_correlations` constraint.
- Source files:
  - `gitgov/gitgov-server/src/handlers/integrations.rs`
  - `gitgov/gitgov-server/src/db.rs`
  - `gitgov/gitgov-server/src/handlers/github_webhook.rs`
  - `gitgov/src-tauri/src/control_plane/server.rs`
- Safe website wording:
  - Jenkins correlation
  - Jira ticket coverage
  - pull request lifecycle evidence
  - pull request review evidence
  - pull request discussion/comment evidence when comments are linked to PRs
  - GitHub status-check evidence (check runs/suites + commit status)
  - GitHub webhook context
- Website consequence:
  - `/features` can claim PR lifecycle + reviews + PR-linked comments + status-check evidence ingestion.
  - Keep wording scoped: comment evidence correlates tickets only when the comment/title includes a ticket ID and a PR/comment SHA is available.

### 4. Risk, Readiness, and Reporting

- `Implemented with scope limits`
- What is real:
  - Control Plane dashboard includes pipeline health, ticket coverage, risk outcomes, recent commits, policy editor, export panel, and chat panel.
  - Dashboard reporting surfaces GitHub PR lifecycle, review, PR comment, and status-check evidence counts.
  - Dashboard reporting includes operator-captured local GitHub evidence trend snapshots for coverage delta/history.
  - Ticket coverage UI explains that commit-ticket coverage can come from commits, branches, PR titles, and PR comments when ticket IDs are present.
  - Release readiness scoring exists.
  - Tier-aware scoring and SLA profiles exist.
  - Export flow exists with content hash generation and export history.
  - Dashboard JSON exports include an executive GitHub evidence summary snapshot alongside raw audit records.
  - Standalone Markdown report generation exists for GitHub executive evidence coverage.
  - GitHub Actions artifact monitoring and trend reporting exist for executive GitHub evidence reports.
  - GitHub evidence operational cadence is documented in `docs/runbooks/github-evidence-operations.md`.
  - Post-merge validation for the runbook rollout passed on `main` commit `7577f90`: CI `24940874607`, Quality Gate Policy Matrix `24940874602`, Release Readiness Gate `24940874616`, Secret Scan `24940874599`, SonarQube Governance `24940874600`, Public Naming Guard `24940874603`, Governance Correlation Smoke `24940874611`, and Desktop Updater Readiness `24940874597`.
  - Risk outcomes widget is operational.
  - Risk outcomes widget surfaces informational `MTTR pipeline` and `Time-to-Evidence` from Jenkins commit-pipeline correlations.
  - `Time-to-Evidence` is calculated as commit timestamp to correlated pipeline ingestion timestamp, with duplicate pipeline evidence ignored.
  - `MTTR pipeline` is calculated as recoverable non-green pipeline event to the next successful run for the same job.
  - These operational metrics render `N/A` when the evidence sample is insufficient.
- Source files:
  - `gitgov/src/components/control_plane/ServerDashboard.tsx`
  - `gitgov/src/components/control_plane/PipelineHealthWidget.tsx`
  - `gitgov/src/components/control_plane/EventBreakdownGrid.tsx`
  - `gitgov/src/components/control_plane/GitHubEvidenceTrendWidget.tsx`
  - `gitgov/src/components/control_plane/TicketCoverageWidget.tsx`
  - `gitgov/src/components/control_plane/RiskOutcomesWidget.tsx`
  - `gitgov/src/components/control_plane/dashboard-helpers.ts`
  - `gitgov/src/components/control_plane/risk-scoring.ts`
  - `gitgov/src/components/control_plane/ExportPanel.tsx`
  - `gitgov/src/test/components/dashboard-helpers.test.ts`
  - `gitgov/gitgov-server/src/handlers/violations_policy_export.rs`
  - `docs/runbooks/github-evidence-operations.md`
  - `docs/reports/operational-mttr-time-to-evidence-2026-04-25.md`
  - `scripts/control-plane/generate_github_evidence_report.ps1`
  - `scripts/control-plane/validate_github_evidence_report_artifact.ps1`
  - `scripts/control-plane/generate_github_evidence_trend_report.ps1`
- Safe website wording:
  - release readiness scoring
  - tier-aware governance visibility
  - exportable audit evidence
  - centralized reporting
- Avoid overstating:
  - do not use invented sample metrics as product facts
  - if the website shows numeric examples, label them clearly as illustrative or remove them
  - `MTTR pipeline` and `Time-to-Evidence` are sample-based operational metrics, not SLO-backed product guarantees
  - do not include these metrics in composite risk/readiness scoring until tier-specific SLO thresholds are calibrated

### 5. Website Gating Rule

Before adding or keeping any `/features` claim:
1. Confirm the implementation exists in code.
2. Confirm it is listed in this section.
3. If scope-limited, write the website copy to match the real scope.
4. If still missing, move it to roadmap/internal planning, not public marketing.

## Next Technical Steps

1. Keep SonarQube local as the Sonar source of truth.
   - SonarCloud onboarding is not applicable for the current personal GitHub account.
   - GitHub-hosted Sonar scan is optional and should skip while `SONAR_HOST_URL=http://localhost:9000`; hosted runners cannot reach the workstation.
   - Jenkins/local validation is the supported Sonar path for this environment.
   - Last operational validation: local Sonar token valid, project `yohandry10_git-gov` quality gate `OK`, Jenkins job `gitgov-demo-pipeline` build `#30` `SUCCESS`, GitGov Render has Sonar/Jenkins evidence for `main`.
2. Keep weekly tier/SLO calibration active and review drift in the generated artifacts.
   - Local multi-tier baseline completed (critical/standard/internal).
   - Production 720h calibration completed on 2026-04-25 with all tier profiles and domain SLOs passing after org-scoped targets were aligned.
   - Repo/branch-scoped calibration is implemented for `calibrate_risk_tier_baseline.ps1`, `validate_domain_slo_targets.ps1`, `risk-tier-baseline-calibration.yml`, and `domain-slo-validation.yml`.
   - Static target-scope validation is enforced by `validate_domain_slo_target_config.ps1` in CI and the domain SLO workflow.
   - Last post-merge live readiness validation for `yohandry10/Git-Gov` on `main`: Release Readiness Gate run `24927603352` passed for commit `f0a3470`.
   - `SQ-07` implementation gap is closed for repo/branch scoping; remaining product gap is improving traceability evidence so readiness can pass without lowering SLO targets.
   - Weekly automation is active (`risk-tier-baseline-calibration.yml` + `enterprise-readiness-bundle.yml` + `domain-slo-validation.yml`).
   - `ops/slo/domain-slo-targets.json` is now the lock file and includes repo/branch scope for the current GitGov repo.
3. Keep GitHub evidence operation on its weekly cadence:
   - PR discussion/comment evidence (`pull_request_review_comment`, PR-linked `issue_comment`) is now ingested and can create ticket correlations from comment/title ticket IDs.
   - Merged PR title evidence now also correlates the merge commit SHA, closing the gap where `main` merge commits were counted as commits without tickets even when the PR title contained a ticket ID.
   - Batch Jira correlation now scans recent merged PR titles as a backfill path, so operators can improve historical coverage without synthetic commit events.
   - Dashboard/reporting now shows PR comment evidence as a distinct GitHub evidence signal and labels coverage scope explicitly.
   - Public `/features` wording is aligned to the real scope: comments improve ticket traceability only when they are PR-linked and contain ticket IDs.
   - Extraction contract tests now protect `check_run`, `check_suite`, `status`, and `pull_request_review_comment` evidence fields before storage.
   - Last GitHub-hosted validation for the extraction contract passed on `main` commit `946fac3` in CI run `24927816238`.
   - GitHub webhook delivery, PR merge materialization, and PR-title correlations are now working in production for `KAN-4`.
   - Ticket coverage/readiness semantics now include `pull_request_merges` in the commit universe.
   - Production validation passed after Render deploy: readiness is currently above target (`77/100` vs `75`) for `yohandry10/Git-Gov` on `main`.
   - Traceability guardrail is active in `Security Guard`; remaining work is operational data quality, not platform plumbing.
   - Latest production validation after the guardrail raised readiness to `79/100`; continue monitoring coverage as new PRs land.
   - GitHub evidence dashboard/report/artifact/trend operation now has an executable runbook: `docs/runbooks/github-evidence-operations.md`.
   - GitHub evidence operational adoption baseline completed on 2026-04-25; `KAN-7` closed the report artifact visibility issue from `0/4` to `4/4` by applying `supabase_schema_v22.sql` and validating a real `pull_request_review` event.
   - Remaining work here is operational monitoring, not new ingestion plumbing.
4. Use the full manual Jira ingest header contract for future GitGov admin operations.
   - Render backend is healthy and webhooks are active.
   - Local ignored `GITGOV_API_KEY` authenticates against production.
   - Manual `/integrations/jira` calls must include both Bearer admin auth and `x-gitgov-jira-secret` when `JIRA_WEBHOOK_SECRET` is configured, plus `org_name` for global admin scope.
5. Decide whether OpenAPI completeness is worth implementing.
   - Current `/api-docs` claim is intentionally partial and safe.
   - Full path annotation is only needed if Swagger becomes a generated SDK or contract-testing source.
6. Keep the website publication flow on the same traceability standard as backend/docs work.
   - `KAN-12` proved the repo policy works: recreate non-traceable local changes on a Jira branch instead of pushing ad-hoc commits on `main`.
   - Treat transient workflow failures like the `actionlint` download issue as rerun candidates only after confirming the code-path checks are already green.
7. Apply the `KAN-13` publication governance rule to new docs.
   - Use placeholders for examples and reusable setup instructions.
   - Keep real repo/service identifiers only in agent memory and evidence snapshots where validation scope matters.
   - Continue relying on `.gitignore`, `publication_guard.ps1`, and `Security Guard` to block restricted forensic/strategy docs.
8. Keep operational validation snapshots current when services are restarted.
   - `KAN-14` refreshed the current state on 2026-04-28.
   - Docker Desktop was started, Compose profiles `sonar` and `jenkins` came online, Render production health passed, and release readiness was `91/100`.
9. Keep OpenAPI as a guarded schema explorer unless product requirements change.
   - `KAN-15` protects the disclaimer that `/api-docs` is intentionally partial.
   - Full `#[utoipa::path]` rollout should remain a deliberate product decision tied to SDK generation or Swagger contract tests.
10. Use the provider access validator before external-service work.
   - `KAN-16` added `scripts/control-plane/validate_provider_access.ps1`.
   - Run `.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness` to validate GitGov, local backend, Sonar, Jenkins, Jira, and readiness without printing secrets.

## Operating Memory Rule

After each major change that affects access, external services, deployment, CI, webhooks, evidence ingestion, validation status, or next-step blockers:

1. Update `AGENTS.md` with the operational fact needed by the next agent.
2. Update this implementation status file or add a dated report under `docs/reports/`.
3. Do not include secrets, token values, private API keys, or raw provider credentials.
4. Prefer concrete IDs, URLs, PR numbers, run IDs, and validation results when they are non-sensitive.

## Sonar Runtime Configuration

Selected runtime:

- Local SonarQube (`http://localhost:9000` for local API access).
- Jenkins/local pipelines are the supported route for Sonar telemetry in this account.
- GitHub-hosted Sonar workflow is intentionally non-blocking and skips unless explicitly configured with a reachable SonarQube endpoint.
- Latest validated state on 2026-04-28: SonarQube system `UP`, project quality gate `OK`; Jenkins `gitgov-demo-pipeline` build `#30` `SUCCESS`; Render-backed readiness for `main` `91/100` with signal coverage `3/3`.
- Provider access validator: `scripts/control-plane/validate_provider_access.ps1`. Latest KAN-16 run with `-IncludeReleaseReadiness` returned all checks `ok`, readiness `91/100`, pipeline success `98.7%`, Jira coverage `67.11%`, and Sonar pass `98.7%`.

Required local variables:

- `SONAR_HOST_URL=http://localhost:9000`
- `SONAR_TOKEN`
- `SONAR_PROJECT_KEY=yohandry10_git-gov`

Required GitHub Actions telemetry variables:

- Secret: `GITGOV_API_KEY`
- Variable: `GITGOV_URL=https://gitgov-api.onrender.com`
- Variable: `SONAR_HOST_URL=http://localhost:9000`
- Variable: `SONAR_PROJECT_KEY=yohandry10_git-gov`
- Secret `SONAR_TOKEN` is not required for GitHub-hosted runners while SonarQube remains local; the non-blocking workflow skips that scan by design.
