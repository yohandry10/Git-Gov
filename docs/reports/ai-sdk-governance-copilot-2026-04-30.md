# KAN-38 Vercel AI SDK Governance Copilot MVP

Updated: 2026-04-30

## Summary

KAN-38 starts the Vercel AI SDK Copilot implementation.

This first MVP adds a server-side Next.js API route that gathers GitGov evidence, builds an evidence-grounded prompt, and uses Vercel AI SDK `generateText()` when AI Gateway/OIDC is available.

If AI generation is unavailable, it returns a deterministic evidence brief instead of exposing an error to the caller.

## Changes

- Created Jira issue `KAN-38 - Vercel AI SDK governance copilot MVP`.
- Added dependency `ai@^6.0.0` to `gitgov-web`.
- Added route:
  - `POST /api/copilot/governance`
- Added helper module:
  - `gitgov-web/lib/copilot/governance.ts`
- Added evidence collection for:
  - Evidence Packets.
  - Jira ticket coverage.
  - Formal release approvals.
  - Enterprise adoption profile.
- Added prompt guardrails requiring source citations and prohibiting invented provider/release/security state.
- Added fallback mode for environments without AI Gateway/OIDC.

## Security

- Caller-provided `Authorization: Bearer ...` is forwarded only to GitGov backend evidence endpoints.
- The route does not log or return Authorization headers.
- Server-key mode is disabled by default and requires `GITGOV_COPILOT_ACCESS_TOKEN`.
- Request body is limited to 12 KB.
- GitGov evidence base URL is not request-controlled.
- Non-HTTP GitGov base URLs are rejected.
- The prompt includes only summarized evidence, not raw secrets.

## Local Validation

Website typecheck:

```powershell
cd gitgov-web
pnpm run typecheck
```

Result:

- passed.

Website lint:

```powershell
cd gitgov-web
pnpm run lint
```

Result:

- passed.

Website build:

```powershell
cd gitgov-web
pnpm run build
```

Result:

- passed.
- Next.js registered dynamic route `/api/copilot/governance`.

Production dependency audit:

```powershell
cd gitgov-web
pnpm audit --prod
```

Result:

- no known vulnerabilities found.

Local route smoke:

```powershell
cd gitgov-web
pnpm exec next start -p 3108
```

Then `POST /api/copilot/governance` was called with a GitGov bearer token from ignored local env files and `GITGOV_COPILOT_DISABLE_AI=true`.

Result:

- `success=true`.
- `mode=fallback`.
- `4` citations.
- `4` evidence sources.
- `1` expected warning because local AI Gateway/OIDC generation was disabled.

Note: one parallel validation attempt ran `pnpm run typecheck` while `next build` was regenerating `.next/types`, which produced transient missing `.next/types` errors. Re-running `pnpm run typecheck` sequentially passed.

## Remaining Work

- Dashboard UI for governance copilot.
- Streaming chat UI using `@ai-sdk/react`.
- AI SDK `ToolLoopAgent` once tool-calling behavior is explicitly needed.
- MCP integration.
- Persisted copilot transcripts or evidence-linked explanations.
- Production runtime validation after merge/deploy with the configured GitGov bearer path and AI Gateway/OIDC state.
