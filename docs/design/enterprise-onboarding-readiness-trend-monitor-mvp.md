# Enterprise Onboarding Readiness Trend Monitor MVP

Updated: 2026-05-01

Ticket: `KAN-56`

## Purpose

KAN-56 adds a monitor for the KAN-55 Enterprise Onboarding Readiness trend artifact.

KAN-55 tells whether onboarding readiness is stable, improving, or declining. KAN-56 checks whether that trend evidence is fresh and whether the latest trend shows deterioration that an operator should review.

This is not a customer release blocker by default.

## Workflow

File:

```text
.github/workflows/enterprise-onboarding-readiness-trend-monitor.yml
```

Script:

```text
scripts/control-plane/validate_enterprise_onboarding_readiness_trend_monitor.ps1
```

Source workflow:

```text
enterprise-onboarding-readiness-trend-report.yml
```

Source artifact:

```text
enterprise-onboarding-readiness-trend-report
```

Monitor artifact:

```text
enterprise-onboarding-readiness-trend-monitor
```

## Triggers

- manual `workflow_dispatch`.
- weekly schedule on Thursday at `14:27 UTC`.

This schedule runs after the readiness automation, artifact monitor, and trend report sequence.

## Inputs

| Input | Default | Purpose |
| --- | ---: | --- |
| `max_age_hours` | `192` | Maximum accepted age for the latest trend artifact. |
| `min_latest_score` | `75` | Minimum accepted latest readiness score before reporting deterioration. |
| `report_only` | `true` | Keeps the workflow non-blocking even when findings are reported. |

## Monitor Rules

The monitor reports:

- `ready` when the trend artifact is fresh, parseable, and no deterioration rule fired.
- `needs-action` when the trend is parseable but shows customer onboarding deterioration.
- `blocked` when the trend evidence cannot be trusted because it is missing, stale, expired, or not parseable.

Rules checked:

- latest successful trend workflow run exists.
- expected trend artifact exists.
- artifact is not expired.
- artifact age is within `max_age_hours`.
- trend JSON exists and can be parsed.
- trend parsed the latest successful onboarding readiness artifact.
- latest score is at least `min_latest_score`.
- readiness score did not decline versus the oldest analyzed report.
- blocked stage count did not increase.
- latest blocked stage count is zero.

## Output

The workflow uploads:

```text
enterprise-onboarding-readiness-trend-monitor.md
enterprise-onboarding-readiness-trend-monitor.json
```

The JSON includes:

- monitor status.
- report-only mode.
- source workflow run and artifact metadata.
- freshness thresholds.
- latest trend snapshot.
- findings list.
- `release_blocking_default=false`.

## Safety Boundaries

The monitor:

- reads GitHub Actions run and artifact metadata.
- downloads only the sanitized trend artifact emitted by GitGov workflows.
- does not read `.env` files.
- does not read provider tokens.
- does not print Authorization headers.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not dispatch workflows.
- does not alter branch protection.
- does not make onboarding readiness or release governance blocking by default.

## Non-Goals

- replacing KAN-52 current readiness reporting.
- replacing KAN-53 readiness automation.
- replacing KAN-54 artifact freshness monitoring.
- replacing KAN-55 trend reporting.
- failing releases because onboarding is `needs-action`.
- enforcing release approvals or quorum.
- connecting providers directly.
- creating GitHub Actions variables or secrets.

## Acceptance Criteria

- Local script can validate the latest KAN-55 trend artifact and produce Markdown/JSON.
- Workflow can run manually and upload monitor evidence.
- Scheduled workflow is enabled for recurring monitor evidence.
- Monitor output remains secret-safe and non-mutating.
- Documentation clearly states that report-only is the default and blocking is opt-in only.
