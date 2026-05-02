# GitGov Current Context Handoff

Updated: 2026-05-02
Ticket: `KAN-59`

Read this file first when resuming work. It is the compact operational handoff for the current GitGov state.

## Exact Current Point

- Local workspace: `C:\Users\PC\Desktop\GitGov`.
- Expected branch before new work: `main`.
- Latest completed KAN-24 implementation baseline: `126167f security(KAN-24): product vulnerability review and hardening (#97)`.
- KAN-24 implementation PR: `#97` - `security(KAN-24): product vulnerability review and production hardening`.
- KAN-24 post-merge context refresh PR: `#98` - `docs(KAN-24): record post-merge validation`.
- Recent prior PR: `#96` - `docs(KAN-23): record evidence packet merge validation`.
- Treat commit/PR fields in this file as validated KAN-24 implementation and validation baselines, not an auto-updating source of truth for later docs-only refresh commits; always run `git status --short --branch` and `git log -1 --oneline main` before new work.
- Worktree expectation before new work: clean and aligned with `origin/main`.
- Implementation-status backlog is closed. Remaining items are operational decisions, optional future enhancements, or evidence hygiene.
- Latest completed follow-up: `KAN-25 - Automate product vulnerability review evidence`.
- Latest completed follow-up: `KAN-26 - Monitor product vulnerability review artifact freshness`.
- Latest completed follow-up: `KAN-27 - Trend product vulnerability review artifacts`.
- Latest completed follow-up: `KAN-28 - Vulnerability trend enforcement gate`.
- Latest completed follow-up: `KAN-29 - Enterprise self-service adoption MVP`.
- Latest completed follow-up: `KAN-30 - Adoption profile dashboard MVP`.
- Latest completed follow-up: `KAN-31 - Adoption profile persistence`.
- Latest completed follow-up: `KAN-32 - Enterprise provider health validation MVP`.
- Latest completed follow-up: `KAN-33 - Workflow template generation from adoption profile`.
- Latest completed follow-up: `KAN-34 - Dashboard workflow template pack download`.
- Latest completed follow-up: `KAN-35 - Reviewed workflow installation from template pack`.
- Latest completed follow-up: `KAN-36 - Direct provider connection validation for enterprise onboarding`.
- Latest completed follow-up: `KAN-37 - Formal enterprise release approval MVP`.
- Latest completed follow-up: `KAN-38 - Vercel AI SDK governance copilot MVP`.
- Latest completed follow-up: `KAN-39 - Governance copilot dashboard UI MVP`.
- Latest completed follow-up: `KAN-40 - Governance copilot AI mode validation`.
- Latest completed follow-up: `KAN-41 - Activate governance copilot AI mode on Vercel`.
- Latest completed follow-up: `KAN-42 - Enforce governance copilot AI mode validation`.
- Latest completed follow-up: `KAN-43 - Dashboard release approval wizard MVP`.
- Latest completed follow-up: `KAN-44 - Document configurable release governance defaults`.
- Latest completed follow-up: `KAN-45 - Add configurable release governance profile policy`.
- Latest completed follow-up: `KAN-46 - Add release governance evaluator`.
- Latest completed follow-up: `KAN-47 - Add optional release governance enforcement gate`.
- Latest completed follow-up: `KAN-48 - Add environment-scoped release governance policy overrides`.
- Latest completed follow-up: `KAN-49 - Monitor release governance gate artifacts`.
- Latest completed follow-up: `KAN-50 - Remote workflow installation PR for customer repositories`.
- Latest completed follow-up: `KAN-51 - Remote workflow installation readiness validation`.
- Latest completed follow-up: `KAN-52 - Enterprise onboarding readiness report`.
- Latest completed follow-up: `KAN-53 - Automate enterprise onboarding readiness evidence`.
- Latest completed follow-up: `KAN-54 - Monitor enterprise onboarding readiness evidence artifacts`.
- Latest completed follow-up: `KAN-55 - Trend enterprise onboarding readiness evidence artifacts`.
- Latest completed follow-up: `KAN-56 - Monitor enterprise onboarding readiness trend deterioration`.
- Latest completed follow-up: `KAN-57 - Generate enterprise onboarding remediation plan`.
- Latest completed follow-up: `KAN-58 - Dashboard onboarding remediation export`.
- Latest completed follow-up: `KAN-59 - Dashboard guided enterprise onboarding checklist`.
- Current follow-up: none selected after `KAN-59`.
- Any future branch, commit, and PR title must include the relevant Jira ticket ID.

## Latest Verified GitHub Checks

Latest post-merge validation for handoff baseline commit `126167f` passed:

- `CI` - run `25156959926`
- `Release Readiness Gate` - run `25156959919`
- `Quality Gate Policy Matrix (Optional)` - run `25156959901`
- `Secret Scan` - run `25156959895`
- `SonarQube Governance (Non-Blocking)` - run `25156959902`
- `Public Naming Guard` - run `25156959899`
- `Governance Correlation Smoke (Optional)` - run `25156959914`
- `Desktop Updater Readiness (Optional)` - run `25156959949`

Latest KAN-25 automation baseline:

- Implementation commit: `7c260fe security(KAN-25): automate vulnerability review evidence`.
- PR: `#100` - `security(KAN-25): automate product vulnerability review evidence`.
- Post-merge checks passed:
  - `CI` - run `25157965635`
  - `Release Readiness Gate` - run `25157965664`
  - `Quality Gate Policy Matrix (Optional)` - run `25157965674`
  - `Secret Scan` - run `25157965657`
  - `SonarQube Governance (Non-Blocking)` - run `25157965627`
  - `Public Naming Guard` - run `25157965648`
  - `Governance Correlation Smoke (Optional)` - run `25157965686`
  - `Desktop Updater Readiness (Optional)` - run `25157965670`
- First manual `Product Vulnerability Review` run passed:
  - Run `25157972836`
  - Mode `DependenciesOnly`
  - Artifact `product-vulnerability-review-25157972836`
  - Artifact status: not expired

Latest KAN-26 artifact monitor baseline:

- Implementation commit: `89a234c security(KAN-26): monitor vulnerability review artifacts`.
- PR: `#102` - `security(KAN-26): monitor product vulnerability review artifacts`.
- Post-merge checks passed:
  - `CI` - run `25158430862`
  - `Release Readiness Gate` - run `25158431062`
  - `Quality Gate Policy Matrix (Optional)` - run `25158430899`
  - `Secret Scan` - run `25158430868`
  - `SonarQube Governance (Non-Blocking)` - run `25158430873`
  - `Public Naming Guard` - run `25158430891`
  - `Governance Correlation Smoke (Optional)` - run `25158430896`
  - `Desktop Updater Readiness (Optional)` - run `25158430919`
- First manual `Product Vulnerability Review Artifact Monitor` run passed:
  - Run `25158436168`
  - Artifact `product-vulnerability-review-artifact-monitor`
  - Artifact ID `6727075935`
  - Artifact status: not expired

Latest KAN-27 trend report baseline:

- Implementation commit: `6fd8de8 security(KAN-27): add product vulnerability review trend reporting`.
- PR: `#104` - `security(KAN-27): add product vulnerability review trend reporting`.
- Post-merge checks passed:
  - `CI` - run `25159025219`
  - `Release Readiness Gate` - run `25159025186`
  - `Quality Gate Policy Matrix (Optional)` - run `25159025384`
  - `Secret Scan` - run `25159025195`
  - `SonarQube Governance (Non-Blocking)` - run `25159025371`
  - `Public Naming Guard` - run `25159025481`
  - `Governance Correlation Smoke (Optional)` - run `25159025229`
  - `Desktop Updater Readiness (Optional)` - run `25159025182`
- First manual `Product Vulnerability Review Trend Report` run passed:
  - Run `25159031614`
  - Artifact `product-vulnerability-review-trend-report`
  - Artifact ID `6727320469`
  - Artifact status: not expired

Latest KAN-28 trend enforcement baseline:

- Implementation commit: `7b36cec security(KAN-28): enforce product vulnerability trend baseline`.
- PR: `#106` - `security(KAN-28): enforce product vulnerability trend baseline`.
- Post-merge checks passed:
  - `CI` - run `25160187848`
  - `Release Readiness Gate` - run `25160187829`
  - `Quality Gate Policy Matrix (Optional)` - run `25160187813`
  - `Secret Scan` - run `25160187847`
  - `SonarQube Governance (Non-Blocking)` - run `25160187844`
  - `Public Naming Guard` - run `25160187839`
  - `Governance Correlation Smoke (Optional)` - run `25160187818`
  - `Desktop Updater Readiness (Optional)` - run `25160187859`
- First manual `Product Vulnerability Review Trend Enforcement` run passed:
  - Run `25160194313`
  - Artifact `product-vulnerability-review-trend-enforcement`
  - Artifact ID `6727810243`
  - Artifact status: not expired

Latest KAN-29 enterprise adoption baseline:

- Implementation commit: `bf8e378 product(KAN-29): add enterprise self-service adoption MVP`.
- PR: `#108` - `product(KAN-29): add enterprise self-service adoption MVP`.
- Post-merge checks passed:
  - `CI` - run `25160842461`
  - `Release Readiness Gate` - run `25160842032`
  - `Quality Gate Policy Matrix (Optional)` - run `25160842064`
  - `Secret Scan` - run `25160842081`
  - `SonarQube Governance (Non-Blocking)` - run `25160842041`
  - `Public Naming Guard` - run `25160842023`
  - `Governance Correlation Smoke (Optional)` - run `25160842049`
  - `Desktop Updater Readiness (Optional)` - run `25160842036`

Latest KAN-30 adoption profile dashboard baseline:

- Implementation commit: `0412574 product(KAN-30): add adoption profile dashboard MVP`.
- PR: `#110` - `product(KAN-30): add adoption profile dashboard MVP`.
- Post-merge checks passed:
  - `CI` - run `25161644820`
  - `Release Readiness Gate` - run `25161644879`
  - `Quality Gate Policy Matrix (Optional)` - run `25161644854`
  - `Secret Scan` - run `25161644841`
  - `SonarQube Governance (Non-Blocking)` - run `25161644861`
  - `Public Naming Guard` - run `25161644857`
  - `Governance Correlation Smoke (Optional)` - run `25161644871`
  - `Desktop Updater Readiness (Optional)` - run `25161644824`

Latest KAN-31 adoption profile persistence baseline:

- Implementation commit: `509e2a2 product(KAN-31): persist adoption profiles`.
- PR: `#112` - `product(KAN-31): persist adoption profiles`.
- Post-merge checks passed:
  - `CI` - run `25186881414`
  - `Release Readiness Gate` - run `25186881375`
  - `Quality Gate Policy Matrix (Optional)` - run `25186881361`
  - `Secret Scan` - run `25186881344`
  - `SonarQube Governance (Non-Blocking)` - run `25186881363`
  - `Public Naming Guard` - run `25186881451`
  - `Governance Correlation Smoke (Optional)` - run `25186881376`
  - `Desktop Updater Readiness (Optional)` - run `25186881345`
- Documentation validation PR: `#113` - `docs(KAN-31): record adoption profile validation`.
- Documentation validation commit: `171d43d docs(KAN-31): record adoption profile validation`.
- Post-merge docs refresh checks passed:
  - `CI` - run `25187583892`
  - `Release Readiness Gate` - run `25187583994`
  - `Quality Gate Policy Matrix (Optional)` - run `25187583967`
  - `Secret Scan` - run `25187583907`
  - `SonarQube Governance (Non-Blocking)` - run `25187583895`
  - `Public Naming Guard` - run `25187584004`
  - `Governance Correlation Smoke (Optional)` - run `25187583992`
  - `Desktop Updater Readiness (Optional)` - run `25187583943`
- Production DB migration `v23` was applied on 2026-04-30 using ignored local `DATABASE_URL` without printing credentials.
- `gitgov/gitgov-server/supabase/checks/v23_postcheck.sql` passed:
  - `enterprise_adoption_profiles.table_exists` - `PASS`
  - `enterprise_adoption_profiles.primary_key` - `PASS`
  - `enterprise_adoption_profiles.updated_at_index` - `PASS`
- Production route validation after migration:
  - `GET /health` returned `200`.
  - Anonymous `GET /enterprise/adoption-profile?org_name=yohandry10` returned `401`.
  - Authenticated `GET /enterprise/adoption-profile?org_name=yohandry10` returned `200` with `found=false`.

Latest KAN-38 AI SDK governance copilot baseline:

- Implementation commit: `9742472 product(KAN-38): add AI SDK governance copilot`.
- PR: `#127` - `product(KAN-38): add AI SDK governance copilot`.
- Jira final comment: `10197`.
- Post-merge checks passed:
  - `CI` - run `25194421718`
  - `Release Readiness Gate` - run `25194421743`
  - `Quality Gate Policy Matrix (Optional)` - run `25194421721`
  - `Secret Scan` - run `25194421747`
  - `SonarQube Governance (Non-Blocking)` - run `25194421756`
  - `Public Naming Guard` - run `25194421752`
  - `Governance Correlation Smoke (Optional)` - run `25194421750`
  - `Desktop Updater Readiness (Optional)` - run `25194421717`
