# KAN-38 Vercel AI SDK Governance Copilot MVP

Updated: 2026-04-30

## Summary

KAN-38 starts the Vercel AI SDK Copilot implementation.

This first MVP adds a server-side Next.js API route that gathers GitGov evidence, builds an evidence-grounded prompt, and uses Vercel AI SDK `generateText()` when AI Gateway/OIDC is available.

If AI generation is unavailable, it returns a deterministic evidence brief instead of exposing an error to the caller.

## Merge And Traceability

- Jira issue: `KAN-38 - Vercel AI SDK governance copilot MVP`.
- Implementation branch: `product/KAN-38-ai-sdk-copilot`.
- Implementation PR: `#127 - product(KAN-38): add AI SDK governance copilot`.
- Merged commit: `9742472 product(KAN-38): add AI SDK governance copilot`.
- Jira final comment: `10197`.

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

## Post-Merge Validation

GitHub checks passed on `main` commit `9742472`:

- `CI` - run `25194421718`.
- `Release Readiness Gate` - run `25194421743`.
- `Quality Gate Policy Matrix (Optional)` - run `25194421721`.
- `Secret Scan` - run `25194421747`.
- `Public Naming Guard` - run `25194421752`.
- `SonarQube Governance (Non-Blocking)` - run `25194421756`.
- `Governance Correlation Smoke (Optional)` - run `25194421750`.
- `Desktop Updater Readiness (Optional)` - run `25194421717`.

Vercel deployment:

- Deployment URL: `https://git-ih2bzdqq5-trivia1.vercel.app`.
- Status: `Ready`.
- Aliases include `https://www.gitgov.cloud`, `https://git-gov.vercel.app`, and `https://gitgov.cloud`.

Production route smoke:

- `POST https://www.gitgov.cloud/api/copilot/governance` returned `200`.
- `POST https://git-gov.vercel.app/api/copilot/governance` returned `200`.
- Sanitized result: `success=true`, `mode=fallback`, `4` citations, `4` evidence sources, and `1` expected warning.
- Direct deploy URL returned `401` with HTML, consistent with Vercel deployment URL protection rather than the application route.
- Apex `https://gitgov.cloud/api/copilot/governance` returned `401`; the canonical `www` and Vercel production aliases were used for validation.

Interpretation:

- The product endpoint is live and can generate a deterministic evidence brief from real GitGov evidence.
- AI generation did not run in production during this validation because AI Gateway/OIDC was not available to the route, so the expected fallback mode was used.
- No secret values, Authorization headers, or provider credentials were printed during validation.

## Remaining Work

- Dashboard UI for governance copilot is started in `KAN-39`.
- Streaming chat UI using `@ai-sdk/react`.
- AI SDK `ToolLoopAgent` once tool-calling behavior is explicitly needed.
- MCP integration.
- Persisted copilot transcripts or evidence-linked explanations.
- Enable and validate production AI Gateway/OIDC if the desired production behavior is `mode=ai` instead of deterministic `mode=fallback`.
