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

### 2) Internal

Content allowed only in private/internal storage:

- Operational runbooks with environment topology details
- Incident analysis and postmortems
- Security hardening checklists tied to real infrastructure
- Backlog prioritization with commercial strategy

### 3) Restricted

Never publish to public repo:

- Forensic audits (`AUDIT_*`, `INTEGRATIONS_AUDIT_*`, enterprise readiness assessments)
- Credentials, tokens, keys, webhook secrets
- Database dumps, Jenkins backups, local backup artifacts
- Internal hostnames, private URLs, account IDs, or PII

## Mandatory Rules

1. No `.env` files in version control.
2. `.env.example` must remain sanitized and placeholder-only.
3. Use placeholders for repo/org/domain examples (`<owner>/<repo>`, `<your-domain>`).
4. Any doc that includes real incidents, forensic findings, or strategy goes to internal storage.
5. Before publishing, run secret scan and review diff for sensitive context.

## Pre-Publish Checklist

1. Secret scan passes.
2. No restricted docs in staged changes.
3. No hardcoded secrets, tokens, or private URLs.
4. Public docs are explanatory only, not forensic.

## Automated Guardrails

- CI workflow: `.github/workflows/secret-scan.yml` (job `Security Guard`)
- The guard enforces restricted-doc exclusions and blocks legacy repository markers.
- The guard blocks tracked `.env` files (except `.env.example`).
- `gitleaks` runs in the same workflow for secret detection on PR/push.
- Local pre-push check available:
  - `powershell -ExecutionPolicy Bypass -File scripts/security/publication_guard.ps1`

## Ownership

- Engineering owner: validates technical accuracy.
- Security owner: validates publication safety.
- Final approver: repository admin.
