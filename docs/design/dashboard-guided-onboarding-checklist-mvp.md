# KAN-59 Dashboard Guided Onboarding Checklist MVP

Updated: 2026-05-02

## Summary

KAN-59 turns the Enterprise Adoption dashboard remediation data into a visible guided checklist.

KAN-58 already lets an operator download a remediation plan JSON. KAN-59 keeps that export, then shows the same onboarding state directly in the dashboard so a customer can see:

- what is already complete.
- what should be handled next.
- who usually owns that step.
- how to validate the step after it is done.
- how many GitHub Actions variable and secret names are still part of the setup pack.

## Scope

- Add a typed dashboard helper: `buildEnterpriseOnboardingGuide`.
- Render a compact `Guided checklist` section in `EnterpriseAdoptionPanel`.
- Preserve existing `Readiness` and `Plan` JSON downloads.
- Add focused unit coverage for checklist status, next-step selection, configuration summary, and safety flags.

## Data Model

The guide is derived from:

- dashboard onboarding readiness report.
- dashboard remediation plan.

It does not introduce a persisted backend record.

Checklist step statuses:

- `complete`: readiness stage is ready.
- `next`: first actionable non-ready remediation step.
- `todo`: later non-ready remediation step.
- `blocked`: invalid or internally inconsistent stage.

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
- No secret values are printed or embedded.
- Secret names may be displayed, but values are never read or generated.
- Placeholder commands remain placeholders only.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No workflow dispatch occurs.
- No branch protection is changed.
- Release blocking remains opt-in only.

## Non-Goals

- No remote customer repository mutation.
- No automatic GitHub Actions variable/secret creation.
- No provider setup wizard that calls provider APIs.
- No new backend route or database migration.
- No change to release governance defaults.
