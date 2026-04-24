# Public Infra Validation Report

Generated (UTC): 2026-04-20 08:06:18
Summary: **WARN**
Base URL: http://127.0.0.1:3001

## Checks

| Check | Status | Details |
|---|---|---|
| Scheme | PASS | Using scheme 'http'. |
| DNS Resolution | PASS | IPs: 127.0.0.1 |
| TLS Certificate | WARN | Skipped (non-HTTPS base URL). |
| GET /health | PASS | HTTP 200. |
| GET /stats (auth) | PASS | HTTP 200. |
| Route probe /webhooks/github | PASS | HTTP 200. Endpoint is reachable. |
| Route probe /integrations/jenkins | PASS | HTTP 200. Endpoint is reachable. |
| Route probe /integrations/jira | PASS | HTTP 200. Endpoint is reachable. |

## Warnings

- TLS check skipped due to non-HTTPS base URL

## Failures

- none
