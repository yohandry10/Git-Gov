# Governance Copilot AI Mode Validation

Updated: 2026-04-30

## Purpose

This runbook validates the production governance copilot route without printing secrets.

The check answers three operational questions:

- Does `POST /api/copilot/governance` respond successfully?
- Did it load enough GitGov evidence sources and citations?
- Is it running in `mode=ai`, or is it still using the deterministic `fallback` brief?

`fallback` is acceptable while Vercel AI Gateway/OIDC is not active. Once AI Gateway/OIDC is enabled for production, run the validator with `-RequireAiMode` or dispatch the workflow with `require_ai_mode=true`.

## Local Command

Run from the repository root:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -OutputPath out\governance-copilot-ai-mode-validation.json
```

Strict mode after AI Gateway/OIDC activation:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\governance-copilot-ai-mode-validation.json
```

The script loads `GITGOV_API_KEY` from ignored local env files by default:

- `gitgov\.env`
- `gitgov\gitgov-server\.env`

It does not print the key, Authorization header, provider tokens, or raw answer text.

## GitHub Workflow

Workflow:

```text
.github/workflows/governance-copilot-ai-mode-validation.yml
```

Default behavior:

- scheduled weekly on Monday at `13:31 UTC`;
- manual dispatch supports `require_ai_mode=false|true`;
- uses `secrets.GITGOV_API_KEY`;
- optionally uses repository variable `GITGOV_COPILOT_URL`;
- uploads artifact `governance-copilot-ai-mode-validation`.

## Expected States

| State | Meaning | Action |
| --- | --- | --- |
| `ai` | Copilot used AI generation and evidence citations were present. | Target production state after AI Gateway/OIDC is active. |
| `fallback` | Copilot route worked, but AI generation was skipped or unavailable. | Acceptable before AI Gateway/OIDC activation; investigate if strict mode was expected. |
| `failed` | Route, evidence thresholds, or strict AI requirement failed. | Review artifact and route configuration. |
| `skipped` | Workflow secret was not configured. | Add `GITGOV_API_KEY` if this validation should run in GitHub Actions. |

## Activation Notes

For AI SDK v6, GitGov should prefer Vercel AI Gateway with OIDC rather than provider-specific API keys.

Operational sequence:

1. Enable AI Gateway for the Vercel project.
2. Ensure the production deployment has OIDC available.
3. Keep `GITGOV_COPILOT_DISABLE_AI` unset or not equal to `true`.
4. Keep `GITGOV_COPILOT_MODEL` on a valid `provider/model` slug if overriding the default.
5. Run this validator with `-RequireAiMode`.

Do not store provider API keys in docs or command output.
