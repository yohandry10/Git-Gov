# KAN-63 Enterprise Route Auth Smoke Artifact Monitor MVP

Updated: 2026-05-02

## Purpose

KAN-63 monitors whether KAN-62 is still producing reviewable Enterprise Route Auth Smoke evidence.

It answers one operational question:

```text
Did the latest successful Enterprise Route Auth Smoke workflow upload a fresh, non-expired evidence artifact?
```

This is evidence freshness monitoring. It does not change release governance defaults and does not make customer onboarding or release approval blocking.

## Workflow

File:

```text
.github/workflows/enterprise-route-auth-smoke-artifact-monitor.yml
```

Monitored workflow:

```text
enterprise-route-auth-smoke.yml
```

Expected artifact prefix:

```text
enterprise-route-auth-smoke-
```

Monitor artifact:

```text
enterprise-route-auth-smoke-artifact-monitor
```

## Triggers

- manual `workflow_dispatch`.
- weekly schedule on Tuesday at `15:47 UTC`.

The monitor runs after the Monday KAN-62 smoke cadence, giving the source workflow time to produce a new artifact.

## Policy

Default maximum artifact age:

```text
192 hours
```

This gives the weekly smoke cadence one extra day of tolerance before the monitor treats evidence as stale.

## Behavior

The monitor uses the existing shared validator:

```text
scripts/control-plane/validate_github_evidence_report_artifact.ps1
```

It checks:

- the latest successful KAN-62 workflow run exists.
- the latest run uploaded an artifact with the expected prefix.
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
- does not call GitGov production routes directly; KAN-62 owns the route smoke.

## Non-Goals

- rerunning the route smoke itself.
- parsing route-level status codes inside the smoke artifact.
- enforcing release approvals.
- creating provider credentials.
- installing workflows into customer repositories.
- replacing KAN-62 smoke generation.

## Acceptance Criteria

- The monitor workflow can run manually.
- The monitor validates the latest KAN-62 artifact by prefix.
- The monitor uploads a JSON summary artifact.
- Local validation can call the shared validator against the current repository.
- Documentation explains that this is artifact freshness monitoring, not route probing or release blocking.
