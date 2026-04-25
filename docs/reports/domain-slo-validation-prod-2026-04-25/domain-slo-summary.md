# Domain SLO Validation Report

Generated (UTC): 2026-04-25 09:07:22
GitGov URL: https://gitgov-api.onrender.com
Targets file: ops\slo\domain-slo-targets.json
Window hours: 720

Overall status: **PASS**

## Summary

| Domain | Tier | Status | Breaches |
|---|---|---|---|
| core-platform | critical | PASS | 0 |
| standard-services | standard | PASS | 0 |
| internal-tools | internal | PASS | 0 |

## Totals

- Domains validated: 3
- Passed: 3
- Failed: 0

## Domain: core-platform

- Tier: critical
- Org filter: yohandry10
- Repo: yohandry10/Git-Gov
- Branch: main
- Status: **PASS**
- Baseline report: docs\reports\domain-slo-validation-prod-2026-04-25\domain-core-platform-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | PASS | 96.0 | >= 85 | Within target |
| blocked_push_rate | PASS | 0.0% | <= 5% | Within target |
| traceability_gap | PASS | 11.8% | <= 15% | Within target |
| pipeline_failure_rate_7d | PASS | 2.5% | <= 10% | Within target |
| sonar_failure_rate_sample | PASS | 2.4% | <= 12% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 30% | Within target |

## Domain: standard-services

- Tier: standard
- Org filter: yohandry10
- Repo: yohandry10/Git-Gov
- Branch: main
- Status: **PASS**
- Baseline report: docs\reports\domain-slo-validation-prod-2026-04-25\domain-standard-services-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | PASS | 95.0 | >= 75 | Within target |
| blocked_push_rate | PASS | 0.0% | <= 10% | Within target |
| traceability_gap | PASS | 11.8% | <= 25% | Within target |
| pipeline_failure_rate_7d | PASS | 2.5% | <= 20% | Within target |
| sonar_failure_rate_sample | PASS | 2.4% | <= 20% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 40% | Within target |

## Domain: internal-tools

- Tier: internal
- Org filter: yohandry10
- Repo: yohandry10/Git-Gov
- Branch: main
- Status: **PASS**
- Baseline report: docs\reports\domain-slo-validation-prod-2026-04-25\domain-internal-tools-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | PASS | 96.0 | >= 65 | Within target |
| blocked_push_rate | PASS | 0.0% | <= 15% | Within target |
| traceability_gap | PASS | 11.8% | <= 35% | Within target |
| pipeline_failure_rate_7d | PASS | 2.5% | <= 30% | Within target |
| sonar_failure_rate_sample | PASS | 2.4% | <= 30% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 50% | Within target |
