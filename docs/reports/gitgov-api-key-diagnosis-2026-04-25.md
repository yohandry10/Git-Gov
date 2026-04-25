# GitGov API Key Diagnosis - 2026-04-25

## Result

The local ignored `GITGOV_API_KEY` is valid for production GitGov admin authentication.

The previous manual Jira ingest `401` was not caused by a stale GitGov API key. The manual Jira ingest endpoint also requires the Jira shared-secret header when `JIRA_WEBHOOK_SECRET` is configured.

## Evidence

- Checked ignored local env files without printing values:
  - `gitgov/.env`
  - `gitgov/gitgov-server/.env`
  - `gitgov/src-tauri/.env`
- Confirmed `GITGOV_API_KEY` is present in ignored local env files.
- Confirmed production admin auth by calling `https://gitgov-api.onrender.com/stats` with Bearer auth; response status was HTTP `200`.
- Confirmed Render has `JIRA_WEBHOOK_SECRET` configured for `gitgov-api`.
- Confirmed Render does not need `GITGOV_API_KEY` as an env var for current DB-backed admin auth. The backend can bootstrap or ensure the env key on startup when configured, but existing API keys are validated from the `api_keys` database table.
- Confirmed manual Jira ingest to `https://gitgov-api.onrender.com/integrations/jira` succeeds when the request includes:
  - `Authorization: Bearer <GITGOV_API_KEY>`
  - `x-gitgov-jira-secret: <JIRA_WEBHOOK_SECRET>`
  - `org_name=yohandry10` in the JSON payload for global admin scope

## Operator Guidance

- Do not rotate `GITGOV_API_KEY` solely because of the earlier `401`; the key is valid.
- For manual Jira ingest, always load both `GITGOV_API_KEY` and `JIRA_WEBHOOK_SECRET` from ignored local env files.
- Do not print either value in logs or documentation.
- Native Jira webhooks still use the signed `/webhooks/jira?org_name=yohandry10` endpoint with `X-Hub-Signature`; do not put `GITGOV_API_KEY` in native Jira webhook URLs.
