# KAN-124 Change Risk Review Queue And CAB Evidence Filter

## Decision

KAN-124 turns the KAN-123 per-evaluation manual review state into an operator/CAB review queue.

The slice is intentionally small: filter existing Change Risk evaluations by `review_status` so an
Admin or Auditor can find work that still needs human review.

## Why Now

KAN-121 creates deterministic advisory risk evidence. KAN-122 explains why a risk was assigned.
KAN-123 lets a human record review, mitigation, and risk acceptance. The next manual-first step is
not scoring or enforcement; it is making pending review work easy to find.

## Non-Goals

- No numeric risk or compliance score.
- No release blocking or default enforcement.
- No deployment execution.
- No provider or repository mutation.
- No AI/LLM, BYOM, MCP, chatbot behavior, or Agent Governance dependency.
- No compliance, certification, legal, or regulatory claim.
- No notifications, approval quorum, or multi-reviewer workflow.

## Implementation Shape

- Extend `ChangeRiskEvaluationQuery` with optional `review_status`.
- Validate `review_status` against the existing KAN-123 states:
  - `needs_review`
  - `reviewed`
  - `accepted_risk`
  - `needs_mitigation`
  - `rejected`
- Filter `GET /change-risk/evaluations` by `review_status`.
- Reuse the existing KAN-123 database columns and index; no new migration is required.
- Add the query param to Tauri and frontend store contracts.
- Add a Desktop `Review queue` status selector in `ChangeRiskPanel`.
- Keep Admin/Auditor read semantics and Admin-only review updates.

## Validation Requirements

- Real Postgres integration test must prove:
  - `needs_review` queue includes an unreviewed evaluation.
  - after Admin moves the same evaluation to `accepted_risk`, it leaves the `needs_review` queue.
  - the same evaluation appears in the `accepted_risk` queue.
  - invalid review queue status returns `400`.
  - Auditor can read queue filters.
  - another tenant cannot see a different tenant's review queue items.
- Store test must prove:
  - `review_status` is sent to Tauri list command.
  - updating an item out of the active queue removes it locally.
- No test may rely on AI, Agent Governance, scoring, release blocking, provider mutation, or
  repository mutation.
