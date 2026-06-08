# KAN-69 Desktop Runtime QA

Date: 2026-06-07
Last updated: 2026-06-08
Ticket: `KAN-69`
Branch: `fix/KAN-69-action-center-desktop-qa`
Baseline: `main` at `afa4aa1 docs(KAN-69): record action center verification merge (#207)`

## Current State

KAN-69 is implemented and merged as the dedicated `/action-center` route. This report records the follow-up Desktop runtime QA work that started after the merge verification.

Current local changes are intentionally limited to:

- `gitgov/src/components/action_center/ActionCenterWorkspace.tsx`
- `gitgov/src/components/action_center/action-center-helpers.ts`
- `gitgov/src/components/auth/ControlPlaneAuthScreen.tsx`
- `gitgov/src/components/auth/LoginScreen.tsx`
- `gitgov/src/components/cli/PipelineVisualizer.tsx`
- `gitgov/src/components/cli/TerminalPanel.tsx`
- `gitgov/src/components/commit/CommitPanel.tsx`
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`
- `gitgov/src/components/control_plane/ServerConfigPanel.tsx`
- `gitgov/src/components/layout/MainLayout.tsx`
- `gitgov/src/components/layout/Sidebar.tsx`
- `gitgov/src/components/shared/LanguagePreferenceSelector.tsx`
- `gitgov/src/lib/cliEvents.ts`
- `gitgov/src/lib/controlPlaneConfig.ts`
- `gitgov/src/lib/gitIdentityPolicy.ts`
- `gitgov/src/lib/i18n.ts`
- `gitgov/src/main.tsx`
- `gitgov/package.json`
- `gitgov/package-lock.json`
- `gitgov/src/pages/ControlPlanePage.tsx`
- `gitgov/src/pages/GovernancePage.tsx`
- `gitgov/src/pages/HelpPage.tsx`
- `gitgov/src/pages/SettingsPage.tsx`
- `gitgov/src/router.tsx`

The former dashboard-only components were removed instead of left unmounted:
`ServerDashboard`, `DashboardHeader`, `DailyActivityWidget`, `RiskOutcomesWidget`, and `TicketCoverageWidget`.

- `gitgov/src/store/useAuthStore.ts`
- `gitgov/src/store/useControlPlaneStore.ts`
- `gitgov/src-tauri/src/commands/cli_commands.rs`
- `gitgov/src-tauri/src/commands/git_commands.rs`
- `gitgov/src/test/components/action-center-helpers.test.ts`
- `gitgov/src/test/components/governance-navigation.test.tsx`
- `gitgov/src/test/components/help-layout.test.tsx`
- `gitgov/src/test/components/pipeline-visualizer-product-copy.test.ts`
- `gitgov/src/test/components/settings-navigation.test.tsx`
- `gitgov/src/test/controlPlaneConfig.test.ts`
- `gitgov/src/test/gitIdentityPolicy.test.ts`
- `gitgov/src/test/i18n.test.ts`
- `gitgov/src/test/useAuthStore.test.ts`
- `gitgov/src/test/useControlPlaneStore.test.ts`

Documentation updated for this QA pass:

- `README.md`
- `docs/CURRENT_CONTEXT.md`
- `docs/AGENT_PUBLIC_CONTEXT.md`
- `docs/ARCHITECTURE.md`
- `docs/DEPLOYMENT.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/QUICKSTART.md`
- `docs/TROUBLESHOOTING.md`
- `docs/design/enterprise-action-center-guided-ux.md`
- `docs/design/enterprise-action-center-ux-focus.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`
- `docs/design/adoption-profile-dashboard-mvp.md`
- `docs/design/ai-sdk-governance-copilot-mvp.md`
- `docs/design/evidence-packets-mvp.md`
- `docs/design/formal-release-approval-mvp.md`
- `docs/design/governance-copilot-dashboard-mvp.md`
- `docs/design/release-approval-dashboard-mvp.md`
- `docs/design/release-governance-evaluator-mvp.md`
- `docs/runbooks/github-evidence-operations.md`
- `docs/reports/kan-69-desktop-runtime-qa-2026-06-07.md`
- `gitgov-web/content/docs/*` targeted public docs in English and Spanish
- `gitgov-web/lib/config/site.ts`
- `gitgov-web/lib/i18n/translations.ts`
- `gitgov-web/public/robots.txt`
- `gitgov-web/tests/e2e/download-url.mjs`
- `gitgov-web/components/marketing/FeaturesClient.tsx` copy only; no style/layout changes

Public web refresh guardrail: this pass changed information architecture copy, public documentation text, canonical URLs, and feature wording only. It did not change web styles, layout classes, section composition, animation, or visual component styling.

Do not stage or revert unrelated user documentation changes while this branch is in progress.

## Implementation Plan For This Follow-Up

This follow-up is not a new feature wave. It is a Desktop/runtime quality pass over the implemented KAN-69 product surface.

Planned implementation scope:

1. Stabilize Desktop startup/auth without forcing unnecessary repeated GitHub Device Flow.
2. Preserve the known Workspace dashboard flow: file list, CLI, pipeline visualizer, audit trail, manual commit/push controls, and `Gates / Blockers`.
3. Keep `Next Action` visible and useful; fix layout if it clips.
4. Reduce Action Center mount pressure by avoiding automatic heavy evidence refresh when opening the route.
5. Keep heavy evidence refresh behind explicit user action.
6. Validate with focused tests, typecheck, lint, whitespace checks, and publication guard before commit.
7. Do Tauri/Desktop visual validation only when the user explicitly permits runtime interaction.

Out of scope:

- no new backend endpoint.
- no new Control Plane provider mutation.
- no new workflow/template/monitor chain.
- no release-blocking behavior change.
- no provider secret handling change.
- no backend/API redesign of Workspace or Control Plane.

## Control Plane And Auth State

The Control Plane flow currently uses two different credentials with different meanings:

- GitHub local session: identifies the desktop operator.
- GitGov API key: authorizes Control Plane role, organization, and evidence access.

The product problem was not that this split exists. The product problem was that the split could feel like double-login if the app forces GitHub Device Flow repeatedly or clears Control Plane config when GitHub session state is temporarily unavailable.

Current intended behavior:

- restore a valid local GitHub session by default.
- keep the GitGov API key/url in local secure config when possible.
- explain the split in the UI instead of making it feel accidental.
- only ask GitHub again when the GitHub token is missing, invalid, expired, or the user explicitly changes account.
- only force Device Flow on every app launch when an explicit hardening env flag is enabled.

Current local code direction:

- `useAuthStore` defaults to session restore instead of forced Device Flow.
- Device Flow polling has a bounded recovery path instead of indefinite spinner behavior.
- `ControlPlaneAuthScreen` can prefill saved API key/url from server config.
- `MainLayout` no longer disconnects/clears Control Plane config just because GitHub auth is temporarily missing.
- `LoginScreen` explains the local GitHub session reuse model.
- Control Plane URL resolution is centralized and no longer forces `http://127.0.0.1:3000` only because the UI is running in Vite dev mode.
- Control Plane URL inputs remain editable in the auth gate and Control Plane settings panel.
- raw localhost network failures are normalized into actionable product messages.

Security boundary:

- this follow-up does not print, expose, or document any API key value.
- it does not change backend authorization policy.
- it does not bypass Control Plane role checks.

## Runtime Findings

The observed Desktop freeze happened while validating the authenticated Tauri app, not while running a browser-only Vite smoke.

Evidence gathered:

- Supabase dashboard showed the project healthy.
- Direct database connectivity checks succeeded.
- Local backend `/health` returned `200`.
- Action Center input endpoints returned `200` with normal latency for local validation.
- Vite served the frontend route successfully.
- The visible freeze appeared after Action Center rendered inside Tauri/Desktop.

Current working conclusion: the freeze is more likely Desktop/Tauri/WebView/client mount pressure than a Supabase or backend outage.

Current mitigation in code: opening Action Center should not automatically trigger a heavy `refreshForCurrentRole({ forceHeavy: true })` on mount. Manual `Refresh` remains the explicit path for heavier evidence refresh.

## Control Plane URL Resolution Finding

During Desktop auth validation, GitHub Device Flow completed successfully, but the second step failed against `http://127.0.0.1:3000/health`.

Observed local state at the time of this finding:

- no process was listening on port `3000`.
- Vite was listening on `::1:1420`.
- `VITE_SERVER_URL` was present but classified as a local URL; no secret values were printed.
- the displayed failure was therefore not a GitHub authentication failure.

Root cause in product code:

- `useControlPlaneStore.resolveServerConfig` forced `DEV_LOCAL_SERVER_URL` whenever `import.meta.env.DEV` was true.
- `ControlPlaneAuthScreen` and `ServerConfigPanel` duplicated that dev-only forced localhost behavior and disabled URL editing.
- this made a stale or unavailable local Control Plane an unavoidable auth blocker even when a different URL should be configured.

Corrected behavior:

- `gitgov/src/lib/controlPlaneConfig.ts` is the single URL normalization/resolution contract.
- explicit user-entered URLs win.
- stale forced localhost defaults are migrated when a configured env URL exists.
- localhost is now only a fallback/default, not a hard override.
- IPv4, `localhost`, and IPv6 loopback targets are all classified as local URLs for actionable connection messages.
- the auth gate and server settings keep the URL editable.
- raw network errors such as `error sending request for url (.../health)` are converted to an actionable message that names the target and explains whether the local Control Plane is not listening.

Validation:

- `npm run typecheck` passed.
- focused lint for Control Plane/auth/i18n files passed.
- `npm test -- src/test/controlPlaneConfig.test.ts src/test/useControlPlaneStore.test.ts src/test/useAuthStore.test.ts src/test/i18n.test.ts` passed with `50` tests.
- `cargo check` passed.
- `git diff --check` passed.

## Git Identity Policy Finding

Classification: product concept plus data/state.

During Desktop Workspace validation, the authenticated GitHub user and the effective Git author identity could differ. This is not a GitHub auth failure. GitHub auth identifies the Desktop operator, while `git config user.name` and `git config user.email` define what Git CLI/manual commits will use in the local repository.

Corrected behavior:

- the warning no longer says the Git identity differs from the user's "GitGov account".
- the warning says the effective Git identity is incomplete or not provably aligned with the authenticated GitHub user.
- the Tauri identity command now returns the effective value plus the Git config scope/source when available.
- the Workspace warning includes a `Ver prueba` action that writes read-only `git config --get user.name` and `git config --get user.email` diagnostic evidence into the GitGov CLI panel.
- the diagnostic action does not mutate `git config`, does not overwrite local/global identity, and does not print provider secrets.
- the Workspace warning recommends explicit `git config --local user.name/user.email` commands instead of the broader `scripts/setup-dev.ps1` helper, because the helper also configures repository hooks and is too broad for an identity-only warning.
- identity alignment is exact/provable rather than substring-based: `user.name` must exactly match the authenticated GitHub login or public name, or `user.email` must exactly match the public GitHub email or a GitHub noreply address for that login.
- Commit and Push remain blocked by policy until the effective Git identity is complete and verifiable against the authenticated GitHub user.

Validation:

- `npm run typecheck` passed.
- focused lint for `CommitPanel`, `gitIdentityPolicy`, and the new policy test passed.
- `npm test -- src/test/gitIdentityPolicy.test.ts` passed with `7` tests.
- `cargo check` passed.
- `git diff --check` passed.

## Control Plane Enterprise Adoption Layout Finding

Classification: layout/visual.

During Desktop Control Plane validation, the Enterprise Adoption panel showed a large empty left column while the guided checklist, provider evidence, workflow plan, and policy details continued down the right column.

Root cause:

- `EnterpriseAdoptionPanel` used one global two-column grid for the whole adoption workflow.
- The left column only contained the configuration form and ended early.
- The right column contained the long operational checklist and evidence details, so scrolling exposed a dead left column instead of usable enterprise content density.

Corrected behavior:

- the top region remains a responsive configuration plus readiness-summary area.
- the guided checklist now renders as a full-width operational section below the top configuration area.
- checklist steps use a responsive two-column grid at wide desktop sizes.
- provider health, workflow plan, policy rules, and required configuration remain visible below the checklist instead of being trapped in a narrow right rail.
- no useful onboarding, provider, workflow, policy, or `Next` action content was removed.

Validation:

- `npm run typecheck` passed.
- `npm run lint` passed.
- `npm test -- src/test/controlPlaneConfig.test.ts` passed with `6` tests.

## Control Plane Information Architecture Finding

Classification: product concept plus layout/visual.

During Desktop Control Plane validation, the page had become a vertically stacked collection of unrelated enterprise tools: daily activity, raw event breakdowns, recent commits, policy editor, adoption workflow, release approval form, export panel, and copilot. On a wide desktop display this required many full-screen scroll captures, and on a laptop it would be effectively unscannable.

Product decision:

- Control Plane is configuration, not a primary product module and not another dashboard.
- `/control-plane` is retained only as a backwards-compatible route that redirects to `/settings#control-plane`.
- Settings owns endpoint, API key, role, org scope, transport state, and Control Plane API configuration.
- A primary sidebar module, `/governance`, owns operational governance.
- Governance is split by domain:
  - `/governance/evidence`: traceability, pipeline evidence, GitHub evidence signals, evidence gaps, evidence packets, event breakdown, trend snapshots, recent commits, and audit export.
  - `/governance/policy`: policy editor and operational rules.
  - `/governance/adoption`: Enterprise Adoption profile, provider health, onboarding checklist, workflow pack, readiness, and remediation.
  - `/governance/releases`: release readiness summary, release approvals, evidence hash binding, governance evaluation, and recent decisions.
  - `/governance/copilot`: the single governance copilot surface.
- `/governance` defaults to `Evidence`; there is no generic `Dashboard` tab inside Governance.
- `DailyActivityWidget` is not mounted in `/control-plane` or Governance. Daily commits/pushes are diagnostic telemetry, not a primary GitGov product decision.
- The old overview contents were distributed by product meaning:
  - readiness moved to `/governance/releases`.
  - traceability, pipeline health, GitHub signals, and evidence gaps moved to `/governance/evidence`.
  - generic snapshot counters such as active repos/devs/tracked pushes were removed from the primary information architecture because they did not answer a governance decision by themselves.
- the redundant `Governance tools` quick-link section was removed because it linked back into the same module and made Control Plane compete with Governance.
- `ActionCenterPage` and `GovernancePage` are route-level lazy chunks, so the heavy recommendation/governance surfaces are not loaded through the main router path until their route is opened.

Next-action ownership decision:

- `/action-center` remains the only owner of the global `Next Action`.
- Workspace keeps `Current Focus` and relabels the execution hint as `Next local step`.
- `Gates / Blockers` in Workspace no longer duplicates the global `Next Action`; it only shows traceability, review gate, and CI gate.
- Enterprise Adoption uses `Next onboarding task`, because that is a customer setup task, not the global recommendation engine.

This is not a layout shortcut. The earlier rule still stands: if useful content clips, wrap/scroll/resize it instead of deleting it. The duplicate Workspace `Next Action` was removed because ownership moved to Action Center as a product/concept decision.

Validation:

- `npm run typecheck` passed.
- `npm test -- src/test/components/action-center-helpers.test.ts src/test/components/governance-navigation.test.tsx` passed with `12` tests after moving Control Plane into Settings and removing the Governance Dashboard tab.
- `npm run lint` passed.
- unmounted dashboard-only components were removed: `ServerDashboard`, `DashboardHeader`, `DailyActivityWidget`, `RiskOutcomesWidget`, and `TicketCoverageWidget`.
- `npm test` passed with `321` tests in `30` files.
- `npm run build` passed and emitted separate `ActionCenterPage` and `GovernancePage` chunks; the remaining `>500 kB` Vite warning is for the base application chunk and remains a future bundle-splitting optimization outside this Control Plane/Governance IA change.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.
- Manual Desktop smoke is intentionally not run in this pass because the user is manually validating the active Tauri/Desktop session and did not request a restart or relaunch.

## Settings Information Architecture Finding

Classification: layout/visual plus product concept.

The Settings page had become a long centered stack with large empty side gutters. The fix is organization, not removal: Settings now uses the same tab pattern as Governance while keeping every existing settings capability.

Tab ownership:

- `Preferences`: content language, Audit Trail timezone, and desktop notifications.
- `Organization`: admin onboarding, team management, API key management, and governance rules.
- `Account`: GitHub session, Control Plane role summary, logout/change user, and local PIN.
- `Repository`: current repository path, repo change action, and GitGov config JSON preview.
- `System`: Control Plane endpoint, API key, role, organization scope, transport state, Desktop updater channel, update status, changelog, download/install, retry, and manual download fallback.

Tab order:

- `Preferences`, `Organization`, `Account`, `Repository`, `System`.
- `Account` sits next to `Organization` because both are identity/access concerns.
- `System` sits last after `Repository` because it contains technical connection/update settings, not daily workflow controls.

Compatibility:

- `/settings#control-plane` opens the `System` tab.
- `/settings#updates` also opens the `System` tab for legacy hash compatibility.
- `/control-plane` still redirects to `/settings#control-plane`.
- No Settings section was removed; inactive sections are organized by tab instead of being stacked in one centered column.
- The `Organization` tab uses full-width vertical flow instead of a two-column parent grid, because the admin/team/API-key stack is much taller than the governance rules card and a two-column parent recreates the empty-side defect.

Validation:

- `npm run typecheck` passed.
- `npm run lint` passed.
- `npm test -- src/test/components/settings-navigation.test.tsx src/test/components/governance-navigation.test.tsx` passed with `10` tests.
- `npm test` passed with `331` tests in `32` files.
- `npm run build` passed; the existing Vite `>500 kB` base chunk warning remains.

## Language Runtime Finding

Classification: product concept plus data/state plus text/UI.

The language selector was not the failing state primitive. It persisted the selected language and called `i18n.changeLanguage`, but major first-class chrome still used hardcoded strings. That made Spanish appear partial immediately after clicking the Settings language button.

Fix in this pass:

- Expanded `gitgov/src/lib/i18n.ts` with first-class navigation, Settings, and Governance resource keys for English and Spanish.
- Moved Settings tab labels, section headings, body copy, updater status copy, account/repository labels, and modal copy onto `t(...)`.
- Moved primary sidebar labels and logout title/aria text onto `t(...)`.
- Moved Governance section labels/descriptions, header copy, access notice, and summary metrics onto `t(...)`.
- Updated `LanguagePreferenceSelector` so option subtitles and pending state come from i18n instead of English-only labels.

Scope note:

- This fixes the language behavior for the Settings surface where the user changes the language, plus primary navigation and Governance shell.
- Nested feature panels mounted inside Governance/Settings still need targeted i18n passes before GitGov can claim every deep module string is fully localized.

Validation:

- `npm run typecheck` passed.
- `npm run lint` passed.
- `npm test -- src/test/i18n.test.ts src/test/components/settings-navigation.test.tsx src/test/components/governance-navigation.test.tsx` passed with `12` tests.
- `npm test` passed with `327` tests in `31` files.

## Help Layout Finding

Classification: layout/visual.

Help/FAQ had the same visual defect as the old Settings page: a narrow centered document column inside a very wide Desktop viewport, leaving both sides empty and making the app look under-designed. The issue is layout, not content quality, so no FAQ categories were removed.

Fix in this pass:

- Removed the `max-w-2xl mx-auto` document-column composition.
- Replaced old `git-gov.vercel.app` links with the canonical `https://gitgov.cloud` domain.
- Added a full-width operational header with product-support context and a clear documentation link.
- Promoted the metadata/privacy guarantee into a wide first-row banner with two supporting cards.
- Added a category rail with counts and anchors for each FAQ domain.
- Rendered FAQ sections in a responsive content grid, including a sticky side rail on wide screens and a six-column FAQ grid on extra-wide screens: the two primary sections span half-width each, while the remaining three sections span one third each. This avoids a final orphan card and keeps the row balanced.

Validation:

- `npm run typecheck` passed.
- `npm run lint` passed.
- `npm test -- src/test/components/help-layout.test.tsx` passed with `3` tests.
- `npm test` passed with `332` tests in `32` files.
- `npm run build` passed; the existing Vite `>500 kB` base chunk warning remains.

## CLI Implementation Review Finding

Classification: data/state plus performance.

Desktop currently has two CLI surfaces with different semantics:

- native terminal PTY: the interactive PowerShell surface rendered by `TerminalPanel`.
- structured command/audit events: `emitCliLine`, `gitgov:cli-output`, `gitgov:cli-finished`, and Control Plane CLI command ingestion.

Findings and fixes in this pass:

- identity-proof lines are diagnostic UI evidence, not executed process commands. They now set `auditable: false`, so `AuditTrailPanel` and `PipelineVisualizer` ignore them while `TerminalPanel` still displays them.
- this prevents read-only diagnostics such as `git config --get user.email` from appearing as failed button commands in the Audit Trail.
- `cmd_execute_cli` no longer reads stdout to completion before stderr. It now drains stdout and stderr concurrently to avoid pipe backpressure deadlocks when a command writes enough data to stderr.
- `cmd_execute_cli` now keeps `command_id` on completion audit metadata and parses quoted safe-mode command arguments instead of splitting blindly on whitespace.
- native PTY startup now kills the spawned shell if initialization fails after spawn but before the session is registered, avoiding orphan shell processes.
- shell-session command submission now rejects overlapping structured commands, because the legacy shell-session attribution model has a single active command id.
- shell-session completion audit now preserves both stdout and stderr previews.
- the Windows shell-session wrapper now resets stale `LASTEXITCODE`, captures PowerShell cmdlet success/failure through `$?`, and still records native process exit codes.
- `TerminalPanel` avoids React state updates after unmount/stale session start and no longer labels an idle/no-repo terminal as `connecting...`.

Known limitation:

- the native PTY is a real interactive shell. GitGov does not currently parse every manually typed PowerShell command into structured audit records. Button-driven actions and managed command events are structured; raw PTY input/output is terminal display. A future audited-terminal mode should use explicit command submission or shell integration, not naive keystroke parsing.

Validation:

- `npm test -- src/test/gitIdentityPolicy.test.ts` passed with `7` tests.
- `npm run typecheck` passed.
- focused lint for CLI event/audit/pipeline/identity files passed.
- `cargo test commands::cli_commands::tests -- --nocapture` passed with `10` tests.
- `cargo check` passed.
- `git diff --check` passed.

## Consolidated Validation

Latest validation for the Desktop runtime QA restructuring:

- `npm --prefix gitgov run typecheck` passed.
- `npm --prefix gitgov run lint` passed.
- `npm --prefix gitgov test -- src/test/components/help-layout.test.tsx` passed with `3` tests.
- `npm --prefix gitgov test -- src/test/components/settings-navigation.test.tsx src/test/i18n.test.ts` passed with `10` tests.
- Focused Action Center/Governance/i18n/Help/Settings tests passed during the IA pass.
- Full `npm --prefix gitgov test` passed with `332` tests in `32` files.
- `npm --prefix gitgov run build` passed.
- The existing Vite `>500 kB` base chunk warning remains a future bundle-splitting optimization; Action Center and Governance are lazy route chunks.
- `pnpm --dir gitgov-web typecheck` passed after the public documentation/copy refresh.
- `pnpm --dir gitgov-web lint` passed after the public documentation/copy refresh.
- `pnpm --dir gitgov-web build` passed after the public documentation/copy refresh.
- `git diff --check` passed after the final documentation refresh.
- `.\scripts\security\publication_guard.ps1` passed after the final documentation refresh.

Manual Desktop smoke remains pending by design. The user has an active Tauri/Desktop validation session, so the agent must not restart, kill, or relaunch Desktop unless explicitly asked.

## Freeze Investigation Status

The freeze investigation is not closed yet.

Already checked:

- Supabase project status: healthy.
- database connectivity: responsive.
- backend `/health`: responsive.
- Action Center-related backend endpoints: responsive during local checks.
- Vite route serving: responsive.
- Tauri command timing observed before freeze: backend command calls completed quickly enough that they do not explain the hang alone.

Most likely current classes:

- WebView/Tauri UI thread pressure.
- React mount/render work from Action Center route.
- large state refresh triggered at route entry.
- IPC/state update pressure when multiple evidence resources refresh together.

Less likely based on current evidence:

- Supabase outage.
- local backend outage.
- single slow backend endpoint.
- missing GitGov API key.

Current mitigation:

- remove automatic heavy refresh on Action Center route mount.
- keep user-controlled refresh for explicit evidence reload.

Still needs validation:

- open Desktop after the mitigation and confirm Action Center does not freeze.
- inspect whether `Gates / Blockers` layout and `Next Action` render correctly in actual Tauri window.
- capture logs only if the user permits runtime inspection.

## Auth Product Decision

The normal Desktop product flow should not force GitHub Device Flow on every app start.

Current intended model:

- GitHub identifies the human operator in Desktop.
- The GitGov API key authorizes Control Plane role, organization, and evidence access.
- Both credentials should be explained to the user and persisted locally when available.
- Re-authentication should happen when the local GitHub token is missing, expired, invalid, or the user explicitly changes account.
- Forced Device Flow on every launch is a hardening mode only and should require an explicit env flag.

Current local changes align with that model:

- session restore is default.
- Device Flow polling has a bounded recovery path instead of an indefinite spinner.
- Control Plane API key/url state can prefill from saved server config.
- missing GitHub auth no longer automatically clears the saved GitGov Control Plane config.

## UI/Product Responsibility Rule

Before changing existing UI or behavior, classify the problem:

1. Product concept.
2. Layout/visual.
3. Data/state.
4. Performance.
5. Security.

If the issue is visual or layout, fix layout. Do not remove useful UI behavior, labels, or product information as a shortcut.

Specific KAN-69 example:

- Before the information architecture decision, `Gates / Blockers` could include `Next Action`; if it clipped, the fix was layout.
- After the information architecture decision, the global `Next Action` belongs only to `/action-center`.
- Workspace now shows `Next local step` in `Current Focus` and keeps `Gates / Blockers` focused on traceability, review gate, and CI gate.
- Enterprise Adoption uses `Next onboarding task` for setup work.

## Desktop Session Safety Rule

Do not restart, kill, or relaunch the Tauri/Desktop app while the user is manually logged in or validating a flow unless the user explicitly asks for that runtime action.

Allowed without restarting the app:

- inspect files.
- patch code.
- run typecheck/tests/lint.
- inspect process state.
- document findings.

Require explicit user instruction first:

- kill `gitgov.exe`.
- start `npm run tauri dev`.
- relaunch GitGov Desktop.
- reset local auth/session state.
- paste or manipulate local API keys.

## Open QA Items

- Validate the Action Center global `Next Action`, Workspace `Next local step`, `/settings#control-plane`, `/control-plane` redirect compatibility, and `/governance/*` routes in the actual Tauri Desktop window after the user permits runtime inspection.
- Confirm Action Center no longer freezes after removing automatic heavy refresh on mount.
- Run final commit/PR publication checks again after any additional edits.

## Non-Goals

- No SonarCloud work.
- No Jenkins trigger-only setup.
- No OpenAPI/SDK work.
- No provider mutation.
- No customer repository mutation.
- No release-blocking behavior change.
- No secret printing.
