# KAN-40 Governance Copilot AI Mode Validation

Updated: 2026-04-30

## Summary

KAN-40 adds a reproducible, secret-safe validation layer for the Vercel AI SDK governance copilot.

KAN-38 created the route. KAN-39 added the dashboard UI. KAN-40 makes the remaining production state explicit: the route can be continuously checked for successful evidence loading and for whether it is running in `mode=ai` or deterministic `fallback`.

## Merge And Traceability

- Jira issue: `KAN-40 - Governance copilot AI mode validation`.
- Implementation branch: `product/KAN-40-governance-copilot-ai-validation`.
- Implementation PR: `#131 - product(KAN-40): validate governance copilot AI mode`.
- Merged commit: `2b507bc product(KAN-40): validate governance copilot AI mode`.

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

Implementation PR checks passed before merge:

- `Security Guard`.
- `Server Clippy + Check`.
- `Desktop Rust Clippy`.
- `Frontend Lint + Typecheck`.
- `Website Lint + Typecheck + Build`.
- `Workflow Lint`.
- `Validate quality_gates warn/block matrix`.
- `Sonar Scan + Quality Gate`.
- `Vercel`.
- `Vercel Preview Comments`.
- `Block internal-assistant markers`.

Post-merge `main` checks passed on commit `2b507bc`:

- `CI` - run `25196003313`.
- `Release Readiness Gate` - run `25196003326`.
- `Quality Gate Policy Matrix (Optional)` - run `25196003325`.
- `Secret Scan` - run `25196003309`.
- `Governance Correlation Smoke (Optional)` - run `25196003311`.
- `SonarQube Governance (Non-Blocking)` - run `25196003302`.
- `Public Naming Guard` - run `25196003318`.
- `Desktop Updater Readiness (Optional)` - run `25196003351`.

First manual workflow validation passed:

- Workflow: `Governance Copilot AI Mode Validation`.
- Run: `25196010712`.
- Trigger: `workflow_dispatch`.
- Branch: `main`.
- Artifact: `governance-copilot-ai-mode-validation`.
- Artifact ID: `6742816838`.
- Artifact expiry: `2026-07-30T00:21:30Z`.
- Artifact result: `status=fallback`, `ok=true`, HTTP `200`, `4` citations, `4` sources, `4` ok sources, `1` warning.

## Residual Work

- Enable and validate Vercel AI Gateway/OIDC when production AI generation is required.
- Re-run the validator with `-RequireAiMode`.
- Consider making strict AI mode a release gate only after the team decides that deterministic fallback is no longer acceptable.
