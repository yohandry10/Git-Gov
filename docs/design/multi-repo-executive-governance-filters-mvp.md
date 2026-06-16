# KAN-130 Multi-Repo Executive Governance Filters MVP

## Product Decision

KAN-130 extends the KAN-129 executive repository view with read-only filters.

The product goal is to let a CISO/CTO narrow the fleet view to the repositories that match a
specific governance signal without creating another workflow, changing deployment decisions, or
claiming certification/compliance status.

GPT/product-leader consultation was attempted after KAN-129 in the existing ChatGPT thread, but the
response rendered blank again after the prompt was accepted. The decision was made from roadmap
block `0.8 Multi-Repo Executive Governance View`, whose next gap is filters by environment and
violation/signal type.

## Scope

- Extend existing backend route `GET /executive/repositories`.
- Add filters:
  - `repository`
  - `environment`
  - `posture`
  - `gate_decision`
  - `risk_level`
  - `review_status`
- Keep the response shape and no-claim flags from KAN-129.
- Update Tauri query DTOs, Desktop store, and the Governance > Releases executive panel.
- Add focused design/report/context docs.

## Filter Semantics

- `repository`: case-insensitive repository name search.
- `environment`: filters Deployment Gate and Change Risk evidence by environment.
- `gate_decision`: requires at least one matching Deployment Gate decision.
- `risk_level`: requires at least one matching Change Risk evaluation.
- `review_status`: requires at least one matching Change Risk review state.
- `posture`: filters the computed executive triage posture after source evidence filters are
  applied.

When gate and risk filters are combined, a repository must satisfy both dimensions.

## Non-Goals

- No new table or migration.
- No enforcement.
- No release blocking changes.
- No deployment execution.
- No provider or repository mutation.
- No risk recalculation.
- No CAB packet or manifest creation.
- No AI, LLM, BYOM, MCP, or chatbot dependency.
- No Agent Governance dependency.
- No compliance score, certification, legal attestation, or official regulatory claim.

## Access

- Admin and Auditor can read the filtered view through Compliance Reviewer access.
- Developer and Agent keys are denied.
- Tenant isolation remains enforced before aggregation.
- Other-tenant repositories must never leak through filters.

## Validation Requirements

The real Postgres integration test must verify:

1. Baseline unfiltered KAN-129 behavior.
2. `posture=attention&environment=production` returns only the high-risk blocked repo.
3. `environment=staging&review_status=needs_review` returns only the pending manual-review repo.
4. `gate_decision=blocked` returns only repositories with a blocked gate.
5. `repository=...&risk_level=low` returns only the matching low-risk repo.
6. Conflicting gate/risk filters return an empty result set.
7. Invalid enum filters fail closed with HTTP `400`.
8. Admin/Auditor read, Developer denial, tenant isolation, and no source mutation.
