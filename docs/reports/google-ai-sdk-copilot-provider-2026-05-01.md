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

Configuration correction:

- The first production validation attempt showed the initial uploaded key value contained an invisible UTF-8 BOM character, which made the Google provider reject the header before reaching Gemini.
- After stripping BOM/whitespace in code, production validation showed the first local env file contained an expired Gemini key.
- The production Vercel secret was reconfigured from the effective local Gemini key used by the working local/server bot path, without printing the value.
- `POST /api/copilot/governance` now strips a leading BOM and surrounding whitespace from server-side Google/Gemini env values before creating the provider client.

Preview remains unconfigured for Gemini so PR deployments continue to use fallback unless a preview-specific decision is made later.

## Implementation

- Added `@ai-sdk/google` to `gitgov-web`.
- Updated `POST /api/copilot/governance` to resolve provider mode:
  - `google` uses `createGoogleGenerativeAI()` with `gemini-2.5-flash`.
  - `gateway` keeps the previous AI Gateway-compatible string model path.
  - `disabled` skips generation.
  - `auto` chooses Google when a Google key is available, otherwise Gateway if configured, otherwise fallback.
- Updated fallback text and runbook language to mention Google Gemini and AI Gateway.
- Added sanitized AI generation diagnostics that expose only safe error class/code/status/message fragments after redacting known secrets, bearer tokens, and `key=` query values.
- Added server-side cleanup for Google/Gemini env values before using them as provider keys.

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

## Production Validation

Command:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\KAN-41-governance-copilot-google-ai-mode-validation.json
```

Final production deployment:

- URL: `https://git-8gwowu155-trivia1.vercel.app`.
- Aliased to `https://www.gitgov.cloud`.
- Trigger: Vercel redeploy after correcting `GOOGLE_GENERATIVE_AI_API_KEY`.

Final sanitized result:

- HTTP `200`.
- `success=true`.
- `mode=ai`.
- `model=google/gemini-2.5-flash`.
- citations: `4`.
- sources: `4`.
- ok sources: `4`.
- warnings: `0`.
- answer length: `419`.
- answer SHA-256: `dcfc9ec8f49a5c91ccd0e0fafc0df4bef1903c9704a7349b7d27c5c8bd2f72d3`.
- no provider secrets, Authorization headers, or raw answer text were printed or stored.

Post-merge checks for implementation commit `ba61d16` passed:

- `CI` - run `25199526039`.
- `Release Readiness Gate` - run `25199526047`.
- `Quality Gate Policy Matrix (Optional)` - run `25199526028`.
- `Secret Scan` - run `25199526038`.
- `Governance Correlation Smoke (Optional)` - run `25199526055`.
- `SonarQube Governance (Non-Blocking)` - run `25199526033`.
- `Public Naming Guard` - run `25199526037`.
- `Desktop Updater Readiness (Optional)` - run `25199526031`.

## Residual Risk

- Gemini free-tier quota can still force fallback or temporary failures under usage spikes.
- Preview deployments intentionally remain fallback-only unless preview Gemini env vars are configured later.
- AI Gateway remains blocked by Vercel billing-card requirements; this is no longer the selected production path.
