# GitGov Implementation Status

Updated: 2026-06-16

## KAN-140 Native Terminal Branch Gate Status Advisory - 2026-06-16

`KAN-140 - Native Terminal Branch Gate Status Advisory` is completed. GitHub issue `#487`
shipped through PR `#488`, merged to `main` as `b3b21f66`.

Product decision:

- Add a compact branch gate status badge to the Desktop native terminal header.
- Keep it advisory-only and visually quiet.
- Do not block terminal commands, commits, pushes, PRs, deployments, or any other local/manual
  workflow.
- Reuse existing Deployment Gate evidence instead of adding a new enforcement model.

Implemented:

- Added `terminalBranchGateStatus.ts` for pure status mapping.
- Added `TerminalBranchGateStatusBadge` beside the existing native terminal repo/branch label.
- Reused `buildTerminalGovernanceTarget` so the badge uses the same safe GitHub `owner/repo` target
  as KAN-135 Governance Context.
- Reused existing read-only Tauri command `cmd_server_list_deployment_gate_authorizations` with
  `limit=1`, `repository_full_name`, and `branch`.
- Added focused tests in `gitgov/src/test/components/terminal-branch-gate-status.test.tsx`.
- Added design/report docs for the advisory-only UX.

Guardrails:

- No backend/API route change.
- No DB migration.
- No Control Plane audit write.
- No command interception, approval, blocking, or auto-run.
- No commit/push/deploy enforcement.
- No provider, repository, or deployment mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.

Validation:

- Focused branch gate test passed:
  `npm --prefix gitgov run test -- --run src/test/components/terminal-branch-gate-status.test.tsx`
  (`6` tests).
- Focused terminal suite passed:
  `npm --prefix gitgov run test -- --run src/test/components/terminal-branch-gate-status.test.tsx src/test/components/terminal-governance-context.test.ts src/test/components/terminal-git-context.test.ts src/test/components/terminal-quick-commands.test.ts`
  (`18` tests).
- Frontend typecheck and lint passed.
- Full frontend Vitest passed (`410` tests).
- Frontend build passed with the pre-existing Vite large chunk warning.
- `git diff --check`, publication guard, and static no-mutation grep passed.
- PR checks passed, including Security Guard, Frontend Lint + Typecheck, Desktop Rust Clippy,
  Server Clippy + Check, Website Lint + Typecheck + Build, Validate Policy-as-Code, Validate
  quality_gates warn/block matrix, Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal
  marker guard.
- Post-merge `main` checks passed: CI, Release Readiness Gate, Secret Scan, Public Naming Guard,
  Quality Gate Policy Matrix, Governance Correlation Smoke, Desktop Updater Readiness, and
  SonarQube Governance.
- No Render/API deploy was required because KAN-140 is Desktop/frontend local and reuses existing
  read endpoints.

## KAN-139 Correct OpenAPI Route-Source Technical Debt - 2026-06-16

`KAN-139 - Correct OpenAPI route-source technical debt` is completed. GitHub issue `#484`
shipped through PR `#485`, merged to `main` as `34e6a540`.

Product/technical decision:

- Keep `/api-docs` intentionally partial; do not implement full generated SDK/contract coverage in
  this slice.
- Correct the technical debt where the OpenAPI disclaimer and backend docs still pointed operators
  to the old `main.rs` route table after route composition moved to `src/server/routes.rs`.
- Record the current verified route count as `158` Axum `.route(...)` registrations.

Implemented:

- Updated the generated OpenAPI description and unit guard to reference
  `gitgov-server/src/server/routes.rs`.
- Updated living backend/architecture/status/context docs to use `src/server/routes.rs` as the
  operational route source of truth.
- Added report `docs/reports/openapi-route-source-technical-debt-2026-06-16.md`.

Guardrails:

- No new route behavior.
- No DB migration.
- Render may redeploy only to refresh the served `/api-docs` description.
- No API route behavior change.
- No full OpenAPI path annotation rollout or SDK generation.

Validation:

- Local validation passed: focused OpenAPI unit test, backend `cargo fmt --check`, backend
  `cargo check`, `git diff --check`, publication guard, route-source stale grep, and route-count
  verification (`158`).
- Required PR checks passed, including Workflow Lint, Security Guard, Server Clippy + Check,
  Desktop Rust Clippy, Frontend Lint + Typecheck, Website Lint + Typecheck + Build,
  Validate Policy-as-Code, Validate quality_gates warn/block matrix, Sonar Scan + Quality Gate,
  Vercel, and internal marker guard.
- Render deploy `dep-d8oo81k2m8qs73augv70` reached `live`; production `/api-docs/openapi.json`
  returned HTTP `200`, contained `gitgov-server/src/server/routes.rs`, and no longer contained
  `main.rs`.

## KAN-137 Remove External Editor Extension Direction - 2026-06-16

`KAN-137 - Remove external editor extension product direction` is completed. GitHub issue `#479`
shipped through PR `#480`, merged to `main` as `be9aed9e`.

Product decision:

- Remove the external editor extension path from the active roadmap/current repository.
- Keep `0.10 Developer Distribution Surfaces` focused on Desktop/Workspace native terminal surfaces.
- Reopen external editor plugins only through a future explicit product decision.

Implemented:

- Deleted the dedicated external editor package from the tracked repo.
- Removed the dedicated extension CI job.
- Removed the dedicated design/report docs for that direction.
- Updated roadmap/context/status docs so the active direction is Desktop/Workspace terminal only.

Guardrails:

- No backend migration or new Control Plane API.
- No Render/API deploy expected.
- No provider/repo/deploy mutation.
- No product claim that external editor plugins are part of the active roadmap.

Validation:

- Local `git diff --check`, publication guard, and repository grep for the removed direction passed.
- Required PR checks passed, including Workflow Lint, Security Guard, Frontend Lint + Typecheck,
  Desktop Rust Clippy, Server Clippy + Check, Website Lint + Typecheck + Build, Validate
  Policy-as-Code, Validate quality_gates warn/block matrix, Sonar Scan + Quality Gate, Vercel, and
  internal marker guard.

## KAN-135 Native Terminal Governance Context - 2026-06-16

`KAN-135 - Native Terminal Governance Context Panel MVP` is completed. GitHub issue `#473` shipped
through PR `#474`, merged to `main` as `b9dbb57c`.

Product decision:

- Add a read-only Governance Context panel to the Desktop native terminal after KAN-132 session
  history, KAN-133 repo/branch context, and KAN-134 safe quick commands.
- Reuse existing Deployment Gate, Change Risk, and Executive Governance read endpoints through
  existing Tauri commands.
- Keep the terminal as a convenience surface only. It does not approve, block, certify, deploy,
  execute commands, or create a second enforcement model.

Implemented:

- Added `terminalGovernanceContext.ts` to derive a safe `owner/repo` target from KAN-133 Git context
  plus existing repo validation remote metadata.
- Added `TerminalGovernanceContextPanel` with a `Context` drawer in the native terminal header.
- The panel loads latest Deployment Gate authorization, latest Change Risk evaluation, and
  Executive Governance posture for the detected repository/branch.
- Added safe states for pending Git context, non-git directories, missing GitHub remotes, Control
  Plane not configured, permission denied, empty governance data, loading, and success.
- Extracted `TerminalSessionHistoryDrawer` so `TerminalPanel` remains focused and below the normal
  maintainability ceiling while preserving KAN-132 behavior.

Guardrails:

- No backend migration or new Control Plane API.
- No backend persistence or audit write.
- No command execution, command interception, command approval, command blocking, or auto-run.
- No mutating Git/provider/deploy command.
- No provider/repo/deploy mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.
- No absolute local path exposure in governance target labels.

Validation:

- Focused terminal governance tests prove GitHub remote parsing, no cwd leak, pending/non-git/missing
  remote empty states, and evidence detection from gate/risk/executive rows.
- Focused terminal regression tests cover KAN-132 history, KAN-133 git context, and KAN-134 quick
  commands.
- Focused terminal test set passed (`18` tests).
- Frontend typecheck, lint, build, and full Vitest passed (`404` frontend tests).
- Tauri fmt/check/clippy/full tests passed (`52` Tauri tests).
- `git diff --check` and publication guard passed.
- Static grep verified the new panel does not write to the terminal PTY and does not call mutating
  Control Plane commands.
- PR checks passed: Security Guard, Frontend Lint + Typecheck, Desktop Rust Clippy, Server Clippy +
  Check, Website Lint + Typecheck + Build, Validate Policy-as-Code, Validate quality_gates
  warn/block matrix, Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal marker guard.
- No Render/API deploy was required because KAN-135 reuses existing read endpoints and changes
  Desktop/frontend only.

## KAN-134 Native Terminal Safe Quick Commands - 2026-06-16

`KAN-134 - Native Terminal Safe Quick Commands MVP` is completed. GitHub issue `#470` shipped
through PR `#471`, merged to `main` as `6b8faef9`.

Product decision:

- Add a local quick-command palette to the Desktop native terminal after KAN-132 session history and
  KAN-133 repo/branch context.
- Keep quick commands insert-only. The user must still press Enter manually.
- Limit the MVP to read-only Git inspection commands. This is a developer convenience surface, not
  automation, enforcement, release approval, policy evidence, or deploy execution.

Implemented locally:

- Added `terminalQuickCommands.ts` with a hardcoded read-only allowlist:
  `git status --short`, `git branch --show-current`, `git log --oneline -5`, `git diff --stat`, and
  `git remote -v`.
- Added structural rejection for compound, redirected, non-git, and mutating commands.
- Added `TerminalQuickCommandsMenu` with preview, disabled non-git state, and recent commands used
  in the current session.
- Updated `TerminalPanel` to insert selected quick commands into the native PTY without newline or
  auto-execution, while updating the KAN-132 draft so manual Enter still records history.

Guardrails:

- No backend migration or Control Plane API.
- No backend persistence or audit write.
- No auto-run, command interception, command approval, or command blocking.
- No mutating commands such as push, pull, fetch, merge, rebase, checkout, commit, reset, deploy, or
  apply.
- No provider/repo/deploy mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- Branch gate status was intentionally out of scope for KAN-134; the advisory-only version is later
  handled by KAN-140.
- No compliance/certification/legal/regulatory claim.

Validation:

- Focused quick-command/history/git-context tests prove allowlist shape, mutating command rejection,
  non-git disablement, no cwd exposure in labels, insert-only text with no newline, and KAN-132
  manual submission capture after the user presses Enter (`14` passed).
- Frontend typecheck, lint, build, and full Vitest passed (`400` frontend tests).
- Tauri fmt/check/clippy/full tests passed (`52` Tauri tests).
- `git diff --check` and publication guard passed.
- PR checks passed.
- Post-merge `main` checks passed: CI, Release Readiness Gate, Secret Scan, Public Naming Guard,
  Quality Gate Policy Matrix, Governance Correlation Smoke, Desktop Updater Readiness, and
  SonarQube Governance.
- No Render/API deploy is expected because KAN-134 is local Desktop/frontend only.

