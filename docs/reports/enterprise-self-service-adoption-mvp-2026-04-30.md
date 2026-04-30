# KAN-29 Enterprise Self-Service Adoption MVP

Updated: 2026-04-30

## Summary

KAN-29 starts the Enterprise Self-Service Adoption feature by adding a reproducible adoption pack generator.

This is the first product step after the KAN-24 to KAN-28 security work. It packages GitGov's existing evidence, readiness, traceability, and security automation into a reusable customer setup plan.

## Changes

- Added `scripts/control-plane/generate_enterprise_adoption_pack.ps1`.
- Added `docs/examples/enterprise-adoption-profile.example.json`.
- Added `docs/design/enterprise-self-service-adoption-mvp.md`.
- Added `docs/runbooks/enterprise-self-service-adoption.md`.
- Updated the KAN-28 roadmap to point to the KAN-29 MVP.
- Updated operating memory after validation.

## MVP Behavior

The generator accepts:

- customer name.
- repository.
- default branch.
- Jira project key.
- provider list.
- module list.
- policy preset: `audit-only`, `moderate`, or `strict`.

It outputs:

- Markdown adoption plan.
- JSON adoption plan.

The output includes:

- recommended workflow files.
- required GitHub Actions variables by name.
- required GitHub Actions secrets by name.
- policy rules.
- manual setup checklist.
- open product gaps.

No secret values are read or written.

## Validation

Local validation passed before PR:

- Command: `.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json`.
- Output Markdown: `enterprise-adoption-pack.md`.
- Output JSON: `enterprise-adoption-pack.json`.
- Profile: `ExampleCo`, `example-org/example-repo`, policy preset `moderate`.
- Generated workflow recommendations: `13`.
- Generated variable names: `3`.
- Generated secret names: `2`.
- Generated policy rules: `6`.
- Generated manual setup steps: `5`.
- Validation confirmed output includes secret names only, not secret values.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.

Pending before PR:

- GitHub PR checks.
