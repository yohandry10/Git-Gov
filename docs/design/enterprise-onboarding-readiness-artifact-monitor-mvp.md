# Enterprise Onboarding Readiness Artifact Monitor MVP

Updated: 2026-05-01

Ticket: `KAN-54`

## Purpose

KAN-54 monitors whether KAN-53 is still producing reviewable Enterprise Onboarding Readiness evidence.

This is not a release gate and does not judge whether onboarding is fully ready. It only answers a simpler operational question:

```text
Did the latest successful Enterprise Onboarding Readiness workflow upload a fresh, non-expired evidence artifact?
```

## Workflow

File:

```text
.github/workflows/enterprise-onboarding-readiness-artifact-monitor.yml
```

Monitored workflow:

```text
enterprise-onboarding-readiness.yml
```

Expected artifact prefix:

```text
enterprise-onboarding-readiness-
```

Monitor artifact:

```text
enterprise-onboarding-readiness-artifact-monitor
```

## Triggers

- manual `workflow_dispatch`.
- weekly schedule on Thursday at `14:07 UTC`.

The monitor runs after the Wednesday KAN-53 readiness cadence, giving the source workflow time to produce a new artifact.

## Policy

Default maximum artifact age:

```text
192 hours
```

This gives the weekly readiness cadence one extra day of tolerance before the monitor treats evidence as stale.

## Behavior

The monitor uses the existing shared validator:

```text
scripts/control-plane/validate_github_evidence_report_artifact.ps1
```

It checks:

- the latest successful KAN-53 workflow run exists.
- the latest run uploaded an artifact with the expected prefix.
- the artifact is not expired.
- the artifact age is less than or equal to the configured maximum.

## Safety Boundaries

The monitor:

- uses `actions: read` and `contents: read` only.
- uses the GitHub Actions run token only for artifact metadata lookup.
- does not read `.env` files.
- does not read or print provider secrets.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not dispatch workflows.
- does not alter branch protection.
- does not make onboarding readiness or release governance blocking by default.

## Non-Goals

- parsing the onboarding readiness score.
- failing because the readiness report says `needs-action`.
- enforcing release approvals.
- creating provider credentials.
- installing workflows into customer repositories.
- replacing KAN-52 readiness generation or KAN-53 readiness automation.

## Acceptance Criteria

- The monitor workflow can run manually.
- The monitor validates the latest KAN-53 artifact by prefix.
- The monitor uploads a JSON summary artifact.
- Local validation can call the shared validator against the current repository.
- Documentation explains that this is artifact freshness monitoring, not release blocking.