## KAN-133 Native Terminal Repo/Branch Context - 2026-06-16

`KAN-133 - Native Terminal Repo/Branch Context MVP` is completed. GitHub issue `#467` shipped
through PR `#468`, merged to `main` as `5d45c2dd`.

Product decision:

- Show safe local Git repo/branch context in the Desktop native terminal header.
- Keep it as a developer convenience surface only. It is not command interception, policy
  enforcement, audit evidence, release approval, release blocking, or compliance evidence.

Implemented locally:

- Added local Tauri command `cmd_get_native_terminal_git_context`.
- Added Rust helpers to detect non-git/git/detached context using `git2::Repository::discover`
  without executing Git commands.
- Added safe cwd inference for simple directory-change commands (`cd`, `chdir`, `sl`,
  `Set-Location`) and rejection of compound shell commands.
- Updated `TerminalPanel` to show `repo:branch`, detached, pending, or non-git context labels while
  preserving KAN-132 session history.
- Added frontend helpers/tests for refresh trigger detection and safe labels.

Guardrails:

- No backend migration or Control Plane API.
- No backend persistence or audit write.
- No command blocking, approval, interception, or automatic re-run.
- No Git push/pull/fetch/checkout.
- No provider/repo/deploy mutation.
- No quick commands, branch gate status, or policy preview.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.

Local validation:

- Tauri `cargo fmt --check`.
- Focused Tauri tests for non-git/real git context and safe `cd` resolution.
- Tauri `cargo check`, `cargo clippy -- -D warnings`, and full Tauri tests (`52` passed).
- Focused frontend terminal git-context/history tests.
- Frontend `typecheck`, `lint`, build with the pre-existing Vite chunk warning, and full Vitest
  (`395` passed).
- `git diff --check` and publication guard.
- PR checks passed: Security Guard, Frontend Lint + Typecheck, Desktop Rust Clippy, Server Clippy +
  Check, Website Lint + Typecheck + Build, Validate Policy-as-Code, Validate quality_gates
  warn/block matrix, Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal marker guard.
- No Render/API deploy was required because KAN-133 is local Desktop/Tauri/frontend only.

Report: `docs/reports/native-terminal-git-context-2026-06-16.md`.

## KAN-132 Native Terminal Session History - 2026-06-16

`KAN-132 - Native Terminal Session History MVP` is completed. GitHub issue `#464` shipped through
PR `#465`, merged to `main` as `86d70861`.

Product decision:

- Add a local/session-scoped native terminal command history to Desktop Workspace.
- Keep it as a developer convenience surface only. It is not policy, audit evidence, enforcement,
  approval, release blocking, or compliance evidence.

Implemented locally:

- Added `terminalSessionHistory.ts` with pure native-terminal input parsing and capped history
  helpers.
- Updated `TerminalPanel` with a compact history button/drawer showing session command count,
  command text, shell, repo, branch, and timestamp.
- Added focused tests for Enter submission, pasted multi-command input, Backspace, Ctrl+C, ANSI
  navigation sequences, newest-first retention, empty command rejection, and safe metadata defaults.

Guardrails:

- No backend migration.
- No Render/API deploy requirement.
- No command interception or automatic re-run.
- No Control Plane audit write.
- No provider/repo/deploy mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.

Local validation:

- `pnpm --dir gitgov exec vitest run src/test/components/terminal-session-history.test.ts src/test/components/terminal-status.test.ts` (`9` tests passed).
- `pnpm --dir gitgov typecheck`.
- `pnpm --dir gitgov lint`.
- `pnpm --dir gitgov build` passed with the pre-existing Vite chunk-size warning.
- PR checks passed: Security Guard, Frontend Lint + Typecheck, Desktop Rust Clippy, Server Clippy +
  Check, Website Lint + Typecheck + Build, Validate Policy-as-Code, Validate quality_gates
  warn/block matrix, Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal marker guard.

Report: `docs/reports/native-terminal-session-history-2026-06-16.md`.

## KAN-131 Multi-Repo Executive Governance Snapshot Export - 2026-06-16

`KAN-131 - Multi-Repo Executive Governance Snapshot Export` is completed. GitHub issue `#459`
shipped through PR `#460`, with production archive-contract hardening follow-ups PR `#461` and
PR `#462`. Final `main` commit: `44e2a492`.

Scope:

- Create/list/get/download/archive executive governance snapshots.
- Persist `executive_governance_snapshots` through Supabase migration `v68`.
- Reuse KAN-130 filtered `GET /executive/repositories` as the only source.
- Add Tauri/store/Desktop controls and focused real tests.

Guardrails: read-only, manual-first, advisory-only, no scoring, no enforcement, no deploy
execution, no provider/repo mutation, no AI/Agent Governance dependency, and no
compliance/certification/legal claim.

## KAN-130 Multi-Repo Executive Governance Filters - 2026-06-16

`KAN-130 - Multi-Repo Executive Governance Filters MVP` is completed. PR `#457` merged to `main` as
`6d1bcf4f`.

Product decision:

- Extend the KAN-129 executive repository view with read-only filters over existing governance
  evidence.
- Keep the feature manual-first and advisory-only: filters are executive triage, not deployment
  authorization, enforcement, compliance scoring, certification, or AI/Agent Governance.
- Do not add a migration, provider mutation, repository mutation, deployment execution, release
  blocking, automatic CAB artifact creation, risk recalculation, or legal/regulatory claims.

Implemented locally:

- Extended `GET /executive/repositories` query with `repository`, `environment`, `posture`,
  `gate_decision`, `risk_level`, and `review_status`.
- Added backend validation for filter enum values and safe text filters.
- Updated DB aggregation to apply filters over existing Deployment Gate and Change Risk evidence,
  with CAB packet/manifest counts derived through filtered evaluations.
- Updated Tauri DTOs, Control Plane store query types, and Governance > Releases executive panel
  filter controls.
- Design and validation report docs.

Validation and production:

- Local backend fmt/check/clippy/no-run passed during implementation.
- Focused real Postgres test
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped` passed with snapshot
  create/list/get/download/archive, hash recomputation, RBAC, tenant isolation, missing-name
  validation, archived download conflict, and no-mutation assertions.
- Tauri fmt/check/clippy/tests passed.
- Frontend typecheck/lint/full Vitest/focused store and panel tests passed.
- Production `v68` migration/postcheck passed.
- PR checks passed for PRs `#460`, `#461`, and `#462`.
- Final Render deploy `dep-d8olc88jo6nc73b94n4g` for `44e2a492` reached `live`.
- Final production smoke passed: `/health=ok`, filtered executive view returned
  `yohandry10/Git-Gov` with posture `review`, snapshot
  `egs_06f228f93f184aeeb182e5932b98f4cc` was created/downloaded/archived, artifact hash
  `sha256:27e21be0854ecd8ad459551176f8de3aab6b487ec902896d7139923a9dcfb24c` recomputed
  successfully, archived download returned HTTP `409`, and source governance counts stayed
  unchanged at `2,6,8,6,7`.
- Aggressive smoke found the archive route initially required `name` due to DTO reuse. PR `#461`
  split the archive DTO, and PR `#462` hardened deserialization so missing create names return
  controlled `400` validation while archive with only `org_name` is accepted.

Local validation completed:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Focused backend real Postgres test
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped` passed with baseline,
  posture/environment, gate decision, risk level, review status, repository search, conflicting
  filter, invalid enum, RBAC, tenant isolation, and no-mutation assertions.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri tests (`49` passed).
- Frontend `npm --prefix gitgov run typecheck`.
- Frontend `npm --prefix gitgov run lint`.
- Frontend full Vitest (`384` passed).
- Focused frontend store/panel tests (`50` passed).
- Frontend build passed with the pre-existing Vite large chunk warning.
- `git diff --check`.
- `scripts/security/publication_guard.ps1`.

Production validation:

- No production migration was required.
- Post-merge `main` checks passed for `6d1bcf4f`, including CI, Release Readiness Gate, Quality Gate
  Policy Matrix, Secret Scan, Public Naming Guard, Governance Correlation Smoke, Desktop Updater
  Readiness, and SonarQube Governance.
- Render deploy `dep-d8ojmp58nd3s73ai40e0` for `6d1bcf4f` reached `live`.
- Production smoke passed with `/health=ok`, authenticated
  `GET /executive/repositories?org_name=yohandry10&limit=10` returning `repositories=1`, first
  repository `yohandry10/Git-Gov`, first posture `review`, filtered
  `environment=production&posture=review` returning `repositories=1`,
  `repository=Git-Gov&risk_level=medium` returning `repositories=1`, conflicting
  `gate_decision=blocked&risk_level=low` returning `repositories=0`, invalid `posture=critical`
  returning HTTP `400`, and safe no-claim flags.
- Production no-mutation check passed: Deployment Gate authorizations, Change Risk evaluations, CAB
  packets, CAB decision manifests, and Agent Governance evaluations stayed `2,6,8,6,7` before and
  after filtered executive reads.

Report: `docs/reports/multi-repo-executive-governance-filters-2026-06-16.md`.

## KAN-129 Multi-Repo Executive Governance View - 2026-06-16

`KAN-129 - Multi-Repo Executive Governance View MVP` is completed. PR `#454` merged to `main` as
`bcf8e8f8`.

Product decision:

- Add a tenant-scoped executive overview of repository governance posture using existing Deployment
  Gate, Change Risk, CAB packet, and CAB decision manifest evidence.
- Keep the feature manual-first and read-only: it is an executive triage view, not a deployment
  approval, enforcement gate, compliance score, certification, or agent/AI workflow.
- Do not add a migration, provider mutation, repository mutation, deployment execution, release
  blocking, automatic CAB artifact creation, risk recalculation, or legal/regulatory claims.

Implemented locally:

- Backend route `GET /executive/repositories`.
- No migration: the view is composed from `deployment_gate_authorizations`,
  `change_risk_evaluations`, `change_risk_cab_packets.evaluation_ids_json`, and
  `change_risk_cab_decision_manifests`.
- Response includes repository summaries, posture, Deployment Gate counts, Change Risk counts, CAB
  packet/manifest counts, latest evidence pointers, page totals, and explicit no-claim flags.
- Tauri DTO/client method/command/invoke registration.
- Control Plane store state/action `loadMultiRepoExecutiveGovernance`.
- Governance > Releases `MultiRepoExecutiveGovernancePanel`.
- Design and validation report docs.

