# KAN-41 Google AI SDK Copilot Provider

Updated: 2026-05-01

## Summary

KAN-41 activates a practical production AI path for the Governance Copilot without requiring Vercel AI Gateway billing.

The selected path is direct Google Gemini through Vercel AI SDK provider package `@ai-sdk/google`. The existing deterministic fallback remains in place and the current bot flow is not removed.

## Decision

- Use `@ai-sdk/google` for production copilot generation.
- Read Gemini credentials only from server-side environment variables.
- Keep Vercel AI Gateway as an optional future provider, not the current blocker.
- Keep `fallback` behavior when no AI provider is configured or generation fails.
- Do not add provider keys to docs, git, command output, or browser-visible `NEXT_PUBLIC_*` variables.

## Vercel Configuration

Configured on Vercel project `trivia1/git-gov` for `Production` only:

```text
GOOGLE_GENERATIVE_AI_API_KEY
GITGOV_COPILOT_PROVIDER=google
GITGOV_COPILOT_GOOGLE_MODEL=gemini-2.5-flash
```

The key value was loaded from ignored local env files and was not printed.

Preview remains unconfigured for Gemini so PR deployments continue to use fallback unless a preview-specific decision is made later.

## Implementation

- Added `@ai-sdk/google` to `gitgov-web`.
- Updated `POST /api/copilot/governance` to resolve provider mode:
  - `google` uses `createGoogleGenerativeAI()` with `gemini-2.5-flash`.
  - `gateway` keeps the previous AI Gateway-compatible string model path.
  - `disabled` skips generation.
  - `auto` chooses Google when a Google key is available, otherwise Gateway if configured, otherwise fallback.
- Updated fallback text and runbook language to mention Google Gemini and AI Gateway.

## Local Validation

Commands:

```powershell
cd gitgov-web
pnpm run typecheck
pnpm run lint
pnpm run build
```

Results:

- `pnpm run typecheck`: passed.
- `pnpm run lint`: passed.
- `pnpm run build`: passed; Next.js registered `/api/copilot/governance`.

Local route smoke:

- local `next start` on loopback.
- `GITGOV_COPILOT_PROVIDER=google`.
- `GOOGLE_GENERATIVE_AI_API_KEY` mapped from ignored local `GEMINI_API_KEY`.
- caller GitGov bearer token loaded from ignored local env files.

Sanitized result:

- HTTP status: `200`.
- success: `true`.
- mode: `ai`.
- model: `google/gemini-2.5-flash`.
- citations: `4`.
- sources: `4`.
- ok sources: `4`.
- warnings: `0`.
- raw answer was not stored in this report.

## Production Validation Plan

After merge and Vercel production deploy:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\KAN-41-governance-copilot-google-ai-mode-validation.json
```

Expected result:

- HTTP `200`.
- `success=true`.
- `mode=ai`.
- `model=google/gemini-2.5-flash`.
- evidence thresholds pass.
- no provider secrets printed or stored.

## Residual Risk

- Gemini free-tier quota can still force fallback or temporary failures under usage spikes.
- Preview deployments intentionally remain fallback-only unless preview Gemini env vars are configured later.
- AI Gateway remains blocked by Vercel billing-card requirements; this is no longer the selected production path.
