# Governance Copilot AI Mode Validation

Updated: 2026-04-30

## Purpose

This runbook validates the production governance copilot route without printing secrets.

The check answers three operational questions:

- Does `POST /api/copilot/governance` respond successfully?
- Did it load enough GitGov evidence sources and citations?
- Is it running in `mode=ai`, or is it still using the deterministic `fallback` brief?

`fallback` is acceptable while no production AI provider is active. Once Google Gemini or AI Gateway is enabled for production, run the validator with `-RequireAiMode` or dispatch the workflow with `require_ai_mode=true`.

## Local Command

Run from the repository root:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -OutputPath out\governance-copilot-ai-mode-validation.json
```

Strict mode after AI provider activation:

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
| `ai` | Copilot used AI generation and evidence citations were present. | Target production state after Google Gemini or AI Gateway is active. |
| `fallback` | Copilot route worked, but AI generation was skipped or unavailable. | Acceptable before AI provider activation; investigate if strict mode was expected. |
| `failed` | Route, evidence thresholds, or strict AI requirement failed. | Review artifact and route configuration. |
| `skipped` | Workflow secret was not configured. | Add `GITGOV_API_KEY` if this validation should run in GitHub Actions. |

## Activation Notes

For this repository, the current production path is direct Google Gemini through `@ai-sdk/google`. This avoids the Vercel AI Gateway billing-card requirement while keeping the AI SDK abstraction and deterministic fallback.

Production Vercel environment variables:

```text
GOOGLE_GENERATIVE_AI_API_KEY
GITGOV_COPILOT_PROVIDER=google
GITGOV_COPILOT_GOOGLE_MODEL=gemini-2.5-flash
```

Do not prefix the Google API key with `NEXT_PUBLIC_`; it must stay server-only.

Operational sequence:

1. Add `GOOGLE_GENERATIVE_AI_API_KEY` to the Vercel project as a sensitive production environment variable.
2. Set `GITGOV_COPILOT_PROVIDER=google`.
3. Set `GITGOV_COPILOT_GOOGLE_MODEL=gemini-2.5-flash` unless a reviewed Gemini model change is intended.
4. Keep `GITGOV_COPILOT_DISABLE_AI` unset or not equal to `true`.
5. Deploy a build that includes `@ai-sdk/google` support.
6. Run this validator with `-RequireAiMode`.

AI Gateway remains an optional future provider path. If using Gateway instead, set `GITGOV_COPILOT_PROVIDER=gateway` and configure Vercel AI Gateway/OIDC or `AI_GATEWAY_API_KEY`.

Do not store provider API keys in docs, git history, command output, or browser-visible `NEXT_PUBLIC_*` variables.
