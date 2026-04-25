# Operational MTTR and Time-to-Evidence

Date: 2026-04-25

## Summary

GitGov now exposes two informational operational metrics in the Control Plane risk outcomes widget:

- `Time-to-Evidence`: average time from a commit timestamp to ingestion of the correlated Jenkins pipeline evidence.
- `MTTR pipeline`: average time from a recoverable non-green Jenkins pipeline event to the next successful run for the same job.

These metrics are intentionally not part of composite risk or release readiness scoring yet. They are displayed as evidence-derived operational indicators until tier-specific SLO thresholds are calibrated.

## Implementation

- `gitgov/src/components/control_plane/dashboard-helpers.ts`
  - Added `buildOperationalEvidenceMetrics`.
  - Deduplicates pipeline evidence by `pipeline_event_id`, `pipeline_id`, or a fallback job/time/status key.
  - Ignores negative commit-to-ingestion deltas.
  - Treats `success`, `ok`, and `passed` as recovery statuses.
  - Treats `failure`, `failed`, `error`, `unstable`, `aborted`, `cancelled`, and `canceled` as recoverable non-green statuses.
- `gitgov/src/components/control_plane/ServerDashboard.tsx`
  - Computes metrics from existing `jenkinsCorrelations`.
  - Passes the values into `RiskOutcomesWidget`.
- `gitgov/src/components/control_plane/RiskOutcomesWidget.tsx`
  - Displays `Time-to-Evidence` and `MTTR pipeline`.
  - Renders `N/A` when no valid sample exists.
  - Documents in-widget that both metrics are informational until SLOs are defined.

## Validation

Commands run locally:

```powershell
npm test -- --run src/test/components/dashboard-helpers.test.ts
npm test -- --run
npm run typecheck
npm run lint
git diff --check
.\scripts\security\publication_guard.ps1
```

Results:

- `dashboard-helpers.test.ts`: 4 tests passed.
- Full frontend test suite: 25 files passed, 267 tests passed.
- TypeScript project build passed.
- ESLint passed.
- Whitespace diff check passed.
- Publication guard passed.

## Scope Limits

- Metrics are based on the current Jenkins correlation sample loaded into the dashboard.
- Metrics do not assert an enterprise SLO yet.
- Metrics do not affect release readiness scoring.
- Product/website claims should describe these as operational indicators, not contractual MTTR guarantees.
