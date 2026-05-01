# KAN-38 Vercel AI SDK Governance Copilot MVP

Updated: 2026-04-30

## Summary

KAN-38 starts the Vercel AI SDK Copilot feature.

This MVP adds a server-side Next.js API route that gathers GitGov evidence first, then uses Vercel AI SDK to generate a concise governance answer with cited source IDs.

The route is deliberately not an autonomous agent yet. It is an evidence brief generator. That keeps the first AI feature predictable: GitGov fetches bounded evidence, and the model explains only that evidence.

## Validation Status

- Implementation PR `#127` merged on `main` as `9742472`.
- Production Vercel deployment `https://git-ih2bzdqq5-trivia1.vercel.app` reached `Ready`.
- Production route smoke passed on `https://www.gitgov.cloud/api/copilot/governance` and `https://git-gov.vercel.app/api/copilot/governance`.
- KAN-38 production validation originally returned `mode=fallback` because AI Gateway/OIDC generation was not active. KAN-41 selects direct Google Gemini through `@ai-sdk/google` as the practical production AI path, while preserving fallback when provider generation is unavailable.

## Route

```text
POST /api/copilot/governance
```

Location:

```text
gitgov-web/app/api/copilot/governance/route.ts
```

Shared evidence helpers:

```text
gitgov-web/lib/copilot/governance.ts
```

## Request

```json
{
  "question": "Is KAN-37 ready for production?",
  "org_name": "yohandry10",
  "repository_full_name": "yohandry10/Git-Gov",
  "branch": "main",
  "ticket_id": "KAN-37",
  "release_id": "KAN-37-runtime-smoke",
  "environment": "production",
  "hours": 720
}
```

The route accepts either snake_case or camelCase input names for the common fields.

## Authentication

Default mode requires the caller to send a GitGov admin API key as:

```text
Authorization: Bearer <gitgov-api-key>
```

The route forwards that header to GitGov backend evidence endpoints and never returns it.

Optional server-key mode exists but is intentionally gated:

- `GITGOV_COPILOT_USE_SERVER_API_KEY=true`
- `GITGOV_API_KEY`
- `GITGOV_COPILOT_ACCESS_TOKEN`
- request header `x-gitgov-copilot-token`

This prevents the public website from becoming an unauthenticated proxy to private GitGov evidence.

## Evidence Sources

The MVP fetches bounded evidence from:

- `GET /evidence/packets/tickets/{ticket_id}`
- `GET /integrations/jira/ticket-coverage`
- `GET /enterprise/release-approvals`
- `GET /enterprise/adoption-profile`

The response includes source metadata:

```json
{
  "id": "evidence-packet",
  "label": "Evidence Packet KAN-37",
  "endpoint": "/evidence/packets/tickets/KAN-37?...",
  "status": "ok"
}
```

## AI SDK Usage

The route imports:

```ts
import { generateText } from 'ai'
```

KAN-41 adds direct Google Gemini provider support through:

```ts
import { createGoogleGenerativeAI } from '@ai-sdk/google'
```

Provider selection:

```text
GITGOV_COPILOT_PROVIDER=auto|google|gateway|disabled
```

Current production decision:

- `google` is the selected provider for production because Vercel AI Gateway generation requires a billing card on the Vercel account.
- AI Gateway remains supported as an optional future provider path.
- deterministic fallback remains the safe behavior when no AI provider is configured or generation fails.

Default Google model:

```text
gemini-2.5-flash
```

Google model override:

```text
GITGOV_COPILOT_GOOGLE_MODEL
```

Gateway default model:

```text
openai/gpt-5.4
```

Gateway model override:

```text
GITGOV_COPILOT_GATEWAY_MODEL
```

AI generation is attempted only when the selected provider has the required server-side runtime configuration. If AI generation is unavailable, the route returns a deterministic evidence brief instead of failing.

This follows the current AI SDK v6 guidance:

- `generateText()` is the basic text generation API.
- `@ai-sdk/google` exposes the Google Generative AI provider.
- `google('gemini-2.5-flash')` calls Gemini directly with the Google API key.
- plain `provider/model` strings can route through Vercel AI Gateway.
- `maxOutputTokens` and `temperature` are passed as generation settings.

References:

- `https://ai-sdk.dev/docs/ai-sdk-core/generating-text`
- `https://ai-sdk.dev/docs/reference/ai-sdk-core/generate-text`
- `https://ai-sdk.dev/providers/ai-sdk-providers/google-generative-ai`

## Guardrails

- no browser-side model token.
- no raw Authorization header in output.
- no provider credential logging.
- request body is limited to 12 KB.
- repository and Jira ticket fields are validated.
- evidence base URL comes from environment/default only, not request body.
- non-HTTP GitGov base URLs are rejected.
- responses must cite source IDs such as `[source:evidence-packet]`.
- prompt instructs the model not to invent approvals, provider state, vulnerabilities or production status.

## Response

Key fields:

```json
{
  "success": true,
  "mode": "ai",
  "model": "openai/gpt-5.4",
  "answer": "...",
  "citations": [],
  "sources": [],
  "warnings": []
}
```

`mode` is:

- `ai` when Vercel AI SDK generation succeeded through Google Gemini or AI Gateway.
- `fallback` when AI generation was skipped or unavailable.

## Non-Goals

- No dashboard chat UI in this ticket.
- No AI SDK tool-loop agent yet.
- No MCP integration.
- No model-side writes or approvals.
- No direct provider mutation.
- No replacement for the existing Rust `/chat/ask` endpoint.

Follow-up work can add dashboard UI and agent/tool-loop behavior after this route is validated.
