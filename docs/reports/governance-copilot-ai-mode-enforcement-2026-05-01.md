# KAN-42 Governance Copilot AI Mode Enforcement

Updated: 2026-05-01

## Summary

KAN-42 turns Governance Copilot AI mode validation from an optional activation check into the default production expectation.

After KAN-41, production is configured to use direct Google Gemini through `@ai-sdk/google`. A silent return to deterministic `fallback` would hide a provider, environment, quota, or deployment regression. The weekly validation workflow now requires `mode=ai` by default.

## Scope

- Jira issue: `KAN-42 - Enforce governance copilot AI mode validation`.
- Implementation branch: `ops/KAN-42-enforce-copilot-ai-validation`.
- Implementation PR: `#138 - ops(KAN-42): enforce governance copilot AI validation`.
- Workflow: `.github/workflows/governance-copilot-ai-mode-validation.yml`.
- Runbook: `docs/runbooks/governance-copilot-ai-mode-validation.md`.
- Prior validation report: `docs/reports/governance-copilot-ai-mode-validation-2026-04-30.md`.

## Behavior Change

- Scheduled workflow runs now require `mode=ai`.
- Manual workflow dispatch defaults to `require_ai_mode=true`.
- Manual dispatch can still set `require_ai_mode=false` for fallback diagnostics.
- If `GITGOV_API_KEY` is missing, strict runs fail instead of silently skipping.
- Non-strict diagnostic runs still skip safely when `GITGOV_API_KEY` is absent.

## Security Model

- The validator still uses the GitGov API key only as a Bearer token to the copilot route.
- It does not print the token, Authorization header, provider API keys, provider payloads, or raw AI answers.
- Evidence output stores counts, statuses, answer length, and answer SHA-256 only.

## Local Validation

Workflow YAML parse:

```powershell
@'
import sys
from pathlib import Path
import yaml
path = Path('.github/workflows/governance-copilot-ai-mode-validation.yml')
with path.open('r', encoding='utf-8') as f:
    yaml.safe_load(f)
print('workflow yaml parsed')
'@ | python -
```

Result: `workflow yaml parsed`.

Strict production validation:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\KAN-42-governance-copilot-ai-mode-enforcement.json
```

Sanitized result:

- status: `ai`.
- ok: `true`.
- HTTP status: `200`.
- response success: `true`.
- mode: `ai`.
- model: `google/gemini-2.5-flash`.
- citations: `4`.
- sources: `4`.
- ok sources: `4`.
- warnings: `0`.
- raw answer text was not stored; only answer length and SHA-256 hash were recorded.

## Residual Risk

- Gemini quota, provider outage, or Vercel env drift can now fail the weekly validator. That is intentional: the workflow is an operational regression signal.
- Manual `require_ai_mode=false` should be used only to diagnose the route when AI provider mode is already known to be unavailable.
