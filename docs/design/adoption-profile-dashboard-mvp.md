# Adoption Profile Dashboard MVP

Updated: 2026-04-30

Ticket: `KAN-30`

## Goal

Turn the `KAN-29` enterprise adoption pack into a visible dashboard workflow.

The MVP lets an admin shape a customer adoption profile from the GitGov UI, preview the generated governance plan, and download a secret-safe JSON adoption pack.

## Scope

Implemented in the admin Control Plane dashboard:

- customer name, repository, default branch, and Jira project key inputs.
- policy preset selector: `audit-only`, `moderate`, or `strict`.
- provider toggles: GitHub, Jira, Jenkins, SonarQube, Render, and Vercel.
- module toggles: traceability, GitHub evidence, release readiness, quality gates, evidence packets, vulnerability review, artifact monitoring, trend enforcement, and formal approval.
- live workflow plan, policy rules, required variable names, and required secret names.
- profile validation before JSON download.
- JSON export using the same profile/pack shape as the `KAN-29` generator.

## Validation Rules

The UI blocks download when:

- customer name is empty.
- repository does not look like `owner/repo`.
- default branch is empty.
- traceability is enabled but Jira project key is empty.
- Jira project key is not uppercase letters/numbers.
- no provider is selected.
- no module is selected.

## Safety

The dashboard pack includes secret names only.

It does not:

- read local `.env` files.
- read provider tokens.
- store secret values.
- call external provider APIs.
- mutate GitHub, Jira, Jenkins, SonarQube, Render, or Vercel settings.

## Non-Goals

- No backend persistence yet.
- No automatic workflow installation yet.
- No provider health validation yet.
- No full formal release approval engine yet.
- No Vercel AI SDK Copilot yet.

## Follow-Ups

1. Persist adoption profiles per tenant/org.
2. Add provider health validation endpoints.
3. Generate or install workflow templates for selected repositories.
4. Add formal release approval with approvers, expiration, risk acceptance, and evidence binding.
5. Build the Vercel AI SDK Copilot over adoption profiles and evidence APIs.