- Vercel production deployment `https://git-ih2bzdqq5-trivia1.vercel.app` reached `Ready`.
- Production smoke passed on `https://www.gitgov.cloud/api/copilot/governance` and `https://git-gov.vercel.app/api/copilot/governance` with `success=true`, `mode=fallback`, `4` citations, `4` sources, and `1` expected warning because AI Gateway/OIDC generation was not active.

KAN-24 local validation before PR creation:

- `.\scripts\security\run_product_vulnerability_review.ps1 -Full -OutputDir docs/reports/product-vulnerability-review-2026-04-30 -CommandTimeoutSeconds 1200`
- Result: `20` pass, `1` expected finding, `0` fail.
- Remaining expected finding: backend `cargo audit` reports `rsa` through inactive `sqlx-mysql`; reachability checks showed no active dependency path in the current backend feature graph.

Production validation after Render deploy `dep-d7phm1m8bjmc73fko1lg`:

- Render deployed commit `126167ff1c4ad9756f2e3f78fcb69f9fcf14f2f1` and reached `live` on 2026-04-30.
- `GET https://gitgov-api.onrender.com/health` returned `status=ok`.
- Anonymous `GET /stats` returned `401`.
- Authenticated `GET /stats` returned `200` without printing token values.

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

KAN-24 product vulnerability review runner:

```powershell
.\scripts\security\run_product_vulnerability_review.ps1 -Full -OutputDir docs/reports/product-vulnerability-review-2026-04-30 -CommandTimeoutSeconds 1200
```

KAN-25 automation workflow:

```text
.github/workflows/product-vulnerability-review.yml
```

Default scheduled mode is `DependenciesOnly`; manual modes are `DependenciesOnly`, `StaticOnly`, `RuntimeSmoke`, and `Full`.

KAN-26 artifact monitor workflow:

```text
.github/workflows/product-vulnerability-review-artifact-monitor.yml
```

It checks latest successful `product-vulnerability-review.yml` runs for artifacts with prefix `product-vulnerability-review-`.

KAN-27 trend report workflow:

```text
.github/workflows/product-vulnerability-review-trend-report.yml
```

It builds Markdown/JSON trend evidence from sanitized `summary.json` files in recent `product-vulnerability-review-*` artifacts.

KAN-28 trend enforcement workflow:

```text
.github/workflows/product-vulnerability-review-trend-enforcement.yml
```

It fails when the latest trend has failures, findings exceed the accepted baseline, findings/failures increase, or the latest successful review run lacks a parseable artifact.

KAN-29 enterprise adoption pack generator:

```powershell
.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-adoption-pack
```

It writes a Markdown/JSON customer adoption pack with providers, modules, policy preset, workflow plan, variable/secret names, and manual setup checklist. It does not read or write secret values.

