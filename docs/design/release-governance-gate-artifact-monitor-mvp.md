# KAN-49 Release Governance Gate Artifact Monitor MVP

Updated: 2026-05-01

## Summary

KAN-49 adds a freshness monitor for artifacts produced by the optional Release Governance Gate.

This is evidence hygiene, not a new default blocker:

- `record-only` customers still do not get release governance enforcement.
- The monitor is generated for enterprise packs only when release governance was explicitly enabled and the `artifact-monitoring` module is selected.
- The monitor checks for a recent `release-governance-gate-*` artifact and records a sanitized JSON summary.
- It does not read provider secrets, call GitGov APIs, mutate customer repositories, or approve releases.

## Components

- GitGov repo workflow: `.github/workflows/release-governance-gate-artifact-monitor.yml`.
- Shared artifact validator: `scripts/control-plane/validate_github_evidence_report_artifact.ps1`.
- Enterprise CLI template support: `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- Dashboard workflow pack support: `gitgov/src/components/control_plane/dashboard-helpers.ts`.
- Runbook: `docs/runbooks/release-governance-gate.md`.

## GitGov Workflow Behavior

The repository workflow is manual only:

```text
workflow_dispatch
```

It validates the latest successful `release-governance-gate.yml` run and requires an artifact whose name starts with:

```text
release-governance-gate-
```

Default accepted age:

```text
720 hours
```

The output artifact is:

```text
release-governance-gate-artifact-monitor
```

The JSON summary includes workflow run id, artifact name/id, age, expiry status, and PASS/FAIL status. It does not include secret values.

## Enterprise Template Behavior

Generated workflow template packs include:

```text
.github/workflows/release-governance-gate-artifact-monitor.yml
```

only when all are true:

- `formal-approval` module is enabled.
- `release_governance` has at least one non-`record-only` policy, including environment overrides.
- `artifact-monitoring` module is enabled.

That means a customer can choose:

- release approval evidence only: no gate, no monitor.
- release governance gate only: gate template, no monitor.
- release governance gate plus artifact hygiene: gate template and monitor template.

## Safety

- No secret values are written to templates, reports, or logs.
- The monitor needs GitHub `actions: read` permission only.
- Customer repositories are not changed automatically.
- The generated monitor is manual in the dashboard pack and manual in the GitGov repo workflow.
- Existing artifact-monitoring behavior for vulnerability and GitHub evidence templates is unchanged.

## Non-Goals

- No release approval enforcement change.
- No quorum default change.
- No database migration.
- No Render deploy.
- No provider mutation.
- No artifact retention override beyond GitHub Actions artifact retention settings.
