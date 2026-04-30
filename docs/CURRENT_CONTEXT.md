# GitGov Current Context Handoff

Updated: 2026-04-30
Ticket: `KAN-31`

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
- Any future branch, commit, and PR title must include a Jira ticket ID such as `KAN-31`.

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

## Current Product Roadmap

- Current major product feature: Enterprise Self-Service Adoption MVP (`KAN-29`/`KAN-30`/`KAN-31`).
  - KAN-29 packages the proven GitGov operating model into a reusable adoption pack generator.
  - KAN-30 adds the first dashboard profile builder with provider/module toggles, policy presets, validation, workflow/policy preview, and secret-safe JSON export.
  - KAN-31 persists adoption profiles per org with admin save/load.
  - Remaining future work: live integration validation, workflow installation, and formal release approval.
- Next major AI feature: Vercel AI SDK Copilot.
  - Explain readiness, findings, tickets, pipelines, evidence packets, accepted risks, and blockers in plain language with cited GitGov evidence.
- Current hardening step before those larger features: KAN-28 vulnerability trend enforcement.
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
