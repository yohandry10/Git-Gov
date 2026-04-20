---
title: Risk Outcomes
description: Turn governance telemetry into measurable engineering risk outcomes for leadership and audits.
order: 9
category: Operate
---

GitGov already captures policy, CI, and traceability signals. The next step is to present these as **business outcomes** your engineering leadership can track over time.

This page defines the current KPI model used in the Control Plane and how to interpret each signal.

---

## Why This Matters

Enterprise buyers do not purchase logs alone. They buy:

- lower delivery risk,
- faster audit evidence,
- and predictable governance behavior.

Risk outcomes convert technical events into language that security, compliance, and leadership teams can act on.

---

## Core KPIs (Current)

### 1. Trusted Path Rate

How much delivery activity follows the expected governed route.

Formula:

```text
trusted_path_rate = tracked_pushes / (tracked_pushes + blocked_pushes)
```

Interpretation:
- Higher is better.
- A drop usually means policy friction, unsafe behavior attempts, or onboarding gaps.

---

### 2. Blocked Push Rate

How often policy blocked push attempts.

Formula:

```text
blocked_push_rate = blocked_pushes / (tracked_pushes + blocked_pushes)
```

Interpretation:
- Moderate values can be healthy during rollout.
- Sustained high values indicate policy mismatch or repeated unsafe patterns.

---

### 3. Traceability Gap

Share of commits without ticket correlation.

Formula:

```text
traceability_gap = 100 - ticket_coverage_percentage
```

Interpretation:
- Lower is better.
- High values reduce audit defensibility and release accountability.

---

### 4. Pipeline Failure Rate (7d)

Build instability in the recent 7-day window.

Formula:

```text
pipeline_failure_rate = failed_pipelines_7d / total_pipelines_7d
```

Interpretation:
- Correlates engineering delivery risk and operational rework.

---

### 5. Sonar Failure Rate (Sample)

Quality gate failures in Sonar-correlated pipeline runs.

Formula:

```text
sonar_failure_rate = sonar_failed_runs / sonar_total_runs
```

Interpretation:
- Helps quantify technical debt and quality regressions in new code.

---

### 6. Unresolved Violation Rate

Open governance violations as a share of total violations.

Formula:

```text
unresolved_violation_rate = unresolved_violations / total_violations
```

Interpretation:
- Measures backlog pressure in governance and response workflows.

---

## Composite Risk Score (0–100)

The dashboard computes a weighted score from currently available signals:

- blocked push rate,
- traceability gap,
- pipeline failure rate,
- sonar failure rate,
- unresolved violation rate.

When some signals are missing, GitGov shows **signal coverage** (`n/5`) so readers can evaluate confidence in the score.

---

## Operating Bands (Recommended)

- **Low Risk**: `< 35`
- **Medium Risk**: `35–59`
- **High Risk**: `>= 60`

These bands are operational defaults. Tune thresholds per repository criticality once you have stable telemetry.

---

## Tier Profiles (Now in Dashboard)

The Control Plane now supports tier-aware scoring profiles in the admin dashboard selector (`Critical`, `Standard`, `Internal`).

Each profile changes:
- score weights (readiness and composite risk),
- readiness color bands,
- and KPI SLA targets used for visual alerts.

### Default SLA targets by tier

| Tier | Min readiness | Max blocked push | Max traceability gap | Max pipeline failures | Max sonar failures | Max unresolved violations |
| --- | --- | --- | --- | --- | --- | --- |
| Critical | `>= 85` | `<= 5%` | `<= 15%` | `<= 10%` | `<= 12%` | `<= 30%` |
| Standard | `>= 75` | `<= 10%` | `<= 25%` | `<= 20%` | `<= 20%` | `<= 40%` |
| Internal | `>= 65` | `<= 15%` | `<= 35%` | `<= 30%` | `<= 30%` | `<= 50%` |

Use `Standard` as baseline, then move each repository to `Critical` or `Internal` once telemetry is stable for at least 2-4 weeks.

---

## Practical Rollout Pattern

1. Baseline each KPI per repo tier (critical backend, internal tooling, etc.).
2. Track trend weekly, not just snapshots.
3. Set one owner for each KPI (engineering manager or platform lead).
4. Use policy changes and quality gates to move one KPI at a time.
5. Export monthly evidence for governance and audits.

---

## Next Iteration

Roadmap metrics under active implementation:

- MTTR for noncompliance resolution,
- time-to-evidence for audits,
- release readiness trend by repository tier.

These will be added without changing the existing event contracts.
