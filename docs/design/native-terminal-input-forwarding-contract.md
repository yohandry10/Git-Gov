# Native Terminal Input Forwarding Contract

Updated: 2026-06-18
Ticket: `KAN-146`

## Decision

Close the remaining active `0.10 Developer Distribution Surfaces` guardrail by making the Desktop
native terminal non-interception behavior explicit and tested.

GitGov may observe manual terminal input locally for session history and safe context refresh. It
must still forward that input unchanged to the native PTY by default. Policy enforcement belongs in
reviewed GitGov flows such as Deployment Gates, not in an implicit local terminal filter.

## Product Scope

- Add a testable forwarding contract for manual native terminal input.
- Preserve the existing local observation behavior for session history and context refresh.
- Forward the original input bytes unchanged to `cmd_write_native_terminal`.
- Keep quick-command insertion separate from manual terminal input forwarding.

## Non-Scope

- No command blocking, approval, interception, filtering, rewriting, or auto-run.
- No backend route, API contract, DB migration, Control Plane audit write, or Render deploy.
- No provider, repository, cluster, deployment, or workflow mutation.
- No AI, Agent Governance, OPA/Rego, MCP, compliance score, certification, legal attestation, or
  regulatory claim.

## Contract

Manual terminal input forwarding returns:

```json
{
  "shouldForward": true,
  "interception": "none",
  "policyEvaluation": "not-run",
  "mutatesInput": false
}
```

The `data` field is the original input string.

## Validation Strategy

- Unit tests verify ordinary manual input is observed for history while forwarded unchanged.
- Unit tests verify compound-looking or redirected manual input is still forwarded unchanged.
- Unit tests verify control bytes and pasted multi-line input are preserved for the PTY.
- Existing terminal tests continue to cover session history, quick commands, governance context,
  branch gate status, Git context, and disabled terminal state.
