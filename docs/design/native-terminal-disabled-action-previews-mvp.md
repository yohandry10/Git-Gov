# Native Terminal Quiet Disabled Action Previews MVP

Updated: 2026-06-18
Ticket: `KAN-145`

## Decision

Continue `0.10 Developer Distribution Surfaces` with a minimal disabled-preview slice inside the
Desktop Workspace native terminal quick-command menu.

The preview must explain why some action categories are intentionally absent from shortcuts without
teaching users to run unsafe commands. This keeps the UI helpful for DevSecOps and platform users
while preserving the manual-first operating model.

## Product Scope

- Show passive advisory text only when local provider/tool context is detected.
- Explain excluded categories at a high level:
  - state-changing tool actions.
  - cloud/provider API actions.
  - secret or value inspection.
  - repository write actions.
- Keep the section visually quiet, below the existing safe quick-command groups.
- Keep the existing safe quick-command insertion behavior unchanged.

## Non-Scope

- No runnable unsafe command strings.
- No disabled buttons for unsafe commands.
- No command interception, blocking, approval, auto-run, or newline insertion.
- No backend route, API contract, DB migration, Control Plane audit write, or Render deploy.
- No provider, repository, cluster, deployment, or workflow mutation.
- No cloud/provider API calls, token reads, or secret reads.
- No AI, Agent Governance, OPA/Rego, MCP, compliance score, certification, legal attestation, or
  regulatory claim.

## Implementation Shape

- `terminalQuickCommands.ts` owns the preview metadata and returns previews only when the local
  provider/tool context reports at least one detected tool.
- `TerminalQuickCommandsMenu.tsx` renders the previews as static text. The rows are not buttons and
  do not call `onInsert`.
- Existing `SAFE_TERMINAL_QUICK_COMMANDS` remains the only source that can insert text into the
  native terminal.

## Validation Strategy

- Helper tests verify previews are hidden without detected provider/tool context.
- Helper tests verify preview objects do not expose a `command` property.
- UI tests verify preview labels are not inside buttons and do not call insertion callbacks.
- UI/helper tests check common unsafe command strings are absent from rendered preview text.
