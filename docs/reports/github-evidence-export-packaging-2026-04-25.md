# GitHub Evidence Export Packaging

Date: 2026-04-25

## Scope

Admin Control Plane audit export packaging for GitHub executive evidence.

## Change

- `ExportPanel` now downloads a JSON package instead of only the raw `data` payload.
- The package includes:
  - export metadata (`export_id`, `export_type`, `record_count`, `source_content_hash`, `created_at`)
  - `packaged_at`
  - `executive_summary.github_evidence`
  - raw audit records under `data`
- `executive_summary.github_evidence` reuses the dashboard signal model:
  - PR lifecycle
  - reviews
  - PR comments
  - checks/status

## Files

- `gitgov/src/components/control_plane/ExportPanel.tsx`
- `gitgov/src/components/control_plane/ServerDashboard.tsx`
- `gitgov/src/components/control_plane/dashboard-helpers.ts`
- `gitgov/src/test/components/EventBreakdownGrid.test.tsx`

## Validation

Pending in this branch:

```powershell
npm test -- EventBreakdownGrid
npm run typecheck
npm run lint
git diff --check
.\scripts\security\publication_guard.ps1
```

## Notes

No provider secrets or token values are stored in this report. The executive summary is a dashboard snapshot at export time; canonical audit records remain in `data`.
