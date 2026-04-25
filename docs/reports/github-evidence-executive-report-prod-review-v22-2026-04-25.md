# GitHub Evidence Executive Report

Generated: `2026-04-25T22:31:02.3951381Z`

Source: `https://gitgov-api.onrender.com/stats`

Organization: `yohandry10`

## Executive Summary

- Status: `Completo`
- Coverage: `4/4 signals`
- Missing signals: `none`

## Signal Coverage

| Signal | Count | Source event types |
|---|---:|---|
| PR lifecycle | 78 | `pull_request` |
| Reviews | 1 | `pull_request_review` |
| Comentarios PR | 97 | `pull_request_review_comment + issue_comment` |
| Checks/status | 2798 | `check_run + check_suite + status` |

## Top GitHub Event Types

| Event type | Count |
|---|---:|
| `check_run` | 2019 |
| `check_suite` | 625 |
| `status` | 154 |
| `push` | 114 |
| `issue_comment` | 97 |
| `pull_request` | 78 |
| `create` | 39 |
| `pull_request_review` | 1 |

## Interpretation

- `Completo` means GitGov has evidence across PR lifecycle, reviews, PR comments, and checks/status.
- `Parcial` means at least one evidence family is missing and operators should verify webhook event selection or recent repo activity.
- Counts come from `/stats.github_events.by_type`; this report does not expose provider secrets or raw webhook payloads.
