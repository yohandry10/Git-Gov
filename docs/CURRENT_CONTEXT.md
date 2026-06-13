# GitGov Current Context Handoff

Updated: 2026-06-12
Ticket: `KAN-78` Jira planning archive migrated to GitHub Issues

Read this file first when resuming work. It is the compact operational handoff for the current GitGov state.

## Exact Current Point

- Local workspace: `C:\Users\PC\Desktop\GitGov`.
- Current planning source: GitHub Issues. The former Jira Cloud project is deactivated and should not block ongoing work.
- Active implementation ticket: GitHub issue `#295`, retitled to `KAN-80: First Governed Repo Setup MVP`.
- Active branch for KAN-80: `feature/KAN-80-first-repo-setup`. The repository publication guard rejects branch names containing `codex`, so this branch intentionally does not use the normal Codex prefix while keeping the required `KAN-80` traceability ID.
- KAN-80 implements the first concrete Deployment Gates 0.1 slice, not a broad integration wizard: one Admin-managed first-repo setup per org, stable `run_id`, repo/branch selection, provider/module/preset selection, policy/workflow preview acknowledgement, backend-normalized baseline readiness, Action Center gaps, Desktop UI under `Governance > Adoption`, and CTA into advisory gate simulation.
- KAN-80 backend route: `GET/PUT /enterprise/first-governed-repo-setup`. It is Admin-only, org-scoped like the enterprise adoption routes, rejects secret-looking baseline JSON, requires GitHub as selected provider, preserves `run_id` across upserts, and writes `upsert_first_governed_repo_setup` audit entries.
- KAN-80 migration: `gitgov/gitgov-server/supabase/supabase_schema_v32.sql` creates `enterprise_first_governed_repo_setups`.
- KAN-80 documentation: `docs/design/first-governed-repo-setup-mvp.md` and `docs/reports/first-governed-repo-setup-2026-06-13.md`.
- KAN-80 local validation run on 2026-06-13: backend check/clippy/full test (`253` tests), focused KAN-80 test with explicit `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov`, Tauri check/clippy/full test (`49` tests), frontend typecheck/lint/full test (`352` tests)/build, `git diff --check`, and `.\scripts\security\publication_guard.ps1` all passed. Vite still reports the existing large chunk warning while completing the production build.
- KAN-80 production validation on 2026-06-13: PR `#296` merged to `main` as `fae9e69`, all post-merge GitHub checks passed, Render deploy `dep-d8maj2u8bjmc73eakeq0` for `fae9e69` reached `live`, Supabase migration `v32` was applied manually through ignored `DATABASE_URL`, `GET https://gitgov-api.onrender.com/health` returned `200`/`ok`, authenticated `/stats` returned `200`, and authenticated `GET /enterprise/first-governed-repo-setup?org_name=yohandry10` returned `200` with `{"found":false}`.
- Active implementation ticket after KAN-80: GitHub issue `#298`, `KAN-81: Platform Superadmin tenant administration foundation`.
- KAN-81 decision: `Platform Founder` is a platform principal outside all tenants (`org_id=null`), not the GitGov internal tenant. `GitGov Internal` remains a normal dogfood tenant; its tenant admins cannot create sibling tenants.
- KAN-81 backend/DB shape on branch `feature/KAN-81-platform-superadmin`: `/me` returns `principal_type` and `requires_workspace_for_tenant_surfaces`; `/platform/tenants` lists/provisions tenants for Platform Founder; `/platform/tenants/{login}/lifecycle` changes tenant lifecycle; `/orgs` create remains compatibility over the same audited platform provisioning semantics; `orgs` is now the tenant catalog with `tenant_type`, `lifecycle_status`, `provisioning_source`, `provisioned_by`, `platform_metadata`, and lifecycle timestamps. Migration: `gitgov/gitgov-server/supabase/supabase_schema_v33.sql`.
- KAN-81 local validation so far: `cargo fmt --manifest-path .\gitgov\gitgov-server\Cargo.toml --check`, `cargo check --manifest-path .\gitgov\gitgov-server\Cargo.toml`, `cargo clippy --manifest-path .\gitgov\gitgov-server\Cargo.toml -- -D warnings`, focused Postgres integration tests `platform_tenant_administration_requires_founder_and_audits_lifecycle`, `create_org_requires_founder_global_admin_key`, and `org_discovery_and_me_return_human_scope`, plus full backend `cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml` (`254` tests) passed with `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov`.
- KAN-81 production validation on 2026-06-13: PR `#299` merged to `main` as `0d2e5e2`; production DB migration `v33` was applied manually through ignored `DATABASE_URL` before merge; postcheck found `8` tenant catalog columns and `3` tenant constraints; post-merge GitHub `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed; Render deploy `dep-d8mc9stckfvc73e5umn0` reached `live`; production `/health` returned `200`, authenticated `/stats` returned `200`, authenticated `/me` returned `principal_type=platform_founder` with `requires_workspace_for_tenant_surfaces=true`, and authenticated `GET /platform/tenants` returned `200` with `21` tenants and lifecycle fields present.
- KAN historical planning records were migrated to GitHub Issues on 2026-06-12:
  - `KAN-4` through `KAN-77` were created as closed historical GitHub issues `#217` through `#290`.
  - Labels created/used: `migrated-from-jira`, `historical-record`, `gitgov-recovered`, and `reconstructed-from-github`.
  - GitGov production had `73` Jira `project_tickets` snapshots (`KAN-4` through `KAN-76`) plus GitHub PR evidence; `KAN-77` was reconstructed from GitHub/GitGov evidence without a Jira snapshot.
  - Migration audit artifacts are generated under ignored `out/jira-migration-audit/` (`summary.json`, `migration-inventory.json`, `gitgov-project-tickets.json`, `github-prs.json`, and `github-created-issues.json`).
  - Active follow-up issue: GitHub issue `#291`, title `KAN-78: Record Jira to GitHub Issues migration`.
