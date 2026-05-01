# KAN-39 Governance Copilot Dashboard MVP

Updated: 2026-04-30

## Summary

KAN-39 adds the first operator dashboard UI for the KAN-38 governance copilot route.

The feature keeps the Vercel AI SDK route server-side and adds a Tauri desktop proxy command so the dashboard can use the existing GitGov API key without exposing it in a public web page.

## Merge And Traceability

- Jira issue: `KAN-39 - Governance copilot dashboard UI MVP`.
- Implementation branch: `product/KAN-39-governance-copilot-dashboard`.
- Implementation PR: `#129 - product(KAN-39): add governance copilot dashboard`.
- Merged commit: `eda2f13 product(KAN-39): add governance copilot dashboard`.
- Jira final comment: `10198`.

## Changes

- Created Jira issue `KAN-39 - Governance copilot dashboard UI MVP`.
- Added Tauri command:
  - `cmd_server_governance_copilot_ask`.
- Added dashboard store action:
  - `askGovernanceCopilot`.
- Added admin dashboard component:
  - `gitgov/src/components/control_plane/GovernanceCopilotPanel.tsx`.
- Added the panel to `ServerDashboard`.
- Added focused store tests for copilot success/error behavior.

## Security

- The dashboard browser does not call the public copilot endpoint directly.
- The Tauri command sends the configured GitGov API key only as an Authorization header to the copilot route.
- The default copilot URL is fixed to `https://www.gitgov.cloud/api/copilot/governance`.
- Optional `GITGOV_COPILOT_URL` is process-env controlled and must target an allowlisted GitGov/Vercel host or loopback URL with no embedded credentials.
- No provider tokens, Authorization headers, or `.env` values are displayed or persisted.

## Validation

Local validation:

- `cargo fmt` from `gitgov/src-tauri`: passed.
- `cargo check` from `gitgov/src-tauri`: passed.
- `cargo clippy -- -D warnings` from `gitgov/src-tauri`: passed.
- `cargo test` from `gitgov/src-tauri`: passed (`23` tests).
- `npm test -- --run src/test/useControlPlaneStore.test.ts` from `gitgov`: passed (`19` tests).
- `npm test -- --run` from `gitgov`: passed (`25` files, `278` tests).
- `npm run typecheck` from `gitgov`: passed.
- `npm run lint` from `gitgov`: passed.
- `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
- Local Vite smoke `GET http://127.0.0.1:5174/`: returned `200` with title `GitGov`.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

PR and post-merge validation completed after merge.

## Post-Merge Validation

GitHub checks passed on `main` commit `eda2f13`:

- `CI` - run `25195469511`.
- `Release Readiness Gate` - run `25195469482`.
- `Quality Gate Policy Matrix (Optional)` - run `25195469485`.
- `Secret Scan` - run `25195469486`.
- `Governance Correlation Smoke (Optional)` - run `25195469490`.
- `Desktop Updater Readiness (Optional)` - run `25195469496`.
- `SonarQube Governance (Non-Blocking)` - run `25195469502`.
- `Public Naming Guard` - run `25195469507`.

## Remaining Work

- Production AI generation mode remains separate from this UI MVP.
