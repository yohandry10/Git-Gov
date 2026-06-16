# GitGov VS Code Extension

KAN-136 adds a local VS Code developer surface for read-only GitGov governance context.

## Scope

- Detect the current workspace Git repository and branch.
- Store the GitGov API key in VS Code SecretStorage.
- Read existing GitGov Deployment Gate, Change Risk, and Executive Governance context.
- Render the context in the `GitGov Governance` tree view.

## Commands

- `GitGov: Configure Connection`
- `GitGov: Refresh Governance Context`
- `GitGov: Clear Connection`

## Guardrails

This extension is a convenience surface only. It does not enforce policy, intercept commands, execute deploys, mutate providers or repositories, create approvals, update CAB records, create Change Risk evaluations, use AI, or create compliance/certification/legal claims.

## Local Validation

```powershell
npm --prefix gitgov-vscode ci
npm --prefix gitgov-vscode run lint
npm --prefix gitgov-vscode run typecheck
npm --prefix gitgov-vscode test
npm --prefix gitgov-vscode run build
```
