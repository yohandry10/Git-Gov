# KAN-13 Publication Governance - 2026-04-28

## Outcome

Documentation publication rules now distinguish reusable public examples from operational memory and historical validation evidence.

The earlier backlog item said to replace remaining hardcoded repository URLs with placeholders wherever found. That was too broad for this repo because some tracked reports and agent memory intentionally preserve real repository or service identifiers to keep validation evidence auditable.

## Rule

Use placeholders in:

- Examples
- Templates
- Setup snippets
- Reusable public guides

Real repository or service identifiers may remain in:

- Agent operating memory required to validate the current environment.
- Historical evidence snapshots where replacing the identifier would make the result unverifiable.
- Security-safe status reports that intentionally document production validation scope.

These exceptions still must not include token values, private hostnames, personal data, account IDs, or credentials.

## Validation

- `docs/ENTERPRISE_READINESS_DECISION.md`, `docs/AUDIT_*.md`, and `docs/INTEGRATIONS_AUDIT_*.md` are ignored by `.gitignore`.
- Restricted forensic/strategy docs are not tracked.
- `scripts/security/publication_guard.ps1` blocks restricted tracked docs, tracked `.env` files, non-placeholder sensitive `.env.example` values, non-neutral branch/commit names, and missing Jira ticket IDs.

## Follow-Up

For future docs, classify the file first:

- Public reusable docs should use placeholders.
- Operational memory can name the real current environment when that is necessary for automation.
- Historical reports can preserve real validation scope if no sensitive values are included.
