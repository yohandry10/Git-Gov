# GitGov Documentation Publication Policy

## Purpose

Define what can be published to a public/company-facing repository and what must remain internal or restricted.

## Classification

### 1) Public

Content allowed in public repo:

- Product architecture overview (high level)
- Setup and deployment guides without secrets
- API usage examples with placeholders
- User guides and troubleshooting without sensitive data
- Sanitized evidence summaries that do not expose secrets, private hostnames, account IDs, or personal data

### 2) Internal

Content allowed only in private/internal storage:

- Operational runbooks with environment topology details
- Incident analysis and postmortems
- Security hardening checklists tied to real infrastructure
- Backlog prioritization with commercial strategy
- Agent operating memory that names real local services, repository scope, or provider endpoints needed for day-to-day validation

### 3) Restricted

Never publish to public repo:

- Forensic audits (`AUDIT_*`, `INTEGRATIONS_AUDIT_*`, enterprise readiness assessments)
- Credentials, tokens, keys, webhook secrets
- Database dumps, Jenkins backups, local backup artifacts
- Internal hostnames, private URLs, account IDs, or PII
- Referencias a tooling interno/asistentes locales, prompts operativos o artefactos de automatización no orientados a producto

## Mandatory Rules

1. No `.env` files in version control.
2. `.env.example` must remain sanitized and placeholder-only.
3. Use placeholders for repo/org/domain examples (`<owner>/<repo>`, `<your-domain>`).
4. Any doc that includes real incidents, forensic findings, or strategy goes to internal storage.
5. Before publishing, run secret scan and review diff for sensitive context.
6. Public docs must describe product behavior and operator workflows, not internal assistant/tooling traces.
7. Branch names, PR titles, and commit messages must be neutral and product-oriented; internal assistant/vendor/tooling identifiers are forbidden.
8. Branch names, PR titles, and commit messages must include a Jira-style ticket ID such as `KAN-4` to preserve traceability coverage.

## Agent-Readable Public Context

When external agents need repo context, publish sanitized, current, product-facing summaries instead of force-adding restricted local memory.

Use `docs/AGENT_PUBLIC_CONTEXT.md` as the public bridge for:

- conclusions from ignored forensic or strategy documents that are still useful.
- current product phase and non-goals.
- a reading path for agents that cannot access ignored local files.
- corrections where old external analysis is now stale after later tickets.

Do not publish ignored forensic files directly. If a restricted note contains a still-valid conclusion, extract only the sanitized conclusion into a tracked public doc or ticket report.

## Identifier Handling

Use placeholders such as `<owner>/<repo>`, `<org>`, `<your-domain>`, and `<service-url>` in examples, templates, setup guides, and reusable runbooks.

Real repository or service identifiers may remain only when they are part of:

- Agent operating memory needed to validate the current environment.
- Historical evidence snapshots where replacing the value would make the result unverifiable.
- Security-safe status reports that intentionally document production validation scope.

These exceptions still must not include token values, private hostnames, personal data, account IDs, or credentials.

## Pre-Publish Checklist

1. Secret scan passes.
2. No restricted docs in staged changes.
3. No hardcoded secrets, tokens, or private URLs.
4. Public docs are explanatory only, not forensic.
5. Real repo/service identifiers appear only under the allowed identifier exceptions above.

## Automated Guardrails

- CI workflow: `.github/workflows/secret-scan.yml` (job `Security Guard`)
- The guard enforces restricted-doc exclusions and blocks legacy repository markers.
- The guard blocks tracked `.env` files (except `.env.example`).
- The guard validates tracked `.env.example` files and fails if sensitive keys contain non-placeholder values.
- The guard blocks non-neutral naming in branch/PR/commit metadata (internal tooling markers).
- The guard blocks missing Jira ticket IDs in branch/PR/commit metadata.
- `gitleaks` runs in the same workflow for secret detection on PR/push.
- Local pre-push check available:
  - `powershell -ExecutionPolicy Bypass -File scripts/security/publication_guard.ps1`
- Local commit-message hook available through `.githooks/commit-msg` when `core.hooksPath` is set to `.githooks`.

## Ownership

- Engineering owner: validates technical accuracy.
- Security owner: validates publication safety.
- Final approver: repository admin.
