# KAN-144 Native Terminal Local Provider/Tool Context Detection MVP

Date: 2026-06-16
Issue: `#498`

## Product Decision

Implement the next `0.10 Developer Distribution Surfaces` slice as local provider/tool context
detection for the Desktop native terminal.

KAN-143 added a safe provider/tool quick-command registry. KAN-144 makes that menu more useful by
detecting which local tools appear relevant in the current terminal workspace, without running
commands, reading secrets, calling the backend, or creating a second governance model.

This remains a convenience surface only.

## Scope

KAN-144 adds a local Tauri command:

```text
cmd_get_native_terminal_tool_context
```

The command returns safe metadata only:

- `cwd_kind`: `git_repo`, `non_git`, or `unknown`.
- detected tool records for Terraform, Docker Compose, Helm, and Kubernetes context.
- generic detection reasons such as `terraform_files_present`.
- safe KAN-143 command IDs associated with a detected tool.
- `scan_limited`, `secrets_read=false`, and `network_used=false`.

The command does not return absolute paths or file contents.

The Desktop terminal stores tool context by native terminal session and uses request-id protection so
stale async results cannot populate a newer session. The quick-command menu shows a small
`Available in this workspace` section when a detected local tool maps to KAN-143 safe commands. Other
safe commands remain available in a quiet `Other safe commands` section.

## Allowed Signals

Detection may inspect only file and directory names, bounded to the current cwd:

- Terraform:
  - `*.tf`
  - `*.tfvars.example`
  - `.terraform.lock.hcl`
- Docker Compose:
  - `compose.yaml`
  - `compose.yml`
  - `docker-compose.yml`
  - `docker-compose.yaml`
- Helm:
  - `Chart.yaml`
  - `templates/`
- Kubernetes local manifests:
  - `kustomization.yaml`
  - `kustomization.yml`
  - `manifests/`

Scan limits:

- maximum depth: `2`.
- maximum entries: `200`.
- ignored directories: `.git`, `.next`, `.terraform`, `build`, `dist`, `node_modules`, and `target`.

## Prohibited Behavior

- Reading file contents.
- Returning absolute paths.
- Reading `.env`, `*.tfvars`, `*.tfstate`, kubeconfig, cloud credentials, secret manifests, values
  files, or large files.
- Executing commands.
- Adding newline or auto-running quick commands.
- Command interception, approval, blocking, or rewriting.
- Backend/API/DB/Render changes.
- Control Plane audit writes.
- Provider, repository, cluster, deployment, or workflow mutation.
- Cloud CLI calls or token/API key usage.
- AI, Agent Governance, OPA, Rego, MCP, or chatbot dependency.
- Compliance, certification, legal, or regulatory claims.

## UX

The UI stays minimal:

- no modal.
- no banner.
- no popup.
- no approval wording.
- no "recommended deploy" wording.
- no local path display.
- detected labels are small chips inside the quick-command menu.

The feature must never make a command look approved by GitGov. It only says the local workspace has
safe signals for a tool.

## Acceptance Criteria

- Existing KAN-132 session history still works.
- Existing KAN-133 repo/branch context still works.
- Existing KAN-134/KAN-143 quick-command insert-only behavior still works.
- Existing KAN-135 governance drawer still works.
- Tool context refreshes after safe directory-change commands.
- Detected provider/tool commands appear under `Available in this workspace`.
- Other safe commands remain available without being marked as detected.
- No path, secret filename, file content, token, or credential is shown in the UI or returned by the
  tool context result.
- Scan limits and ignored directories are tested.
