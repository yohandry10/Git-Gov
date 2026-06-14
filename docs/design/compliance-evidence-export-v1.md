# Compliance Evidence Export v1

Updated: 2026-06-14

Ticket: `KAN-99`

## Decision

After `KAN-98`, the product decision is to pause MCP/agent expansion and implement a manual-first compliance evidence export. GPT/product review and local roadmap analysis agreed that the next enterprise-safe slice should strengthen audit packaging for Deployment Gates, not add another agent capability.

`KAN-99` creates read-only JSON evidence packages from existing Deployment Gate authorizations. It does not authorize deployments, mutate provider configuration, map regulatory controls, generate PDF, invoke OPA/Rego, or create Agent Governance evaluation rows.

## Scope

The MVP supports one source:

```text
scope = deployment_gate
format = json
```

Admin users can:

- `POST /compliance/evidence-exports` to generate a completed export from a `dga_` Deployment Gate authorization.
- `GET /compliance/evidence-exports/{export_id}` to read export metadata without the artifact payload.
- `GET /compliance/evidence-exports/{export_id}/download` to download the redacted JSON artifact.

The artifact includes:

- Deployment Gate decision and target metadata.
- Policy checksum/source and `llm_decision=false`.
- Readiness status, issues, next steps, and missing evidence.
- Release approvals and break-glass status.
- Evidence counts/references for GitHub/Jira/Jenkins/Sonar sources already ingested by GitGov.
- Audit timestamps and redaction markers.
- `agent_governance_used=false` when the source is a Deployment Gate.
- `compliance_claim=false` and `framework_mapping=false`.

## Non-Scope

`KAN-99` deliberately does not include:

- Regulatory framework mapping such as PCI-DSS, ISO 27001, SBS Peru, or LGPD.
- PDF generation.
- Compliance certification claims.
- External auditor portal.
- Agent/MCP/chatbot surfaces.
- BYOM, LLM routing, or AI explanation generation.
- OPA/Rego policy execution.
- Provider mutation or CI/CD installation.
- New Deployment Gate decision logic.

## Persistence

Migration `supabase_schema_v43.sql` creates `compliance_evidence_exports`.

Stored fields are metadata plus the redacted JSON artifact:

- `export_id`
- `org_id`
- `created_by_user_id`
- `scope`
- `deployment_gate_id`
- `release_id`
- `status`
- `format`
- `artifact_hash`
- `policy_checksum`
- `gate_decision`
- `payload_json_redacted`
- timestamps and safe error text

The artifact hash is `sha256:<hex>` over the serialized JSON artifact returned by the download route.

## Safety Rules

- Admin-only API surface.
- Tenant-scoped lookup; cross-tenant Deployment Gate ids return not found.
- JSON only.
- Raw event payloads, request payloads, provider tokens, and secret-like fixture fields are not exported.
- Export generation writes an admin audit event.
- Export generation does not create `agent_governance_evaluations`.
- The sensitive admin route classifier includes `/compliance/evidence-exports`.

## Real Validation

The integration tests use real Postgres through the normal Axum test app.

The main fixture creates:

- a repository;
- a GitHub-like `client_events` row with secret-like metadata that must not export;
- a Jenkins-like `pipeline_events` row with secret-like payload that must not export;
- a Jira `project_tickets` row with secret-like raw payload that must not export;
- a release approval;
- a Deployment Gate authorization with missing Sonar evidence and secret-like request payload.

Assertions verify:

- create returns `201` and a `cee_` id;
- artifact hash is stable against the downloaded JSON;
- metadata read does not include the artifact;
- secret-like fixture values are absent from create/download responses;
- `agent_governance_used=false`;
- `llm_decision=false`;
- missing evidence is explicit;
- Agent Governance evaluation count does not change;
- developers cannot export;
- tenant A cannot export tenant B's gate;
- unsupported scope/format are rejected.
