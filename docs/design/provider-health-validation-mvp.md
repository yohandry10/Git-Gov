# Provider Health Validation MVP

Updated: 2026-04-30

Ticket: `KAN-32`

## Goal

Add the first customer-facing provider health view to Enterprise Self-Service Adoption.

After `KAN-31`, GitGov can persist a customer's adoption profile. `KAN-32` uses that profile plus existing dashboard evidence to show whether selected providers look ready, need configuration, or still need evidence.

## Scope

Implemented in the Enterprise Adoption dashboard panel:

- Provider health checks for selected adoption providers:
  - GitHub.
  - Jira.
  - Jenkins.
  - SonarQube.
  - Render.
  - Vercel.
- Status model:
  - `ready`: required adoption intent exists and GitGov has observable evidence.
  - `needs-evidence`: provider is selected but GitGov has not observed enough telemetry yet.
  - `needs-config`: selected provider is missing required profile/module/config intent.
- Evidence inputs:
  - GitHub event totals from `serverStats.github_events`.
  - Jira ticket coverage from `ticketCoverage`.
  - Jenkins pipeline health from `serverStats.pipeline`.
  - Sonar/quality evidence from loaded Jenkins correlations whose job name includes `sonar`.
  - Active repo count from `serverStats.active_repos`.
- UI summary:
  - ready provider count.
  - per-provider status badge.
  - evidence sentence.
  - next operational step.

## Safety

This MVP is secret-safe.

It does not:

- read `.env` files.
- read API keys.
- read provider tokens.
- display secret values.
- call GitHub, Jira, Jenkins, SonarQube, Render, or Vercel APIs directly.
- mutate customer provider settings.

The UI only uses already-loaded GitGov Control Plane evidence and the secret-name-only adoption pack model.

## Non-Goals

- No direct provider credential validation yet.
- No automatic webhook installation.
- No automatic GitHub Actions variable/secret creation.
- No workflow template installation.
- No formal release approval engine.
- No Vercel AI SDK Copilot.

## Next Product Steps

1. Add customer workflow template generation/installation for selected modules.
2. Add direct provider connection validation where safe and explicitly authorized.
3. Add formal release approval with approvers, expiration, risk acceptance, and evidence binding.
4. Build the Vercel AI SDK Copilot over adoption readiness, provider health, evidence packets, and security findings.
