# Domain SLO Validation Report

Generated (UTC): 2026-04-20 08:37:11
GitGov URL: http://127.0.0.1:3001
Targets file: ops/slo/domain-slo-targets.json
Window hours: 168

Overall status: **FAIL**

## Summary

| Domain | Tier | Status | Breaches |
|---|---|---|---|
| core-platform | critical | FAIL | 2 |
| standard-services | standard | FAIL | 2 |
| internal-tools | internal | FAIL | 1 |

## Totals

- Domains validated: 3
- Passed: 0
- Failed: 3

## Domain: core-platform

- Tier: critical
- Org filter: none
- Status: **FAIL**
- Baseline report: docs\reports\domain-slo-validation-local-2026-04-20\domain-core-platform-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | FAIL | 77.0 | >= 85 | Below target |
| blocked_push_rate | PASS | 0.0% | <= 5% | Within target |
| traceability_gap | FAIL | 100.0% | <= 15% | Exceeded target |
| pipeline_failure_rate_7d | PASS | 5.9% | <= 10% | Within target |
| sonar_failure_rate_sample | PASS | 0.0% | <= 12% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 30% | Within target |

## Domain: standard-services

- Tier: standard
- Org filter: none
- Status: **FAIL**
- Baseline report: docs\reports\domain-slo-validation-local-2026-04-20\domain-standard-services-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | FAIL | 72.0 | >= 75 | Below target |
| blocked_push_rate | PASS | 0.0% | <= 10% | Within target |
| traceability_gap | FAIL | 100.0% | <= 25% | Exceeded target |
| pipeline_failure_rate_7d | PASS | 5.9% | <= 20% | Within target |
| sonar_failure_rate_sample | PASS | 0.0% | <= 20% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 40% | Within target |

## Domain: internal-tools

- Tier: internal
- Org filter: none
- Status: **FAIL**
- Baseline report: docs\reports\domain-slo-validation-local-2026-04-20\domain-internal-tools-baseline.md

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
| release_readiness | PASS | 78.0 | >= 65 | Within target |
| blocked_push_rate | PASS | 0.0% | <= 15% | Within target |
| traceability_gap | FAIL | 100.0% | <= 35% | Exceeded target |
| pipeline_failure_rate_7d | PASS | 5.9% | <= 30% | Within target |
| sonar_failure_rate_sample | PASS | 0.0% | <= 30% | Within target |
| unresolved_violation_rate | PASS | 0.0% | <= 50% | Within target |
