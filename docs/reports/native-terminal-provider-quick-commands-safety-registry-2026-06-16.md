# Native Terminal Provider Quick Commands Safety Registry - 2026-06-16

Ticket: `KAN-143`
Issue: `#495`
Branch: `product/KAN-143-provider-quick-commands-registry`

## Summary

KAN-143 implements the next `0.10 Developer Distribution Surfaces` terminal slice after KAN-141 and
KAN-142. It adds a local provider/tool quick-command safety registry to the existing native terminal
quick-command menu.

The feature remains manual-first:

- commands are inserted only.
- no newline is added.
- no command is executed automatically.
- no backend or provider API is called.
- no audit evidence or enforcement decision is created.

## Implemented

- Added safety metadata to every terminal quick command.
- Changed read-only validation to an exact enabled allowlist rather than broad command parsing.
- Added provider/tool commands for Terraform, Kubernetes local config, Docker Compose config, and
  local Helm lint.
- Replaced the existing `git remote -v` quick command with `git remote` to avoid printing remote
  URLs that could contain misconfigured credentials.
- Grouped the menu into `Git inspection` and `Provider / Tool context`.
- Added focused helper and UI tests for:
  - safety metadata.
  - exact registry acceptance.
  - rejection of mutating/network/secret-exposing commands.
  - provider/tool insert-only behavior.
  - disabled reasons outside a Git repository.
  - no local cwd leakage in labels/descriptions.

## Explicitly Rejected Commands

The registry does not include:

- `terraform plan`, `terraform apply`, `terraform destroy`, or `terraform output -json`.
- `kubectl get`, `kubectl apply`, `kubectl delete`, or rollout commands.
- `helm install`, `helm upgrade`, or `helm uninstall`.
- `docker compose up`, `docker compose down`, or `docker compose logs`.
- `aws`, `az`, `gcloud`, `vercel`, or `render` CLI commands.
- `env`, `printenv`, `cat .env`, redirection, shell chaining, or command substitution.
- `git remote -v` because remote URLs can accidentally contain credentials.

## Validation

Passed locally:

```powershell
npm --prefix gitgov run test -- --run src/test/components/terminal-quick-commands.test.ts
npm --prefix gitgov run test -- --run src/test/components/terminal-quick-commands.test.ts src/test/components/terminal-quick-commands-menu.test.tsx
npm --prefix gitgov run test -- --run src/test/components/terminal-quick-commands.test.ts src/test/components/terminal-quick-commands-menu.test.tsx src/test/components/terminal-branch-gate-status.test.tsx src/test/components/terminal-governance-context.test.ts src/test/components/terminal-git-context.test.ts src/test/components/terminal-session-history.test.ts src/test/components/terminal-status.test.ts
npm --prefix gitgov run typecheck
npm --prefix gitgov run lint
npm --prefix gitgov run test -- --run
npm --prefix gitgov run build
git diff --check
.\scripts\security\publication_guard.ps1
```

Results:

- quick-command helper tests: `7` passed.
- quick-command helper + menu tests: `10` passed.
- focused terminal suite: `36` passed.
- frontend typecheck passed.
- frontend lint passed.
- full frontend Vitest: `419` passed.
- frontend build passed with the pre-existing Vite large chunk warning.
- `git diff --check` passed.
- publication guard passed.
- static product-code grep found no mutating/network/secret-exposing provider commands in
  `gitgov/src/components/cli`.

## Guardrails

- No backend/API route change.
- No DB migration.
- No Render deploy requirement.
- No Control Plane audit write.
- No command interception, approval, blocking, or auto-run.
- No provider, repository, cluster, deployment, or workflow mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.
