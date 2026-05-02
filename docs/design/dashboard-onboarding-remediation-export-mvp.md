# Dashboard Onboarding Remediation Export MVP

Updated: 2026-05-02

Ticket: `KAN-58`

## Purpose

KAN-58 exposes the KAN-57 Enterprise Onboarding Remediation Plan from the Enterprise Adoption dashboard.

KAN-57 added the CLI generator. KAN-58 makes the same remediation concept available to self-service users who are already shaping the adoption profile in the dashboard.

## UI Surface

File:

```text
gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx
```

The Enterprise Adoption header now includes a `Plan` download action next to the existing adoption pack, workflow template pack, and readiness exports.

## Helper Surface

File:

```text
gitgov/src/components/control_plane/dashboard-helpers.ts
```

New helpers:

```text
buildEnterpriseOnboardingRemediationPlan
buildEnterpriseOnboardingRemediationPlanFilename
```

The helper builds the plan from:

- current dashboard readiness report.
- current dashboard adoption pack.

## Output

The dashboard export is JSON only:

```text
{customer}-{owner-repo}-onboarding-remediation-plan.json
```

The JSON includes:

- readiness status and score.
- remediation status.
- prioritized actions.
- suggested owner per action.
- validation evidence per action.
- placeholder-only GitHub Actions variable/secret commands.
- safety flags.

## Safety Boundaries

The dashboard remediation export:

- is built locally in the browser from already-loaded dashboard state.
- does not read `.env` files.
- does not call provider APIs.
- does not read provider tokens.
- does not read or print secret values.
- may list required secret names, but never secret values.
- uses `<value>` placeholders for variable commands.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not dispatch workflows.
- does not alter branch protection.
- does not make onboarding readiness or release governance blocking by default.

## Non-Goals

- creating variables or secrets from the dashboard.
- storing customer secret values.
- opening a remote PR.
- changing branch protection.
- replacing provider validation.
- replacing workflow readiness validation.
- making release governance blocking by default.

## Acceptance Criteria

- Dashboard can export remediation plan JSON.
- Helper tests cover actions, placeholder commands, filename, and safety flags.
- Export contains no secret values or secret assignments.
- Frontend tests, typecheck, lint, and build pass.
