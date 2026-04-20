# GitGov Implementation Status

Updated: 2026-04-20

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
  - Produces JSON artifact with score, signal coverage, and fail reasons per run.
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
  - Added explicit `pnpm/action-setup@v4` bootstrap before `actions/setup-node@v4` cache resolution (prevents `pnpm` missing executable failures on hosted runners).
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
  - Latest local evidence report:
    - `docs/reports/quality-gate-policy-matrix-local-2026-04-20.md`
    - `docs/reports/quality-gate-policy-matrix-auto-local-2026-04-20.md`
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
- Public infra preflight automation added:
  - `scripts/deploy/validate_public_infra.ps1` validates domain DNS, TLS certificate, health endpoint, authenticated stats, and webhook/integration route reachability.
  - Local dry-run evidence generated at `docs/reports/public-infra-validation-local-2026-04-20.md` (expected `WARN` on non-HTTPS localhost).
- Desktop updater readiness automation added:
  - `scripts/deploy/validate_desktop_updater_readiness.ps1` validates `plugins.updater` config, endpoint syntax, and live `latest.json` reachability/manifest shape.
  - Local evidence generated at `docs/reports/desktop-updater-readiness-local-2026-04-20.md` (current warning: updater endpoint returns `404` for `latest.json` and requires publish step).
- Legacy migration hardening added:
  - `Security Guard` in `.github/workflows/secret-scan.yml` blocks forbidden legacy-repo markers in tracked files.
- CI lint stability hardening:
  - Refactored `gitgov-server` DB insert APIs to typed input structs to satisfy `clippy -D warnings` (removed `too_many_arguments` failures).
  - Local validation completed: `cargo clippy -- -D warnings` and `cargo test` (150 passed).

## In Progress

- SonarCloud rollout for GitHub-hosted CI in environments without org constraints.
- Consolidating governance telemetry in dashboards and executive reporting.
- GitHub Actions CI config visibility is currently blocked by PAT scope (`secrets=read`, `actions_variables=read`, `administration=read` missing on current token preflight).

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
  - GitHub webhook ingestion exists for `push`, `create`, and merged `pull_request` events.
  - Merged PR records can enrich approvers through GitHub reviews API when `GITHUB_PERSONAL_ACCESS_TOKEN` is configured.
- Source files:
  - `gitgov/gitgov-server/src/handlers/integrations.rs`
  - `gitgov/gitgov-server/src/db.rs`
  - `gitgov/gitgov-server/src/handlers/github_webhook.rs`
  - `gitgov/src-tauri/src/control_plane/server.rs`
- Safe website wording:
  - Jenkins correlation
  - Jira ticket coverage
  - merged pull request evidence
  - GitHub webhook context
- Not implemented yet:
  - GitHub review-event ingestion as first-class stored evidence
  - GitHub status checks / check runs / check suites ingestion as first-class stored evidence
  - full PR lifecycle capture beyond merged PR storage
- Website consequence:
  - `/features` must not claim broad GitHub reviews/status-check capture until the above is implemented and moved to `Implemented` here

### 4. Risk, Readiness, and Reporting

- `Implemented with scope limits`
- What is real:
  - Control Plane dashboard includes pipeline health, ticket coverage, risk outcomes, recent commits, policy editor, export panel, and chat panel.
  - Release readiness scoring exists.
  - Tier-aware scoring and SLA profiles exist.
  - Export flow exists with content hash generation and export history.
  - Risk outcomes widget is operational.
- Source files:
  - `gitgov/src/components/control_plane/ServerDashboard.tsx`
  - `gitgov/src/components/control_plane/PipelineHealthWidget.tsx`
  - `gitgov/src/components/control_plane/RiskOutcomesWidget.tsx`
  - `gitgov/src/components/control_plane/risk-scoring.ts`
  - `gitgov/src/components/control_plane/ExportPanel.tsx`
  - `gitgov/gitgov-server/src/handlers/violations_policy_export.rs`
- Safe website wording:
  - release readiness scoring
  - tier-aware governance visibility
  - exportable audit evidence
  - centralized reporting
- Avoid overstating:
  - do not use invented sample metrics as product facts
  - if the website shows numeric examples, label them clearly as illustrative or remove them
  - `MTTR` and `Time-to-Evidence` are not complete yet and should not be presented as finished capabilities

### 5. Website Gating Rule

Before adding or keeping any `/features` claim:
1. Confirm the implementation exists in code.
2. Confirm it is listed in this section.
3. If scope-limited, write the website copy to match the real scope.
4. If still missing, move it to roadmap/internal planning, not public marketing.

## Next Technical Steps

1. Configure repository-level CI secrets/variables per rollout mode (Sonar scan vs telemetry publish).
   - Current live status in GitHub-hosted CI: **UNKNOWN** with current PAT (limited token cannot read Actions secrets/variables).
   - Token preflight evidence (`scripts/github/check_token_permissions.ps1`): `403` on Actions secrets, Actions variables, and branch protection.
   - Pending for Sonar scan mode: `SONAR_TOKEN` (+ strict visibility check with PAT that has `secrets=read` / `actions_variables=read`).
   - Pending for telemetry mode (`-RequireGitGovTelemetry`): `GITGOV_API_KEY` + `GITGOV_URL`.
2. Validate the same `quality_gates=warn/block` matrix on GitHub-hosted CI once SonarCloud org onboarding is available (local/Jenkins validation already complete; runbook: `docs/QUALITY_GATE_POLICY_VALIDATION.md`).
3. Calibrate tier profiles with production telemetry (weekly) and lock tier-specific SLO baselines per business domain.
   - Local multi-tier baseline completed (critical/standard/internal); current main gap is high `traceability_gap` in all profiles.
   - Pending: run same calibration against production telemetry and lock business-domain SLO targets.
4. Expand GitHub evidence ingestion beyond current scope:
   - store review activity as first-class evidence
   - ingest status checks / check runs / check suites
   - decide whether `/features` should market merged PR evidence only or full PR lifecycle coverage

## Required GitHub Configuration (for Sonar workflow)

Base Sonar scan mode:

- Secret: `SONAR_TOKEN`
- Variable: `SONAR_PROJECT_KEY`

Telemetry publish mode (`-RequireGitGovTelemetry`):

- Secret: `GITGOV_API_KEY`
- Variable: `GITGOV_URL`
- Secret opcional: `GITGOV_JENKINS_SECRET`

Always optional:

- Variable: `SONAR_HOST_URL` (default `https://sonarcloud.io`)
