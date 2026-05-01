# KAN-40 Governance Copilot AI Mode Validation

Updated: 2026-04-30

## Summary

KAN-40 adds a reproducible, secret-safe validation layer for the Vercel AI SDK governance copilot.

KAN-38 created the route. KAN-39 added the dashboard UI. KAN-40 makes the remaining production state explicit: the route can be continuously checked for successful evidence loading and for whether it is running in `mode=ai` or deterministic `fallback`.

## Scope

- Script: `scripts/control-plane/validate_governance_copilot_ai_mode.ps1`.
- Workflow: `.github/workflows/governance-copilot-ai-mode-validation.yml`.
- Runbook: `docs/runbooks/governance-copilot-ai-mode-validation.md`.

## Validation Model

The validator checks:

- route returns HTTP `2xx`;
- response has `success=true`;
- at least three evidence sources are present by default;
- at least two sources are `ok` by default;
- at least three citations are present by default;
- warnings are captured without printing secrets;
- raw answer text is not stored, only length and SHA-256 hash;
- optional strict mode fails if `mode` is not `ai`.

## Current Expected Production Interpretation

Current production is expected to pass in non-strict mode and report `status=fallback` until Vercel AI Gateway/OIDC is active for the route.

That means:

- the product route is usable;
- evidence grounding is validated;
- AI-generated narrative mode is still an activation/configuration step, not a route implementation blocker.

## Local Validation

Non-strict production validation:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -OutputPath out\KAN-40-governance-copilot-ai-mode-validation.json
```

Result:

- status: `fallback`.
- ok: `true`.
- HTTP status: `200`.
- response success: `true`.
- mode: `fallback`.
- citations: `4`.
- sources: `4`.
- ok sources: `4`.
- warnings: `1`.
- raw answer text was not stored; the report stores only length and SHA-256 hash.

Strict-mode validation:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 `
  -TicketId KAN-39 `
  -ReleaseId KAN-39 `
  -RequireAiMode `
  -OutputPath out\KAN-40-governance-copilot-ai-mode-validation-strict.json
```

Result:

- expected controlled failure confirmed.
- route still returned HTTP `200` with valid evidence.
- failure reason: `copilot did not return mode=ai while RequireAiMode was set`.

Interpretation:

- The copilot route is healthy and evidence-grounded.
- Production AI generation mode is not active yet.
- The new strict gate is ready to use after Vercel AI Gateway/OIDC activation.

Repository guard validation:

- new workflow YAML parsed successfully with local Python/PyYAML.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.

## GitHub Validation

Pending.

## Residual Work

- Enable and validate Vercel AI Gateway/OIDC when production AI generation is required.
- Re-run the validator with `-RequireAiMode`.
- Consider making strict AI mode a release gate only after the team decides that deterministic fallback is no longer acceptable.
