# KAN-122 Change Risk Rule Catalog & Evaluation Trace MVP

KAN-122 extends the KAN-121 Change Risk Advisory with rule-level explainability. The product decision is to keep Change Risk deterministic, qualitative, manual-first, and advisory-only, while making every evaluation auditable by ruleset version and trace hash.

## Problem

KAN-121 persisted `risk_level`, reasons, missing evidence, blocking gaps, and recommended manual actions. That made the advisory useful, but an auditor still needed to infer which deterministic rules produced the result.

KAN-122 answers:

- Which ruleset version evaluated the change?
- Which rules triggered and which did not?
- Which evidence inputs were considered missing or present?
- What manual action should a release owner take?
- What trace hash binds the explanation to the stored evaluation?

## Implemented Scope

- Ruleset version: `change_risk_rules.v1`.
- Deterministic rule catalog endpoint: `GET /change-risk/rules`.
- Evaluation trace endpoint: `GET /change-risk/evaluations/{evaluation_id}/trace`.
- Persisted fields on `change_risk_evaluations`:
  - `ruleset_version`.
  - `triggered_rules`.
  - `non_triggered_rules`.
  - `evaluation_trace`.
  - `trace_hash`.
- Desktop/Tauri typed DTOs, client methods, commands, store state/actions.
- Governance > Releases `ChangeRiskPanel` section: `Why this risk?`.
- Read access moves to `Admin | Auditor`; create remains `Admin`.

## Rule Catalog

`change_risk_rules.v1` includes:

- `missing_release_approval`.
- `missing_ci_evidence`.
- `missing_code_review`.
- `missing_change_link`.
- `provider_unhealthy`.
- `policy_source_conflict`.
- `production_environment`.
- `break_glass_involved`.
- `stale_evidence`.
- `gate_requires_approval`.
- `gate_blocked`.
- `insufficient_evidence`.

Each rule exposes `rule_id`, `title`, `description`, `severity`, `evidence_inputs`, `manual_action_hint`, and `enabled=true`.

## Non-Goals

KAN-122 does not add AI/LLM, BYOM, MCP, chatbot behavior, Agent Governance dependency, automatic deploy execution, provider mutation, repository mutation, compliance score, certification, regulatory/legal claim, customer rule editor, or default blocking behavior.

## Verification Requirements

- New evaluations persist `ruleset_version=change_risk_rules.v1`.
- Trace hashes are `sha256:` hashes and deterministic for identical inputs.
- Medium/high evaluations show expected triggered rules.
- Missing CI/review/linkage evidence triggers explicit rules.
- Production triggers `production_environment`.
- Break-glass triggers `break_glass_involved`.
- `GET /change-risk/rules` returns no secrets and no claim flags.
- `GET /change-risk/evaluations/{evaluation_id}/trace` is tenant-scoped.
- Admin can create/read; Auditor can read; Developer and Agent Governance keys are denied.
- Agent Governance evaluation rows are not created by Change Risk.
- Deployment Gate rows are not mutated by Change Risk.
