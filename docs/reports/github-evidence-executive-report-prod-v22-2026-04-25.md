# GitHub Evidence Executive Report

Generated: `2026-04-25T22:17:56.0618521Z`

Source: `https://gitgov-api.onrender.com/stats`

Organization: `yohandry10`

## Executive Summary

- Status: `Parcial`
- Coverage: `3/4 signals`
- Missing signals: `Reviews`

## Signal Coverage

| Signal | Count | Source event types |
|---|---:|---|
| PR lifecycle | 75 | `pull_request` |
| Reviews | 0 | `pull_request_review` |
| Comentarios PR | 93 | `pull_request_review_comment + issue_comment` |
| Checks/status | 2684 | `check_run + check_suite + status` |

## Top GitHub Event Types

| Event type | Count |
|---|---:|
| `check_run` | 1937 |
| `check_suite` | 599 |
| `status` | 148 |
| `push` | 110 |
| `issue_comment` | 93 |
| `pull_request` | 75 |
| `create` | 37 |

## Interpretation

- `Completo` means GitGov has evidence across PR lifecycle, reviews, PR comments, and checks/status.
- `Parcial` means at least one evidence family is missing and operators should verify webhook event selection or recent repo activity.
- Counts come from `/stats.github_events.by_type`; this report does not expose provider secrets or raw webhook payloads.
