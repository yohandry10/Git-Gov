# VS Code Governance Context MVP

Ticket: KAN-136

## Decision

KAN-136 continues `0.10 Developer Distribution Surfaces` after the native terminal slices
KAN-132 through KAN-135. The terminal now has local history, repo/branch context, safe read-only
quick commands, and a read-only governance context drawer. The next useful developer surface is VS
Code, where developers can inspect governance state for the current workspace without opening the
Desktop app.

## Scope

- Add a new local `gitgov-vscode` VS Code extension package.
- Detect the current workspace Git repository and branch with read-only Git commands.
- Normalize GitHub remotes into `owner/repo` for GitGov filtering.
- Configure GitGov API URL and org through VS Code settings.
- Store the GitGov API key in VS Code SecretStorage.
- Read existing Deployment Gate, Change Risk, and Executive Governance endpoints with GET requests.
- Show a read-only `GitGov Governance` tree view.
- Add commands to configure, refresh, and clear the connection.

## Guardrails

- No backend/API/DB change.
- No Render deploy.
- No enforcement, approvals, command interception, deploy execution, or provider/repo mutation.
- No Git push/pull/fetch/checkout.
- No CAB review updates or Change Risk creation.
- No Agent Governance, AI/LLM/BYOM/MCP/chatbot dependency.
- No compliance/certification/legal claims.
- No API key storage in plain settings and no secret logging.
- No background polling.

## Validation Strategy

The implementation keeps most logic outside `vscode` imports so it can be tested directly:

- Real temporary Git repository detection.
- Non-git workspace safe state.
- SecretStorage-compatible API key store/read/delete behavior.
- GET-only endpoint allowlist.
- 401/403 error sanitization without token leakage.
- Governance snapshot loading without calling GitGov when repo/config/key prerequisites are absent.

CI runs the extension package separately with lint, typecheck, tests, and build.
