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
- Current production behavior is `mode=fallback` because AI Gateway/OIDC generation was not active during validation. This is the expected safe behavior until production AI generation is explicitly enabled.

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

Default model:

```text
openai/gpt-5.4
```

Model override:

```text
GITGOV_COPILOT_MODEL
```

AI generation is attempted only when the runtime looks AI Gateway/OIDC-capable. If AI generation is unavailable, the route returns a deterministic evidence brief instead of failing.

This follows the current AI SDK v6 guidance:

- `generateText()` is the basic text generation API.
- plain `provider/model` strings can route through Vercel AI Gateway.
- `maxOutputTokens` and `temperature` are passed as generation settings.

References:

- `https://ai-sdk.dev/docs/ai-sdk-core/generating-text`
- `https://ai-sdk.dev/docs/reference/ai-sdk-core/generate-text`

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

- `ai` when Vercel AI SDK generation succeeded.
- `fallback` when AI generation was skipped or unavailable.

## Non-Goals

- No dashboard chat UI in this ticket.
- No AI SDK tool-loop agent yet.
- No MCP integration.
- No model-side writes or approvals.
- No direct provider mutation.
- No replacement for the existing Rust `/chat/ask` endpoint.

Follow-up work can add dashboard UI and agent/tool-loop behavior after this route is validated.