Local validation completed:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri tests (`49` passed).
- Frontend `npm --prefix gitgov run typecheck`.
- Frontend `npm --prefix gitgov run lint`.
- Frontend full Vitest (`383` passed).
- Focused frontend tests for store and panel (`49` passed).
- Frontend build passed with the pre-existing Vite large chunk warning.
- Focused backend real Postgres test
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped` passed with real
  tenant-scoped Deployment Gate, Change Risk, CAB packet, CAB manifest, Auditor/Developer RBAC,
  other-tenant isolation, no-claim flags, and no source mutation assertions.
- `git diff --check`.
- `scripts/security/publication_guard.ps1`.

Production validation:

- No production migration was required.
- Post-merge `main` checks passed for `bcf8e8f8`, including CI, Release Readiness Gate, Quality Gate
  Policy Matrix, Secret Scan, Public Naming Guard, Governance Correlation Smoke, Desktop Updater
  Readiness, and SonarQube Governance.
- Render deploy `dep-d8oiuan7f7vs73amvbig` for `bcf8e8f8` reached `live`.
- Production smoke passed with `/health=ok`, authenticated `/stats=200`, authenticated
  `GET /executive/repositories?org_name=yohandry10&limit=10` returning HTTP `200`,
  `repositories=1`, first repository `yohandry10/Git-Gov`, first posture `review`, `gate_count=2`,
  `change_risk_count=6`, `cab_packet_count=8`, `cab_manifest_count=6`, and safe no-claim flags.
- Production no-mutation check passed: Deployment Gate authorizations, Change Risk evaluations, CAB
  packets, CAB decision manifests, and Agent Governance evaluations stayed `2,6,8,6,7` before and
  after reading the executive route.

Report: `docs/reports/multi-repo-executive-governance-view-2026-06-16.md`.

## KAN-128 Deployment Gate Risk & CAB Evidence Context - 2026-06-16

`KAN-128 - Deployment Gate Risk & CAB Evidence Context` is completed. PR `#451` merged to `main` as
`27b2b5d5`.

Product decision:

- Return the completed KAN-121 through KAN-127 Change Risk/CAB evidence chain to the main
  Deployment Gates workflow.
- Add read-only context from a Deployment Gate authorization to its related Change Risk evaluations,
  CAB packets, and CAB decision manifests.
- Do not change gate decisions, block releases, execute deploys, mutate providers/repos, recalculate
  risk, create artifacts automatically, depend on AI/Agent Governance, or create compliance/legal
  claims.

Implemented locally:

- Backend route `GET /deployment-gates/{deployment_gate_id}/risk-context`.
- No migration: the context is composed from existing tenant-scoped relationships:
  `change_risk_evaluations.deployment_gate_id`, CAB packet `evaluation_ids_json`/`filters_json`, and
  CAB decision manifest `cab_packet_id`.
- Response includes Deployment Gate authorization, Change Risk evaluations, CAB packets, CAB
  decision manifests, latest risk/review status, triggered rule count, and no-claim flags.
- Tauri DTO/client method/command/invoke registration.
- Control Plane store cache/action `getDeploymentGateRiskContext`.
- Governance > Releases Deployment Gate History `Risk & CAB Context` section with context loading and
  CAB manifest download.
- Design and validation report docs.

