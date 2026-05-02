# KAN-67 Enterprise Route Auth Smoke Trend Enforcement Artifact Monitor MVP

Updated: 2026-05-02

## Purpose

KAN-67 monitors whether KAN-66 is still producing reviewable Enterprise Route Auth Smoke Trend Enforcement evidence.

It answers one operational question:

```text
Did the latest successful Enterprise Route Auth Smoke Trend Enforcement workflow upload a fresh, non-expired evidence artifact?
```

This is artifact freshness monitoring. It does not parse route results itself, rerun auth smoke checks, or change release governance defaults.

## Workflow

File:

```text
.github/workflows/enterprise-route-auth-smoke-trend-enforcement-artifact-monitor.yml
```

Monitored workflow:

```text
enterprise-route-auth-smoke-trend-enforcement.yml
```

Expected artifact:

```text
enterprise-route-auth-smoke-trend-enforcement
```

Monitor artifact:

```text
enterprise-route-auth-smoke-trend-enforcement-artifact-monitor
```

## Triggers

- manual `workflow_dispatch`.
- weekly schedule on Friday at `15:37 UTC`.

The monitor runs after the Friday KAN-66 enforcement cadence, giving the source workflow time to produce a new artifact.

## Policy

Default maximum artifact age:

```text
192 hours
```

This gives the weekly enforcement cadence one extra day of tolerance before the monitor treats evidence as stale.

## Behavior

The monitor uses the existing shared validator:

```text
scripts/control-plane/validate_github_evidence_report_artifact.ps1
```

It checks:

- the latest successful KAN-66 workflow run exists.
- the latest run uploaded the expected artifact.
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
- rebuilding the auth smoke trend.
- enforcing release approvals.
- changing KAN-66 enforcement rules.
- creating provider credentials.
- installing workflows into customer repositories.
- replacing KAN-66 trend enforcement.

## Acceptance Criteria

- The monitor workflow can run manually.
- The monitor validates the latest KAN-66 artifact by exact name.
- The monitor uploads a JSON summary artifact.
- Local validation can call the shared validator against the current repository.
- Documentation explains that this is artifact freshness monitoring, not route probing, trend generation, or release blocking.
