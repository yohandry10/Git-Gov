# KAN-39 Governance Copilot UI MVP

Updated: 2026-04-30

## Summary

KAN-39 turns the KAN-38 server-side copilot route into a usable operator surface inside GitGov Desktop.

The MVP adds an operator copilot panel that asks governance/readiness questions, sends bounded context to the Vercel AI SDK copilot route, and renders the answer with citations, source statuses, and warnings.

## Scope

- Add `GovernanceCopilotPanel` to Desktop. After the KAN-69 Desktop runtime QA information-architecture decision, this panel belongs in `Governance > Copilot`.
- Add a Tauri command that calls `POST /api/copilot/governance` from the desktop side.
- Reuse the configured GitGov API key only as a Bearer token for the copilot route.
- Keep the copilot endpoint URL out of browser-controlled state.
- Show:
  - answer mode: `ai` or `fallback`.
  - response text.
  - citation pills.
  - source status table.
  - warnings.

## Security Model

- The browser UI does not call the public website endpoint directly.
- The Tauri command performs the HTTPS request, avoiding browser CORS workarounds.
- The GitGov API key is never displayed in UI and is not written into docs, logs, or response output.
- The default copilot URL is `https://www.gitgov.cloud/api/copilot/governance`.
- Optional `GITGOV_COPILOT_URL` is read from the desktop process environment for local/dev override and must target an allowlisted GitGov/Vercel host or loopback URL, with no embedded credentials.
- The request is limited to product context fields: question, org, repository, branch, ticket, release, environment, and lookback hours.

## Non-Goals

- No streaming chat UI yet.
- No autonomous tool-loop agent.
- No model-side writes or approvals.
- No production AI Gateway/OIDC configuration change.
- No direct customer repository mutation.

## Follow-Ups

- Enable production AI generation mode if `mode=ai` is required.
- Add streaming via `@ai-sdk/react`.
- Add persisted copilot transcripts if audit retention becomes a requirement.
- Add ToolLoopAgent/MCP only after read-only copilot behavior is validated.
