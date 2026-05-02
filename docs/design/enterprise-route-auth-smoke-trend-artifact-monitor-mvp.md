# KAN-65 Enterprise Route Auth Smoke Trend Artifact Monitor MVP

Updated: 2026-05-02

## Purpose

KAN-65 monitors whether KAN-64 is still producing reviewable Enterprise Route Auth Smoke Trend evidence.

It answers one operational question:

```text
Did the latest successful Enterprise Route Auth Smoke Trend workflow upload a fresh, non-expired evidence artifact?
```

This is artifact freshness monitoring. It does not parse route results itself and does not change release governance defaults.

## Workflow

File:

```text
.github/workflows/enterprise-route-auth-smoke-trend-artifact-monitor.yml
```

Monitored workflow:

```text
enterprise-route-auth-smoke-trend-report.yml
```

Expected artifact prefix:

```text
enterprise-route-auth-smoke-trend-report
```

Monitor artifact:

```text
enterprise-route-auth-smoke-trend-artifact-monitor
```

## Triggers

- manual `workflow_dispatch`.
- weekly schedule on Thursday at `15:47 UTC`.

The monitor runs after the Wednesday KAN-64 trend cadence, giving the source workflow time to produce a new artifact.

## Policy

Default maximum artifact age:

```text
192 hours
```

This gives the weekly trend cadence one extra day of tolerance before the monitor treats evidence as stale.

## Behavior

The monitor uses the existing shared validator:

```text
scripts/control-plane/validate_github_evidence_report_artifact.ps1
```

It checks:

- the latest successful KAN-64 workflow run exists.
- the latest run uploaded an artifact with the expected name/prefix.
- the artifact is not expired.
- the artifact age is less than or equal to the configured maximum.

## Safety Boundaries

The monitor:

- uses `actions: read` and `contents: read` only.
- uses the GitHub Actions run token only for artifact metadata lookup.
- does not read `.env` files.
- does not read or print provider secrets.
- does not read or print GitGov API keys.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not dispatch workflows.
- does not alter branch protection.
- does not change release governance defaults.
- does not call GitGov production routes directly.

## Non-Goals

- rerunning the auth smoke itself.
- parsing route-level status codes inside the trend artifact.
- enforcing release approvals.
- creating provider credentials.
- installing workflows into customer repositories.
- replacing KAN-64 trend generation.

## Acceptance Criteria

- The monitor workflow can run manually.
- The monitor validates the latest KAN-64 artifact by name/prefix.
- The monitor uploads a JSON summary artifact.
- Local validation can call the shared validator against the current repository.
- Documentation explains that this is artifact freshness monitoring, not trend generation or release blocking.
