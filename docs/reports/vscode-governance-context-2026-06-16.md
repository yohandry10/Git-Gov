# VS Code Governance Context Report

Date: 2026-06-16
Ticket: KAN-136
Issue: #476
PR: #477
Main commit: 23c8b551

## Summary

KAN-136 implements the first VS Code distribution surface for GitGov governance context. The MVP is
read-only and uses existing GitGov APIs; it does not add backend routes, persistence, enforcement,
deploy execution, provider/repo mutation, AI, Agent Governance, or compliance/certification claims.

## Implemented

- Added `gitgov-vscode` package.
- Added VS Code extension manifest with `GitGov Governance` tree view.
- Added commands:
  - `GitGov: Configure Connection`
  - `GitGov: Refresh Governance Context`
  - `GitGov: Clear Connection`
- Added SecretStorage-backed API key handling.
- Added read-only Git context detection.
- Added read-only GitGov HTTP client for:
  - `/deployment-gates/authorizations`
  - `/change-risk/evaluations`
  - `/executive/repositories`
- Added CI job for the extension.

## Validation

Local validation:

- `npm --prefix gitgov-vscode ci`
  - Result: passed, `0` vulnerabilities.
- `npm --prefix gitgov-vscode run lint`
  - Result: passed.
- `npm --prefix gitgov-vscode run typecheck`
  - Result: passed for extension source and tests.
- `npm --prefix gitgov-vscode test`
  - Result: `4` files passed, `11` tests passed.
- `npm --prefix gitgov-vscode run build`
  - Result: passed.
- `git diff --check`
  - Result: passed.
- `powershell -ExecutionPolicy Bypass -File scripts/security/publication_guard.ps1`
  - Result: passed.
- Static grep over extension source/tests/docs verified the only HTTP method used by the client is
  `GET`; mutating words appear only in documented no-goals or test assertions.

PR checks passed:

- Security Guard.
- Frontend Lint + Typecheck.
- Desktop Rust Clippy.
- Server Clippy + Check.
- VS Code Extension Lint + Typecheck + Test.
- Website Lint + Typecheck + Build.
- Validate Policy-as-Code.
- Validate quality_gates warn/block matrix.
- Workflow Lint.
- Sonar Scan + Quality Gate.
- Vercel.
- Block internal-assistant markers in branch/commits.

No Render/API deploy was required because KAN-136 is a local VS Code extension and reuses existing
read-only endpoints.
