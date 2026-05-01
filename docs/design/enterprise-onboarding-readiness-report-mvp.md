# Enterprise Onboarding Readiness Report MVP

Updated: 2026-05-01

Ticket: `KAN-52`

## Purpose

GitGov already generates adoption profiles, workflow template packs, provider checks, workflow installation plans, remote installation PRs, and remote workflow readiness reports.

KAN-52 adds one customer-facing readiness snapshot that answers:

- is this enterprise onboarding profile ready?
- which setup areas are ready?
- what still needs action?
- is any release-blocking behavior enabled?
- can the evidence be handed to a customer without exposing secrets?

## Scope

The MVP adds:

- dashboard helper `buildEnterpriseOnboardingReadinessReport`.
- dashboard download action for onboarding readiness JSON.
- CLI generator `scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1`.
- Markdown and JSON output for adoption records.

The readiness report combines:

- adoption profile validation.
- provider health or provider connection report status.
- workflow template pack status.
- optional remote workflow readiness status from KAN-51.
- GitHub Actions variable/secret name readiness from KAN-51.
- release governance policy posture.

## Stages

The report uses six stages:

| Stage | Meaning |
|---|---|
| Adoption profile | Customer, repo, branch, Jira key, modules, and release governance are coherent. |
| Provider connections | Selected providers are configured and have either direct validation or observable evidence. |
| Workflow template pack | The customer has reviewed workflow templates to install. |
| Remote workflow readiness | Installed workflow files match the reviewed pack at the selected GitHub ref. |
| GitHub Actions configuration | Required variable and secret names exist; secret values are never read. |
| Release governance policy | Release approval/enforcement mode is explicit and record-only remains the safe default. |

Stage statuses:

- `ready`: no action needed for that stage.
- `needs-action`: customer/operator must still configure, run validation, or wait for evidence.
- `blocked`: the adoption profile is invalid or internally inconsistent.

The overall readiness score is a simple stage score:

- `ready` = 1.0
- `needs-action` = 0.5
- `blocked` = 0

## Dashboard UX

The Enterprise Adoption panel now shows an `Onboarding` readiness card with:

- overall status.
- readiness score.
- ready stage count.
- first next action.

The `Readiness` download button exports JSON only. It does not call provider APIs and does not read local `.env` files.

## CLI UX

The CLI generator can be run with only a profile:

```powershell
.\scripts\control-plane\generate_enterprise_onboarding_readiness_report.ps1 `
  -ProfilePath docs/examples/enterprise-adoption-profile.example.json `
  -OutputDir out/enterprise-onboarding-readiness `
  -ReportOnly
```

It can also consume existing reports:

```powershell
.\scripts\control-plane\generate_enterprise_onboarding_readiness_report.ps1 `
  -AdoptionPackPath out/enterprise-adoption-pack/enterprise-adoption-pack.json `
  -ProviderConnectionsPath out/provider-connections-report-only.json `
  -WorkflowReadinessPath out/workflow-readiness.json `
  -OutputDir out/enterprise-onboarding-readiness `
  -ReportOnly
```

Outputs:

```text
out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.md
out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.json
```

Without `-ReportOnly`, the script exits non-zero when the overall status is not `ready`.

## Safety Boundaries

The report:

- does not read secret values.
- does not print secret values.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider state.
- does not open PRs.
- does not dispatch workflows.
- does not change branch protection.
- does not make release blocking the default.

Secret and variable handling is name-only. Values must remain in customer secret stores or GitHub Actions secrets.

## Non-Goals

- Direct GitHub App installation.
- Automatic GitHub Actions variable/secret creation.
- Branch protection mutation.
- Automatic release blocking.
- Provider webhook creation.
- Replacing KAN-36 direct provider validation or KAN-51 remote workflow readiness.

## Acceptance Criteria

- Dashboard can export a secret-safe onboarding readiness JSON snapshot.
- CLI can generate Markdown and JSON readiness reports.
- CLI can run from profile-only input and from existing provider/workflow readiness reports.
- Missing remote readiness is visible as `needs-action`, not hidden.
- Record-only release governance remains documented as the safe default.
- Tests cover ready, needs-action, invalid-profile, filename, and secret-safety behavior.
