# KAN-44 Configurable Release Governance Defaults

Updated: 2026-05-01

## Summary

KAN-44 documents the product rule that future multi-approver quorum and release-blocking enforcement must be configurable, customer-selected capabilities.

They are not GitGov defaults.

Default behavior remains evidence capture and approval recordkeeping. Blocking release gates or requiring multiple approvers should happen only after a customer explicitly chooses that policy.

## Traceability

- Jira issue: `KAN-44 - Document configurable release governance defaults`.
- Branch: `docs/KAN-44-configurable-release-governance`.
- PR: `#142 - docs(KAN-44): clarify release governance defaults`.
- Commit: `eb15b084124a85b16a384c651dde61dbef4f3e2f`.
- Design: `docs/design/configurable-release-governance-defaults.md`.

## Documentation Changes

- Added `docs/design/configurable-release-governance-defaults.md`.
- Updated formal release approval documentation to point future quorum/enforcement work to opt-in policy.
- Updated release approval dashboard documentation to explain that KAN-43 creates approval records but does not block releases.
- Updated the enterprise self-service roadmap to preserve non-blocking defaults unless a customer explicitly chooses enforcement.
- Updated agent/current context memory so future work does not accidentally treat quorum or enforcement as default behavior.

## Product Decision

Default mode:

```text
record-only
```

Meaning:

- GitGov can store release approvals.
- GitGov can display release approval status.
- GitGov can include release approval evidence in reports.
- GitGov does not fail customer pipelines by default.
- GitGov does not require multiple approvers by default.

Future optional modes:

- `advisory`: warn but do not block.
- `approval-required`: block only when customer config says approval is required.
- `quorum-required`: block only when customer config says specific approver roles or counts are required.

## Security And Adoption Notes

- This is a documentation/product-default change only.
- No secrets, env files, provider tokens, Authorization headers, or sensitive payloads are read or printed.
- No customer repository, workflow, database, Render service, Vercel env, or provider setting is changed.
- The decision reduces adoption risk by preventing silent release blocking.

## Local Validation

Completed locally:

- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

## GitHub Validation

PR `#142` checks passed before merge:

- `Security Guard`.
- `Server Clippy + Check`.
- `Desktop Rust Clippy`.
- `Frontend Lint + Typecheck`.
- `Website Lint + Typecheck + Build`.
- `Workflow Lint`.
- `Validate quality_gates warn/block matrix`.
- `Sonar Scan + Quality Gate`.
- `Block internal-assistant markers in branch/commits`.
- `Vercel`.
- `Vercel Preview Comments`.

Post-merge checks passed on `main` commit `eb15b084124a85b16a384c651dde61dbef4f3e2f`:

- `CI` run `25203116708`.
- `Release Readiness Gate` run `25203116684`.
- `Quality Gate Policy Matrix (Optional)` run `25203116644`.
- `Secret Scan` run `25203116635`.
- `SonarQube Governance (Non-Blocking)` run `25203116668`.
- `Public Naming Guard` run `25203116673`.
- `Governance Correlation Smoke (Optional)` run `25203116650`.
- `Desktop Updater Readiness (Optional)` run `25203116657`.

## Deployment

- No code, database, Render, Vercel, provider, or customer workflow behavior changed.
- KAN-44 is documentation/product-default memory only.

## Residual Work

- Implement customer-configurable release governance policy storage.
- Add dashboard controls for advisory/blocking/quorum modes.
- Generate workflow templates that respect the selected release governance mode.
- Add blocking release gate validation only when the customer policy explicitly enables it.
- Add quorum role/count evaluation only when the customer policy explicitly enables it.
