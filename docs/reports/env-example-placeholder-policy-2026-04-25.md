# Env Example Placeholder Policy - 2026-04-25

## Scope

Jira ticket: `KAN-9`

This closes the `.env.example` policy confusion by making the intended behavior executable:

- real `.env` files must not be tracked
- `.env.example` files may be tracked
- sensitive keys in `.env.example` must contain only blank or placeholder values

## Current State

Tracked env templates:

- `gitgov/.env.example`
- `gitgov/gitgov-server/.env.example`

`.gitignore` already blocks real env files and explicitly allows `.env.example`:

- `.env`
- `.env.local`
- `.env.*.local`
- `!.env.example`
- `!**/.env.example`

## Guardrail Added

Local guard:

- `scripts/security/publication_guard.ps1`
- New check: `.env.example` placeholder values

GitHub Actions guard:

- `.github/workflows/secret-scan.yml`
- New step: `Enforce sanitized .env.example placeholders`

The check inspects sensitive env key names such as:

- `TOKEN`
- `SECRET`
- `PASSWORD`
- `API_KEY`
- `PRIVATE_KEY`
- `DATABASE_URL`
- `SUPABASE`
- `JIRA`
- `JENKINS`
- `SONAR`
- `RENDER`
- `WEBHOOK`
- `JWT`

Allowed values are blank or explicit placeholders such as:

- `<placeholder>`
- `${PLACEHOLDER}`
- `your-*`
- `example`
- `change-me`
- `replace`
- `dummy`
- `fake`
- `localhost`
- `user:password@host`

## Validation

Local validation passed:

```powershell
.\scripts\security\publication_guard.ps1
git diff --check
```

The Git Bash equivalent of the GitHub Actions placeholder step also passed locally.

## Remaining Debt

None for `.env.example` publication policy. Future env templates should stay sanitized and should rely on placeholders, not copied local values.