- Expected branch before new work: `main`; latest validated main commit is `e4bec3f fix(KAN-77): align Render Docker context for policy core (#215)`.
- KAN-77 implementation PR `#214` merged as `0acfd26 security(KAN-77): harden event capture and policy as code (#214)`.
- KAN-77 Render hotfix PR `#215` merged as `e4bec3f fix(KAN-77): align Render Docker context for policy core (#215)`.
- Render deploy `dep-d8lsqf7avr4c73fsemlg` for `0acfd26` failed because the previous Render root/context `gitgov/gitgov-server` excluded the new sibling `gitgov/policy-core` crate, so Cargo could not read `/policy-core/Cargo.toml`.
- Render service `gitgov-api` was updated through the Render API to `rootDir=gitgov`, Docker context `.`, and Dockerfile `gitgov-server/Dockerfile`; local `docker-compose.yml` now uses the same context shape.
- Render deploy `dep-d8lsul8k1i2s73dk1ph0` for `e4bec3f` reached `live` on 2026-06-12, `/health` returned `status=ok`, and authenticated `/stats` returned HTTP `200`.
- Production `supabase_schema_v31.sql` was applied after PR `#216` exposed a `/policy/yohandry10%2FGit-Gov` database error in `Validate quality_gates warn/block matrix`; `v31` now drops/recreates `get_policy_history(UUID, INTEGER)` so the migration is re-runnable when the OUT row changes.
- After applying `v31`, `source_metadata` exists on `policies`, `policy_history`, and `policy_change_requests`; `get_policy_history` exists once; authenticated production `GET /policy/yohandry10%2FGit-Gov` returned HTTP `200`; local rerun of `scripts/jenkins/validate_quality_gate_policy_matrix.ps1` against production passed.
- Post-merge GitHub checks for `e4bec3f` passed, including `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix (Optional)`, `Governance Correlation Smoke (Optional)`, `Desktop Updater Readiness (Optional)`, and `SonarQube Governance (Non-Blocking)`.
- Latest KAN-72 audit baseline: PR `#193` merged as `655478e`, handoff refresh PR `#194` merged as `2ab821e`, and stable wording PR `#195` merged as `0ccef26`.
- Latest completed KAN-24 implementation baseline: `126167f security(KAN-24): product vulnerability review and hardening (#97)`.
- KAN-24 implementation PR: `#97` - `security(KAN-24): product vulnerability review and production hardening`.
- KAN-24 post-merge context refresh PR: `#98` - `docs(KAN-24): record post-merge validation`.
- Recent prior PR: `#96` - `docs(KAN-23): record evidence packet merge validation`.
- Treat commit/PR fields in this file as validated KAN-24 implementation and validation baselines, not an auto-updating source of truth for later docs-only refresh commits; always run `git status --short --branch` and `git log -1 --oneline main` before new work.
- Worktree expectation before new work: clean and aligned with `origin/main`.
- Implementation-status backlog is closed. Remaining items are operational decisions, optional future enhancements, or evidence hygiene.
- Completed ticket chain: `KAN-25` through `KAN-68` (vulnerability review automation, enterprise self-service adoption, release governance, onboarding readiness/remediation, and route auth smoke chains), `KAN-69 - Enterprise Action Center guided UX` (product/UX), and `KAN-70` through `KAN-76` (documentation reality audits and public agent context). Per-ticket titles are in `Recent Ticket Chain` below; per-ticket implementation/validation notes live in `docs/reports/current-context-kan-notes-archive-2026-06-09.md`.
- `KAN-70`, `KAN-71`, `KAN-72`, `KAN-73`, `KAN-74`, and `KAN-75` were documentation/CI hygiene follow-ups. They audited living documentation against actual repository state before returning to `KAN-69`.
- `KAN-75` scope: public web docs, roadmap/context/product-state docs, and systematic cleanup of stale public claims that were not covered by the backend/API, Desktop/dashboard, or workflows/scripts/ops audit phases.
- `KAN-76` scope: publish a sanitized public agent-readable context bridge so external models can understand current product state without force-adding restricted forensic/strategy docs.
- KAN-69 implementation PR: `#204 - product(KAN-69): add guided Action Center workspace`.
- KAN-69 implementation commit on main: `aa7e352 product(KAN-69): add guided action center workspace (#204)`.
- KAN-69 implementation shape: dedicated `/action-center` desktop route, sidebar navigation entry, deterministic `Goal + Evidence + Permission` recommendations, and deep links into existing Control Plane/Workspace surfaces. It is not another panel inside Workspace or Enterprise Adoption.
- KAN-69 verification follow-up PR: `#206 - fix(KAN-69): harden Action Center verification logic`.
- KAN-69 verification follow-up commit on main: `8a55a6d fix(KAN-69): harden action center verification logic (#206)`.
- KAN-69 follow-up verification: `docs/reports/enterprise-action-center-verification-2026-06-07.md` records the product/infrastructure Q/A review. The follow-up fixed release prep so missing or empty Jira coverage remains conservative before Evidence Packet/release decision guidance, and it avoids known-forbidden admin-only adoption-profile/checklist reads for non-admin users.
- KAN-69 Desktop runtime QA is completed and merged to `main` through PR `#209` (`fix/KAN-69-desktop-runtime-qa-maintainability`) and PR `#211` (`fix/KAN-69-control-plane-workspace-auth`); latest main commit `e0c769d`. Report: `docs/reports/kan-69-desktop-runtime-qa-2026-06-07.md`.
- The merged Desktop QA code changes were limited to Action Center mount behavior, Desktop auth/session UX, Workspace pipeline visualizer layout/copy, Control Plane technical connection/configuration UX, Governance information architecture, Control Plane Enterprise Adoption layout, and focused auth/navigation/product-copy tests.
- Desktop QA implementation approach (executed): stabilize startup/auth, preserve Workspace local execution flow, keep heavy evidence refresh explicit, reduce Action Center route mount pressure, move Control Plane connection/configuration into Settings, keep `/control-plane` only as a redirect to `/settings#control-plane`, move operational governance to `/governance/*`, keep `Governance > Evidence` first with no generic Governance Dashboard tab, keep Action Center as the only global `Next Action` owner, and validate without relaunching Desktop unless the user permits runtime interaction.
- Current Control Plane/auth decision: GitHub identifies the Desktop operator; the GitGov API key authorizes Control Plane role/org/evidence. Restore valid local GitHub sessions by default, preserve saved Control Plane config, explain the split in UI, and reserve forced Device Flow on every launch for explicit hardening mode.
- Current Control Plane workspace/auth implementation: Desktop now treats GitHub identity, GitGov API key authorization, and active workspace/tenant as separate product concepts. Scoped API keys get `org_name` from `/me`; global/founder Admin keys with `org_id=null` must validate an active workspace via `/orgs/{login}` before admin tenant surfaces unlock. The active workspace is persisted locally per GitHub login, Control Plane URL, and a non-secret API-key fingerprint. `/orgs` now lists visible workspaces, `/orgs/{login}` validates scope, and `/api-keys?org_name=...` is scoped while unqualified `/api-keys` remains the explicit global Admin catalog.
- Runtime QA finding: Supabase and local backend health were validated as healthy; the observed Action Center freeze is more likely Desktop/Tauri/WebView/client mount pressure than database or backend failure. Opening Action Center must not trigger heavy background refresh automatically; manual Refresh remains the explicit path for heavier evidence refresh.
- Runtime QA product decision: Desktop should reuse a valid local GitHub session by default. GitHub identifies the human operator; the GitGov API key authorizes Control Plane role/org/evidence. The two-step model is acceptable only when it is explained and persisted; it should not force GitHub Device Flow on every app start unless an explicit hardening env flag enables that behavior.
- Runtime QA Control Plane URL finding: GitHub Device Flow can succeed while Step 2 fails if Desktop is forced to `http://127.0.0.1:3000` and no local Control Plane is listening. The fix direction is centralized URL resolution, editable Control Plane URL fields, localhost as fallback only, IPv4/localhost/IPv6 loopback detection for local-target hints, and actionable connection errors instead of raw `Network error ... /health` output.
- Runtime QA Git identity finding: classify as product concept plus data/state. GitHub auth identifies the Desktop operator, while effective `git config user.name/user.email` controls Git CLI/manual commit authorship. The Workspace warning should say "effective Git identity incomplete/not provably aligned", not "cuenta GitGov"; `Ver prueba` writes read-only `git config --get` evidence to the GitGov CLI panel; no automatic `git config` mutation is allowed. The warning should recommend explicit `git config --local user.name/user.email`, not the broader `scripts/setup-dev.ps1` helper, because that script also configures repo hooks. Identity alignment is exact/provable only: login or public name must exactly match `user.name`, or `user.email` must match the public GitHub email or GitHub noreply pattern, including numbered noreply addresses. Commit/Push remain blocked by policy until the effective Git identity is complete and verifiable against the authenticated GitHub user.
- Runtime QA CLI finding: classify as data/state plus performance. Diagnostic CLI proof lines must be visible in the terminal but not treated as executed/audited commands; `emitCliLine` now supports `auditable: false`, and Audit Trail/Pipeline ignore those lines. `cmd_execute_cli` now drains stdout/stderr concurrently to avoid pipe backpressure deadlocks, preserves `command_id` on completion audit metadata, and parses quoted arguments instead of whitespace-splitting safe-mode commands. Native PTY startup kills spawned shells if initialization fails before registration. Legacy shell-session now rejects overlapping structured commands, captures stderr previews, and has a safer PowerShell exit wrapper. Native PTY manual input remains raw terminal I/O, not command-by-command structured audit; future audited-terminal work should use explicit command submission or shell integration, not naive keystroke parsing.
- Runtime QA UI rule: before removing UI or behavior, classify the problem as concept, layout, data/state, performance, or security. If useful content clips or overflows, fix layout/scroll/wrap instead of deleting it. The current Workspace `Gates / Blockers` removal of global `Next Action` is a product/concept decision, not a visual workaround: Action Center is the only global `Next Action` owner, Workspace uses `Next local step`, and Adoption uses `Next onboarding task`.
- Runtime QA Control Plane layout finding: classify as layout/visual. Enterprise Adoption must not keep a long guided checklist and evidence detail rail inside one narrow right column while the left form column ends early. The fix is responsive composition: top configuration/readiness, then full-width checklist/evidence sections, with no useful onboarding or `Next` action content removed.
- Runtime QA Control Plane information architecture finding: classify as product concept plus layout/visual. Control Plane is configuration, not a primary product module. The sidebar no longer shows Control Plane; Settings owns endpoint, API key, role, org scope, and transport state; `/control-plane` remains only as a compatibility redirect to `/settings#control-plane`. Operational governance moved to sidebar route `/governance`, where `/governance` defaults to `Evidence` and there is no `/governance/dashboard` tab. Governance sections are `/governance/evidence`, `/governance/policy`, `/governance/adoption`, `/governance/releases`, and `/governance/copilot`. Former overview contents were redistributed: traceability, pipeline health, GitHub signals, and evidence gaps live in Evidence; release readiness lives in Releases; generic snapshot counters such as active repos/devs/tracked pushes were removed from the primary IA. `DailyActivityWidget` is not mounted because daily commits/pushes are diagnostic telemetry, not a primary product decision surface. `ActionCenterPage` and `GovernancePage` are route-level lazy chunks to reduce main router load.
- Runtime QA Settings layout finding: classify as layout/visual plus product concept. Settings now uses Governance-style tabs instead of a long centered column. Tabs are ordered `Preferences`, `Organization`, `Account`, `Repository`, and `System`. `System` merges the former `Connection` and `Updates` surfaces: Control Plane endpoint/API key/role/scope/transport plus Desktop updater. Account sits next to Organization in the tab order, and System sits last after Repository. No settings capability was removed; `/settings#control-plane` and legacy `/settings#updates` both land on the System tab.
- Runtime QA Organization tab layout finding: classify as layout/visual. The first Settings tab layout pass incorrectly used a two-column parent grid for Organization even though its left admin/API-key stack is much taller than the governance-rules card. That recreated the same empty-right-column defect. Organization now uses a full-width vertical flow; Repository remains the only Settings tab that uses the two-column parent grid when config preview is present.
- Runtime QA Help layout finding: classify as layout/visual plus product concept. Help/FAQ had the same centered-document problem as the old Settings view, and its links still pointed at the old `git-gov.vercel.app` URL. The fix keeps all FAQ sections, removes the narrow `max-w-2xl mx-auto` composition, uses `https://gitgov.cloud` links, adds full-width operational header cards, category navigation, and a responsive FAQ grid (`xl` side rail, `2xl` six-column content grid: first two sections half-width, remaining three sections one-third width) so wide Desktop windows do not leave a final orphan card or dead side space.
- Runtime QA language finding: classify as product concept plus data/state plus text/UI. The language selector persisted and switched i18next correctly, but Settings, Governance, and primary sidebar chrome still used hardcoded labels, so Spanish appeared partial. Local fix expanded `gitgov/src/lib/i18n.ts` and moved first-class Settings/Governance/sidebar labels, descriptions, and status copy onto translation keys. Nested feature panels still need targeted i18n coverage before claiming every deep module string is localized.
- Runtime QA security/business-logic follow-up: classify as security plus data/state plus product concept. Desktop no longer treats `/stats` success as a fallback Admin identity when `/me` fails; role context must come from `/me`. Control Plane API key persistence no longer fails silently: keyring errors keep the session disconnected/degraded with an explicit error. Control Plane URL validation now rejects invalid schemes, embedded credentials, and non-loopback `http://` before persistence. CLI command auditing now redacts URL credentials, bearer/API/token/password/secret values, common GitHub/GitLab/OpenAI-style token prefixes, and stdout/stderr previews before outbox or direct Control Plane ingestion.
- Runtime QA Settings/Governance policy follow-up: classify as product concept. Organization Settings no longer mounts a second governance policy editor; it keeps organization onboarding/team/API-key administration and links to `Governance > Policy` as the single policy owner. Organization admin UI is gated by Control Plane `Admin` role, not local GitHub admin state. `GovernanceRulesPanel` dirty-state tracking now includes forbidden patterns so those changes can be saved when the panel is used.
- Runtime QA Release/Governance performance follow-up: classify as security plus performance plus product concept. Release approvals no longer default to the real `yohandry10/Git-Gov` repository or `KAN-43` when no profile/evidence context exists, and evidence URIs now allow relative API paths or `https://` only. Governance route entry no longer performs the previous heavy role refresh that pulled daily activity plus 500 logs; it loads base stats and defers the smaller log window to `Governance > Evidence`.
- Runtime QA performance follow-up: classify as performance plus data/state. Default governance/event refresh windows are capped at `120` logs instead of `500`, `dailyActivity` is no longer loaded by general dashboard refresh because it has no mounted product consumer, SSE refreshes are batched for `1000 ms`, incremental log refreshes are serialized to avoid overlapping store/API work, and the Workspace pipeline visualizer deduplicates concurrent graph/signal refreshes while limiting Control Plane signal pulls to `50` records per source. Manual explicit heavy refresh still keeps the heavy evidence path available when needed. Validation passed with frontend typecheck, lint, focused store/settings/config tests, full frontend tests (`333` tests), build, and `git diff --check`.
- Runtime QA header refresh follow-up: classify as product concept plus data/state. The global Workspace `Actualizar` button mixed local repository refresh with an interactive Control Plane `checkConnection()` call, so a transient `/me` role/context revalidation could replace the Workspace with the Control Plane access screen before returning. The global button was removed. Header connection checks remain background-only; repo status refresh is handled by route polling and explicit local actions, while manual Control Plane reconnect belongs in Settings/System.
- Local maintenance cleanup: `gitgov/gitgov-server/target_forensic` was inventoried in `docs/reports/target-forensic-cleanup-2026-06-08.md` and removed as a local Rust/Cargo forensic/debug build artifact directory. It contained generated `.rlib`, `.pdb`, `.o`, `.exe`, `.dll`, and incremental cache files, not source, docs, migrations, tests, or runtime configuration.
- Runtime QA validation after Control Plane/Governance/Settings/Help/i18n restructure: after moving Control Plane into Settings, removing the Governance Dashboard tab, deleting the unmounted dashboard-only components, organizing Settings into tabs, making Settings/Governance/sidebar chrome language-reactive, widening Help/FAQ, correcting Organization to full-width flow, merging Connection/Updates into the final System tab, and moving Help links to `gitgov.cloud`, `npm --prefix gitgov run typecheck`, `npm --prefix gitgov run lint`, focused Settings/Governance/i18n/Help layout tests (`17` tests), full frontend tests (`332` tests in `32` files), `npm --prefix gitgov run build`, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. Build still reports the existing Vite `>500 kB` base chunk warning; Action Center and Governance emit separate chunks. Manual Desktop smoke remains pending by design because the active Tauri/Desktop session must not be restarted or relaunched without explicit user instruction.
- Runtime QA documentation/web refresh finding: README, architecture, quickstart, troubleshooting, deployment, public agent context, implementation status, Action Center design docs, GitHub evidence runbook, and public web docs were aligned with the new IA. Public web copy now uses canonical `https://gitgov.cloud`, describes Governance/Action Center instead of the old Admin Dashboard/Control Plane dashboard, and preserves page/component styling while changing only informational copy/URLs.
- Runtime QA operating rule: do not restart, kill, or relaunch the Tauri/Desktop app while the user is manually logged in or validating unless the user explicitly asks for that runtime action.
- Runtime QA maintainability rule: hand-maintained source files should not become giant mixed-responsibility modules. Single Responsibility Principle comes first: split files that mix UI, fetch/state, business rules, data transforms, templates, and types even before they become huge. Practical targets are 300-600 lines for most source files, 800 lines as the normal upper bound, and 1,200 lines only as an exceptional justified ceiling. UI components/pages should usually stay around 150-350 lines and split before 500-600 lines; domain helpers should usually stay around 200-500 lines and below 800; tests can be larger when they cover one coherent module but should normally stay below 800-1,000; type/interface files may grow only while they remain one clear domain contract. When reducing an existing large file, split by responsibility and keep a compatibility facade if existing imports depend on the old path. Generated outputs, lockfiles, vendored artifacts, fixtures, and historical reports are exempt.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/integration_tests.rs` giant backend integration-test file was split into a small facade plus focused modules under `gitgov/gitgov-server/src/integration_tests/` by responsibility: shared helpers, auth, events/admin, policy enforcement, coverage/compliance, and alerts/exports/policy requests. No endpoint behavior was intentionally changed.
- Runtime QA maintainability refactor: the former `gitgov/src/components/control_plane/dashboard-helpers.ts` giant Control Plane helper file was split into a compatibility facade plus focused helper modules under `gitgov/src/components/control_plane/dashboard-helpers/`. Existing imports from `dashboard-helpers.ts` remain valid; the split is organizational only and keeps adoption/profile/workflow/policy/release helper behavior intact.
- Runtime QA maintainability refactor: the former `gitgov/src-tauri/src/control_plane/server.rs` giant Desktop Control Plane client/DTO file was split into a compatibility facade plus `server/models/*` domain DTO modules and `server/client/*` endpoint-group modules. Public import path remains `crate::control_plane::server::*`/`crate::control_plane::*`; no endpoint URL, payload shape, or store/backend behavior was intentionally changed. Validation passed with `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml`, `cargo test --manifest-path gitgov/src-tauri/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`31` tests), `git diff --check` for the split files, no-BOM check, and a public surface comparison confirming `82` public DTO/error types and `46` public client methods before and after.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/db.rs` backend database layer was split by SRP into modules under `gitgov/gitgov-server/src/db/` while preserving the old import path through the `db.rs` module facade. Per the staged migration plan, the original full file is still retained inside `db.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, public surface comparison (`17` public types and `144` public methods unchanged), and full function-name coverage (`180/180`, no missing function names).
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/models.rs` backend model contract file was split by domain into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/models/`. Per the same staged migration plan used for `db.rs`, the original full file is still retained inside `models.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, module/file parity (`32` modules and `32` files), archive integrity (`0` uncommented archive lines), and public surface comparison (`167` public types, `4` public functions, and `1` public const unchanged). The largest live model module is now `tests.rs` at `378` lines; the large `models.rs` facade size is temporary archive evidence only.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/chat_handler.rs` backend chat orchestration file was split into a live include facade plus focused modules under `gitgov/gitgov-server/src/handlers/chat_handler/` for helpers, short-circuit intents, query families, and the public `chat_ask` route handler. Per the staged migration plan, the original full file is still retained inside `chat_handler.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with explicit `rustfmt --edition 2021` over the included module files, `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, publication guard, archive exact-match comparison against the original `HEAD` file (`3795` archived lines), archive integrity (`13` `PART` markers, `0` uncommented archive lines), `ChatQuery` dispatch coverage (`31` variants, `0` missing), and public surface comparison confirming only `pub async fn chat_ask` remains public in the split module.
- Runtime QA maintainability refactor: the former `gitgov/src/store/useControlPlaneStore.ts` giant Desktop Control Plane Zustand store was split into a compatibility facade plus focused modules under `gitgov/src/store/useControlPlaneStore/`. The root file still exports the same public path and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` uncommented archive lines. Live modules now separate constants, types, helpers, runtime in-flight guards, initial state, and action slices for connection/auth, dashboard/evidence, enterprise/adoption/releases/export, organization/team/API keys, chat/copilot, and policy/SSE. The largest live module after the second-level split is `types.ts` at `695` lines; action modules are `150-530` lines and the live `store.ts` composer is `19` lines. Validation passed with `npm run typecheck`, `npm run lint`, focused store/config/settings tests (`36` tests), full frontend tests (`333` tests), `npm run build`, `git diff --check`, no-BOM check, archive integrity check, and `.\scripts\security\publication_guard.ps1`. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/main.rs` backend crate-root/server bootstrap file was split into a small crate-root facade plus focused modules under `gitgov/gitgov-server/src/server/`. The root `main.rs` still declares crate modules, calls `server::run().await`, and retains the original full file as a line-commented migration archive with `8` `PART` markers and `0` uncommented archive lines. The archive reconstructs the `HEAD` original exactly (`2188` lines). Live modules now separate env/CLI config, rate limiting, HTTP middleware, distributed SSE listener, job worker, route composition, startup/runtime orchestration, and the moved rate-limit tests. The largest live module is `startup.rs` at `748` lines, followed by `rate_limit.rs` at `450` and `routes.rs` at `411`; `main.rs` size is temporary archive evidence only. Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), route-path parity check against `HEAD`, no-BOM check, `git diff --check`, and exact archive comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/client_ingest_dashboard.rs` backend handler bundle was split into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/handlers/client_ingest_dashboard/`. The root file still exports the same handler names through `include!` and retains the original full source as a line-commented migration archive with `5` `PART` markers and `0` live function declarations. The archive reconstructs the `HEAD` original exactly (`1886` lines). Live modules now separate client event ingest, outbox lease telemetry/acquisition, stats/log/repo lookup caches, dashboard/log/team query handlers, and policy-check helpers/endpoint. Current live module sizes are `policy_check.rs` (`491` lines), `cache.rs` (`460`), `dashboard_queries.rs` (`406`), `ingest.rs` (`338`), and `outbox_lease.rs` (`191`). Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, `git diff --check`, and exact archive comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/github_webhook.rs` backend GitHub webhook handler bundle was split into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/handlers/github_webhook/`. The root file still exports `handle_github_webhook` and all existing private helpers/tests through `include!`, and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` live declarations. The archive matches the concatenated live modules exactly (`1749` lines). Live modules now separate webhook entry/signature validation, push/create/review processing, generic check/status repository evidence, PR review-comment/issue-comment correlation helpers, PR merge/approval processing plus repo upsert, and existing webhook unit tests. Current live module sizes are `pr_comments.rs` (`452` lines), `pr_events.rs` (`368`), `repo_evidence.rs` (`303`), `push_create_review.rs` (`264`), `entry.rs` (`191`), and `tests.rs` (`171`). Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, `git diff --check`, and live/archive concatenation comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/src-tauri/src/commands/cli_commands.rs` Desktop CLI command bundle was split into a compatibility facade plus focused modules under `gitgov/src-tauri/src/commands/cli_commands/`. The root file still exports the same `commands::cli_commands::*` surface and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` live declarations. The archive matches the concatenated live modules exactly (`1785` lines), preserving local Desktop runtime QA changes. Live modules now separate CLI types/managers, parsing/env/redaction/audit helpers, shell-session commands, native-terminal commands, structured command execution, and whitelist/pipeline graph/tests. Current live module sizes are `helpers.rs` (`529` lines), `shell_session.rs` (`384`), `execute.rs` (`285`), `types.rs` (`201`), `native_terminal.rs` (`199`), and `pipeline.rs` (`187`). Validation passed with `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`, `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`, `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/src-tauri/Cargo.toml --no-run`, full Tauri `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`31` tests), no-BOM check, `git diff --check`, and live/archive concatenation comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- KAN-69 local validation: `npm --prefix gitgov run typecheck`, focused Action Center helper tests (`8` tests), full frontend tests (`304` tests in `26` files), `npm --prefix gitgov run lint`, `npm --prefix gitgov run build`, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. Browser/Vite smoke for `/action-center` returned HTTP `200`; full authenticated UI validation remains a Tauri/Desktop runtime concern.
- KAN-69 post-merge checks on `main` commit `aa7e352` passed: `CI` run `27086413044`, `Release Readiness Gate` run `27086413043`, `Secret Scan` run `27086413053`, `Public Naming Guard` run `27086413041`, `SonarQube Governance (Non-Blocking)` run `27086413042`, `Quality Gate Policy Matrix (Optional)` run `27086413040`, `Governance Correlation Smoke (Optional)` run `27086413050`, and `Desktop Updater Readiness (Optional)` run `27086413038`.
- KAN-69 verification follow-up post-merge checks on `main` commit `8a55a6d` passed: `CI` run `27100640858`, `Release Readiness Gate` run `27100640831`, `Secret Scan` run `27100640840`, `Public Naming Guard` run `27100640856`, `SonarQube Governance (Non-Blocking)` run `27100640837`, `Quality Gate Policy Matrix (Optional)` run `27100640835`, `Governance Correlation Smoke (Optional)` run `27100640862`, and `Desktop Updater Readiness (Optional)` run `27100640864`.
- Any future branch, commit, and PR title must keep the `KAN-*` traceability ID. New planning records should be opened in GitHub Issues unless Jira Cloud is deliberately reactivated later.

## Local Security Review (uncommitted, 2026-06-09)

A multi-surface security review was run against the backend and several findings were
fixed locally in the working tree (not yet committed; no Jira ID was attached because the
Jira account was temporarily disabled). Validated against a local Postgres (`5434`, since a
native Windows Postgres collides on `5433`) with the full suite green and `clippy -D warnings`
clean. Fixed: traceability coverage now counts only Jira-verified tickets (not pattern-only
matches); multi-tenant org scoping closed on the `integrations.rs` read/evidence/correlation
endpoints, the Jira/Jenkins status aggregates, `append_project_ticket_relations*`, and the
`commit_ticket_correlations` uniqueness (new migration `supabase_schema_v26.sql`); SSE is now
org-scoped per subscriber, admin-gated, and invalidates caches per-org; the governance copilot
was hardened so the LLM cannot forge deterministic-provenance refs and the system prompt has an
explicit prompt-injection guardrail.

### Finding E1 — client-controlled event timestamp (Medium)

- **Issue**: `POST /events` stored `created_at` directly from the client-supplied
  `input.timestamp` (`handlers/client_ingest_dashboard/ingest.rs`) with no bound. A
  server-authoritative `synced_at` exists on `client_events` but no governance query uses it —
  coverage, release readiness, daily activity, `?hours=N` windows and log ordering all filter by
  the client `created_at`. An authenticated client could postdate/backdate events to move them
  in or out of reporting windows or corrupt audit ordering.
- **Fix applied**: ingest now rejects events whose timestamp is more than `5` minutes in the
  future (`event_timestamp_too_far_in_future`, `EVENT_FUTURE_SKEW_MS`). Past timestamps remain
  allowed on purpose — the offline outbox legitimately backfills older events. Tests:
  `client_event_timestamp_in_future_is_rejected` (unit) and
  `events_with_future_timestamp_are_rejected` (integration).
- **Residual / recommended next step**: future-rejection closes postdating but not backdating
  within the past (bounding the past at ingest would break the offline outbox). The complete
  anti-evasion fix is to anchor security-sensitive time windows on the server `synced_at`
  (or `GREATEST(created_at, synced_at)`) instead of the client `created_at`. Tracked as
  follow-up, not yet implemented.
- **Org-invitation identity binding (FIXED)**: `accept` previously let the acceptor override the
  invited identity via `login` and mutate an existing `org_user`. Now the invited identity is
  authoritative: `OrgInvitation::resolved_accept_login()` resolves the login from `invite_login`
  (then the `invite_email` local-part) and never from acceptor input; `accept_org_invitation` no
  longer takes a `requested_login`; and the handler rejects a mismatched acceptor `login` with
  `400 "login does not match the invitation target"` (`accepts_requested_login`). Unit tests in
  `models/tests.rs` cover both helpers, including the spoofing case being rejected.

### Finding W1 — webhook replay via unsigned delivery_id (Low/Medium)

- **Issue**: GitHub/Jira HMAC signs only the request body; `X-GitHub-Delivery` /
  `X-GitHub-Event` are unsigned, sender-controlled headers. Idempotency keyed on
  `github_events.delivery_id` meant a captured, validly-signed payload could be replayed with a
  fresh `delivery_id` and re-injected as duplicate audit evidence (which feeds coverage,
  readiness, and PR-merge correlations). No replay/timestamp window existed. The raw signed
  payloads are also persisted in `webhook_events`.
- **Fix applied**: content-bound idempotency. New migration `supabase_schema_v30.sql` adds
  `webhook_events.payload_sha256` + a unique index. `handlers/github_webhook/entry.rs` now hashes
  the signed material (`SHA256(event_type ‖ raw_body)`) and `store_webhook_event` returns a
  `WebhookIngestDecision`. A content-hash collision is only skipped when the prior occurrence was
  already processed successfully (`processed = TRUE`); a prior delivery whose processing FAILED is
  returned for reprocessing, so a transient failure is not silently lost (retry-safety). A replay
  with a fresh `delivery_id` but the same already-processed signed body collides on the content
  hash and is skipped. Test:
  `webhook_replay_with_fresh_delivery_id_is_deduped_by_content_hash` (integration) covers both the
  retry-safe and the dedup paths. The harness `webhook_events` table was aligned (it was missing
  `signature` / `payload_sha256`).
  - Self-review note: the first cut of this fix skipped processing on ANY content-hash collision,
    which would have permanently dropped an event whose first delivery stored the row but then
    failed processing (GitHub retries would be answered `200 duplicate`). Corrected to the
    processed-aware decision above.

### Finding W2 — Jira webhook stale-replay overwrite (Low)

- **Issue**: `upsert_project_ticket` used `ON CONFLICT (org_id, ticket_id) DO UPDATE` with no
  version guard, so replaying an older Jira webhook overwrote a newer ticket state
  (last-write-wins).
- **Fix applied**: the `DO UPDATE` now carries
  `WHERE project_tickets.updated_at IS NULL OR EXCLUDED.updated_at IS NULL OR EXCLUDED.updated_at >= project_tickets.updated_at`,
  so a strictly-older replay is ignored. Validated directly against the production-shaped schema
  (the harness `project_tickets` is drifted — `project_key NOT NULL`, no `ticket_url`/`title` — so
  `upsert_project_ticket` cannot run against it; verified via `psql`: a stale replay returns
  `INSERT 0 0` and the newer status is preserved).

### Migration numbering note

Local webhook idempotency migration is `supabase_schema_v30.sql`. Migrations `v27`–`v29`
(api-key role integrity, release-approval evidence-packet binding, push-outcome event fidelity)
are separate concurrent local work; `v26` is the `commit_ticket_correlations` org-scoped
uniqueness from the multi-tenant fix.

### Multi-tenant join hardening — loose `org_id IS NULL` pattern (FIXED)

- **Issue**: several SQL joins across the correlation/coverage/noncompliance queries matched org
  with a loose predicate, e.g. `X.org_id = Y.org_id OR X.org_id IS NULL`. A second-pass review
  found 6 instances. None was a cross-tenant breach — the driving table is always strictly scoped
  (`WHERE <pk>.org_id = $N`), so org A never sees org B; the loose branch only let `org_id IS NULL`
  (unowned/legacy) rows bleed into a scoped result, and could over- or under-count.
- **Fix applied**: every instance was tightened to the strict form
  `(X.org_id IS NULL AND Y.org_id IS NULL) OR X.org_id = Y.org_id`. Locations: `db/jira_coverage.rs`
  orphan-ticket join and the `get_commit_pipeline_correlations` lateral; `db/noncompliance_detection.rs`
  in `detect_v2_commit_no_ticket_signals`, `detect_v2_stale_in_progress_signals` (×2), and
  `detect_v2_done_not_deployed_signals`. The remaining coverage/flow joins were already strict from
  concurrent local work.
- **Validation**: a repo-wide grep confirms zero loose column-to-column `org_id IS NULL OR` patterns
  remain. Because the noncompliance orchestrator swallows V2 SQL errors (`Err(e) => warn!`), a green
  suite alone does not prove SQL validity, so the most complex rewritten query was `EXPLAIN`-checked
  against the production-shaped schema and plans correctly. Build + `clippy -D warnings` + full suite
  (`230` tests) green. Post-migration/scoping/E1 work means `org_id IS NULL` data rows should no
  longer be produced, so the change is defense-in-depth with no expected effect on current data.

### KAN-77 Event capture fidelity (local implementation)

- **Scope implemented**: Desktop/Tauri branch and checkout capture now emits only backend-supported
  event types. `cmd_create_branch` no longer writes unsupported `attempt_create_branch` or
  `branch_failed`; failed branch creation is a `create_branch` event with `failed` status, blocked
  branch creation is `blocked_branch`, and successful checkout now emits `checkout_branch` with
  actor, repo/org, branch, HEAD SHA, and `from_branch`/`to_branch` metadata.
- **Remote parser**: `repo_event_context` no longer depends only on `origin`; it prefers the current
  branch upstream remote, then `origin`, then other configured remotes, while still accepting only
  parseable GitHub SSH/HTTPS remote URLs and rejecting ambiguous/non-GitHub remotes.
- **Backend guardrail**: `/events` now rejects evidence-bearing Desktop events that are incomplete:
  `stage_files`, branch/checkout, commit, and push/governance push events must carry
  `repo_full_name` and `branch`; commit/push evidence events must carry `commit_sha`; `stage_files`
  must include at least one file. Non-evidence telemetry such as heartbeat/login is not made
  artificially strict.
- **Desktop native terminal correction**: the Workspace terminal is a core product surface and
  must be operational by default in Desktop. A local hardening pass had accidentally changed
  `GITGOV_ENABLE_NATIVE_TERMINAL` from opt-out to opt-in, leaving the Workspace terminal offline
  unless the variable was set to `true`. Restored the product contract: native PTY is enabled by
  default, `GITGOV_ENABLE_NATIVE_TERMINAL=false` remains an explicit restricted-runtime opt-out,
  and `TerminalPanel` now treats that explicit opt-out as a degraded configuration state instead
  of a repeated red technical error.
- **Tests/validation run locally**: `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml`
  (`230` passed), `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`,
  `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`,
  `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`,
  `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`,
  `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`47` passed),
  `npm --prefix gitgov test -- --run` (`345` passed), `npm --prefix gitgov run typecheck`,
  `npm --prefix gitgov run lint -- --quiet`, `npm --prefix gitgov run build` (existing Vite
  chunk-size warning only), `git diff --check`, and manual Desktop validation confirming the
  Workspace terminal starts as `powershell` at the repo prompt.

## Documentation Intake - 2026-06-12

Session request: read as much repository documentation as practical and preserve the operating
context. Files reviewed included `AGENTS.md`, `README.md`, `CONTRIBUTING.md`,
`docs/CURRENT_CONTEXT.md`, `docs/AGENT_PUBLIC_CONTEXT.md`, `docs/IMPLEMENTATION_STATUS.md`,
`docs/ARCHITECTURE.md`, `docs/QUICKSTART.md`, `docs/DEPLOYMENT.md`,
`docs/TROUBLESHOOTING.md`, `docs/OPERATIONS_ACCESS.md`, `docs/PUBLICATION_POLICY.md`,
`docs/QUALITY_GATE_POLICY_VALIDATION.md`, current Action Center and roadmap design docs,
recent KAN-69/KAN-70/KAN-71/KAN-72/KAN-73/KAN-74/KAN-75/KAN-76 reports, integration-test
harness drift notes, enterprise adoption/GitHub evidence/release governance runbooks,
`gitgov/README.md`, `gitgov/gitgov-server/README.md`, `gitgov-web/README.md`, public web
content headings, and `gitgov-web/CONTENT_ARCHITECTURE_GUIDE.md`.

Key preserved context:

- This worktree already had substantial uncommitted `KAN-77` changes before the documentation
  intake. Treat them as existing local work and do not revert or overwrite them casually.
- Current branch observed during intake: `security/KAN-77-event-capture-fidelity`.
- `main` observed at `e1cba5d security(KAN-77): harden webhook replay idempotency (#213)`.
- Product direction remains consolidation, not another default hardening/report chain. New work
  should improve usability, package existing capabilities into a clearer workflow, fix a real bug,
  close a confirmed security/production risk, or support an explicit customer-selected policy.
- Desktop information architecture remains: `/action-center` owns the global `Next Action`;
  Workspace owns local execution and `Next local step`; Governance owns Evidence, Policy, Adoption,
  Releases, and Copilot; Settings/System owns Control Plane connection, API key, role/scope,
  transport, and updater configuration; `/control-plane` is compatibility redirect only.
- Desktop runtime safety remains non-negotiable: do not restart, kill, or relaunch Tauri/Desktop
  during a user's manual validation session unless the user explicitly asks.
- Publication safety remains non-negotiable: no token values, no real `.env` files, no restricted
  forensic/strategy docs force-added; use `docs/AGENT_PUBLIC_CONTEXT.md` as the public bridge.
- Enterprise onboarding and release-governance tooling is dry-run/report-only by default; mutations
  require explicit reviewed flags such as `-Apply`, and release blocking is customer opt-in.
- The integration-test harness drift report leaves an active durability concern: the backend
  integration harness still relies on a hand-maintained inline schema and CI does not exercise it
  with `TEST_DATABASE_URL`; durable fix is to apply real migrations or add schema parity plus a CI
  Postgres service.
- Some living docs are historical snapshots and may cite older route/test/migration counts. Prefer
  this handoff plus current repo inspection when facts differ from older KAN reports.
- No external-service validation or secret-bearing env-file inspection was performed during this
  documentation intake.

Additional deep intake from the same 2026-06-12 session:

- Public website/docs context: `gitgov-web` is the presentation layer, not the product source of
  truth. Future public copy should follow `docs/IMPLEMENTATION_STATUS.md`,
  `docs/ARCHITECTURE.md`, `README.md`, and actual Desktop/backend behavior before older web copy.
  The commercial category to preserve is "engineering governance with operational evidence".
- Public content risk noted for future cleanup, not changed in this intake: some public docs still
  carry historical wording such as Jira coverage/correlation marked as `Preview` in CI trace
  tables and Desktop commit capture described as including "message". Current security/product
  context is more precise: Jira API plus signed native webhook are operational, and GitGov must not
  imply source content or diff bodies leave the workstation.
- Release governance defaults are a closed product decision: default mode is `record-only`.
  Advisory status is allowed; blocking release gates, approval-required mode, quorum, and
  environment-specific overrides require explicit customer-selected policy. Do not infer blocking
  just because release approval records exist.
- Enterprise onboarding/readiness/remediation/checklist features are evidence and workflow guidance
  surfaces. Readiness reports, remediation plans, checklist tracking, artifact monitors, and trend
  reports must not read secret values, create GitHub variables/secrets, mutate provider settings,
  dispatch workflows, alter branch protection, or make release blocking the default.
- Remote workflow installation and readiness tooling is deliberately review-first: dry-run/plan by
  default, remote PR mutation only with explicit `-Apply`, overwrite only with explicit
  `-Overwrite`, and readiness validators compare workflow hashes/configuration names without
  reading GitHub Actions secret values.
- Product vulnerability review status remains: no critical/high reachable product vulnerability was
  left open after KAN-24. The recurring expected scanner finding is the inactive `sqlx-mysql`/`rsa`
  path classified as not reachable; if MySQL/sqlx-mysql features are enabled later, revisit that
  classification. Website contact/download rate limiting and ecosystem dependency warnings remain
  maintenance/deferred hygiene, not current blockers.
- Restricted local forensic/strategy docs (`docs/ENTERPRISE_READINESS_DECISION.md`,
  `docs/AUDIT_*.md`, `docs/INTEGRATIONS_AUDIT_*.md`) were inspected only as local memory. Do not
  force-add or quote them into public context; extract only sanitized current conclusions into
  tracked docs when needed.
- Older readiness and integration audit notes still support the current direction: enterprise value
  is traceable evidence, risk/readiness outcomes, and deterministic governance. SSO/SCIM, broader
  MCP, and autonomous AI agency remain deal-driven or future work, not default next steps.
- Route auth smoke chains (KAN-61 through KAN-68) are a guardrail family for enterprise route
  authorization and artifact freshness/trends. Treat them as regression evidence; do not add new
  monitor/enforcement chains unless they protect a concrete route/security risk or a selected
  customer policy.
- Policy-as-Code product decision captured in
  `docs/design/policy-as-code-flexible-source-mvp.md`: keep one canonical internal
  `GitGovConfig` model, but let customers choose `control-plane-managed`,
  `repo-policy-as-code`, or `hybrid-advisory` source mode and support TOML/YAML/JSON repo policy
  files. Implementation is merged on `main` through PR `#214`: `gitgov/policy-core` provides shared
  TOML/YAML/JSON parsing, discovery, canonical JSON, checksum, semantic diff, and real Git PR
  validation; backend/Tauri reexport the shared model; `supabase_schema_v31.sql` adds
  `source_metadata`; overrides and policy requests use canonical checksums; merged PR webhooks can
  activate the exact policy blob from GitHub when a token is configured; Governance displays policy
  source and blocks silent direct overrides for `repo-policy-as-code`. OPA/Rego is now supported as
  an optional external adapter, not as the default embedded engine: policy config can define
  `adapters.opa.*`, `enforcement.external_policy`, `effect = advisory|required`,
  `failure_mode = fail-open|fail-closed`, Data API `decision_path`, result mapping, timeout, and
  `token_env_var` by env-var name only. `/policy/check` calls the configured external OPA Data API
  when enabled, sends repo/branch/commit/actor, policy source metadata, and the native GitGov
  result, then returns OPA evidence under `external_decisions`. Required OPA plus
  `external_policy=block` can deny; advisory OPA never blocks. OPA response mapping supports boolean
  `allow`, custom allowed keys, boolean `deny`, and common Rego `deny` collections. An OPA `200`
  response without a mapped boolean decision is treated as adapter failure and obeys
  `fail-open`/`fail-closed`, matching the official Data API behavior where an undefined document can
  return `200` without `result`. Runtime and committed OPA URLs reject inline credentials/token query
  strings, query/fragment suffixes, invalid ports, and non-loopback `http://`; loopback checks parse
  the host so `localhost.example.com` / `127.0.0.1.example.com` are not accepted. Stored policy change
  requests are revalidated and checksum-checked again at approval time before activation. Local
  validation after the second OPA pass: policy-core tests `12` passed, backend OPA adapter tests `10`
  passed including real HTTP mock OPA Data API calls, backend policy-change approval tests `3`
  passed, full backend tests `250` passed, Tauri tests `49` passed, frontend tests `349` passed,
  policy-core/backend clippy `-D warnings` passed, Tauri `cargo check` passed, frontend typecheck/lint
  passed. Targeted `policy_check` integration tests compile the OPA endpoint path but still depend on
  a dedicated `TEST_DATABASE_URL` for non-skipped DB-backed runtime coverage. Remaining work:
  Governance patch/PR proposal UX,
  explicit emergency override UX, periodic drift comparison, Evidence Packet source metadata,
  customer examples/schema docs, controlled GitHub API activation test, persisted OPA decision audit
  history/export, and a real `opa run --server` smoke script. Production packaging was completed by
  PR `#215`, which aligned Render and local Docker context with the new sibling crate layout.
- KAN-77 security/event-fidelity and flexible Policy-as-Code work is no longer local-only. It is
  merged to `main`, production deployed on Render deploy `dep-d8lsul8k1i2s73dk1ph0`, and validated
  with `/health`, authenticated `/stats`, and post-merge GitHub checks.

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
- Render root directory: `gitgov`.
- Render Docker context: `.` within `gitgov`.
- Render Dockerfile path: `gitgov-server/Dockerfile`.
- The Docker build context must include both `gitgov-server` and `policy-core` because the backend depends on `gitgov-policy-core` through a relative Cargo path.
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

## Archived Ticket Notes

- Historical per-ticket implementation/validation notes (KAN-24 through KAN-68) were moved verbatim to `docs/reports/current-context-kan-notes-archive-2026-06-09.md` to keep this handoff compact.
- Treat archived notes as evidence snapshots for completed tickets, not as active backlog.

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
