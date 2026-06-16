# KAN-143 Native Terminal Provider Quick Commands Safety Registry MVP

Date: 2026-06-16
Issue: `#495`

## Product Decision

Implement the next `0.10 Developer Distribution Surfaces` slice as a local provider/tool quick
command safety registry inside the Desktop native terminal.

The feature is a convenience surface only. It helps a developer insert reviewed read-only inspection
commands while they remain in the terminal. It does not run commands automatically and does not
create a second governance or enforcement model.

External editor extensions remain out of scope. The active direction stays focused on
Desktop/Workspace native terminal surfaces.

## Scope

KAN-143 extends the existing KAN-134 quick-command menu with a second grouped section:

- `Git inspection`
- `Provider / Tool context`

The provider/tool registry initially includes only local read-only commands:

- `terraform fmt -check -recursive`
- `terraform validate -no-color`
- `kubectl config current-context`
- `kubectl config get-contexts`
- `docker compose config --services`
- `docker compose config --quiet`
- `helm lint .`

Every command carries safety metadata:

- command group and tool.
- enabled state.
- safety level.
- whether network access is required.
- whether the command may expose secrets.
- whether a Git repository terminal is required.

The insert path accepts only exact enabled registry commands. Broad parsing of arbitrary
provider/tool commands is intentionally not supported.

KAN-143 also tightens the existing Git quick-command registry by using `git remote` instead of
`git remote -v`, so the menu lists remote names without printing remote URLs that could contain
misconfigured credentials.

## Out Of Scope

- Auto-run or appending newline.
- Command interception, approval, blocking, or rewriting.
- Backend/API/DB/Render changes.
- Control Plane audit writes.
- Provider, repository, cluster, deployment, or workflow mutation.
- Cloud API commands such as `aws`, `az`, `gcloud`, `vercel`, or `render`.
- Mutating local/provider commands such as `terraform apply`, `terraform destroy`, `kubectl apply`,
  `kubectl delete`, `helm install`, `helm upgrade`, `docker compose up`, or `docker compose down`.
- Commands that dump environment variables, `.env` files, tokens, credentials, or secrets.
- Remote URL printing such as `git remote -v`.
- AI, Agent Governance, OPA, Rego, MCP, or chatbot dependency.
- Compliance, certification, legal, or regulatory claims.

## Safety Model

The registry is an exact allowlist. A command is insertable only when all of these are true:

- it is present in the local registry.
- it is enabled.
- it has `requiresNetwork=false`.
- it has `mayExposeSecrets=false`.
- it does not contain shell chaining, redirection, command substitution, or newline characters.
- the terminal is inside a Git repository when the command requires a repo context.

Disabled commands show a reason and are not inserted.

## Acceptance Criteria

- Existing Git quick commands still work.
- Provider/tool commands are grouped separately and do not clutter the terminal header.
- Provider/tool commands are inserted through the existing insert-only path.
- Inserted text has no newline and is not executed automatically.
- Mutating, networked, secret-exposing, compound, redirected, or non-registry commands are rejected.
- The feature does not call the backend or mutate any provider/repository/deployment state.
- UI text does not expose the local working directory.