Local validation completed:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri tests (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest (`381` passed).
- Focused store test `pnpm --dir gitgov test -- --run src/test/useControlPlaneStore.test.ts`
  passed (`47` tests).
- Frontend build passed with the pre-existing Vite large chunk warning.
- Focused backend real Postgres test
  `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation` passed with KAN-128
  assertions for gate -> risk -> CAB packet -> disposition -> manifest, Admin/Auditor read,
  Developer/tenant/agent denial, no source mutation, no AI, and no claims.
- `git diff --check`.
- `scripts/security/publication_guard.ps1`.

Production validation:

- No production migration was required.
- Post-merge `main` CI and guard workflows passed.
- Render deploy `dep-d8oi2p99rddc73d37320` for `27b2b5d5` reached `live`.
- Production smoke passed with `/health=ok`, authenticated `/stats=200`, Deployment Gate total `2`,
  Agent Governance evaluation total `7`, source gate `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`,
  linked Change Risk evaluation `cra_b8408c9e4aa44989bd1146d5ff5d4c30`, `risk_level=medium`,
  `review_status=accepted_risk`, trace hash
  `sha256:5c1d4c8504c0a52c42176f525b8ea9a35a5c1f2826cb5a18af99311dd47b5f46`, CAB packet
  `crcab_cf0af176f7674b16821d5cf61b5225b8`, packet hash
  `sha256:78edf32af71d3d96872b08080dc4c009bac2a7b33fe61d078bf33a3eb4d2ad51`, CAB review
  `needs_mitigation`, CAB decision manifest `crcabdm_8df5e6df7297acb8155730f48b5cc526`, manifest
  hash `sha256:ea93d018394b141665b83698cadcb1a519aef602c82571bfcfb0a385fde1936f`,
  `/deployment-gates/{gate}/risk-context` returning one linked evaluation, one CAB packet, one
  manifest, safe no-claim flags, and manifest status `revoked` after revoke.

Report: `docs/reports/deployment-gate-risk-cab-context-2026-06-16.md`.

## KAN-127 Change Risk CAB Decision Manifest - 2026-06-16

`KAN-127 - Change Risk CAB Decision Manifest` is completed. PR `#444` merged to `main` as
`12aff10d`; production route hotfix PRs `#445`, `#446`, `#447`, and `#448` ended on `main` commit
`9f1c5c9c`.

Product decision:

- Freeze the final evidence of a reviewed KAN-125/KAN-126 CAB Packet into a portable, hashable JSON
  manifest.
- Keep the source CAB Packet artifact hash, source Change Risk evaluations, trace hashes, Deployment
  Gates, providers, repos, and Agent Governance state unchanged.
- Let Admins create/revoke manifests and Admins/Auditors read/download them.
- Keep the feature manual-first, advisory-only, and no-claim.

Implemented locally:

- Supabase migration/postcheck `v67`.
- New `change_risk_cab_decision_manifests` table.
- Backend routes for create/list/get/download/revoke.
- Stable backend detail route
  `GET /change-risk/cab-decision-manifests/{manifest_id}/detail` for read-without-download.
- Manifest schema `gitgov_change_risk_cab_decision_manifest.v1`.
- Audit actions `cab_decision_manifest_created`, `cab_decision_manifest_downloaded`, and
  `cab_decision_manifest_revoked`.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases CAB Packet detail `Decision Manifest` panel.
- Design and validation report docs.

Local validation completed:

- Backend `cargo check`.
- Backend `cargo fmt --check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo check`.
- Tauri `cargo fmt --check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest `pnpm --dir gitgov test` (`380` passed).
- Frontend build `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused store test `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts`
  passed (`46` tests).
- Focused backend real Postgres test
  `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation` passed with the new
  manifest create/list/get/download/revoke assertions.
- Real Postgres `v67` migration/postcheck passed in a rollback transaction.
- `git diff --check`.
- `.\scripts\security\publication_guard.ps1`.

Production validation:

- Production `v67` migration/postcheck passed.
- Render deploy `dep-d8og45t7vvec73fsgk4g` for `12aff10d` reached `live`.
- Production smoke found the terminal detail route with `?org_name=...` unreliable behind the
  deployed path/proxy, so hotfixes added stricter ID parsing plus stable `/detail` route and moved
  the Desktop client to that route.
- Final Render deploy `dep-d8oh7gu7r5hc73c2tt40` for `9f1c5c9c` reached `live`.
- Final smoke passed with `/health=ok`, authenticated `/stats=200`, source packet
  `crcab_23d138be426a4967ae0895810e679a19`, source packet hash
  `sha256:d314caf9c2e41886cdfcbd5e56c841ea84f6329b0c00c7f6f7398a3dbe3b1d9a` unchanged, final
  source review `needs_mitigation`, created/retrieved/downloaded/revoked manifest
  `crcabdm_841ddc3eda30a3b0fceffe27fa7e856a`, manifest hash
  `sha256:45badc8d054d0ad3b8c58a0b2d64eb3e998bac2e27d4940123ce06d122c12733`, revoked download HTTP
  `409`, Deployment Gate authorization count unchanged at `2`, Agent Governance evaluation count
  unchanged at `7`, and audit rows present in `admin_audit_log` for created/downloaded/revoked.

Report: `docs/reports/change-risk-cab-decision-manifest-2026-06-16.md`.

## KAN-126 Change Risk CAB Packet Manual Disposition - 2026-06-16

`KAN-126 - Change Risk CAB Packet Manual Disposition` is completed. PR `#440` merged to `main` as
`b7bc9e81`.

Product decision:

- Record human CAB disposition over an existing KAN-125 packet.
- Keep the KAN-125 artifact JSON and artifact hash immutable.
- Let Admins update disposition metadata.
- Let Admins and Auditors read disposition metadata.
- Do not approve deployments, block releases, execute deploys, mutate providers/repos, mutate source
  Change Risk evaluations, use AI/LLM/agents, create compliance scores, or make certification/legal
  claims.

Implemented:

- Supabase migration/postcheck `v66`.
- Review metadata on `change_risk_cab_packets`.
- Backend routes:
  - `GET /change-risk/cab-packets/{packet_id}/review`.
  - `PATCH /change-risk/cab-packets/{packet_id}/review`.
- Safe text validation, decision-reason/follow-up validation, Admin-only update, Admin/Auditor read,
  tenant isolation, and admin audit events.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases CAB disposition panel.
- Design and validation report docs.

Local validation completed so far:

- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest `pnpm --dir gitgov test` passed (`379` tests).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused store test `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts`
  passed (`45` tests).
- Focused backend real Postgres test
  `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation` passed with
  `TEST_DATABASE_URL` mapped from ignored local `DATABASE_URL`.
- Real Postgres `v66` migration/postcheck passed in a rollback transaction.

Production validation:

- Production `v66` migration/postcheck passed.
- Render deploy `dep-d8ofa167r5hc73c1nf5g` for `b7bc9e81` reached `live`.
- Final production smoke passed: `/health=ok`, authenticated `/stats=200`, CAB packet
  `crcab_23d138be426a4967ae0895810e679a19` created, artifact hash
  `sha256:d314caf9c2e41886cdfcbd5e56c841ea84f6329b0c00c7f6f7398a3dbe3b1d9a` stayed unchanged
  across review updates, review moved through `pending_review`, `reviewed`, `accepted_risk`, and
  final `needs_mitigation`, unsafe secret-looking review note was rejected with HTTP `400`,
  no-claim flags stayed safe, review audit rows were present (`2` viewed, `3` updated), Deployment
  Gate authorization count stayed `2`, and Agent Governance evaluation count stayed `7`.

Report: `docs/reports/change-risk-cab-packet-manual-disposition-2026-06-16.md`.

## KAN-125 Change Risk CAB Review Packet - 2026-06-16

`KAN-125 - Change Risk CAB Review Packet` is completed. PR `#436` merged to `main` as
`92db41ac`, and hotfix PR `#437` merged as `44c0744b`.

Product decision:

- Package existing deterministic Change Risk evaluations into manual CAB/internal-audit review
  packets.
- Let Admins create packets by filters or explicit evaluation IDs.
- Let Admins and Auditors list/read/download active packets.
- Let Admins archive packets.
- Keep the artifact JSON-only, hashable, tenant-scoped, and no-claim.
- Do not add release blocking, deployment execution, provider/repo mutation, policy enforcement,
  AI/LLM/BYOM/MCP/chatbot behavior, Agent Governance dependency, public links, email/Slack,
  scheduler, PDF/DOCX, compliance score, certification, legal attestation, or official regulatory
  claim.

Implemented:

- Supabase migration/postcheck `v65`, including a re-runnable `download_count` type repair for
  existing tables.
- `change_risk_cab_packets` with artifact hash, filters, selected evaluation IDs, lifecycle,
  download count, and no-claim JSON constraints.
- Backend routes:
  - `POST /change-risk/cab-packets`.
  - `GET /change-risk/cab-packets`.
  - `GET /change-risk/cab-packets/{packet_id}`.
  - `GET /change-risk/cab-packets/{packet_id}/download`.
  - `PATCH /change-risk/cab-packets/{packet_id}/archive`.
- Artifact schema `gitgov_change_risk_cab_packet.v1`.
- Admin audit actions for created/downloaded/archived packets.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases `ChangeRiskCabPacketsPanel`.
- Design, roadmap, architecture, report, public-context, and current-context docs.

Local validation completed:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest `pnpm --dir gitgov test` (`378` passed).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused backend real Postgres test
  `change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation` passed.
- Focused backend real Postgres Change Risk suite `cargo test change_risk -- --nocapture` passed
  (`3` tests).
- Focused store test `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts` passed
  (`44` tests).
- Real Postgres `v65` migration/postcheck passed in a rollback transaction.
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

Known local validation limit:

- Full backend `cargo test -- --test-threads=2` exceeded the local `7` minute command timeout
  without returning useful failure output. Backend test compilation and the affected Change Risk
  real Postgres suite passed.

Production validation:

- PR checks and post-merge `main` checks passed.
- Render deploy `dep-d8oemvuq1p3s73fecrug` for `44c0744b` reached `live`.
- Production `v65` migration/postcheck passed.
- Production initially returned HTTP `502` on `POST /change-risk/cab-packets` because an existing
  `download_count integer` column did not match the backend `bigint` model. Render logs confirmed
  the `ColumnDecode` panic. The corrected `v65` migration altered the column to `bigint`, and the
  postcheck now enforces the type.
- Final post-hotfix production smoke passed: `/health=ok`, authenticated `/stats=200`,
  `download_count` type `bigint`, packet `crcab_d48e546c08b844189ec4fe6d7d4ed7b2` was created from
  evaluation `cra_4d59c84859a747789e577ca24945ec50`, list/get/download/archive succeeded, archived
  download returned HTTP `409`, record hash
  `sha256:4a262e527c263a293e8d1febbb756b448bb4400ec63ee92959414e0309bf7199` matched on
  read/download, no-claim flags stayed false where required, and Deployment Gate authorization plus
  Agent Governance evaluation counts stayed unchanged.

Report: `docs/reports/change-risk-cab-review-packet-2026-06-16.md`.

## KAN-124 Change Risk Review Queue And CAB Evidence Filter - 2026-06-16

`KAN-124 - Change Risk Review Queue and CAB Evidence Filter` is completed. PR `#433` merged to
`main` as `d145d6fe`.

Product decision:

- Extend KAN-123 manual review metadata into a CAB/operator review queue.
- Let Admins and Auditors list Change Risk evaluations by `review_status`.
- Keep review status as manual evidence only; do not convert it into release blocking.
- Do not add scoring, enforcement, deploy execution, provider/repo mutation, AI/LLM, BYOM, MCP,
  chatbot behavior, Agent Governance dependency, compliance/certification/legal/regulatory claim,
  notifications, approval quorum, or multi-reviewer workflow.

Implemented:

- Optional `review_status` on backend/Tauri/frontend `ChangeRiskEvaluationQuery`.
- Backend validation for allowed KAN-123 review states.
- SQL filtering in `list_change_risk_evaluations`.
- Tauri query-string support for `review_status`.
- Control Plane store support for applying and explicitly clearing the review status filter.
- Store behavior that removes an evaluation from the active review queue when a review update moves
  it to a different status.
- Governance > Releases `ChangeRiskPanel` `Review queue` selector.
- Design and validation report docs.

Local validation:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Focused backend Change Risk tests with real Postgres passed (`2` tests), covering review queue
  inclusion/exclusion, invalid review status rejection, Auditor read access, tenant isolation, and
  no Deployment Gate or Agent Governance mutation.
- Tauri `cargo check`.
- Tauri `cargo fmt --check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri tests (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest (`377` passed).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused store test `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts` passed
  (`43` tests).
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

PR and production validation:

- PR `#433` checks passed, including Security Guard, Server Clippy + Check, Desktop Rust Clippy,
  Frontend Lint + Typecheck, Website Lint + Typecheck + Build, Validate Policy-as-Code, and the
  quality gate matrix.
- Post-merge `main` checks passed, including CI, Release Readiness Gate, Secret Scan, Public Naming
  Guard, Governance Correlation Smoke, Desktop Updater Readiness, Quality Gate Policy Matrix, and
  SonarQube Governance.
- Render deploy `dep-d8odh5c2m8qs73amf7p0` for commit `d145d6fe` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `GET /change-risk/evaluations?review_status=accepted_risk` returned KAN-123 smoke evaluation
    `cra_4d59c84859a747789e577ca24945ec50`.
  - `GET /change-risk/evaluations?review_status=needs_review` excluded that accepted-risk
    evaluation.
  - Invalid `review_status=approved` returned HTTP `400`.
  - No-claim/manual flags remained `advisory_only=true`, `llm_used=false`,
    `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
  - Read-only smoke left counts unchanged:
    `deployment_gate_authorizations=2;agent_governance_evaluations=7`.

Report: `docs/reports/change-risk-review-queue-cab-filter-2026-06-16.md`.

## KAN-123 Change Risk Manual Review & Mitigation Notes - 2026-06-16

`KAN-123 - Change Risk Manual Review & Mitigation Notes` is completed. PR `#430` merged to
`main` as `3aa5f894`.

Product decision:

- Extend KAN-121/KAN-122 rather than creating a parallel review workflow.
- Let a human reviewer record manual review status, safe notes, mitigation notes, and decision
  reason over an already explained Change Risk evaluation.
- Keep Change Risk advisory-only, qualitative, manual-first, non-mutating, and independent from
  Agent Governance.
- Keep update access Admin-only in this MVP; allow Admin/Auditor read access.
- Do not add enforcement, release blocking, deploy execution, provider mutation, repository
  mutation, AI/LLM, BYOM, MCP, chatbot behavior, compliance score, certification/legal/regulatory
  claim, notifications, approval quorum, or multi-reviewer workflow.

Implemented:

- Supabase migration/postcheck `v64`.
- Review metadata on `change_risk_evaluations`: `review_status`, `reviewed_by_user_id`,
  `reviewed_at`, `review_notes_safe`, `mitigation_notes_safe`, `decision_reason_safe`, and
  `review_updated_at`.
- Backend routes:
  - `GET /change-risk/evaluations/{evaluation_id}/review`.
  - `PATCH /change-risk/evaluations/{evaluation_id}/review`.
- Safe note validation and secret-like text rejection.
- Dedicated review handler module `gitgov-server/src/handlers/change_risk_review.rs`, keeping the
  base Change Risk handler focused on evaluation/catalog/trace behavior.
- Admin audit action `change_risk_review_updated`.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases `ChangeRiskPanel` `Manual Review` panel.
- Architecture, roadmap, design, and validation report docs.

Local validation:

- Backend `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and
  `cargo test --no-run`.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and tests
  (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest `pnpm --dir gitgov test` (`377` passed).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused backend Change Risk tests with real Postgres passed (`2` tests), covering default
  `needs_review`, Admin review updates, safe note rejection, Auditor read/Admin-only update,
  Developer and Agent Governance key denial, tenant isolation, immutable risk/trace fields, audit
  event creation, and no Deployment Gate or Agent Governance mutation.
- `v64` migration/postcheck passed in a real Postgres rollback transaction.
- Focused store test `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts` passed
  (`43` tests).
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

Production validation:

- PR checks passed.
- Production `v64` migration/postcheck passed.
- Render deploy `dep-d8od0bt8nd3s73adtalg` for `3aa5f894` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `POST /change-risk/evaluations` created
    `cra_4d59c84859a747789e577ca24945ec50` for `KAN-123-production-smoke`.
  - Review status moved through `reviewed`, `needs_mitigation`, and final `accepted_risk`.
  - Secret-like review note containing `Authorization: Bearer` was rejected with HTTP `400`.
  - `GET /change-risk/evaluations/{id}/review` returned final `accepted_risk`.
  - `GET /change-risk/evaluations/{id}/trace` preserved the same trace hash after review updates.
  - `3` `change_risk_review_updated` audit events were recorded with `trace_changed=false`.
  - No-claim flags stayed false for LLM, Agent Governance, compliance claim, and certification.
  - Deployment Gate authorization and Agent Governance evaluation counts did not change.

Report: `docs/reports/change-risk-manual-review-mitigation-notes-2026-06-16.md`.

## KAN-122 Change Risk Rule Catalog & Evaluation Trace - 2026-06-16

`KAN-122 - Change Risk Rule Catalog & Evaluation Trace` is completed. PR `#425` merged the
feature, PR `#426` fixed production migration constraint scoping, and PR `#427` fixed GitHub
Actions CI evidence detection. Final production commit: `243b8998`.

Product decision:

- Extend KAN-121 rather than creating a parallel risk engine.
- Explain every Change Risk evaluation with deterministic, versioned rules.
- Keep Change Risk advisory-only, qualitative, manual-first, and non-mutating.
- Keep create access Admin-only; allow Admin/Auditor read access.
- Do not add AI/LLM, BYOM, MCP, chatbot behavior, Agent Governance dependency, compliance score,
  certification/legal/regulatory claim, provider mutation, repository mutation, or deployment
  execution.

Implemented:

- Supabase migration/postcheck `v63`.
- New persisted columns on `change_risk_evaluations`: `ruleset_version`, `triggered_rules`,
  `non_triggered_rules`, `evaluation_trace`, and `trace_hash`.
- Ruleset `change_risk_rules.v1` with 12 deterministic rules.
- Backend routes:
  - `GET /change-risk/rules`.
  - `GET /change-risk/evaluations/{evaluation_id}/trace`.
- `POST /change-risk/evaluations` persists rule trace metadata.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases `ChangeRiskPanel` `Why this risk?` trace view.
- Rule/trace helper split into `gitgov-server/src/handlers/change_risk_rules.rs` for
  maintainability.

Validation:

- Backend `cargo fmt` and `cargo check`.
- Tauri `cargo fmt` and `cargo check`.
- Frontend focused ESLint on changed files.
- Frontend `pnpm --dir gitgov typecheck`.
- Focused store test `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts` passed
  (`42` tests).
- Focused backend Change Risk tests with real Postgres passed (`2` tests).
- `v63` migration/postcheck passed in a real Postgres rollback transaction.
- Backend `cargo clippy -- -D warnings` and `cargo test --no-run`.
- Tauri `cargo clippy -- -D warnings` and `cargo test` (`49` tests).
- Frontend `pnpm --dir gitgov lint`, full Vitest (`376` tests), and build.
- `git diff --check` and publication guard.
- PR `#425`, PR `#426`, and PR `#427` checks passed.
- Post-merge `main` checks for `243b8998` passed: `CI`, `Release Readiness Gate`, `Quality Gate
  Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop
  Updater Readiness`, and `SonarQube Governance`.
- Production `v63` migration/postcheck passed after scoping constraint idempotency checks to
  `public.change_risk_evaluations`.
- Render deploy `dep-d8oc3r8jo6nc73b2s07g` for `243b8998` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `GET /change-risk/rules` returned `change_risk_rules.v1` and `12` rules.
  - `POST /change-risk/evaluations` created `cra_e70a4dfbee3546cd8ae976ff3bcd4ee3`.
  - Created evaluation returned `risk_level=medium`, trace hash
    `sha256:ee2bb0714ce4e83117581f9ab8ea3c98979693d2ce8a7d7f46711ae274790410`, and `12` trace
    rule entries.
  - Real GitHub Actions and PR evidence no longer triggered `missing_ci_evidence`,
    `missing_code_review`, or `missing_change_link`.
  - Agent Governance and Deployment Gate authorization counts stayed unchanged.

Known local validation limit:

- Full backend `cargo test -- --test-threads=2` timed out twice locally after `10` and `15`
  minutes without useful failure output. Focused real Change Risk tests and backend test compilation
  passed; required GitHub CI passed before merge.

Report: `docs/reports/change-risk-rule-catalog-evaluation-trace-2026-06-16.md`.

## KAN-121 Change Risk Assessment Advisory MVP - 2026-06-16

`KAN-121 - Change Risk Assessment Advisory MVP` is completed. PR `#422` merged to `main` as
`eb66480`; production `v62` migration/postcheck passed; Render deploy `dep-d8oanqmq1p3s73fc8u7g`
reached `live`; production smoke passed.

Product decision:

- Implement Change Risk as deterministic advisory evidence, not a blocking deployment decision.
- Reuse existing Deployment Gates, release governance, evidence packet, and first governed repo
  context.
- Keep the feature manual-first for regulated customers.
- Allow Admins to create/list/read advisory evaluations in the MVP.
- Deny Developers, unrelated tenants, and Agent Governance keys.
- Do not use AI, call Agent Governance, mutate providers, mutate repositories, execute deploys,
  create compliance scores, create certification/legal/regulatory claims, or replace manual CAB
  judgment.

Implemented surface:

- Supabase migration/postcheck `v62` for `change_risk_evaluations`.
- Backend routes:
  - `POST /change-risk/evaluations`.
  - `GET /change-risk/evaluations`.
  - `GET /change-risk/evaluations/{evaluation_id}`.
- Deterministic evaluator returning `risk_level`, `risk_reasons`, `missing_evidence`,
  `blocking_gaps`, and `recommended_manual_actions`.
- Database-enforced flags: `advisory_only=true`, `llm_used=false`,
  `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- `ChangeRiskPanel` under Governance > Releases.

Validation completed locally:

- Backend `cargo fmt --check`, `cargo check`, and `cargo clippy -- -D warnings`.
- Focused real Postgres KAN-121 tests covering approved/advisory gates, blocked gates,
  break-glass, missing context, tenant isolation, Developer denial, Agent Governance key denial,
  and no Agent Governance evaluation mutation.
- Full backend real Postgres suite passed (`313` tests).
- `v62` migration/postcheck passed in a real Postgres rollback transaction.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and tests (`49`
  passed).
- Frontend typecheck, lint, focused store test (`41` passed), full Vitest suite (`375` passed), and
  production build.

Report: `docs/reports/change-risk-assessment-advisory-2026-06-16.md`.

Post-merge and production validation:

- PR checks passed, including Security Guard, Server Clippy + Check, Desktop Rust Clippy, Frontend
  Lint + Typecheck, Website Lint + Typecheck + Build, Validate Policy-as-Code, quality gate matrix,
  Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal marker guard.
- Production `v62` migration/postcheck returned `PASS` for table, no-claim constraints, and indexes.
- Render deploy `dep-d8oanqmq1p3s73fc8u7g` for commit `eb66480` reached `live`.
- Production smoke:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `GET /change-risk/evaluations?org_name=yohandry10&limit=1` succeeded.
  - `POST /change-risk/evaluations` created `cra_9d53d9cd29a7439aa0485607edeae64e` for
    `KAN-121`, repo `yohandry10/Git-Gov`, branch `main`, environment `production`, commit
    `eb66480`.
  - Created record returned `risk_level=medium`, `advisory_only=true`, `llm_used=false`,
    `agent_governance_used=false`, `compliance_claim=false`, `certification=false`, and missing
    evidence `deployment_gate_authorization, release_evidence_packet`.
  - `GET /change-risk/evaluations/{evaluation_id}?org_name=yohandry10` returned the same
    evaluation ID.

## KAN-120 First Governed Repo Integration Wizard - 2026-06-16

`KAN-120 - First Governed Repo Setup Integration Wizard` is completed. PR `#419` merged to `main`
as `e244c1c`; Render deploy `dep-d8o9t619rddc73cugjvg` reached `live`; no database migration was
required; production smoke passed.

Product decision:

- Resume the `0.1 Deployment Gates` activation path after KAN-119 instead of adding another
  compliance artifact.
- Reuse KAN-80 `enterprise_first_governed_repo_setups` as the canonical store; no duplicate
  onboarding table.
- Keep the path manual-first for regulated customers.
- Allow Admins to create/resume, update, validate, plan, and complete the first governed repo run.
- Allow Auditors to read state only.
- Deny Developers, unrelated tenants, and Agent Governance keys.
- Do not store provider secrets, mutate providers, mutate customer repositories, execute deploys,
  create compliance/certification/legal claims, or depend on Agent Governance/AI.

Implemented surface:

- Backend routes:
  - `GET /onboarding/first-governed-repo/state`.
  - `POST /onboarding/first-governed-repo/runs`.
  - `PATCH /onboarding/first-governed-repo/runs/{run_id}`.
  - `POST /onboarding/first-governed-repo/runs/{run_id}/validate`.
  - `POST /onboarding/first-governed-repo/runs/{run_id}/plan`.
  - `POST /onboarding/first-governed-repo/runs/{run_id}/complete`.
- Backend wizard helpers split from the base setup handler for maintainability.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- `FirstGovernedRepoSetupPanel` wizard controls for Start, Validate, Plan, and Complete.

Validation completed locally:

- Backend `cargo fmt --check`, `cargo check`, and `cargo clippy -- -D warnings`.
- Focused real Postgres KAN-120 test using local `TEST_DATABASE_URL` on `127.0.0.1:5433`.
- Full backend test suite with local Postgres `TEST_DATABASE_URL` and `--test-threads=2`:
  `311` passed.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and tests (`49` passed).
- Frontend typecheck, lint, focused store test, full Vitest suite (`373` passed), and production
  build.

Post-merge and production validation:

- PR checks passed.
- Post-merge `main` CI and required guardrails passed for `e244c1c`.
- Render deploy `dep-d8o9t619rddc73cugjvg` for `e244c1c` reached `live`.
- Production smoke:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - Initial `GET /onboarding/first-governed-repo/state?org_name=yohandry10` returned `found=false`.
  - `POST /onboarding/first-governed-repo/runs` created run
    `71d55474-0833-4d15-b485-6281792841ae`.
  - `validate` returned `stores_secret_values=false` and `providerHealthCount=3`.
  - `plan` returned `provider_mutation=false` and `deployment_gate_mode=advisory`.
  - `complete` returned `status=completed`, `release_blocking_default=false`,
    `agent_governance_required=false`, and `compliance_claim=false`.

Report: `docs/reports/first-governed-repo-integration-wizard-2026-06-16.md`.

## KAN-119 Period Compliance Report Share Packages - 2026-06-15

`KAN-119 - Period Compliance Report Share Packages` is completed. PR `#416` merged to `main` as
`1d1df77`; Render deploy `dep-d8o8t9u47okc738l7g7g` reached `live`; production `v61`
migration/postcheck and production smoke passed.

Product decision:

- Keep this manual and auditor/customer-review oriented.
- Package already existing Period Compliance Report evidence; do not create a new compliance claim.
- Require the source period report to be `reviewed`.
- Require existing JSON report, PDF export, and provenance manifest before package creation.
- Allow Admins to create/revoke packages.
- Allow Admins and source-authorized Auditors to list/download packages.
- Deny Developers, unrelated tenants, and Agent Governance keys.
- Record custody through append-only access-log actions.

Implemented surface:

- Supabase migration/postcheck `v61`.
- Backend routes:
  - `GET/POST /compliance/period-reports/{period_report_id}/share-packages`.
  - `GET /compliance/period-report-share-packages/{share_package_id}`.
  - `GET /compliance/period-report-share-packages/{share_package_id}/download`.
  - `PATCH /compliance/period-report-share-packages/{share_package_id}/revoke`.
- Artifact schema `gitgov_period_compliance_report_share_package.v1`.
- Package hash over the redacted package payload with period JSON hash, PDF hash, manifest hash,
  review snapshot, retention snapshot, no-claim flags, and manual verification instructions.
- Desktop/Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions and `CompliancePeriodReportSharePackagePanel`.

Explicitly not included:

- No public links.
- No email delivery.
- No scheduler.
- No DOCX/formal regulatory template.
- No compliance score.
- No certification, official regulatory claim, or legal attestation.
- No official regulatory mapping.
- No AI summary, BYOM/MCP/chatbot behavior, or Agent Governance dependency.

Validation completed:

- Backend `cargo fmt --check`, `cargo check`, and `cargo clippy -- -D warnings`.
- Focused real Postgres period-report integration test covering create preconditions, real
  JSON/PDF/manifest/review chain, package hash recomputation, download custody, revoke behavior,
  role denial, tenant isolation, Agent Governance key denial, and no Agent Governance evaluation
  mutation.
- Affected backend module suites in serial:
  `compliance_period_reports`, `compliance_framework_review_reports`,
  `compliance_review_packages`, and `compliance_evidence_exports`.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and full Tauri tests
  (`49` passed).
- Frontend typecheck, lint, focused store/component tests, full Vitest suite (`371` passed), and
  production build.
- `v61` migration/postcheck passed in a real rollback transaction against the configured Postgres
  connection.
- PR checks and post-merge `main` checks passed, including `CI`, `Release Readiness Gate`, `Quality
  Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`,
  `Desktop Updater Readiness`, `SonarQube Governance`, `Security Guard`, `Server Clippy + Check`,
  `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`,
  `Validate Policy-as-Code`, and `Workflow Lint`.
- Production `v61` migration/postcheck passed.
- Production smoke reviewed Period Compliance Report `cpr_9389010c74a34484a8e080942b56956e`,
  verified existing PDF export `cprpdf_0d2e6aad239125a198e64c1a307b158d`, created manifest
  `cprm_c3473263ece408fc12ca0bd5c7adc206`, created share package
  `cprsp_afaaed71cf684e63860915923722ce65`, downloaded it with package hash
  `sha256:49aa46f29c12a8a48286d099f171826c40584afc0094ad592344abd57b822e38`, revoked it, and
  confirmed revoked download returns HTTP `409` with code `share_package_revoked`.

Known local validation limit:

- A full parallel backend suite hit local Supabase/Postgres session exhaustion
  (`EMAXCONNSESSION max clients reached`).
- A full serial backend retry timed out after 15 minutes.
- The affected real KAN-119 integration chain, adjacent backend modules, PR checks, and post-merge
  CI passed.

Report: `docs/reports/period-compliance-report-share-package-2026-06-15.md`.

## KAN-69 Desktop Runtime QA - 2026-06-08

`KAN-69 - Enterprise Action Center guided UX` remains implemented and merged. The Desktop runtime QA and information-architecture pass that followed is also merged to `main` through PR `#209` (`fix/KAN-69-desktop-runtime-qa-maintainability`) and PR `#211` (`fix/KAN-69-control-plane-workspace-auth`); latest main commit `e0c769d`. It was a QA pass, not a new feature wave.

QA decisions from that pass:

- `/action-center` remains the only global `Next Action` owner.
- Workspace keeps local execution: file list, CLI, pipeline visualizer, audit trail, commit/push controls, `Next local step`, and gates/blockers without repeating the global recommendation.
- Opening Action Center no longer performs automatic heavy evidence refresh; heavy refresh remains behind explicit `Refresh`.
- GitHub auth identifies the Desktop operator; the GitGov API key authorizes Control Plane role/org/evidence. Valid local GitHub sessions should be restored by default instead of forcing Device Flow on every app start.
- Effective local Git identity is validated separately from GitHub login. Warnings now say the effective Git identity is incomplete or not provably aligned; `Ver prueba` emits read-only `git config --get` evidence and does not mutate Git config.
- Control Plane is configuration, not a primary dashboard. `/control-plane` redirects to `/settings#control-plane`.
- Settings tabs are `Preferences`, `Organization`, `Account`, `Repository`, and `System`. `System` merges former Connection and Updates surfaces: Control Plane endpoint/API key/role/scope/transport plus Desktop updater.
- Organization settings use full-width vertical flow to avoid a dead right column. Repository is the only Settings tab retaining a two-column parent grid when config preview is present.
- Governance is the operational governance module. `/governance` defaults to `Evidence`; sections are `Evidence`, `Policy`, `Adoption`, `Releases`, and `Copilot`; there is no generic Governance Dashboard tab.
- Former dashboard-only components `ServerDashboard`, `DashboardHeader`, `DailyActivityWidget`, `RiskOutcomesWidget`, and `TicketCoverageWidget` are removed rather than left unmounted.
- Help/FAQ uses a full-width support layout and canonical `https://gitgov.cloud` links instead of the old Vercel app URL.
- Settings, primary sidebar, and Governance shell are language-reactive through `i18n`; nested feature panels still need targeted localization before claiming full app localization.
- Do not restart, kill, or relaunch Tauri/Desktop while a user is manually validating unless explicitly requested.

Current QA report: `docs/reports/kan-69-desktop-runtime-qa-2026-06-07.md`.

Latest local validation for this QA pass:

- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- focused Settings/Governance/i18n/Help layout tests: `17` tests
- full frontend suite: `332` tests in `32` files
- `npm --prefix gitgov run build`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

The build still reports the known Vite `>500 kB` base chunk warning; Action Center and Governance are emitted as separate route chunks.

## KAN-69 Product UX Implementation - 2026-06-07

`KAN-69 - Enterprise Action Center guided UX` is completed.

- Adds a dedicated desktop route at `/action-center`.
- Adds a sidebar `Action Center` navigation item.
- Uses deterministic `Goal + Evidence + Permission` rules to show one primary recommendation plus alternatives.
- Reuses existing Governance, Settings, and Workspace workflows through deep links instead of duplicating dashboard panels.
- Keeps recommendations advisory, non-blocking, and explainable from loaded evidence.
- Keeps AI as an explanation destination only; the Action Center recommendation is not LLM-driven.
- Does not add backend endpoints, provider mutations, customer repository mutations, release blocking defaults, SonarCloud, Jenkins trigger-only setup, or OpenAPI/SDK work.

Design: `docs/design/enterprise-action-center-guided-ux.md`.
Report: `docs/reports/enterprise-action-center-guided-ux-2026-06-07.md`.
PR: `#204 - product(KAN-69): add guided Action Center workspace`.
Main commit: `aa7e352 product(KAN-69): add guided action center workspace (#204)`.
Post-merge checks passed on `main`: `CI` run `27086413044`, `Release Readiness Gate` run `27086413043`, `Secret Scan` run `27086413053`, `Public Naming Guard` run `27086413041`, `SonarQube Governance (Non-Blocking)` run `27086413042`, `Quality Gate Policy Matrix (Optional)` run `27086413040`, `Governance Correlation Smoke (Optional)` run `27086413050`, and `Desktop Updater Readiness (Optional)` run `27086413038`.

Verification follow-up: `docs/reports/enterprise-action-center-verification-2026-06-07.md` records the product/infrastructure Q/A review. It fixed release prep so missing or empty Jira coverage stays conservative before Evidence Packet/release decision guidance, and it avoids known-forbidden admin-only adoption-profile/checklist reads for non-admin users. Follow-up PR `#206 - fix(KAN-69): harden Action Center verification logic` merged on `main` as `8a55a6d`. Follow-up validation passed focused Action Center helper tests (`8` tests), full frontend tests (`304` tests in `26` files), typecheck, lint, build, local Vite HTTP smoke for `/action-center`, and post-merge checks on `main`: `CI` run `27100640858`, `Release Readiness Gate` run `27100640831`, `Secret Scan` run `27100640840`, `Public Naming Guard` run `27100640856`, `SonarQube Governance (Non-Blocking)` run `27100640837`, `Quality Gate Policy Matrix (Optional)` run `27100640835`, `Governance Correlation Smoke (Optional)` run `27100640862`, and `Desktop Updater Readiness (Optional)` run `27100640864`.

## Documentation Reality Audit - 2026-05-02

`KAN-70` started the documentation cleanup track, `KAN-71` completed the backend/API/schema audit phase, `KAN-72` completed the Desktop/dashboard audit phase, `KAN-73` completed the CI/workflows/release automation audit phase, `KAN-74` completed the narrow CI helper/runtime follow-up from that audit, and `KAN-75` completed the remaining public web, roadmap/context, and stale public-claim cleanup phase. The purpose is to update living documentation against the actual repository state in phases, not to add product functionality.

- `KAN-69 - Enterprise Action Center guided UX` is no longer pending; it is completed after `KAN-68` and the documentation audit track.
- Latest completed follow-up: `KAN-74 - CI helper/runtime follow-up`, which aligned branch-protection helper defaults with live required checks and replaced `gitleaks/gitleaks-action@v2` with direct Gitleaks CLI execution.
- Latest completed follow-up: `KAN-75 - Public web roadmap claims documentation audit`, which reconciled public docs and content architecture notes with implemented Jira, governance, Render production, risk-outcome, pricing, metadata, and web runtime facts when `KAN-69` was still pending. PR `#200` merged as `b393a82`; post-merge `CI` run `25265387894` and `Release Readiness Gate` run `25265387888` passed.
- The current repo has `32` active GitHub Actions workflows, schema migrations through `supabase_schema_v25.sql`, `193` backend tests reported by `cargo test -- --list`, `296` desktop frontend tests across `25` files, and `23` Tauri/Rust tests.
- CI/workflow docs were checked against `.github/workflows`, `.github/scripts`, `scripts/github`, `scripts/control-plane`, and live GitHub branch protection metadata; verified facts include `5` pull_request workflows, `9` push workflows, `29` workflow_dispatch workflows, `22` scheduled workflows, `28` artifact-producing workflows, and `6` strict required checks on `main`.
- Backend/API docs are checked against `gitgov/gitgov-server/src/server/routes.rs`, `gitgov/gitgov-server/src/handlers`, `gitgov/gitgov-server/supabase`, and `.env.example`; the verified backend router has `158` production Axum route registrations plus `/api-docs` as a partial schema explorer.
- Desktop/dashboard docs were checked against `gitgov/src`, `gitgov/src-tauri`, `gitgov/package.json`, and `gitgov/src-tauri/tauri.conf.json`; verified facts include `27` Control Plane component modules, `94` registered Tauri commands, React `19.2.0`, and an updater endpoint configured through GitHub Releases.
- Public web docs were checked against `gitgov-web/package.json`, `gitgov-web/README.md`, bilingual public docs, `CONTENT_ARCHITECTURE_GUIDE.md`, and current product-state docs; corrected public claims include Next.js `15.5.15`, Jira operational maturity, governance blocking boundaries, Render-managed production, risk-outcome metrics, pricing/pilot-fit language, and stale `/docs/privacy` references.
- Existing documentation edits in `README.md`, `gitgov/README.md`, `docs/ARCHITECTURE.md`, `docs/DEPLOYMENT.md`, `docs/QUICKSTART.md`, `docs/TROUBLESHOOTING.md`, and `gitgov/gitgov-server/README.md` are being included only where they match code/configuration reality.
- Historical reports remain evidence snapshots. The cleanup target is living docs and current handoff/status material.

## Current Execution Summary - 2026-04-25

This section consolidates the latest completed implementation/documentation points and separates active implementation work from operational decisions.

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
| `KAN-15` | OpenAPI partial-contract guard | Added a regression test that preserves the `/api-docs` partial schema-explorer disclaimer and keeps `docs/ARCHITECTURE.md` plus the backend route composition file as the operational contract source. KAN-139 later updated that pointer to `gitgov/gitgov-server/src/server/routes.rs`. | `gitgov/gitgov-server/src/openapi.rs`, `docs/reports/kan-15-openapi-partial-contract-guard-2026-04-28.md` |
| `KAN-16` | Provider access validation | Added a single secret-safe PowerShell smoke test for GitGov production/local health, SonarQube, Jenkins, Jira, and optional release readiness using ignored env files. | `scripts/control-plane/validate_provider_access.ps1`; latest validation all checks `ok`, readiness `91/100` |
| `KAN-17` | Local Sonar self-hosted runner runbook | Documented how to safely add a dedicated GitHub self-hosted runner for local SonarQube without breaking the current GitHub-hosted/non-blocking CI path. | `docs/runbooks/local-sonar-self-hosted-runner.md` |
| `KAN-18` | Jenkins trigger-only token flow | Added a dry-run-first validator and runbook for the optional `/build?token=...` path while keeping authenticated Jenkins API access as the default verification path. | `scripts/jenkins/validate_trigger_token_flow.ps1`, `docs/runbooks/jenkins-trigger-token-flow.md`, `docs/reports/kan-18-jenkins-trigger-token-flow-2026-04-28.md` |
| `KAN-19` | Jira traceability coverage validator | Added a dedicated validator and runbook for refreshing Jira/PR correlations and measuring ticket coverage independently from the release readiness gate. | `scripts/control-plane/validate_jira_traceability_coverage.ps1`; latest validation coverage `96.43%` (`54/56`) |
| `KAN-20` | Implementation backlog closure | Reframed the last six "remaining" items as operational decisions or optional future enhancements. No required implementation blocker remains in this status list. | `docs/reports/kan-20-implementation-backlog-closure-2026-04-28.md` |
| `KAN-21` | Operating decision clarification | Documented that SonarCloud is not a valid path for this personal repo, Jenkins trigger-only is not needed for normal API-based agent work, and OpenAPI completeness is only required if generated SDK/Swagger contract testing becomes product scope. | `docs/reports/kan-21-operational-decisions-2026-04-28.md` |
| `KAN-22` | Current context handoff | Added a single resume document with exact current state, latest PR/commit, access summary, non-negotiable decisions, validation commands, and practical next steps. | `docs/CURRENT_CONTEXT.md`, `docs/reports/kan-22-current-context-handoff-2026-04-28.md` |
| `KAN-31` | Enterprise adoption persistence | Persisted the dashboard adoption profile per org with admin get/upsert endpoints, Tauri commands, UI save/load, backend validation, audit metadata, and Supabase migration `v23`; production `v23` was applied and postchecked on 2026-04-30. | PR `#112`, docs PR `#113`, main commits `509e2a2` and `171d43d`, `docs/design/adoption-profile-persistence-mvp.md`, `docs/reports/adoption-profile-persistence-2026-04-30.md` |
| `KAN-32` | Enterprise provider health validation | Added a secret-safe Provider Health dashboard section that evaluates selected adoption providers from profile intent plus existing GitGov evidence, without reading or displaying provider credentials. | PR `#115`, main commit `1a16d88`, `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`, `gitgov/src/components/control_plane/dashboard-helpers.ts`, `docs/design/provider-health-validation-mvp.md`, `docs/reports/provider-health-validation-2026-04-30.md` |
| `KAN-33` | Enterprise workflow template generation | Added a secret-safe workflow template generator that converts the adoption profile into reviewed GitHub Actions template packs, manifest, README, variables, secret names, and manual install checklist without mutating customer repositories. | PR `#117`, main commit `62b67e5`, `scripts/control-plane/generate_enterprise_workflow_templates.ps1`, `docs/design/workflow-template-generation-mvp.md`, `docs/reports/workflow-template-generation-2026-04-30.md` |
| `KAN-34` | Dashboard workflow template pack | Added dashboard-side workflow template pack generation/download from the current adoption profile, keeping generated workflow contents inside a secret-safe JSON pack and avoiding automatic repository mutation. | PR `#119`, main commit `31b109d`, `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`, `gitgov/src/components/control_plane/dashboard-helpers.ts`, `docs/design/dashboard-workflow-template-pack-mvp.md`, `docs/reports/dashboard-workflow-template-pack-2026-04-30.md` |
| `KAN-35` | Reviewed workflow installation | Added a dry-run-first installer that applies CLI or dashboard workflow template packs into a local customer repository checkout only after explicit `-Apply`, with path validation and overwrite review. | PR `#121`, main commit `c60c486`, `scripts/control-plane/install_enterprise_workflow_templates.ps1`, `docs/design/reviewed-workflow-installation-mvp.md`, `docs/reports/reviewed-workflow-installation-2026-04-30.md` |
| `KAN-36` | Provider connection validation | Added a secret-safe direct provider connection validator for adoption profiles, covering GitHub, Jira, Jenkins, SonarQube, Render, and Vercel with strict and report-only modes. | PR `#123`, main commit `8c075a4`, `scripts/control-plane/validate_enterprise_provider_connections.ps1`, `docs/design/provider-connection-validation-mvp.md`, `docs/reports/provider-connection-validation-2026-04-30.md` |

### Current Operational Decisions

As of `KAN-20`, this list has no required implementation blocker. The items below are validated operating decisions, optional future enhancements, or ongoing evidence hygiene.

Resume context is centralized in `docs/CURRENT_CONTEXT.md`. Read it first before continuing a future session.

1. `GITGOV_API_KEY` production admin access is usable from ignored local env files.
   - `https://gitgov-api.onrender.com/stats` returned HTTP `200` with the local key.
   - The previous manual `/integrations/jira` `401` was caused by missing Jira shared-secret handling, not by a bad GitGov API key.
   - Manual Jira ingest requires `Authorization: Bearer <GITGOV_API_KEY>`, `x-gitgov-jira-secret: <JIRA_WEBHOOK_SECRET>`, and an `org_name` payload hint such as `yohandry10`.
2. Sonar remains intentionally local.
   - SonarCloud is not applicable for the current personal GitHub repository/account because SonarCloud onboarding for this repo requires a GitHub organization path. Do not ask again to use SonarCloud for this repo unless it is moved to a GitHub organization.
   - GitHub-hosted runners cannot reach `localhost:9000`; keep GitHub Sonar scan optional/non-blocking unless a self-hosted runner is added.
   - Latest local validation on 2026-04-28: SonarQube `UP`, project `yohandry10_git-gov`, quality gate `OK`.
   - `KAN-17` documents the self-hosted runner activation path; no workflow `runs-on` change is enabled by default.
3. Jenkins trigger-only URL flow is still optional and separate from Jenkins API access.
   - API inspection/build access works through `JENKINS_API_TOKEN`; this is already configured and is the normal agent path.
   - The unauthenticated/manual trigger URL requires `JENKINS_BUILD_TRIGGER_TOKEN` only if that flow is needed. It was not required to get Jenkins operational access and is not needed for logs, queue state, build history, or authenticated build operations.
   - Latest local validation on 2026-04-28: job `gitgov-demo-pipeline`, last build `#30`, result `SUCCESS`, not building.
   - `KAN-18` added dry-run validation through `scripts/jenkins/validate_trigger_token_flow.ps1`; latest dry-run passed API inspection but reported `JENKINS_BUILD_TRIGGER_TOKEN` was not loaded. Pass `-Trigger` only to launch a real build.
4. OpenAPI is still partial by design.
   - OpenAPI is the machine-readable API description used by Swagger tools and generated SDKs. It is not the API itself.
   - `/api-docs` is a schema explorer, not the full operational route contract.
   - Implement full `#[utoipa::path]` coverage only if generated SDKs or Swagger-based contract tests become a product requirement. Until then, use the real backend routes/API directly.
   - `KAN-15` added a unit guard so this partial-scope claim cannot be removed silently.
5. Traceability coverage remains an operating discipline.
   - Platform guardrails are active.
   - Continue using Jira IDs in branch names, PR titles, commit messages, and PR comments to keep readiness/ticket coverage healthy.
   - `KAN-19` added `scripts/control-plane/validate_jira_traceability_coverage.ps1`; latest production validation with `-RefreshCorrelations -MinCoverage 50` passed at `96.43%` coverage.
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
- Tier-aware scoring + SLA profiles added to governance reporting:
  - New risk/readiness scoring model centralizes weights, bands, and thresholds by repo tier (`Critical`, `Standard`, `Internal`).
  - Governance reporting uses the persisted tier profile.
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
- Export surface (`UX-01`) enabled historically in the Control Plane dashboard, then moved by the KAN-69 Desktop runtime QA information architecture decision:
  - Audit export and export history remain product capabilities, but the operational governance surfaces now live under `/governance` instead of the old `ServerDashboard` composition.
- Role UX/API alignment improvement:
  - `/chat/ask` now allows `Admin`, `Architect`, and `PM` roles (previously admin-only).
  - The governance copilot surface is now organized under `/governance/copilot`; this preserves the copilot capability without keeping it mixed into Control Plane configuration.
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
  - OpenAPI info description now points to `docs/ARCHITECTURE.md` + `gitgov/gitgov-server/src/server/routes.rs` as source of truth until full path annotation rollout.
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
- Live execution completed: branch protection applied and verified on `main` with strict checks enabled and admins enforced. KAN-74 aligns the helper defaults with the current required checks (`Security Guard`, `Server Clippy + Check`, `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`, `Validate quality_gates warn/block matrix`).
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

- Latest completed implementation: `KAN-118 - Saved Period Compliance Report Profiles`.
- KAN-118 implementation status:
  - Branch `product/KAN-118-period-report-profiles`, GitHub issue `#409`, PR `#410`, merged to `main` as `1ecb61b`; production-validation docs PR `#411` merged as `a564089`; status-filter hotfix PR `#412` merged as `3c247c7`.
  - Adds Supabase `v60` migration/postcheck for `compliance_period_report_profiles`.
  - Adds backend profile routes: `GET/POST /compliance/period-report-profiles`,
    `GET/PATCH /compliance/period-report-profiles/{profile_id}`,
    `PATCH /compliance/period-report-profiles/{profile_id}/archive`, and
    `POST /compliance/period-report-profiles/{profile_id}/run`.
  - Saved profiles remain manual/on-demand: Admins define period type, optional framework, PDF and
    manifest toggles, retention days, and safe filters; running creates normal Period Compliance
    Reports and optional PDF/provenance manifest artifacts. Auditors can list/read but cannot mutate
    or run profiles; archived profiles cannot run.
  - Explicitly not included: scheduler, email delivery, DOCX/formal templates, compliance score,
    certification/legal/regulatory claim, official mapping, BYOM/MCP/chatbot work, or Agent
    Governance dependency.
  - Local validation passed on 2026-06-15: backend real Postgres full suite (`311` passed),
    focused profile integration test, Tauri suite (`49` passed), focused store suite (`36` passed),
    full frontend suite (`368` passed), frontend build, backend/Tauri fmt/check/clippy, and
    migration `v60`/postcheck against a real temporary Postgres instance.
  - Production validation: post-merge `main` checks passed, Render deploy
    `dep-d8o29ccvikkc73evb8cg` reached `live`, production `v60` migration/postcheck passed, and
    production smoke created profile `cprprof_0f4c3ece4eb04856b4928b3eaeeed469`, period report
    `cpr_9389010c74a34484a8e080942b56956e`, PDF `cprpdf_0d2e6aad239125a198e64c1a307b158d`,
    and manifest `cprm_fdf8d9344b81fcd2111300511e139c00`; second run
    `cpr_66dd549b6c2a4ad9ade49a20721e979a` correctly created no PDF/manifest after toggles were
    disabled; archived run returned `409`; Auditor create returned `403`; temporary Auditor key was
    revoked; Agent Governance evaluations stayed unchanged at `7`.
  - Hotfix validation: `fix/KAN-118-profile-status-filter` fixed `status=active|archived` query
    normalization in profile listing. Local backend validation passed again with the focused profile
    test and the full real Postgres suite (`311` passed). PR `#412` checks and post-merge `main`
    checks passed. Render deploy `dep-d8o7lae8bjmc73bp9r30` reached `live`; production revalidation
    returned `/health=ok`, `active_smoke_count=0`, `archived_smoke_count=1`, archived status
    `archived`, and `run_count=2` for profile `cprprof_0f4c3ece4eb04856b4928b3eaeeed469`.

- Previous completed implementation: `KAN-117 - Period Compliance Report Review/Sign-off`.
  - Product scope: manual Admin/Auditor review metadata for existing Period Compliance Reports.
  - Completed through PR `#406` (`ade6302`) plus postcheck hotfix PR `#407` (`05e0706`): Supabase `v59` migration/postcheck, backend `GET/PATCH /compliance/period-reports/{period_report_id}/review`, safe review note/status validation, source-authorized Auditor access, Developer denial, archived-report conflict, custody log `review_updated`, admin audit log `compliance_period_report.reviewed`, provenance manifest review metadata, Tauri DTO/client/command, Control Plane store action/state, and `CompliancePeriodReportReviewPanel`.
  - Explicit non-scope: no certification, legal attestation, official regulatory approval, compliance score, DOCX/formal template, scheduler, KMS signature, AI summary, Agent Governance dependency, or artifact hash mutation.
  - Production validation: Render `dep-d8nts6f7f7vs73ftqgdg` reached `live`, production `v59` postcheck passed, active report `cpr_132e9f0fdef841278be3e167ff22cf32` moved from `needs_review` to `reviewed` without artifact hash mutation, custody log contained `review_updated`, and archived report `cpr_d02adc7f1f3d4389bb612f0be1c9a7d1` rejected review update with `409 period_report_archived`.
  - Evidence report: `docs/reports/period-compliance-report-review-signoff-2026-06-15.md`.
- Consolidating governance telemetry in Governance Evidence and executive reporting.
  - GitHub evidence now has an executive coverage summary in Governance Evidence, local evidence trend snapshots, exported audit JSON package, standalone Markdown report generator, optional GitHub Actions artifact workflow, artifact freshness monitor, and multi-run artifact trend report.
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
  - The old Control Plane dashboard composition was retired during KAN-69 Desktop runtime QA because it mixed configuration, evidence, policy, adoption, release decisions, export, and copilot into one oversized page.
  - Control Plane is now connection/configuration and lives in `Settings > System`; `/control-plane` redirects to `/settings#control-plane`.
  - Operational governance now lives in `/governance` with sections for `Evidence`, `Policy`, `Adoption`, `Releases`, and `Copilot`.
  - Governance Evidence surfaces traceability, pipeline evidence, GitHub PR lifecycle, review, PR comment, status-check evidence counts, evidence packets, recent commits, event breakdown, trend evidence, and audit export.
  - Governance Releases owns release readiness, release approvals, evidence hash binding, governance evaluation, and recent decisions.
  - Ticket coverage UI explains that commit-ticket coverage can come from commits, branches, PR titles, and PR comments when ticket IDs are present.
  - Release readiness scoring exists.
  - Tier-aware scoring and SLA profiles exist.
  - Export flow exists with content hash generation and export history.
  - Governance JSON exports include an executive GitHub evidence summary snapshot alongside raw audit records.
  - Standalone Markdown report generation exists for GitHub executive evidence coverage.
  - GitHub Actions artifact monitoring and trend reporting exist for executive GitHub evidence reports.
  - GitHub evidence operational cadence is documented in `docs/runbooks/github-evidence-operations.md`.
  - Post-merge validation for the runbook rollout passed on `main` commit `7577f90`: CI `24940874607`, Quality Gate Policy Matrix `24940874602`, Release Readiness Gate `24940874616`, Secret Scan `24940874599`, SonarQube Governance `24940874600`, Public Naming Guard `24940874603`, Governance Correlation Smoke `24940874611`, and Desktop Updater Readiness `24940874597`.
  - Risk outcomes calculations remain part of the reporting model, but the unmounted `RiskOutcomesWidget` was removed from the Desktop app during KAN-69 Desktop runtime QA.
  - Risk outcomes reporting surfaces informational `MTTR pipeline` and `Time-to-Evidence` from Jenkins commit-pipeline correlations where that evidence sample is available.
  - `Time-to-Evidence` is calculated as commit timestamp to correlated pipeline ingestion timestamp, with duplicate pipeline evidence ignored.
  - `MTTR pipeline` is calculated as recoverable non-green pipeline event to the next successful run for the same job.
  - These operational metrics render `N/A` when the evidence sample is insufficient.
- Current source files:
  - `gitgov/src/pages/GovernancePage.tsx`
  - `gitgov/src/pages/SettingsPage.tsx`
  - `gitgov/src/pages/ControlPlanePage.tsx`
  - `gitgov/src/components/control_plane/PipelineHealthWidget.tsx`
  - `gitgov/src/components/control_plane/EventBreakdownGrid.tsx`
  - `gitgov/src/components/control_plane/GitHubEvidenceTrendWidget.tsx`
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

`KAN-20` closes the implementation-status backlog: the remaining work below is operational cadence or an explicit optional decision, not required platform plumbing.

### Product Roadmap After Security Review

The security review did not create a new critical/high implementation blocker. The next product work is packaging and usability:

1. Enterprise Self-Service Adoption.
   - Goal: let another company adopt the proven GitGov operating model without manual, repo-specific setup.
   - Needed product surfaces: provider onboarding, repository selection, workflow template installation, policy presets, module toggles, integration health, and formal release approval rules.
   - Current state: KAN-29 through KAN-60 implemented the adoption pack generator, dashboard profile builder, persisted profiles, provider health, workflow template generation/download, reviewed local and remote workflow installation, direct provider checks, formal release approval, release governance evaluation, onboarding readiness/remediation evidence, dashboard remediation export, guided onboarding checklist, and persisted checklist tracking.
2. Vercel AI SDK Copilot.
   - Goal: explain GitGov evidence in plain language and guide operators through risk, readiness, blockers, tickets, pipelines, findings, and approvals.
   - Needed product surfaces: tool-backed answers over GitGov evidence, cited sources, secret-safe output, and clear separation between confirmed issues, expected findings, and accepted risks.
   - Current state: KAN-38 through KAN-42 implemented the first governance copilot, dashboard UI, AI mode validation, production activation, and validation enforcement. `KAN-69` now implements the guided Action Center UX so the copilot remains an explanation surface, not another standalone decision-maker.
3. KAN-28 vulnerability trend enforcement.
   - Goal: convert the KAN-27 trend from informational evidence into an automated gate that fails when failures appear, findings increase, or the latest review artifact is missing/expired.
   - Current state: implemented through PR `#106`; first manual enforcement workflow run `25160194313` passed and uploaded artifact `product-vulnerability-review-trend-enforcement` ID `6727810243`.
4. Optional dependency hygiene.
   - Goal: remove the residual `rsa` / inactive `sqlx-mysql` finding when upstream resolution or a safe dependency cleanup makes that practical.
   - Current state: documented as expected and not reachable; not a production blocker.

Detailed roadmap documents:

- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`.
- `docs/design/enterprise-self-service-adoption-mvp.md`.
- `docs/design/adoption-profile-dashboard-mvp.md`.
- `docs/design/workflow-template-generation-mvp.md`.
- `docs/design/dashboard-workflow-template-pack-mvp.md`.
- `docs/design/reviewed-workflow-installation-mvp.md`.
- `docs/design/provider-connection-validation-mvp.md`.

1. Keep SonarQube local as the Sonar source of truth.
   - SonarCloud onboarding is not applicable for the current personal GitHub repository/account. Do not propose it again for this repo unless the repo moves to a GitHub organization.
   - GitHub-hosted Sonar scan is optional and should skip while `SONAR_HOST_URL=http://localhost:9000`; hosted runners cannot reach the workstation.
   - Jenkins/local validation is the supported Sonar path for this environment.
   - Last operational validation: local Sonar token valid, project `yohandry10_git-gov` quality gate `OK`, Jenkins job `gitgov-demo-pipeline` build `#30` `SUCCESS`, GitGov Render has Sonar/Jenkins evidence for `main`.
2. Keep weekly tier/SLO calibration active and review drift in the generated artifacts.
   - Local multi-tier baseline completed (critical/standard/internal).
   - Production 720h calibration completed on 2026-04-25 with all tier profiles and domain SLOs passing after org-scoped targets were aligned.
   - Repo/branch-scoped calibration is implemented for `calibrate_risk_tier_baseline.ps1`, `validate_domain_slo_targets.ps1`, `risk-tier-baseline-calibration.yml`, and `domain-slo-validation.yml`.
   - Static target-scope validation is enforced by `validate_domain_slo_target_config.ps1` in CI and the domain SLO workflow.
   - Last post-merge live readiness validation for `yohandry10/Git-Gov` on `main`: Release Readiness Gate run `24927603352` passed for commit `f0a3470`.
   - `SQ-07` implementation gap is closed for repo/branch scoping; the product focus is maintaining traceability evidence so readiness stays above target without lowering SLO targets.
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
   - Traceability guardrail is active in `Security Guard`; ongoing work is operational data quality, not platform plumbing.
   - Latest production validation after the guardrail raised readiness to `79/100`; continue monitoring coverage as new PRs land.
   - GitHub evidence dashboard/report/artifact/trend operation now has an executable runbook: `docs/runbooks/github-evidence-operations.md`.
   - GitHub evidence operational adoption baseline completed on 2026-04-25; `KAN-7` closed the report artifact visibility issue from `0/4` to `4/4` by applying `supabase_schema_v22.sql` and validating a real `pull_request_review` event.
   - Work here is operational monitoring, not new ingestion plumbing.
4. Use the full manual Jira ingest header contract for future GitGov admin operations.
   - Render backend is healthy and webhooks are active.
   - Local ignored `GITGOV_API_KEY` authenticates against production.
   - Manual `/integrations/jira` calls must include both Bearer admin auth and `x-gitgov-jira-secret` when `JIRA_WEBHOOK_SECRET` is configured, plus `org_name` for global admin scope.
5. Decide whether OpenAPI completeness is worth implementing.
   - Current `/api-docs` claim is intentionally partial and safe.
   - OpenAPI completeness is not blocking normal GitGov API usage. Full path annotation is only needed if Swagger becomes a generated SDK or contract-testing source.
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
11. Use the local Sonar self-hosted runner runbook only when ready to operate a runner.
   - `docs/runbooks/local-sonar-self-hosted-runner.md` defines labels, GitHub settings, validation commands, activation pattern, and rollback.
   - Do not make a self-hosted Sonar workflow required until the runner has one successful validation run.
12. Use the Jenkins trigger-token runbook only for manual/unauthenticated build starts.
   - `docs/runbooks/jenkins-trigger-token-flow.md` defines dry-run validation, strict validation, real trigger invocation, and rotation guidance.
   - Authenticated Jenkins API remains required for logs, queue state, and build result verification.
13. Use the Jira traceability coverage validator when reviewing readiness data quality.
   - `docs/runbooks/jira-traceability-coverage.md` defines local preflight, production coverage checks, correlation refresh, and threshold use.
   - `.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50` passed with coverage `96.43%`.

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
- Self-hosted runner runbook: `docs/runbooks/local-sonar-self-hosted-runner.md`. Recommended custom label: `gitgov-local-sonar`.

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