KAN-33 workflow template generator:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-workflow-templates -Force
```

It writes ignored onboarding output under `out/enterprise-workflow-templates/`: `README.md`, `workflow-template-manifest.json`, and selected `.github/workflows/*.yml` templates. It records variable and secret names only, does not read `.env`, and does not mutate customer repositories.

KAN-35 reviewed workflow installer dry-run:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out\enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out\workflow-install-plan.json
```

Use `-Apply` only after review. Use `-Overwrite` only for reviewed replacements. The installer also supports dashboard JSON packs with `-PackPath`.

KAN-36 provider connection validator:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -ReportOnly -OutputPath out\provider-connections-report-only.json
```

Use strict mode without `-ReportOnly` when every selected provider must be ready. The validator reports sanitized statuses only and does not print secret values.

KAN-40/KAN-42 governance copilot AI mode validator:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 -TicketId KAN-39 -ReleaseId KAN-39 -RequireAiMode -OutputPath out\governance-copilot-ai-mode-validation.json
```

Google Gemini is active in production after KAN-41. Use `-RequireAiMode` for normal production validation. Non-strict validation is only for explicit fallback diagnostics.

KAN-31 adoption profile persistence migration postcheck:

```powershell
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/supabase_schema_v23.sql
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/checks/v23_postcheck.sql
```

Do not print the database URL or credentials.
Production `v23` has already been applied; rerun the postcheck only when revalidating or provisioning a new environment.

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
- `KAN-23`: implemented ticket-scoped Evidence Packets before a Vercel AI SDK copilot. MVP added `GET /evidence/packets/tickets/{ticket_id}`, a Tauri command, dashboard JSON download UI, and docs under `docs/design/evidence-packets-mvp.md`; follow-up PR `#96` recorded production merge validation on `main` commit `a37d489`.
- `KAN-24`: opened Jira issue `KAN-24 - Product vulnerability review and production hardening` and started branch `security/KAN-24-product-vulnerability-review`. Scope covers end-to-end product vulnerability review across code, architecture, runtime, CI/CD, dependencies, and real user surfaces.
- `KAN-25`: opened Jira issue `KAN-25 - Automate product vulnerability review evidence` and started branch `security/KAN-25-product-vulnerability-review-automation`. Scope is operationalizing the KAN-24 runner as a weekly/manual GitHub Actions workflow with sanitized artifacts.
- `KAN-26`: opened Jira issue `KAN-26 - Monitor product vulnerability review artifact freshness` and started branch `security/KAN-26-product-vulnerability-artifact-monitor`. Scope is monitoring the freshness and presence of Product Vulnerability Review artifacts.
- `KAN-27`: opened Jira issue `KAN-27 - Trend product vulnerability review artifacts` and started branch `security/KAN-27-product-vulnerability-review-trend`. Scope is aggregating recent Product Vulnerability Review artifacts into trend evidence so regressions are visible across runs.
- `KAN-28`: opened Jira issue `KAN-28 - Vulnerability trend enforcement gate` and started branch `security/KAN-28-vulnerability-trend-enforcement`. Scope is converting KAN-27 trend evidence into an enforcement workflow and documenting the next two product features: Enterprise Self-Service Adoption and Vercel AI SDK Copilot.
- `KAN-29`: opened Jira issue `KAN-29 - Enterprise self-service adoption MVP` and started branch `product/KAN-29-enterprise-self-service-adoption`. Scope is creating the first reusable adoption pack generator for customer onboarding.
- `KAN-30`: opened Jira issue `KAN-30 - Adoption profile dashboard MVP`, implemented branch `product/KAN-30-adoption-profile-dashboard`, and merged PR `#110` as `0412574`. Scope moved the KAN-29 adoption profile into the admin dashboard with validation and secret-safe JSON export.
- `KAN-31`: opened Jira issue `KAN-31 - Persist adoption profiles for enterprise onboarding`, implemented branch `product/KAN-31-adoption-profile-persistence`, and merged PR `#112` as `509e2a2`. Scope persists the KAN-30 profile per org with admin get/upsert endpoints, backend validation, Supabase migration `v23`, Tauri commands, dashboard save/load, and secret-safe docs. Documentation refresh PR `#113` merged as `171d43d`, and production migration `v23` was applied and validated on 2026-04-30.
- `KAN-32`: opened Jira issue `KAN-32 - Enterprise provider health validation MVP`, implemented branch `product/KAN-32-provider-health-validation`, and merged PR `#115` as `1a16d88`. Scope adds a secret-safe Provider Health section to the Enterprise Adoption dashboard using already-loaded GitGov evidence instead of provider credentials.
- `KAN-33`: opened Jira issue `KAN-33 - Generate customer workflow templates from adoption profile`, implemented branch `product/KAN-33-workflow-template-generation`, and merged PR `#117` as `62b67e5`. Scope converts the KAN-29/KAN-31 adoption profile into reviewed workflow template packs, manifest, README, variables, secret names, and manual install checklist without mutating customer repositories.
- `KAN-34`: opened Jira issue `KAN-34 - Dashboard workflow template pack download`, implemented branch `product/KAN-34-dashboard-workflow-template-pack`, and merged PR `#119` as `31b109d`. Scope exposes workflow template pack generation in the Enterprise Adoption dashboard using the current/persisted profile, while keeping automatic repository mutation out of scope.
- `KAN-35`: opened Jira issue `KAN-35 - Reviewed workflow installation from template pack`, implemented branch `product/KAN-35-reviewed-workflow-installation`, and merged PR `#121` as `c60c486`. Scope installs CLI or dashboard workflow template packs into a local customer repository checkout only after dry-run review and explicit `-Apply`; remote GitHub mutation remains out of scope.
- `KAN-36`: opened Jira issue `KAN-36 - Direct provider connection validation for enterprise onboarding`, implemented branch `product/KAN-36-provider-connection-validation`, and merged PR `#123` as `8c075a4`. Scope validates explicitly provided provider credentials/reachability for GitHub, Jira, Jenkins, SonarQube, Render, and Vercel without printing secrets or mutating provider state.
- `KAN-37`: opened Jira issue `KAN-37 - Formal enterprise release approval MVP`, implemented branch `product/KAN-37-formal-release-approval`, and merged PR `#125` as `d7ae92e`. Scope is append-only formal release approvals with admin-only org scope, evidence packet hash binding, risk acceptance expiration, audit logging, Supabase migration `v24`, and backend validation tests. Production migration `v24` was applied and validated on 2026-04-30; Render deploy `dep-d7ptsvhoagis738cj88g` reached `live`.
- `KAN-38`: implemented `KAN-38 - Vercel AI SDK governance copilot MVP` on branch `product/KAN-38-ai-sdk-copilot`; PR `#127` merged as `9742472`. Scope is the first server-side Next.js AI SDK copilot route over bounded GitGov evidence with citations and fallback when AI Gateway/OIDC is unavailable.
- `KAN-39`: implemented `KAN-39 - Governance copilot dashboard UI MVP` on branch `product/KAN-39-governance-copilot-dashboard`; PR `#129` merged as `eda2f13`. Scope is the first admin dashboard UI for the KAN-38 copilot route, using a secret-safe Tauri proxy command and displaying cited answers, source statuses, and warnings.

## Current Product Roadmap

- Current major product feature: Enterprise Self-Service Adoption MVP (`KAN-29`/`KAN-30`/`KAN-31`/`KAN-32`/`KAN-33`/`KAN-34`/`KAN-35`/`KAN-36`/`KAN-37`).
  - KAN-29 packages the proven GitGov operating model into a reusable adoption pack generator.
  - KAN-30 adds the first dashboard profile builder with provider/module toggles, policy presets, validation, workflow/policy preview, and secret-safe JSON export.
  - KAN-31 persists adoption profiles per org with admin save/load.
  - KAN-32 adds evidence-based provider health validation in the dashboard.
  - KAN-33 generates reviewed workflow template packs from the adoption profile.
  - KAN-34 adds dashboard download for workflow template packs.
  - KAN-35 adds reviewed local workflow installation from CLI or dashboard workflow packs.
  - KAN-36 adds direct provider credential/reachability checks.
  - KAN-37 adds formal release approval persistence with evidence packet hash and risk expiration.
- Current major AI feature: Vercel AI SDK Copilot.
  - Explain readiness, findings, tickets, pipelines, evidence packets, accepted risks, and blockers in plain language with cited GitGov evidence.
  - KAN-38 implements the first server-side route with `POST /api/copilot/governance`.
  - KAN-39 adds the first admin dashboard surface for that route.
- Completed hardening gate before those larger features: KAN-28 vulnerability trend enforcement.
- Optional later hygiene: remove the residual `rsa` / inactive `sqlx-mysql` dependency finding when upstream resolution or safe dependency cleanup makes that practical.

## Current KAN-29 Implementation Notes

- Script: `scripts/control-plane/generate_enterprise_adoption_pack.ps1`.
- Design: `docs/design/enterprise-self-service-adoption-mvp.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Example profile: `docs/examples/enterprise-adoption-profile.example.json`.
- Report: `docs/reports/enterprise-self-service-adoption-mvp-2026-04-30.md`.
- The generator supports policy presets `audit-only`, `moderate`, and `strict`.
- Local validation generated a pack for `ExampleCo` / `example-org/example-repo` with preset `moderate`, `13` workflow recommendations, `3` variable names, `2` secret names, `6` policy rules, and `5` manual setup steps.
- PR `#108` merged this MVP on `main` as `bf8e378`.

## Current KAN-30 Implementation Notes

- Component: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Helpers: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Design: `docs/design/adoption-profile-dashboard-mvp.md`.
- Report: `docs/reports/adoption-profile-dashboard-mvp-2026-04-30.md`.
- The dashboard builder uses the same profile/pack shape as the KAN-29 generator: customer, repository, default branch, Jira key, policy preset, providers, modules, workflow plan, variable names, secret names, policy rules, manual steps, and open product gaps.
- The JSON export contains secret names only. It does not read local env files or provider tokens.
- Local validation passed with `npm test -- --run src/test/components/dashboard-helpers.test.ts`, `npm run typecheck`, and `npm run lint`.
- Full local preflight also passed with `npm test -- --run`, `npm run build`, `git diff --check`, `.\scripts\security\publication_guard.ps1`, and a browser smoke at `http://127.0.0.1:5174/` with `0` console errors.
- PR `#110` merged this MVP on `main` as `0412574`.

## Current KAN-31 Implementation Notes

- Backend routes: `GET /enterprise/adoption-profile` and `PUT /enterprise/adoption-profile`.
- Backend files: `gitgov/gitgov-server/src/handlers/adoption_profiles.rs`, `models.rs`, `db.rs`, and `main.rs`.
- Migration: `gitgov/gitgov-server/supabase/supabase_schema_v23.sql`.
- Postcheck: `gitgov/gitgov-server/supabase/checks/v23_postcheck.sql`.
- Desktop bridge: `gitgov/src-tauri/src/control_plane/server.rs`, `commands/server_commands.rs`, and command registration in `src-tauri/src/lib.rs`.
- Dashboard: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx` now loads/saves persisted profiles while preserving JSON export.
- Store: `gitgov/src/store/useControlPlaneStore.ts` tracks profile load/save state and errors.
- Design: `docs/design/adoption-profile-persistence-mvp.md`.
- Report: `docs/reports/adoption-profile-persistence-2026-04-30.md`.
- Saved profiles contain configuration intent only: no API keys, tokens, webhook secrets, generated secret values, or `.env` values.
- Production database migration `v23` was applied on 2026-04-30; use `v23_postcheck.sql` for revalidation or new environment provisioning.
- Local validation passed with `cargo test enterprise_adoption_profile_validation`, backend `cargo check`, backend `cargo clippy -- -D warnings`, Tauri `cargo check`, Tauri `cargo clippy -- -D warnings`, `npm run typecheck`, `npm test -- --run src/test/components/dashboard-helpers.test.ts`, full `npm test -- --run`, `npm run lint`, `npm run build`, `git diff --check`, and `.\scripts\security\publication_guard.ps1`.
- PR `#112` merged this MVP on `main` as `509e2a2`; post-merge `CI` run `25186881414` and `Release Readiness Gate` run `25186881375` passed.
- PR `#113` recorded KAN-31 validation on `main` as `171d43d`; post-merge `CI` run `25187583892` and `Release Readiness Gate` run `25187583994` passed.
- Production validation after `v23` migration: `/health` `200`, anonymous adoption-profile GET `401`, authenticated adoption-profile GET `200` with `found=false`.

## Current KAN-32 Implementation Notes

- Implementation commit: `1a16d88 product(KAN-32): add provider health validation`.
- PR: `#115` - `product(KAN-32): add provider health validation`.
- Component: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Helper: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Tests: `gitgov/src/test/components/dashboard-helpers.test.ts`.
- Design: `docs/design/provider-health-validation-mvp.md`.
- Report: `docs/reports/provider-health-validation-2026-04-30.md`.
- Provider Health status values:
  - `ready`: selected provider has required adoption intent and observable GitGov evidence.
  - `needs-evidence`: selected provider is configured in the profile but telemetry has not arrived yet.
  - `needs-config`: selected provider lacks required profile/module/config intent.
- Evidence inputs are already-loaded dashboard data: GitHub event totals, Jira ticket coverage, Jenkins pipeline health, Sonar/quality evidence inferred from Jenkins correlations, and active repository count.
- KAN-32 does not read `.env`, provider tokens, webhook secrets, or raw secret values, and it does not call external provider APIs directly.
- Local validation passed with `npm test -- --run src/test/components/dashboard-helpers.test.ts`, `npm run typecheck`, `npm run lint`, full `npm test -- --run`, and `npm run build`.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Workflow Lint`.
  - `Validate quality_gates warn/block matrix`.
  - `Sonar Scan + Quality Gate`.
  - `Vercel`.
- Post-merge `main` checks passed:
  - `CI` - run `25188414404`
  - `Release Readiness Gate` - run `25188414418`
  - `Quality Gate Policy Matrix (Optional)` - run `25188414443`
  - `Secret Scan` - run `25188414428`
  - `SonarQube Governance (Non-Blocking)` - run `25188414417`
  - `Public Naming Guard` - run `25188414424`
  - `Governance Correlation Smoke (Optional)` - run `25188414421`
  - `Desktop Updater Readiness (Optional)` - run `25188414432`

## Current KAN-33 Implementation Notes

- Implementation commit: `62b67e5 product(KAN-33): generate enterprise workflow templates`.
- PR: `#117` - `product(KAN-33): generate enterprise workflow templates`.
- Script: `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- Design: `docs/design/workflow-template-generation-mvp.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Report: `docs/reports/workflow-template-generation-2026-04-30.md`.
- Example command: `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-workflow-templates -Force`.
- ExampleCo validation generated `13` workflow templates plus `README.md` and `workflow-template-manifest.json`.
- Generated outputs are ignored under `out/`.
- Safety: the generator records variable names and secret names only, does not read `.env`, does not print provider tokens, and does not mutate customer repositories automatically.
- Local validation passed: ExampleCo generation, generated YAML parse with PyYAML, no unresolved template tokens, `git diff --check`, targeted secret-pattern scan over new files, and `.\scripts\security\publication_guard.ps1`.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Workflow Lint`.
  - `Validate quality_gates warn/block matrix`.
  - `Sonar Scan + Quality Gate`.
  - `Vercel`.
- Post-merge `main` checks passed:
  - `CI` - run `25189490341`
  - `Release Readiness Gate` - run `25189490316`
  - `Quality Gate Policy Matrix (Optional)` - run `25189490347`
  - `Secret Scan` - run `25189490317`
  - `SonarQube Governance (Non-Blocking)` - run `25189490329`
  - `Public Naming Guard` - run `25189490343`
  - `Governance Correlation Smoke (Optional)` - run `25189490321`
  - `Desktop Updater Readiness (Optional)` - run `25189490319`

## Current KAN-34 Implementation Notes

- Implementation commit: `31b109d product(KAN-34): add dashboard workflow template pack`.
- PR: `#119` - `product(KAN-34): add dashboard workflow template pack`.
- Component: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Helpers: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Tests: `gitgov/src/test/components/dashboard-helpers.test.ts`.
- Design: `docs/design/dashboard-workflow-template-pack-mvp.md`.
- Report: `docs/reports/dashboard-workflow-template-pack-2026-04-30.md`.
- Dashboard adds a `Workflows` download action next to existing profile save and adoption-pack JSON download.
- Helper output shape: one JSON file with `manifest`, `files`, and `readme`.
- Safety: no `.env` reads, no provider token reads, no secret value display, and no customer repository mutation.
- Local validation passed with `npm test -- --run src/test/components/dashboard-helpers.test.ts` (`13` tests), `npm run typecheck`, `npm run lint`, full `npm test -- --run` (`25` files, `276` tests), `npm run build` with the existing Vite large chunk warning, `git diff --check`, targeted secret-pattern scan, and `.\scripts\security\publication_guard.ps1`.
- Vercel AI SDK Copilot later starts in `KAN-38` after the onboarding/evidence surfaces are ready enough.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Workflow Lint`.
  - `Validate quality_gates warn/block matrix`.
  - `Sonar Scan + Quality Gate`.
  - `Vercel`.
- Post-merge `main` checks passed:
  - `CI` - run `25190963652`
  - `Release Readiness Gate` - run `25190963636`
  - `Quality Gate Policy Matrix (Optional)` - run `25190963623`
  - `Secret Scan` - run `25190963646`
  - `SonarQube Governance (Non-Blocking)` - run `25190963649`
  - `Public Naming Guard` - run `25190963657`
  - `Governance Correlation Smoke (Optional)` - run `25190963633`
  - `Desktop Updater Readiness (Optional)` - run `25190963664`

## Current KAN-35 Implementation Notes

- Implementation commit: `c60c486 product(KAN-35): add reviewed workflow installation`.
- PR: `#121` - `product(KAN-35): add reviewed workflow installation`.
- Script: `scripts/control-plane/install_enterprise_workflow_templates.ps1`.
- Design: `docs/design/reviewed-workflow-installation-mvp.md`.
- Report: `docs/reports/reviewed-workflow-installation-2026-04-30.md`.
- The installer supports both KAN-33 CLI output directories through `-PackDir` and KAN-34 dashboard JSON packs through `-PackPath`.
- Default mode is dry-run. Workflow writes require `-Apply`; replacements require `-Overwrite`.
- Target repository path must have a `.git` marker.
- Writes are constrained to `.github/workflows/*.yml` and `.github/workflows/*.yaml`.
- Unsafe paths, duplicate workflow paths, null bytes, declared secret-value packs, and declared repository-mutation packs are rejected.
- Safety: no `.env` reads, no provider token reads, no secret value printing, and no remote GitHub repository mutation.
- Local validation passed for CLI pack dry-run/apply, dashboard JSON pack dry-run/apply, unsafe path rejection, and differing existing workflow `blocked=1` planning.
- Direct provider credential/reachability checks are covered by `KAN-36`; formal enterprise release approval persistence is covered by `KAN-37`; Vercel AI SDK Copilot later starts in `KAN-38`.
- Post-merge `main` checks passed:
  - `CI` - run `25191857023`
  - `Release Readiness Gate` - run `25191857006`
  - `Quality Gate Policy Matrix (Optional)` - run `25191857008`
  - `Secret Scan` - run `25191856999`
  - `Public Naming Guard` - run `25191857012`
  - `SonarQube Governance (Non-Blocking)` - run `25191857029`
  - `Governance Correlation Smoke (Optional)` - run `25191857024`
  - `Desktop Updater Readiness (Optional)` - run `25191857020`

## Current KAN-36 Implementation Notes

- Implementation commit: `8c075a4 product(KAN-36): add provider connection validation`.
- PR: `#123` - `product(KAN-36): add provider connection validation`.
- Script: `scripts/control-plane/validate_enterprise_provider_connections.ps1`.
- Design: `docs/design/provider-connection-validation-mvp.md`.
- Report: `docs/reports/provider-connection-validation-2026-04-30.md`.
- The validator reads selected providers from an adoption profile by default and supports overrides for providers, repository, Jira project key, Jenkins job name, and Sonar project key.
- Supported providers: GitHub, Jira, Jenkins, SonarQube, Render, and Vercel.
- Status values are `ready`, `missing-config`, and `failed`.
- Default mode exits non-zero unless all selected providers are `ready`; `-ReportOnly` writes evidence without failing the process.
- Safety: no secret value printing, no secret value writing, no provider mutation, no webhook creation, no GitHub Actions variable/secret creation, and no customer repository mutation.
- Local validation passed for GitHub/Jira ready path, full profile `-ReportOnly` with local Jenkins/Sonar offline findings, Vercel missing-config report, and strict-mode missing-config failure.
- Formal enterprise release approval persistence is covered by `KAN-37`; Vercel AI SDK Copilot later starts in `KAN-38`.
- Post-merge `main` checks passed:
  - `CI` - run `25192626074`
  - `Release Readiness Gate` - run `25192626059`
  - `Quality Gate Policy Matrix (Optional)` - run `25192626048`
  - `Secret Scan` - run `25192626067`
  - `Public Naming Guard` - run `25192626061`
  - `SonarQube Governance (Non-Blocking)` - run `25192626079`
  - `Governance Correlation Smoke (Optional)` - run `25192626054`
  - `Desktop Updater Readiness (Optional)` - run `25192626050`

## Current KAN-37 Implementation Notes

- Jira: `KAN-37 - Formal enterprise release approval MVP`.
- Jira final comment: `10196`.
- Implementation commit: `d7ae92e product(KAN-37): add formal release approvals`.
- PR: `#125` - `product(KAN-37): add formal release approvals`.
- API: `GET /enterprise/release-approvals` and `POST /enterprise/release-approvals`.
- Backend table: `enterprise_release_approvals`.
- Migration: `gitgov/gitgov-server/supabase/supabase_schema_v24.sql`.
- Post-check: `gitgov/gitgov-server/supabase/checks/v24_postcheck.sql`.
- Design: `docs/design/formal-release-approval-mvp.md`.
- Report: `docs/reports/formal-release-approval-2026-04-30.md`.
- Local validation already run:
  - `cargo test enterprise_release_approval_validation` from `gitgov/gitgov-server`: `5` passed, `0` failed.
  - `cargo test` from `gitgov/gitgov-server`: `178` passed, `0` failed.
  - `cargo check` from `gitgov/gitgov-server`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Post-merge `main` checks passed:
  - `CI` - run `25193460879`.
  - `Release Readiness Gate` - run `25193460902`.
  - `Quality Gate Policy Matrix (Optional)` - run `25193460904`.
  - `Secret Scan` - run `25193460915`.
  - `Public Naming Guard` - run `25193460892`.
  - `SonarQube Governance (Non-Blocking)` - run `25193460922`.
  - `Governance Correlation Smoke (Optional)` - run `25193460903`.
  - `Desktop Updater Readiness (Optional)` - run `25193460881`.
- Production database migration `v24` was applied on 2026-04-30; `v24_postcheck.sql` passed all checks.
- Production deploy `dep-d7ptsvhoagis738cj88g` for commit `d7ae92e` reached `live`.
- Production validation:
  - `/health` returned `200`.
  - anonymous `GET /enterprise/release-approvals?org_name=yohandry10` returned `401`.
  - authenticated `GET /evidence/packets/tickets/KAN-37` returned `found=true`.
  - authenticated `POST /enterprise/release-approvals` created `KAN-37-runtime-smoke` with decision `approved`.
  - authenticated follow-up list for `KAN-37-runtime-smoke` returned `total=1`.
- Vercel AI SDK Copilot first server-side route is implemented in `KAN-38`.

## Latest KAN-38 Validation Notes

- Jira: `KAN-38 - Vercel AI SDK governance copilot MVP`.
- Implementation branch: `product/KAN-38-ai-sdk-copilot`.
- Implementation PR: `#127 - product(KAN-38): add AI SDK governance copilot`.
- Implementation commit: `9742472 product(KAN-38): add AI SDK governance copilot`.
- Jira final comment: `10197`.
- Route: `POST /api/copilot/governance`.
- Package: `gitgov-web` now depends on `ai@^6.0.0`.
- Implementation:
  - `gitgov-web/app/api/copilot/governance/route.ts`.
  - `gitgov-web/lib/copilot/governance.ts`.
- Design: `docs/design/ai-sdk-governance-copilot-mvp.md`.
- Report: `docs/reports/ai-sdk-governance-copilot-2026-04-30.md`.
- Evidence sources:
  - `GET /evidence/packets/tickets/{ticket_id}`.
  - `GET /integrations/jira/ticket-coverage`.
  - `GET /enterprise/release-approvals`.
  - `GET /enterprise/adoption-profile`.
- Security defaults:
  - caller Bearer token is required by default and forwarded only to GitGov backend.
  - server-key mode requires explicit `GITGOV_COPILOT_USE_SERVER_API_KEY=true` and `GITGOV_COPILOT_ACCESS_TOKEN`.
  - request body is limited to 12 KB.
  - route does not log or return Authorization headers.
- Local validation already run:
  - `pnpm run typecheck` from `gitgov-web`: passed.
  - `pnpm run lint` from `gitgov-web`: passed.
  - `pnpm run build` from `gitgov-web`: passed and registered `/api/copilot/governance`.
  - `pnpm audit --prod` from `gitgov-web`: no known vulnerabilities found.
  - local `next start -p 3108` route smoke with `GITGOV_COPILOT_DISABLE_AI=true`: `success=true`, `mode=fallback`, `4` citations, `4` evidence sources.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Post-merge validation:
  - `CI` run `25194421718`: passed.
  - `Release Readiness Gate` run `25194421743`: passed.
  - `Quality Gate Policy Matrix (Optional)` run `25194421721`: passed.
  - `Secret Scan` run `25194421747`: passed.
  - `Public Naming Guard` run `25194421752`: passed.
  - `SonarQube Governance (Non-Blocking)` run `25194421756`: passed.
  - `Governance Correlation Smoke (Optional)` run `25194421750`: passed.
  - `Desktop Updater Readiness (Optional)` run `25194421717`: passed.
- Vercel production deployment `https://git-ih2bzdqq5-trivia1.vercel.app` reached `Ready`.
- Production route smoke:
  - `POST https://www.gitgov.cloud/api/copilot/governance`: `200`, `success=true`, `mode=fallback`, `4` citations, `4` evidence sources, `1` expected warning.
  - `POST https://git-gov.vercel.app/api/copilot/governance`: `200`, `success=true`, `mode=fallback`, `4` citations, `4` evidence sources, `1` expected warning.
  - Direct deployment URL returned `401` HTML and apex `https://gitgov.cloud/api/copilot/governance` returned `401`; canonical `www` and Vercel production aliases are the validated paths.
- Production AI Gateway/OIDC was not active during KAN-38 validation, so the route used deterministic fallback mode. KAN-41 changes the selected production activation path to direct Google Gemini through `@ai-sdk/google`.

Latest KAN-39 governance copilot dashboard baseline:

- Implementation commit: `eda2f13 product(KAN-39): add governance copilot dashboard`.
- PR: `#129` - `product(KAN-39): add governance copilot dashboard`.
- Jira final comment: `10198`.
- Post-merge checks passed:
  - `CI` - run `25195469511`
  - `Release Readiness Gate` - run `25195469482`
  - `Quality Gate Policy Matrix (Optional)` - run `25195469485`
  - `Secret Scan` - run `25195469486`
  - `Governance Correlation Smoke (Optional)` - run `25195469490`
  - `Desktop Updater Readiness (Optional)` - run `25195469496`
  - `SonarQube Governance (Non-Blocking)` - run `25195469502`
  - `Public Naming Guard` - run `25195469507`

## Latest KAN-39 Validation Notes

- Jira: `KAN-39 - Governance copilot dashboard UI MVP`.
- Implementation branch: `product/KAN-39-governance-copilot-dashboard`.
- Implementation PR: `#129 - product(KAN-39): add governance copilot dashboard`.
- Implementation commit: `eda2f13 product(KAN-39): add governance copilot dashboard`.
- Jira final comment: `10198`.
- Design: `docs/design/governance-copilot-dashboard-mvp.md`.
- Report: `docs/reports/governance-copilot-dashboard-2026-04-30.md`.
- Scope:
  - Tauri command `cmd_server_governance_copilot_ask`.
  - dashboard store action `askGovernanceCopilot`.
  - admin dashboard component `GovernanceCopilotPanel`.
  - citations, source statuses, warnings, answer mode, and response text.
- Security defaults:
  - dashboard browser does not call the public copilot endpoint directly.
  - desktop command forwards the configured GitGov API key only as a Bearer token.
  - default copilot URL is `https://www.gitgov.cloud/api/copilot/governance`.
  - optional `GITGOV_COPILOT_URL` is process-env controlled, host allowlisted, and must not contain embedded credentials.
- Local validation already run:
  - `cargo fmt` from `gitgov/src-tauri`: passed.
  - `cargo check` from `gitgov/src-tauri`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/src-tauri`: passed.
  - `cargo test` from `gitgov/src-tauri`: passed.
  - `npm test -- --run src/test/useControlPlaneStore.test.ts` from `gitgov`: passed.
  - `npm test -- --run` from `gitgov`: passed.
  - `npm run typecheck` from `gitgov`: passed.
  - `npm run lint` from `gitgov`: passed.
  - `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
  - local Vite smoke `GET http://127.0.0.1:5174/`: returned `200`.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Post-merge validation:
  - `CI` run `25195469511`: passed.
  - `Release Readiness Gate` run `25195469482`: passed.
  - `Quality Gate Policy Matrix (Optional)` run `25195469485`: passed.
  - `Secret Scan` run `25195469486`: passed.
  - `Governance Correlation Smoke (Optional)` run `25195469490`: passed.
  - `Desktop Updater Readiness (Optional)` run `25195469496`: passed.
  - `SonarQube Governance (Non-Blocking)` run `25195469502`: passed.
  - `Public Naming Guard` run `25195469507`: passed.

## Current KAN-40 Implementation Notes

- Jira: `KAN-40 - Governance copilot AI mode validation`.
- Implementation branch: `product/KAN-40-governance-copilot-ai-validation`.
- Implementation PR: `#131 - product(KAN-40): validate governance copilot AI mode`.
- Implementation commit: `2b507bc product(KAN-40): validate governance copilot AI mode`.
- Script: `scripts/control-plane/validate_governance_copilot_ai_mode.ps1`.
- Workflow: `.github/workflows/governance-copilot-ai-mode-validation.yml`.
- Runbook: `docs/runbooks/governance-copilot-ai-mode-validation.md`.
- Report: `docs/reports/governance-copilot-ai-mode-validation-2026-04-30.md`.
- Scope:
  - validate the production copilot route without printing secrets.
  - record HTTP status, response mode, source count, ok-source count, citation count, warning count, answer length, and answer SHA-256 hash.
  - support `-RequireAiMode` for post-AI-Gateway/OIDC enforcement.
  - add a weekly/manual GitHub Actions workflow that uploads sanitized validation evidence.
- Local production validation already run:
  - non-strict command returned `status=fallback`, `ok=true`, HTTP `200`, `4` citations, `4` sources, and `4` ok sources.
  - strict mode returned the expected controlled failure because production still reports `mode=fallback`.
- Post-merge checks passed:
  - `CI` run `25196003313`.
  - `Release Readiness Gate` run `25196003326`.
  - `Quality Gate Policy Matrix (Optional)` run `25196003325`.
  - `Secret Scan` run `25196003309`.
  - `Governance Correlation Smoke (Optional)` run `25196003311`.
  - `SonarQube Governance (Non-Blocking)` run `25196003302`.
  - `Public Naming Guard` run `25196003318`.
  - `Desktop Updater Readiness (Optional)` run `25196003351`.
- First manual workflow validation passed:
  - Run `25196010712`.
  - Artifact `governance-copilot-ai-mode-validation`.
  - Artifact ID `6742816838`.
  - Artifact status: not expired; expires `2026-07-30T00:21:30Z`.
  - Result: `status=fallback`, `ok=true`, HTTP `200`, `4` citations, `4` sources, `4` ok sources, and `1` warning.
- Current interpretation:
  - the copilot route is healthy and evidence-grounded.
  - production AI generation mode moved to KAN-41 using direct Google Gemini because Vercel AI Gateway required billing-card activation.

## Current KAN-41 Implementation Notes

- Jira: `KAN-41 - Activate governance copilot AI mode on Vercel`.
- Implementation PRs:
  - `#133 - product(KAN-41): add Google AI SDK copilot provider`.
  - `#134 - fix(KAN-41): add sanitized copilot AI failure diagnostics`.
  - `#135 - fix(KAN-41): include sanitized copilot AI runtime cause`.
  - `#136 - fix(KAN-41): sanitize Google AI env key`.
- Final implementation commit: `ba61d16 fix(KAN-41): sanitize Google AI env key`.
- Report: `docs/reports/google-ai-sdk-copilot-provider-2026-05-01.md`.
- Scope:
  - add direct Google Gemini support to `POST /api/copilot/governance` through `@ai-sdk/google`.
  - keep AI Gateway as an optional provider path.
  - keep deterministic fallback if no provider is configured or generation fails.
  - do not change the existing Rust `/chat/ask` Gemini bot path.
  - strip leading UTF-8 BOM and surrounding whitespace from server-side Google/Gemini env values before using them as provider keys.
- Vercel production env configuration was updated without printing secret values:
  - `GOOGLE_GENERATIVE_AI_API_KEY`.
  - `GITGOV_COPILOT_PROVIDER=google`.
  - `GITGOV_COPILOT_GOOGLE_MODEL=gemini-2.5-flash`.
- Production env correction:
  - first strict validation showed an invisible BOM in the uploaded Google key value.
  - after the code stripped BOM/whitespace, strict validation showed the first local Gemini key was expired.
  - the production secret was reconfigured from the effective local Gemini key used by the working local/server bot path.
  - Vercel redeploy `https://git-8gwowu155-trivia1.vercel.app` reached `Ready` and is aliased to `https://www.gitgov.cloud`.
- Preview remains fallback-only unless explicitly configured later.
- Local validation already run:
  - `pnpm run typecheck` from `gitgov-web`: passed.
  - `pnpm run lint` from `gitgov-web`: passed.
  - `pnpm run build` from `gitgov-web`: passed.
  - `pnpm audit --prod` from `gitgov-web`: no known vulnerabilities.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
  - local route smoke with `GITGOV_COPILOT_PROVIDER=google` and Google key mapped from ignored local `GEMINI_API_KEY`: HTTP `200`, `success=true`, `mode=ai`, `model=google/gemini-2.5-flash`, `4` citations, `4` sources, `4` ok sources, and `0` warnings.
- Final production strict validation passed:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\KAN-41-governance-copilot-google-ai-mode-validation.json
```

Result: `status=ai`, `ok=true`, HTTP `200`, `success=true`, `mode=ai`, `model=google/gemini-2.5-flash`, `4` citations, `4` sources, `4` ok sources, `0` warnings, and no raw answer or secrets stored.

- Post-merge checks for final commit `ba61d16` passed:
  - `CI` - run `25199526039`.
  - `Release Readiness Gate` - run `25199526047`.
  - `Quality Gate Policy Matrix (Optional)` - run `25199526028`.
  - `Secret Scan` - run `25199526038`.
  - `Governance Correlation Smoke (Optional)` - run `25199526055`.
  - `SonarQube Governance (Non-Blocking)` - run `25199526033`.
  - `Public Naming Guard` - run `25199526037`.
  - `Desktop Updater Readiness (Optional)` - run `25199526031`.

## Current KAN-42 Implementation Notes

- Jira: `KAN-42 - Enforce governance copilot AI mode validation`.
- Implementation branch: `ops/KAN-42-enforce-copilot-ai-validation`.
- Implementation PR: `#138 - ops(KAN-42): enforce governance copilot AI validation`.
- Implementation commit: `7ad1c9d ops(KAN-42): enforce governance copilot AI validation`.
- Workflow: `.github/workflows/governance-copilot-ai-mode-validation.yml`.
- Runbook: `docs/runbooks/governance-copilot-ai-mode-validation.md`.
- Report: `docs/reports/governance-copilot-ai-mode-enforcement-2026-05-01.md`.
- Scope:
  - scheduled Governance Copilot AI Mode Validation runs now require `mode=ai`.
  - manual dispatch defaults to `require_ai_mode=true`.
  - manual `require_ai_mode=false` remains available only for fallback diagnostics.
  - missing `GITGOV_API_KEY` fails strict runs instead of silently skipping.
- Local validation already run:
  - workflow YAML parsed successfully.
  - strict production validator passed with HTTP `200`, `success=true`, `mode=ai`, `model=google/gemini-2.5-flash`, `4` citations, `4` sources, `4` ok sources, and `0` warnings.
- Post-merge checks for commit `7ad1c9d` passed:
  - `CI` - run `25200079701`.
  - `Release Readiness Gate` - run `25200079686`.
  - `Quality Gate Policy Matrix (Optional)` - run `25200079694`.
  - `Secret Scan` - run `25200079699`.
  - `Governance Correlation Smoke (Optional)` - run `25200079688`.
  - `SonarQube Governance (Non-Blocking)` - run `25200079685`.
  - `Public Naming Guard` - run `25200079691`.
  - `Desktop Updater Readiness (Optional)` - run `25200079696`.
- First strict manual `Governance Copilot AI Mode Validation` workflow run on `main` passed:
  - Run `25200126845`.
  - Head SHA `7ad1c9dc947a2ff50e451f4caacc8125874527aa`.
  - Result: `status=ai`, `ok=true`, `mode=ai`, `model=google/gemini-2.5-flash`, `4` citations, `4` sources, `4` ok sources, and `0` warnings.
  - Artifact `governance-copilot-ai-mode-validation`, ID `6744359123`, expires `2026-07-30T02:58:40Z`.

## Latest KAN-43 Validation Notes

- Jira: `KAN-43 - Dashboard release approval wizard MVP`.
- Implementation branch: `product/KAN-43-release-approval-dashboard`.
- Implementation PR: `#140 - product(KAN-43): add release approval dashboard`.
- Implementation commit: `10d0c4b product(KAN-43): add release approval dashboard`.
- Design: `docs/design/release-approval-dashboard-mvp.md`.
- Report: `docs/reports/release-approval-dashboard-2026-05-01.md`.
- Scope:
  - add `gitgov/src/components/control_plane/ReleaseApprovalPanel.tsx`.
  - add admin dashboard create/list UI for formal release approvals.
  - add Zustand release approval state and list/create actions.
  - add Tauri client structs, methods, commands and command registration for `GET /enterprise/release-approvals` and `POST /enterprise/release-approvals`.
  - reuse the existing KAN-37 backend API and validation.
- Client validation:
  - release, repository, environment, approver and evidence hash are required.
  - repository must look like `owner/repo`.
  - evidence hash must be 64 hex characters.
  - optional target SHA, ticket ID, and evidence URI are shape-validated.
  - high/critical risk cannot be approved directly.
  - accepted-risk requires non-`none` severity, reason and 1-366 day expiration.
  - operator confirmation is required before submit.
- Local validation already run:
  - `cargo fmt` from `gitgov/src-tauri`: passed.
  - `cargo check` from `gitgov/src-tauri`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/src-tauri`: passed.
  - `cargo test` from `gitgov/src-tauri`: `23` passed.
  - `npm test -- --run src/test/useControlPlaneStore.test.ts` from `gitgov`: `21` passed.
  - `npm run lint` from `gitgov`: passed.
  - `npm run typecheck` from `gitgov`: passed.
  - `npm test -- --run` from `gitgov`: `25` test files passed, `280` tests passed.
  - `npm run build` from `gitgov`: passed with existing Vite large chunk warning.
- PR `#140` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `10d0c4b` passed:
  - `CI` - run `25202577666`.
  - `Release Readiness Gate` - run `25202577665`.
  - `Quality Gate Policy Matrix (Optional)` - run `25202577671`.
  - `Secret Scan` - run `25202577668`.
  - `SonarQube Governance (Non-Blocking)` - run `25202577669`.
  - `Public Naming Guard` - run `25202577675`.
  - `Governance Correlation Smoke (Optional)` - run `25202577688`.
  - `Desktop Updater Readiness (Optional)` - run `25202577680`.
- No new backend route, database migration, Render deploy, or Vercel production env change was needed; KAN-43 reuses the existing KAN-37 backend API.

## Latest KAN-44 Validation Notes

- Jira: `KAN-44 - Document configurable release governance defaults`.
- Implementation branch: `docs/KAN-44-configurable-release-governance`.
- Implementation PR: `#142 - docs(KAN-44): clarify release governance defaults`.
- Implementation commit: `eb15b08 docs(KAN-44): clarify release governance defaults`.
- Design: `docs/design/configurable-release-governance-defaults.md`.
- Report: `docs/reports/configurable-release-governance-defaults-2026-05-01.md`.
- Product decision:
  - default release governance behavior remains `record-only`.
  - release approval records can be stored, displayed and reported by default.
  - release-blocking enforcement must be explicitly selected by customer policy.
  - multi-approver quorum must be explicitly selected by customer policy.
  - generated workflows should not block customer releases unless the adoption profile or equivalent policy clearly enables blocking behavior.
- Current docs updated:
  - `docs/design/configurable-release-governance-defaults.md`.
  - `docs/design/formal-release-approval-mvp.md`.
  - `docs/design/release-approval-dashboard-mvp.md`.
  - `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`.
  - `docs/reports/configurable-release-governance-defaults-2026-05-01.md`.
- Local validation:
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#142` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `eb15b08` passed:
  - `CI` - run `25203116708`.
  - `Release Readiness Gate` - run `25203116684`.
  - `Quality Gate Policy Matrix (Optional)` - run `25203116644`.
  - `Secret Scan` - run `25203116635`.
  - `SonarQube Governance (Non-Blocking)` - run `25203116668`.
  - `Public Naming Guard` - run `25203116673`.
  - `Governance Correlation Smoke (Optional)` - run `25203116650`.
  - `Desktop Updater Readiness (Optional)` - run `25203116657`.
- No code, database, Render, Vercel, provider, or customer workflow behavior changed; KAN-44 is documentation/product-default memory only.

## Latest KAN-45 Validation Notes

- Jira: `KAN-45 - Add configurable release governance profile policy`.
- Implementation branch: `product/KAN-45-release-governance-profile-policy`.
- Implementation PR: `#144 - product(KAN-45): add release governance profile policy`.
- Implementation commit: `dc37e92 product(KAN-45): add release governance profile policy`.
- Design: `docs/design/release-governance-profile-policy-mvp.md`.
- Report: `docs/reports/release-governance-profile-policy-2026-05-01.md`.
- Scope:
  - add explicit `release_governance` to the Enterprise Adoption profile shape.
  - preserve `record-only` / `disabled` as the default release governance mode.
  - expose release governance mode and environment controls in the Enterprise Adoption dashboard.
  - include release governance in dashboard adoption pack and workflow template pack exports.
  - include release governance in the CLI adoption pack Markdown/JSON output.
  - include release governance in the CLI workflow template README/manifest output.
  - validate `release_governance` in `PUT /enterprise/adoption-profile` before persistence.
  - keep non-`record-only` modes gated on the `formal-approval` module.
- Product rule:
  - default behavior remains non-blocking.
  - `advisory`, `approval-required`, and `quorum-required` are customer-selected modes.
  - KAN-45 stores and carries policy intent; it does not add active release blocking by itself.
- Local validation already run:
  - `cargo fmt` from `gitgov/gitgov-server`: passed.
  - `cargo test adoption_profile_tests` from `gitgov/gitgov-server`: passed, `6` tests.
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts` from `gitgov`: passed, `15` tests.
  - `npm run typecheck` from `gitgov`: passed.
  - `.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-45-enterprise-adoption-pack`: passed.
  - `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-45-enterprise-workflow-templates -Force`: passed.
- Secret safety:
  - no provider token, `.env` value, Authorization header, or raw secret payload is read or printed by this change.
- PR `#144` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `dc37e92` passed:
  - `CI` - run `25203785504`.
  - `Release Readiness Gate` - run `25203785499`.
  - `Quality Gate Policy Matrix (Optional)` - run `25203785520`.
  - `Secret Scan` - run `25203785497`.
  - `SonarQube Governance (Non-Blocking)` - run `25203785527`.
  - `Public Naming Guard` - run `25203785483`.
  - `Governance Correlation Smoke (Optional)` - run `25203785490`.
  - `Desktop Updater Readiness (Optional)` - run `25203785503`.
- No database migration, Render deploy, Vercel production env change, provider setting change, or customer workflow installation was needed.

## Latest KAN-46 Validation Notes

- Jira: `KAN-46 - Add release governance evaluator`.
- Implementation branch: `product/KAN-46-release-governance-evaluator`.
- Implementation PR: `#146 - product(KAN-46): add release governance evaluator`.
- Implementation commit: `0252432 product(KAN-46): add release governance evaluator`.
- Design: `docs/design/release-governance-evaluator-mvp.md`.
- Report: `docs/reports/release-governance-evaluator-2026-05-01.md`.
- Scope:
  - add admin endpoint `GET /enterprise/release-governance/evaluate`.
  - evaluate KAN-45 `release_governance` policy against KAN-37 formal release approval records.
  - keep `record-only` non-blocking by default.
  - return `recorded`, `advisory-warning`, `approved`, `would-block`, or `blocked`.
  - include `blocking` and `would_block` booleans for future customer-selected workflow gates.
  - count quorum roles through `evidence_summary.approver_role` without adding a database migration.
  - add Tauri control-plane client structs, method, command, and registration.
  - add dashboard release governance evaluation state, action, button, result panel, and approver role field.
  - include `/enterprise/release-governance/evaluate` in the stale-auth-cache sensitive admin path set.
- Product rule:
  - KAN-46 reports release governance status; it does not mutate customer workflows or block deployments by itself.
  - blocking status can only appear when the customer-selected profile policy is explicitly blocking.
  - future workflow enforcement must opt in to consuming `blocking=true`.
- Local validation already run:
  - `cargo fmt` from `gitgov/gitgov-server`: passed.
  - `cargo fmt` from `gitgov/src-tauri`: passed.
  - `cargo test release_approval_tests` from `gitgov/gitgov-server`: passed, `9` tests.
  - `cargo test sensitive_admin_path_detection_matches_expected_routes` from `gitgov/gitgov-server`: passed, `1` test.
  - `cargo check` from `gitgov/gitgov-server`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
  - `cargo check` from `gitgov/src-tauri`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/src-tauri`: passed.
  - `cargo test` from `gitgov/src-tauri`: passed, `23` tests.
  - `npm test -- --run src/test/useControlPlaneStore.test.ts` from `gitgov`: passed, `22` tests.
  - `npm run lint` from `gitgov`: passed.
  - `npm run typecheck` from `gitgov`: passed.
  - `npm test -- --run` from `gitgov`: passed, `25` test files and `283` tests.
  - `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Secret safety:
  - no provider token, `.env` value, Authorization header, webhook secret, or raw customer credential is read or printed by this change.
- PR `#146` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `0252432` passed:
  - `CI` - run `25207328590`.
  - `Release Readiness Gate` - run `25207328587`.
  - `Quality Gate Policy Matrix (Optional)` - run `25207328585`.
  - `Secret Scan` - run `25207328605`.
  - `SonarQube Governance (Non-Blocking)` - run `25207328608`.
  - `Public Naming Guard` - run `25207328592`.
  - `Governance Correlation Smoke (Optional)` - run `25207328584`.
  - `Desktop Updater Readiness (Optional)` - run `25207328581`.
- Production validation:
  - Render deploy `dep-d7q5qmkvikkc73cmfg0g` reached `live` for commit `025243214639757e901830d958e60e2ba3eb55cd`.
  - `GET https://gitgov-api.onrender.com/health` returned `status=ok`.
  - Anonymous `GET /enterprise/release-governance/evaluate?...` returned `401`.
  - Authenticated `GET /enterprise/release-governance/evaluate?...` returned `200` with `status=recorded`, `policy_mode=record-only`, `blocking=false`, `would_block=false`, `valid=0`, and `required=0`.
- No database migration, provider setting change, customer workflow installation, or Vercel production environment change was needed.

## Latest KAN-48 Validation Notes

- Jira: `KAN-48 - Add environment-scoped release governance policy overrides`.
- Implementation branch: `product/KAN-48-environment-release-governance-policy`.
- Implementation PR: `#150 - product(KAN-48): add environment release governance overrides`.
- Implementation commit: `cba3f9d product(KAN-48): add environment release governance overrides`.
- Design: `docs/design/environment-scoped-release-governance-policy-mvp.md`.
- Report: `docs/reports/environment-scoped-release-governance-policy-2026-05-01.md`.
- Scope:
  - add optional `release_governance.environment_overrides` to enterprise adoption profiles.
  - keep base/default release governance `record-only` and non-blocking.
  - allow explicit per-environment opt-in policy, for example `production: approval-required` while `staging` remains record-only.
  - make the KAN-46 evaluator resolve matching environment override first, then fall back to base policy.
  - make adoption pack and workflow template generation include the release governance gate when any override is non-`record-only` and `formal-approval` is enabled.
  - expose environment overrides in the Enterprise Adoption dashboard without reading `.env` or provider secrets.
- Local validation already run:
  - `cargo fmt` from `gitgov/gitgov-server`: passed.
  - `cargo test release_governance` from `gitgov/gitgov-server`: passed, `6` tests.
  - `cargo test enterprise_adoption_profile_validation` from `gitgov/gitgov-server`: passed, `8` tests.
  - `cargo check` from `gitgov/gitgov-server`: passed.
  - `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
  - `cargo test` from `gitgov/gitgov-server`: passed, `189` tests.
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts` from `gitgov`: passed, `17` tests.
  - `npm run typecheck` from `gitgov`: passed.
  - `npm run lint` from `gitgov`: passed.
  - `npm test -- --run` from `gitgov`: passed, `25` test files and `285` tests.
  - `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
  - CLI adoption pack generation with a production `approval-required` override: passed with `14` workflows and release governance gate included.
  - CLI workflow template generation with a production `approval-required` override: passed with `14` templates; generated release governance gate defaulted to `production` and `enforce_gate=true`.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Remaining local validation before PR:
  - none.
- No database migration, provider setting change, customer repository mutation, Render deploy, or Vercel production environment change is expected for KAN-48.
- PR `#150` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `cba3f9d` passed:
  - `CI` - run `25209198316`.
  - `Release Readiness Gate` - run `25209198277`.

## Latest KAN-49 Validation Notes

- Jira: `KAN-49 - Monitor release governance gate artifacts`.
- Implementation branch: `ops/KAN-49-release-governance-gate-artifact-monitor`.
- Implementation PR: `#152 - ops(KAN-49): monitor release governance gate artifacts`.
- Implementation commit: `4257a95 ops(KAN-49): monitor release governance gate artifacts`.
- Design: `docs/design/release-governance-gate-artifact-monitor-mvp.md`.
- Report: `docs/reports/release-governance-gate-artifact-monitor-2026-05-01.md`.
- Runbook: `docs/runbooks/release-governance-gate.md`.
- Scope:
  - add manual workflow `.github/workflows/release-governance-gate-artifact-monitor.yml`.
  - reuse `scripts/control-plane/validate_github_evidence_report_artifact.ps1` against `release-governance-gate.yml` artifacts named `release-governance-gate-*`.
  - add CLI and dashboard workflow template support for `.github/workflows/release-governance-gate-artifact-monitor.yml`.
  - generate the enterprise monitor only when `formal-approval`, non-`record-only` release governance, and `artifact-monitoring` are selected.
  - keep `record-only` release approval evidence non-blocking and without generated release governance gate/monitor templates by default.
- Local validation already run:
  - `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile release-governance-gate.yml -ArtifactNamePrefix release-governance-gate- -MaxAgeHours 720 -OutputPath out\release-governance-gate-artifact-monitor.json`: passed against run `25208470238`, artifact `release-governance-gate-25208470238`, ID `6747272652`.
  - CLI workflow template generation with a production environment override profile: passed; generated both release governance gate and artifact monitor templates, with monitor max age `720`.
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `18` tests.
  - `npm run typecheck`: passed.
  - `npm run lint`: passed.
  - `npm test -- --run`: passed, `25` test files and `286` tests.
  - `npm run build`: passed with the existing Vite large chunk warning.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#152` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `4257a95` passed:
  - `CI` - run `25209672506`.
  - `Release Readiness Gate` - run `25209672484`.
  - `Quality Gate Policy Matrix (Optional)` - run `25209672487`.
  - `Secret Scan` - run `25209672489`.
  - `Public Naming Guard` - run `25209672473`.
  - `Governance Correlation Smoke (Optional)` - run `25209672471`.
  - `Desktop Updater Readiness (Optional)` - run `25209672476`.
  - `SonarQube Governance (Non-Blocking)` - run `25209672492`.
- First manual `Release Governance Gate Artifact Monitor` workflow run on `main` passed:
  - Run `25209735562`.
  - Artifact `release-governance-gate-artifact-monitor`, ID `6747717581`, not expired.
- No database migration, provider setting change, customer repository mutation, Render deploy, or Vercel production environment change was needed.

## Latest KAN-50 Validation Notes

- Jira: `KAN-50 - Remote workflow installation PR for customer repositories`.
- Implementation branch: `product/KAN-50-remote-workflow-installation-pr`.
- Implementation PR: `#154 - product(KAN-50): add remote workflow installation PR flow`.
- Implementation commit: `eb7482b product(KAN-50): add remote workflow installation PR flow`.
- Script: `scripts/control-plane/open_enterprise_workflow_template_pr.ps1`.
- Design: `docs/design/remote-workflow-installation-pr-mvp.md`.
- Report: `docs/reports/remote-workflow-installation-pr-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Scope:
  - add dry-run-first remote PR creation for GitGov enterprise workflow template packs.
  - support CLI-generated `-PackDir` and dashboard-style `-PackPath`.
  - infer target repository/default branch from the pack manifest when explicit parameters are omitted.
  - compare remote `.github/workflows/*.yml` and `.yaml` files against the target base branch.
  - require `-Apply` before creating a remote branch, single commit, and draft PR.
  - require `-Overwrite` before replacing differing existing workflow files.
  - keep PRs draft by default unless `-ReadyForReview` is passed.
  - avoid GitHub Actions variable/secret creation, branch protection mutation, provider mutation, or automatic merge.
- Local validation already run:
  - CLI workflow template generation with `docs/examples/enterprise-adoption-profile.example.json`: passed.
  - `-PackDir` dry-run against `yohandry10/Git-Gov` on `main`: passed with `create=0`, `update=0`, `skip=0`, `blocked=13`; no remote mutation.
  - `-PackDir -Overwrite` dry-run against `yohandry10/Git-Gov` on `main`: passed with `create=0`, `update=13`, `skip=0`, `blocked=0`; no remote mutation.
  - minimal dashboard-style `-PackPath` dry-run: passed with `create=1`, `update=0`, `skip=0`, `blocked=0`; no remote mutation.
  - PowerShell parse check for `open_enterprise_workflow_template_pr.ps1`: passed.
  - dry-run plan secret/string scan: passed, no token/Authorization/secret-value assignment patterns found.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#154` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `eb7482b` passed:
  - `CI` - run `25210329452`.
  - `Release Readiness Gate` - run `25210329443`.
  - `Quality Gate Policy Matrix (Optional)` - run `25210329442`.
  - `Secret Scan` - run `25210329455`.
  - `Public Naming Guard` - run `25210329454`.
  - `Governance Correlation Smoke (Optional)` - run `25210329459`.
  - `Desktop Updater Readiness (Optional)` - run `25210329441`.
  - `SonarQube Governance (Non-Blocking)` - run `25210329445`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, or remote apply run was needed.

## Latest KAN-51 Validation Notes

- Jira: `KAN-51 - Remote workflow installation readiness validation`.
- Implementation branch: `product/KAN-51-remote-workflow-readiness-validation`.
- Implementation PR: `#156 - product(KAN-51): validate remote workflow readiness`.
- Implementation commit: `dcfb529 product(KAN-51): validate remote workflow readiness`.
- Script: `scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1`.
- Design: `docs/design/remote-workflow-readiness-validation-mvp.md`.
- Report: `docs/reports/remote-workflow-readiness-validation-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Scope:
  - add read-only GitHub repository readiness validation after workflow template installation.
  - support CLI `-PackDir` and dashboard-style `-PackPath` sources.
  - validate expected `.github/workflows/*.yml` / `.yaml` files at a selected ref.
  - compare remote workflow content to the reviewed pack by SHA-256.
  - validate GitHub Actions variable names from the manifest.
  - validate GitHub Actions secret names from the manifest without reading secret values.
  - support `-ReportOnly` for non-blocking onboarding reports.
  - avoid file, branch, PR, variable, secret, provider, workflow dispatch, or branch-protection mutation.
- Local validation already run:
  - PowerShell parse check for `validate_enterprise_workflow_installation_readiness.ps1`: passed.
  - CLI workflow template generation with `docs/examples/enterprise-adoption-profile.example.json`: passed.
  - `-PackDir -ReportOnly` readiness against `yohandry10/Git-Gov` on `main`: passed with expected `needs-action`, `workflows_missing=0`, `workflows_different=13`, `variables_missing=0`, and `secrets_missing=1`.
  - minimal dashboard-style `-PackPath -ReportOnly` readiness against `yohandry10/Git-Gov` on `main`: passed with expected `needs-action`, `workflows_missing=1`, `workflows_different=0`, `variables_missing=0`, and `secrets_missing=0`.
  - output secret/header string scan: passed, no token values or Authorization headers stored.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#156` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `dcfb529` passed:
  - `CI` - run `25210718116`.
  - `Release Readiness Gate` - run `25210718113`.
  - `Quality Gate Policy Matrix (Optional)` - run `25210718112`.
  - `Secret Scan` - run `25210718114`.
  - `Public Naming Guard` - run `25210718108`.
  - `Governance Correlation Smoke (Optional)` - run `25210718092`.
  - `Desktop Updater Readiness (Optional)` - run `25210718096`.
  - `SonarQube Governance (Non-Blocking)` - run `25210718107`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, or remote apply run was needed.

## Latest KAN-52 Validation Notes

- Jira: `KAN-52 - Enterprise onboarding readiness report`.
- Implementation branch: `product/KAN-52-enterprise-onboarding-readiness`.
- Implementation PR: `#158 - product(KAN-52): add onboarding readiness report`.
- Implementation commit: `a64bb30 product(KAN-52): add onboarding readiness report`.
- Main merge commit: `268770a product(KAN-52): add onboarding readiness report`.
- Script: `scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1`.
- Dashboard helper: `buildEnterpriseOnboardingReadinessReport`.
- Dashboard UI: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Design: `docs/design/enterprise-onboarding-readiness-report-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-readiness-report-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Scope:
  - add a consolidated Enterprise Self-Service Adoption readiness snapshot.
  - combine adoption profile validity, provider readiness, workflow pack status, remote workflow readiness, GitHub Actions config-name readiness, and release governance posture.
  - add dashboard `Onboarding` readiness card and `Readiness` JSON export.
  - add CLI Markdown/JSON generator that can consume KAN-36 provider reports and KAN-51 workflow readiness reports.
  - keep report generation read-only and secret-safe.
- Local validation already run:
  - PowerShell parse check for `generate_enterprise_onboarding_readiness_report.ps1`: passed.
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `22` tests.
  - `npm run typecheck`: passed.
  - `npm run lint`: passed.
  - `npm test -- --run`: passed, `25` test files and `290` tests.
  - `npm run build`: passed with the existing Vite large chunk warning.
  - profile-only readiness report: `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
  - provider/workflow-input readiness report: `needs-action`, score `83`, `4` ready stages, `2` needs-action stages, `0` blocked stages.
  - generated output scan for `Authorization`, `Bearer`, `GITGOV_API_KEY=`, `SONAR_TOKEN=`, `ATATT`, and `vck_`: passed with no matches.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#158` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `268770a` passed:
  - `CI` - run `25211254174`.
  - `Release Readiness Gate` - run `25211254160`.
  - `Quality Gate Policy Matrix (Optional)` - run `25211254185`.
  - `Secret Scan` - run `25211254159`.
  - `Public Naming Guard` - run `25211254165`.
  - `Governance Correlation Smoke (Optional)` - run `25211254168`.
  - `Desktop Updater Readiness (Optional)` - run `25211254172`.
  - `SonarQube Governance (Non-Blocking)` - run `25211254202`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, remote apply run, or workflow dispatch was needed.

## Latest KAN-53 Validation Notes

- Jira: `KAN-53 - Automate enterprise onboarding readiness evidence`.
- Implementation branch: `product/KAN-53-enterprise-onboarding-readiness-automation`.
- Implementation PR: `#160 - ops(KAN-53): automate onboarding readiness evidence`.
- Implementation commit: `85d63e1 ops(KAN-53): automate onboarding readiness evidence`.
- Main merge commit: `027a10f Merge pull request #160 from yohandry10/product/KAN-53-enterprise-onboarding-readiness-automation`.
- Workflow: `.github/workflows/enterprise-onboarding-readiness.yml`.
- Design: `docs/design/enterprise-onboarding-readiness-automation-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-readiness-automation-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Scope:
  - add manual and weekly Enterprise Onboarding Readiness automation.
  - generate a temporary adoption profile from workflow inputs or safe GitGov defaults.
  - run adoption pack generation, workflow template generation, optional KAN-51 read-only workflow readiness, and KAN-52 readiness reporting.
  - upload `enterprise-onboarding-readiness-{run_id}` artifacts.
  - keep release blocking opt-in and report-only by default.
- Local validation already run:
  - GitGov adoption pack generation: passed.
  - GitGov workflow template generation: passed.
  - default KAN-53 readiness generation: `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
  - optional KAN-51 remote workflow readiness: `needs-action`, `workflows_missing=0`, `workflows_different=13`, `variables_missing=0`, `secrets_missing=1`.
  - KAN-53 readiness generation with remote readiness input: `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
  - generated output scan for `Authorization`, `Bearer`, `GITGOV_API_KEY=`, `SONAR_TOKEN=`, `ATATT`, and `vck_`: passed with no matches.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#160` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `027a10f` passed:
  - `CI` - run `25211635818`.
  - `Release Readiness Gate` - run `25211635807`.
  - `Quality Gate Policy Matrix (Optional)` - run `25211636125`.
  - `Secret Scan` - run `25211635809`.
  - `Public Naming Guard` - run `25211635814`.
  - `Governance Correlation Smoke (Optional)` - run `25211635806`.
  - `Desktop Updater Readiness (Optional)` - run `25211635830`.
  - `SonarQube Governance (Non-Blocking)` - run `25211635803`.
- First manual workflow validation passed:
  - Run `25211644692`.
  - Artifact `enterprise-onboarding-readiness-25211644692`.
  - Artifact ID `6748421926`.
  - Artifact status: not expired, expires at `2026-07-30T10:46:51Z`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, or provider webhook mutation was needed.

## Latest KAN-54 Validation Notes

- Jira: `KAN-54 - Monitor enterprise onboarding readiness evidence artifacts`.
- Implementation branch: `ops/KAN-54-enterprise-onboarding-readiness-artifact-monitor`.
- Implementation PR: `#162 - ops(KAN-54): monitor onboarding readiness artifacts`.
- Implementation commit: `414f8b0 ops(KAN-54): monitor onboarding readiness artifacts`.
- Main merge commit: `ec99b7c Merge pull request #162 from yohandry10/ops/KAN-54-enterprise-onboarding-readiness-artifact-monitor`.
- Workflow: `.github/workflows/enterprise-onboarding-readiness-artifact-monitor.yml`.
- Design: `docs/design/enterprise-onboarding-readiness-artifact-monitor-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-readiness-artifact-monitor-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Scope:
  - add a weekly/manual monitor for KAN-53 readiness artifacts.
  - validate the latest successful `enterprise-onboarding-readiness.yml` run uploaded a fresh `enterprise-onboarding-readiness-` artifact.
  - upload `enterprise-onboarding-readiness-artifact-monitor`.
  - keep onboarding `needs-action` non-blocking and release blocking opt-in only.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no customer repository mutation.
  - no provider mutation.
  - no GitHub Actions variable/secret creation.
  - no workflow dispatch or branch protection mutation.
  - no release blocking by default.
- Local validation already run:
  - command used existing `scripts/control-plane/validate_github_evidence_report_artifact.ps1`.
  - workflow file: `enterprise-onboarding-readiness.yml`.
  - artifact prefix: `enterprise-onboarding-readiness-`.
  - latest successful source run: `25211644692`.
  - artifact: `enterprise-onboarding-readiness-25211644692`.
  - artifact ID: `6748421926`.
  - result: `PASS`, artifact age `0.19h`, not expired.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#162` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `ec99b7c` passed:
  - `CI` - run `25212018101`.
  - `Release Readiness Gate` - run `25212018102`.
  - `Quality Gate Policy Matrix (Optional)` - run `25212018092`.
  - `Secret Scan` - run `25212018105`.
  - `Public Naming Guard` - run `25212018093`.
  - `Governance Correlation Smoke (Optional)` - run `25212018110`.
  - `Desktop Updater Readiness (Optional)` - run `25212018091`.
  - `SonarQube Governance (Non-Blocking)` - run `25212018095`.
- First manual monitor workflow validation passed:
  - Run `25212021793`.
  - Artifact `enterprise-onboarding-readiness-artifact-monitor`.
  - Artifact ID `6748551922`.
  - Artifact status: not expired, expires at `2026-07-30T11:01:29Z`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-59 Validation Notes

- Jira: `KAN-59 - Dashboard guided enterprise onboarding checklist`.
- Implementation branch: `product/KAN-59-dashboard-guided-onboarding-checklist`.
- Implementation PR: `#172 - product(KAN-59): add guided onboarding checklist`.
- Implementation commit: `a24e34b product(KAN-59): add guided onboarding checklist`.
- Main merge commit: `d2ce33b Merge pull request #172 from yohandry10/product/KAN-59-dashboard-guided-onboarding-checklist`.
- Dashboard helper: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Dashboard UI: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Design: `docs/design/dashboard-guided-onboarding-checklist-mvp.md`.
- Report: `docs/reports/dashboard-guided-onboarding-checklist-2026-05-02.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no provider API calls.
  - no secret value printing.
  - secret names may be displayed, but values are never read or generated.
  - no GitHub Actions variable/secret creation.
  - no customer repository mutation.
  - no provider mutation.
  - no workflow dispatch or branch protection mutation.
  - advisory/non-blocking by default.
  - no release blocking by default.
- Local validation already run:
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `26` tests.
  - `npm run typecheck`: passed.
  - `npm run lint`: passed.
  - `npm test -- --run`: passed, `25` test files and `294` tests.
  - `npm run build`: passed with existing Vite large chunk warning.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#172` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `d2ce33b` passed:
  - `CI` - run `25244188759`.
  - `Release Readiness Gate` - run `25244188770`.
  - `Quality Gate Policy Matrix (Optional)` - run `25244188767`.
  - `Secret Scan` - run `25244188764`.
  - `Public Naming Guard` - run `25244188766`.
  - `Governance Correlation Smoke (Optional)` - run `25244188774`.
  - `Desktop Updater Readiness (Optional)` - run `25244188758`.
  - `SonarQube Governance (Non-Blocking)` - run `25244188762`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-58 Validation Notes

- Jira: `KAN-58 - Add dashboard onboarding remediation export`.
- Implementation branch: `product/KAN-58-dashboard-onboarding-remediation-export`.
- Implementation PR: `#170 - product(KAN-58): export onboarding remediation plan`.
- Implementation commit: `43ac78e product(KAN-58): export onboarding remediation plan`.
- Main merge commit: `4f0eff5 Merge pull request #170 from yohandry10/product/KAN-58-dashboard-onboarding-remediation-export`.
- Dashboard helper: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Dashboard UI: `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- Design: `docs/design/dashboard-onboarding-remediation-export-mvp.md`.
- Report: `docs/reports/dashboard-onboarding-remediation-export-2026-05-02.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no provider API calls.
  - no secret value printing.
  - secret names may be listed, but values are never read or generated.
  - placeholder commands use `<value>` only.
  - no GitHub Actions variable/secret creation.
  - no customer repository mutation.
  - no provider mutation.
  - no workflow dispatch or branch protection mutation.
  - advisory/non-blocking by default.
  - no release blocking by default.
- Local validation already run:
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `24` tests.
  - `npm run typecheck`: passed.
  - `npm run lint`: passed.
  - `npm test -- --run`: passed, `25` test files and `292` tests.
  - `npm run build`: passed with existing Vite large chunk warning.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#170` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `4f0eff5` passed:
  - `CI` - run `25243856927`.
  - `Release Readiness Gate` - run `25243856920`.
  - `Quality Gate Policy Matrix (Optional)` - run `25243856933`.
  - `Secret Scan` - run `25243856930`.
  - `Public Naming Guard` - run `25243856934`.
  - `Governance Correlation Smoke (Optional)` - run `25243856931`.
  - `Desktop Updater Readiness (Optional)` - run `25243856923`.
  - `SonarQube Governance (Non-Blocking)` - run `25243856915`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-57 Validation Notes

- Jira: `KAN-57 - Generate enterprise onboarding remediation plan`.
- Implementation branch: `product/KAN-57-enterprise-onboarding-remediation-plan`.
- Implementation PR: `#168 - product(KAN-57): generate onboarding remediation plan`.
- Implementation commit: `1ef7fce product(KAN-57): generate onboarding remediation plan`.
- Main merge commit: `dca7e0b Merge pull request #168 from yohandry10/product/KAN-57-enterprise-onboarding-remediation-plan`.
- Script: `scripts/control-plane/generate_enterprise_onboarding_remediation_plan.ps1`.
- Design: `docs/design/enterprise-onboarding-remediation-plan-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-remediation-plan-2026-05-02.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no secret value printing.
  - secret names may be listed, but values are never read or generated.
  - no GitHub Actions variable/secret creation.
  - no customer repository mutation.
  - no provider mutation.
  - no workflow dispatch or branch protection mutation.
  - advisory/non-blocking by default.
  - no release blocking by default.
- Local validation already run:
  - PowerShell parser check: passed.
  - `scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1` produced ExampleCo readiness status `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, and `0` blocked stages.
  - `scripts/control-plane/generate_enterprise_onboarding_remediation_plan.ps1` produced remediation status `needs-action`, `3` actions, `3` variable names, and `2` secret names with placeholder-only commands.
  - generated output scan for `Authorization`, `Bearer`, `ATATT`, `vck_`, `gho_`, `JIRA_API_TOKEN=`, `GITGOV_API_KEY=`, and `SONAR_TOKEN=`: passed with no matches.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#168` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `dca7e0b` passed:
  - `CI` - run `25243574261`.
  - `Release Readiness Gate` - run `25243574245`.
  - `Quality Gate Policy Matrix (Optional)` - run `25243574251`.
  - `Secret Scan` - run `25243574256`.
  - `Public Naming Guard` - run `25243574262`.
  - `Governance Correlation Smoke (Optional)` - run `25243574244`.
  - `Desktop Updater Readiness (Optional)` - run `25243574236`.
  - `SonarQube Governance (Non-Blocking)` - run `25243577058`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-56 Validation Notes

- Jira: `KAN-56 - Monitor enterprise onboarding readiness trend deterioration`.
- Implementation branch: `ops/KAN-56-enterprise-onboarding-readiness-trend-monitor`.
- Implementation PR: `#166 - ops(KAN-56): monitor onboarding readiness trend`.
- Implementation commit: `b120174 ops(KAN-56): monitor onboarding readiness trend`.
- Main merge commit: `89175b3 Merge pull request #166 from yohandry10/ops/KAN-56-enterprise-onboarding-readiness-trend-monitor`.
- Script: `scripts/control-plane/validate_enterprise_onboarding_readiness_trend_monitor.ps1`.
- Workflow: `.github/workflows/enterprise-onboarding-readiness-trend-monitor.yml`.
- Design: `docs/design/enterprise-onboarding-readiness-trend-monitor-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-readiness-trend-monitor-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no customer repository mutation.
  - no provider mutation.
  - no GitHub Actions variable/secret creation.
  - no workflow dispatch or branch protection mutation.
  - report-only by default.
  - no release blocking by default.
- Local validation already run:
  - command used `scripts/control-plane/validate_enterprise_onboarding_readiness_trend_monitor.ps1`.
  - source workflow file: `enterprise-onboarding-readiness-trend-report.yml`.
  - source artifact: `enterprise-onboarding-readiness-trend-report`.
  - source trend run: `25212387234`.
  - source trend artifact ID: `6748686954`.
  - result: monitor status `ready`, latest readiness status `needs-action`, score `75`, trend `stable`, `0` blocked stages, `0` findings.
  - strict mode without `-ReportOnly` exited `0` because monitor status was `ready`.
  - PowerShell parser check: passed.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#166` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `89175b3` passed:
  - `CI` - run `25212797552`.
  - `Release Readiness Gate` - run `25212797547`.
  - `Quality Gate Policy Matrix (Optional)` - run `25212797530`.
  - `Secret Scan` - run `25212797571`.
  - `Public Naming Guard` - run `25212797553`.
  - `Governance Correlation Smoke (Optional)` - run `25212797545`.
  - `Desktop Updater Readiness (Optional)` - run `25212797541`.
  - `SonarQube Governance (Non-Blocking)` - run `25212797561`.
  - scheduled `Release Readiness Gate` - run `25212844642`.
- First manual trend monitor workflow validation passed:
  - Run `25212805979`.
  - Artifact `enterprise-onboarding-readiness-trend-monitor`.
  - Artifact ID `6748834779`.
  - Artifact status: not expired, expires at `2026-07-30T11:32:21Z`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-55 Validation Notes

- Jira: `KAN-55 - Trend enterprise onboarding readiness evidence artifacts`.
- Implementation branch: `ops/KAN-55-enterprise-onboarding-readiness-trend`.
- Implementation PR: `#164 - ops(KAN-55): trend onboarding readiness artifacts`.
- Implementation commit: `1699e95 ops(KAN-55): trend onboarding readiness artifacts`.
- Main merge commit: `e5c259d Merge pull request #164 from yohandry10/ops/KAN-55-enterprise-onboarding-readiness-trend`.
- Script: `scripts/control-plane/generate_enterprise_onboarding_readiness_trend_report.ps1`.
- Workflow: `.github/workflows/enterprise-onboarding-readiness-trend-report.yml`.
- Design: `docs/design/enterprise-onboarding-readiness-trend-mvp.md`.
- Report: `docs/reports/enterprise-onboarding-readiness-trend-2026-05-01.md`.
- Runbook: `docs/runbooks/enterprise-self-service-adoption.md`.
- Safety:
  - no `.env` reads.
  - no provider secret reads.
  - no customer repository mutation.
  - no provider mutation.
  - no GitHub Actions variable/secret creation.
  - no workflow dispatch or branch protection mutation.
  - no release blocking by default.
- Local validation already run:
  - command used `scripts/control-plane/generate_enterprise_onboarding_readiness_trend_report.ps1`.
  - workflow file: `enterprise-onboarding-readiness.yml`.
  - artifact prefix: `enterprise-onboarding-readiness-`.
  - latest successful source run: `25211644692`.
  - artifact: `enterprise-onboarding-readiness-25211644692`.
  - result: parsed `1` report, latest status `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages, trend direction `stable`.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- PR `#164` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `e5c259d` passed:
  - `CI` - run `25212383270`.
  - `Release Readiness Gate` - run `25212383263`.
  - `Quality Gate Policy Matrix (Optional)` - run `25212383274`.
  - `Secret Scan` - run `25212383265`.
  - `Public Naming Guard` - run `25212383284`.
  - `Governance Correlation Smoke (Optional)` - run `25212383273`.
  - `Desktop Updater Readiness (Optional)` - run `25212383275`.
  - `SonarQube Governance (Non-Blocking)` - run `25212383279`.
- First manual trend workflow validation passed:
  - Run `25212387234`.
  - Artifact `enterprise-onboarding-readiness-trend-report`.
  - Artifact ID `6748686954`.
  - Artifact status: not expired, expires at `2026-07-30T11:15:52Z`.
- No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.

## Latest KAN-47 Validation Notes

- Jira: `KAN-47 - Add optional release governance enforcement gate`.
- Implementation branch: `ops/KAN-47-release-governance-enforcement-gate`.
- Implementation PR: `#148 - ops(KAN-47): add release governance enforcement gate`.
- Implementation commit: `b6b2854 ops(KAN-47): add release governance enforcement gate`.
- Design: `docs/design/release-governance-enforcement-gate-mvp.md`.
- Runbook: `docs/runbooks/release-governance-gate.md`.
- Report: `docs/reports/release-governance-enforcement-gate-2026-05-01.md`.
- Scope:
  - add `scripts/control-plane/validate_release_governance_gate.ps1`.
  - add manual workflow `.github/workflows/release-governance-gate.yml`.
  - keep workflow `workflow_dispatch` only; no push, PR, or scheduled blocking by default.
  - fail only when `-Enforce` is set and the KAN-46 evaluator returns `blocking=true`.
  - support optional stricter switches `-FailOnWouldBlock` and `-RequirePolicySatisfied`.
  - update CLI workflow template generation to include `release-governance-gate.yml` only for `formal-approval` plus non-`record-only` release governance.
  - update dashboard workflow template pack generation with the same inclusion rule.
- Product rule:
  - KAN-47 supplies an opt-in enforcement mechanism, not a default release blocker.
  - record-only customer profiles do not get the generated release governance gate template.
  - approval-required and quorum-required profiles can get a manual gate that defaults to enforcement because the customer explicitly selected blocking policy.
- Local validation already run:
  - report-only release governance gate script smoke against production: passed with `status=recorded`, `policy_mode=record-only`, `blocking=false`, and `would_block=false`.
  - enforced release governance gate script smoke against production: passed because current profile is `record-only`.
  - CLI workflow template generation with ExampleCo record-only profile: passed with `13` templates and no release governance gate.
  - CLI workflow template generation with quorum opt-in profile: passed with `14` templates including `.github/workflows/release-governance-gate.yml`.
  - YAML parse validation for repo workflow and generated gate template: passed.
  - `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `16` tests.
  - `npm run typecheck`: passed.
  - `npm run lint`: passed.
  - `npm run build`: passed with the existing Vite large chunk warning.
  - `npm test -- --run`: passed, `25` test files and `284` tests.
  - `git diff --check`: passed.
  - `.\scripts\security\publication_guard.ps1`: passed.
- Secret safety:
  - no provider token, `.env` value, Authorization header, webhook secret, or raw customer credential is read, printed, or stored by this change.
- No database migration, backend route change, provider setting change, customer repository mutation, or Vercel production environment change is needed.
- PR `#148` checks passed before merge:
  - `Security Guard`: passed.
  - `Server Clippy + Check`: passed.
  - `Desktop Rust Clippy`: passed.
  - `Frontend Lint + Typecheck`: passed.
  - `Website Lint + Typecheck + Build`: passed.
  - `Workflow Lint`: passed.
  - `Validate quality_gates warn/block matrix`: passed.
  - `Sonar Scan + Quality Gate`: passed.
  - `Block internal-assistant markers in branch/commits`: passed.
  - `Vercel`: passed.
  - `Vercel Preview Comments`: passed.
- Post-merge checks for commit `b6b2854` passed:
  - `CI` - run `25208426343`.
  - `Release Readiness Gate` - run `25208426384`.
  - `Quality Gate Policy Matrix (Optional)` - run `25208426354`.
  - `Secret Scan` - run `25208426359`.
  - `Public Naming Guard` - run `25208426346`.
  - `Governance Correlation Smoke (Optional)` - run `25208426363`.
  - `Desktop Updater Readiness (Optional)` - run `25208426341`.
  - `SonarQube Governance (Non-Blocking)` - run `25208426365`.
- First manual `Release Governance Gate` workflow run on `main` passed:
  - Run `25208470238`.
  - Head SHA `b6b285403455fc929eff903270bc7725a430628f`.
  - Inputs used report/non-blocking mode with `enforce_gate=false`, `fail_on_would_block=false`, and `require_policy_satisfied=false`.
  - Result: `passed=true`, HTTP `200`, `status=recorded`, `policy_mode=record-only`, `policy_enforcement=disabled`, `policy_satisfied=true`, `blocking=false`, `would_block=false`, `valid_approval_count=0`, and `required_approval_count=0`.
  - Artifact `release-governance-gate-25208470238`, ID `6747272652`, expires `2026-05-31T08:44:26Z`, not expired.

## Current KAN-28 Implementation Notes

- Workflow: `.github/workflows/product-vulnerability-review-trend-enforcement.yml`.
- Script: `scripts/control-plane/generate_product_vulnerability_review_trend_report.ps1`.
- Report: `docs/reports/product-vulnerability-trend-enforcement-2026-04-30.md`.
- Roadmap doc: `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`.
- Default enforcement rules:
  - latest parsed report failures must be `0`.
  - latest parsed report findings must be at most `1`.
  - finding count must not increase versus the oldest analyzed report.
  - failure count must not increase versus the oldest analyzed report.
  - latest successful Product Vulnerability Review run must have a parseable `product-vulnerability-review-*` artifact.
- Local validation passed against workflow run `25157972836`, artifact `product-vulnerability-review-25157972836`, with `5` pass, `1` expected finding, and `0` fail.

## Current KAN-27 Implementation Notes

- Workflow: `.github/workflows/product-vulnerability-review-trend-report.yml`.
- Script: `scripts/control-plane/generate_product_vulnerability_review_trend_report.ps1`.
- Report: `docs/reports/product-vulnerability-review-trend-2026-04-30.md`.
- Runbook: `docs/runbooks/product-vulnerability-review-automation.md`.
- The script downloads recent successful `product-vulnerability-review.yml` artifacts with prefix `product-vulnerability-review-`, parses sanitized `summary.json`, and writes Markdown/JSON trend evidence.
- Local validation analyzed workflow run `25157972836`, artifact `product-vulnerability-review-25157972836`, artifact ID `6726899384`, and produced trend status `findings` with `5` pass, `1` expected finding, and `0` fail.
- Scheduled trend report time is Friday `13:03 UTC`.

## Current KAN-26 Implementation Notes

- Workflow: `.github/workflows/product-vulnerability-review-artifact-monitor.yml`.
- Report: `docs/reports/product-vulnerability-review-artifact-monitor-2026-04-30.md`.
- Reuses `scripts/control-plane/validate_github_evidence_report_artifact.ps1`.
- The shared validator now supports `-ArtifactNamePrefix`, needed because Product Vulnerability Review artifacts are named `product-vulnerability-review-{run_id}`.
- Default max artifact age is `192` hours.
- Scheduled monitor time is Friday `12:53 UTC`.

## Current KAN-25 Implementation Notes

- Workflow: `.github/workflows/product-vulnerability-review.yml`.
- Runbook: `docs/runbooks/product-vulnerability-review-automation.md`.
- Report: `docs/reports/product-vulnerability-review-automation-2026-04-30.md`.
- Scheduled default mode: `DependenciesOnly` every Thursday at `12:41 UTC`.
- Manual modes: `DependenciesOnly`, `StaticOnly`, `RuntimeSmoke`, and `Full`.
- The KAN-24 runner is being made cross-platform for Ubuntu GitHub runners and local Windows use.

## Current KAN-24 Implementation Notes

- Master plan: `docs/security/product-vulnerability-review-plan-2026-04-30.md`.
- Live report: `docs/reports/product-vulnerability-review-2026-04-30.md`.
- Reproducible runner: `scripts/security/run_product_vulnerability_review.ps1`.
- Generated sanitized evidence directory: `docs/reports/product-vulnerability-review-2026-04-30/`.
- Main fixes in the KAN-24 branch:
  - GitHub Actions PowerShell script blocks now pass GitHub/input context through `env` instead of direct shell interpolation.
  - Frontend and website dependency advisories were remediated; `npm audit --json` and `pnpm audit --json` pass.
  - Backend and desktop Rust dependency chains were refreshed; `cargo deny check` passes for both, and desktop `cargo audit` exits 0.
  - Backend `cargo audit` still reports `rsa` through inactive `sqlx-mysql`; documented as not reachable after `cargo tree` reachability checks.
  - Windows external URL opening no longer uses `cmd /C start`.
  - Website contact API has explicit body/field bounds and PII-safe logging.
  - Website download metadata is constrained to the `public` root.
  - Website security headers were added with a Next-compatible CSP.
  - Evidence packet JSON download filenames are sanitized.
- No critical/high reachable vulnerability remained open after the latest full runner.

## Latest Workflow Fix Context

- `Risk Tier Baseline Calibration` scheduled run `24999681550` failed on 2026-04-27 because `.github/workflows/risk-tier-baseline-calibration.yml` used array splatting with `"-Param", value` pairs; PowerShell passed those positionally, so `-RepoFullName` reached the `Tier` parameter.
- `.github/workflows/desktop-updater-readiness.yml` used the same pattern and failed inside its optional job when `gitgov/src-tauri/tauri.conf.json` was bound to `TimeoutSeconds`.
- Use hashtable splatting for workflow PowerShell script blocks that call repository scripts with named parameters.
- Local validation for the fix generated a risk-tier baseline report with readiness `92/100`, composite risk `8/100`, and ran desktop updater readiness with endpoint probe skipped, returning the expected optional `WARN` state.
- Manual Risk Tier Baseline runs `25049577630` and `25049782826` on `main` confirmed the calibration step generated a report, then failed artifact upload because `report_path` was not visible to `actions/upload-artifact`; the workflow now uploads the deterministic report path directly.
- Final manual Risk Tier Baseline validation run `25049984199` passed on `main` commit `8e9b043` and uploaded artifact `risk-tier-baseline-25049984199` ID `6682824924`.

## Current Work Classification

No active implementation blocker remains after KAN-24 merge and production smoke validation.

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
4. Use a Jira-traceable branch, commit message, PR title, and Jira comment.
5. Run `.\scripts\security\publication_guard.ps1` before commit.
6. Push, open PR, wait for required checks, merge only when green.
7. After merge, pull `main`, wait for post-merge checks, and comment the Jira ticket with evidence.

## Do Not Reopen Without New Product Decision

- SonarCloud for this personal repo.
- Jenkins trigger-only token for normal agent work.
- Full OpenAPI annotation as a blocker.
- Old EC2/Nginx/systemd deployment path; Render is current production.
- Non-traceable commits or PRs.
