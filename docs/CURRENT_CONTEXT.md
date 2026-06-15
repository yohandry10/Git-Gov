# GitGov Current Context Handoff

Updated: 2026-06-15
Ticket: `KAN-113` Period Compliance Report Generator

Read this file first when resuming work. It is the compact operational handoff for the current GitGov state.

## Exact Current Point

- Local workspace: `C:\Users\PC\Desktop\GitGov`.
- Current planning source: GitHub Issues. The former Jira Cloud project is deactivated and should not block ongoing work.
- Current implementation branch: `product/KAN-113-period-compliance-report-generator`.
- Current implementation ticket: GitHub issue `#393`, `KAN-113: Period Compliance Report Generator`.
- KAN-113 product decision from GPT consultation and local roadmap/repo analysis: after KAN-112, implement a manual/on-demand Period Compliance Report Generator before scheduler, DOCX/PDF formal templates, official regulatory wording, compliance scoring, certification claims, AI summaries, Integration Wizard, Change Risk Score, Multi-Repo Executive View, BYOM, MCP, chatbot behavior, or broader Agent Governance. The first slice generates append-only JSON reports for `org + date_range + optional framework_id` from already reviewed Framework Review Reports.
- KAN-113 implementation status on 2026-06-15: in progress on branch `product/KAN-113-period-compliance-report-generator`. Implemented so far: Supabase migration/postcheck `v55` for `compliance_period_reports`; backend routes `POST/GET /compliance/period-reports`, `GET /compliance/period-reports/{period_report_id}`, and `GET /compliance/period-reports/{period_report_id}/download`; artifact schema `gitgov_period_compliance_report.v1`; Admin-only creation; Admin/Auditor read/download with Auditor access requiring visibility to every source Framework Review Report; no-claim constraints; Tauri models/client/commands; Desktop Evidence Review Period Compliance Report panel; store state/actions/tests; roadmap/architecture/report docs. Local validation passed: backend `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, focused real Postgres test `period_compliance_report_aggregates_reviewed_reports_without_claims`, full backend Postgres suite (`310` passed), local `v55` migration/postcheck through ignored `DATABASE_URL`, Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, Tauri tests (`49` passed), frontend `pnpm --dir gitgov typecheck`, `pnpm --dir gitgov lint`, focused store test (`35` passed), full Vitest run (`367` passed), frontend build, `git diff --check`, and publication guard. Still pending before completion: commit/PR/checks/merge/deploy/prod smoke.
- KAN-112 product decision from GPT consultation attempt and local roadmap/repo analysis: after KAN-111, implement read-only customer framework pack diff before official regulatory mapping, compliance scoring, DOCX export, BYOM, MCP, chatbot behavior, or broader Agent Governance. The diff compares two customer-provided versions of the same original framework id, remains Admin-only and tenant-scoped, preserves no-claim flags, does not persist a new artifact in this MVP, and does not create Agent Governance evaluations.
- KAN-112 implementation status on 2026-06-15: completed by PR `#391`, merged to `main` as `5499f78`. It adds backend `GET /compliance/framework-packs/diff`, raw redacted pack diff source loading, deterministic added/removed/changed/unchanged control comparison, same-original-framework enforcement, no-claim invariant checks, Tauri models/client/command, Desktop Governance Evidence Review diff UI, focused store test coverage, roadmap/architecture/design/report docs. Local validation passed: backend `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, focused real Postgres test `customer_framework_pack_diff_compares_real_versions_without_claims`, full backend Postgres suite (`309` passed), Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, Tauri tests (`49` passed), frontend `pnpm --dir gitgov typecheck`, focused store test (`35` passed), full Vitest run (`367` passed), frontend lint/build, `git diff --check`, and publication guard. PR checks passed. Post-merge `main` checks passed for `5499f78`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8noflu8bjmc73f2u2rg` for `5499f78` reached `live`. Production smoke passed: `/health=ok`, imported and reviewed two temporary customer-provided packs for original framework `bank_release_controls_kan112_20260615001152`, diffed `cfp_8181d41d2bb39ed54af8050056fbb7eb` to `cfp_fdec1d243cad05936aee96a678ca35e1`, summary was `added=1`, `removed=1`, `changed=1`, `unchanged=2`, and no-claim flags stayed `compliance_claim=false`, `regulatory_claim=false`, `gitgov_certifies=false`, `official_regulatory_mapping=false`, `requires_auditor_review=true`.
- KAN-111 product decision from GPT consultation and local roadmap/repo analysis: after KAN-110, implement PDF export for reviewed Framework Review Reports before DOCX, formal regulatory templates, official regulatory mappings, BYOM, MCP, chatbot behavior, or broader Agent Governance. The PDF must be manual-first, bound to the existing source report hash and provenance manifest hash, readable by Admins and assigned Auditors, and explicitly not a certification, compliance score, legal attestation, or official regulatory claim.
- KAN-111 implementation status on 2026-06-15: completed by PR `#388`, merged to `main` as `97b1b94`. It adds Supabase migration/postcheck `v54` for `compliance_framework_review_report_pdf_exports`; backend routes `POST/GET /compliance/framework-review-reports/{report_id}/pdf-export` and `GET /compliance/framework-review-reports/{report_id}/pdf-export/download`; server-side PDF rendering with `application/pdf` bytes and `x-gitgov-artifact-hash`; Tauri models/client/commands; Desktop Governance Evidence Review PDF panel; roadmap/architecture/design/report docs. Local validation passed: backend `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, focused real Postgres Framework Review Report integration test covering full evidence chain, blocked pre-reviewed PDF, assigned Auditor success, unassigned/Developer/other-tenant denial, real PDF bytes/hash/header/content, unchanged source report hash, no-claim flags, and no Agent Governance evaluations, and full backend Postgres suite (`308` passed); Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and tests (`49` passed); frontend `pnpm --dir gitgov typecheck`, focused store test (`34` passed), full tests (`366` passed), lint, and build; local `v54` migration/postcheck through ignored `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed for `97b1b94`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8nnnvgjo6nc73e4u030` for `97b1b94` reached `live`. Production `v54` migration/postcheck passed. Production smoke passed: `/health=ok`, report `frr_ac4ee214bc051caee783485d5755d34a` was `reviewed`, created PDF export `frrpdf_f2bd8e2a866a9194889e05f59d0829b5` from manifest `frrm_a3135e737c43d92fbc3f8b56d19d0a0c`, downloaded PDF was `3930` bytes and `%PDF-1.4`, `Content-Type=application/pdf`, `x-gitgov-artifact-hash` and downloaded bytes SHA-256 matched `sha256:f2bd8e2a866a9194889e05f59d0829b595cc1761af0d12ed8b1c1d85ffaa7e87`, PDF text contained no-claim language/source hash/manifest hash/no-claim flags, and source report `artifact_hash` stayed unchanged.
- Latest completed implementation ticket: GitHub issue `#387`, `KAN-111: Framework Review Report PDF export`.
- Previous completed implementation ticket: GitHub issue `#383`, `KAN-110: Reviewed report provenance manifest`.
- KAN-110 product decision from GPT consultation attempt and local roadmap/repo analysis: after KAN-109, ChatGPT received the current state but returned an empty assistant message despite the UI showing reasoning completed. The decision was made from the repo state and roadmap: implement reviewed Framework Review Report provenance manifests before PDF/DOCX export, official regulatory mapping seeds, Integration Wizard, Change Risk Score, Multi-Repo Executive View, or compliance report generation. This strengthens auditor/customer sharing with hash-chain evidence while preserving manual-first behavior and no official regulatory/compliance/certification claims.
- KAN-110 implementation status on 2026-06-15: completed by PR `#384`, merged to `main` as `a7ab2e5`. It adds Supabase migration/postcheck `v53` for append-only `compliance_framework_review_report_manifests`; backend routes `POST /compliance/framework-review-reports/{report_id}/provenance-manifests` and `GET /compliance/framework-review-reports/{report_id}/provenance-manifests/{manifest_id}`; manifest schema `gitgov_framework_review_report_provenance_manifest.v1`; deterministic `sha256-provenance-manifest-v1` hash signature, `manifest_hash`, and `previous_manifest_hash`; assignment-aware authorization; `409 report_not_reviewed` until manual review status is `reviewed`; Tauri models/client/command; Desktop Governance Evidence Review manifest generation/download subcomponent; docs under `docs/design/framework-review-report-provenance-manifest-mvp.md` and `docs/reports/framework-review-report-provenance-manifest-2026-06-15.md`. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; real Postgres focused Framework Review Report integration test covering full KAN-99/KAN-105/KAN-109 chain, blocked pre-reviewed manifest, assigned Auditor success, unassigned Auditor/Developer denial, first/second hash-chain manifests, manifest download, no artifact mutation, no-claim flags, and no Agent Governance evaluation creation; full backend Postgres suite (`308` passed); Tauri `cargo check`; Tauri `cargo fmt --check`; Tauri `cargo clippy -- -D warnings`; Tauri tests (`49` passed); frontend `pnpm --dir gitgov typecheck`; focused store test (`34` passed); full frontend tests (`366` passed); frontend lint/build; local `v53` migration/postcheck through ignored `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed for `a7ab2e5`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8nmoluq1p3s738cqdl0` for `a7ab2e5` reached `live`. Production `v53` migration/postcheck passed. Production smoke passed: `/health=200`, report `frr_ac4ee214bc051caee783485d5755d34a` was `reviewed`, two append-only manifests were created (`frrm_ed5a356ad89f406b90e236756655183c` then `frrm_a3135e737c43d92fbc3f8b56d19d0a0c`), the second manifest `previous_manifest_hash` matched the first manifest hash, downloaded manifest hash-chain matched storage, schema was `gitgov_framework_review_report_provenance_manifest.v1`, signature algorithm was `sha256-provenance-manifest-v1`, report `artifact_hash` stayed unchanged, no-claim flags stayed valid, `agent_governance_required=false`, and `source_report_artifact_mutated=false`.
- KAN-109 product decision from GPT consultation and local roadmap/repo analysis: after KAN-108, do not jump to signed provenance manifests, PDF/DOCX export, official regulatory mappings, BYOM, MCP, chatbot behavior, or broader Agent Governance. First close the collaboration gap over existing Framework Review Reports: Admins assign reports to active tenant Auditors; assigned Auditors can list assigned reports, comment, and update manual review metadata; unassigned same-tenant Auditors are blocked once active assignments exist.
- KAN-109 implementation status on 2026-06-15: completed by PR `#381`, merged to `main` as `cee4594`. It adds Supabase migration/postcheck `v52` for `compliance_framework_review_report_assignments` and `compliance_framework_review_report_comments`; backend routes `GET /compliance/framework-review-reports/assigned-to-me`, `GET/PUT /compliance/framework-review-reports/{report_id}/assignments`, and `GET/POST /compliance/framework-review-reports/{report_id}/comments`; assignment-aware review authorization; Tauri models/client/commands; Desktop Governance Evidence Review assignment/comment controls; store state/actions/tests; docs under `docs/design/framework-review-report-auditor-collaboration-mvp.md` and `docs/reports/framework-review-report-auditor-collaboration-2026-06-15.md`. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; real Postgres focused Framework Review Report integration test creating the full KAN-99/KAN-105 evidence chain, assigning one Auditor, blocking an unassigned same-tenant Auditor, creating comments, rejecting secret-like text, preserving artifact hash/no-claim flags, and confirming no Agent Governance evaluation changes; full backend Postgres suite (`308` passed); Tauri `cargo check`; Tauri `cargo fmt --check`; Tauri `cargo clippy -- -D warnings`; Tauri tests (`49` passed); frontend `pnpm --dir gitgov typecheck`; focused store test (`34` passed); full frontend tests (`366` passed); frontend lint/build; local `v52` migration/postcheck through ignored `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed for `cee4594`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8nlhi6q1p3s738c7f10` for `cee4594` reached `live`. Production `v52` migration/postcheck passed. Production smoke passed: `/health=200`, authenticated `/stats=200`, temporary Auditor A was assigned to report `frr_ac4ee214bc051caee783485d5755d34a`, Auditor A `assigned-to-me` returned the report, temporary Auditor B `assigned-to-me` did not return the report, Auditor A comment returned `201`, Auditor B comment and review returned `403 auditor_not_assigned`, Auditor A review returned `200`, `artifact_hash` stayed unchanged, no-claim flags stayed unchanged, active assignments were cleared after smoke, and all `kan109-prod-*` temporary API keys were revoked (`activeTemp=0`).
- Previous completed implementation ticket before KAN-109: GitHub issue `#377`, `KAN-108: Tenant Auditor role for compliance evidence review`.
- KAN-108 product decision from GPT consultation and local roadmap/repo analysis: after KAN-107, do not jump to signed manifests, PDF/DOCX, official regulatory mappings, BYOM, MCP, chatbot, or Agent Governance. First add separation of duties with a tenant-scoped `Auditor` role. Auditors can read/download existing compliance evidence and submit Framework Review Report review metadata, but cannot configure tenants, providers, policies, Deployment Gates, Agent Governance, framework packs, users, invitations, or API keys.
- KAN-108 implementation status on 2026-06-14: completed by PR `#378`, merged to `main` as `98cf543`. It adds `Auditor` to tenant `UserRole`, Supabase migration/postcheck `v51` for `api_keys`, `org_users`, and `org_invitations` role constraints, `require_compliance_reviewer` for Admin/Auditor evidence-review surfaces, Auditor access to KAN-99 export get/download, KAN-100 mapping get/control framework read, KAN-101 package get/download, KAN-105/KAN-106 report list/get/download, and KAN-107 report review. Creation/configuration surfaces remain Admin-only. Desktop Admin onboarding can provision/invite Auditor users and API key badges recognize Auditor. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; focused Framework Review Report Postgres tests (`2` passed) covering real Admin-generated evidence chain, Auditor read/download/review, Auditor mutation denials, other-tenant Auditor isolation, Developer denial, no-claim flags, unchanged artifact hash, and no Agent Governance evaluations; full backend Postgres tests with `--test-threads=2` (`308` passed); Tauri `cargo check`; Tauri `cargo fmt --check`; Tauri `cargo clippy -- -D warnings`; Tauri tests (`49` passed); frontend `pnpm --dir gitgov typecheck`; focused store test (`34` passed); full frontend tests (`366` passed); frontend lint/build; `v51` migration and postcheck through ignored local `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed for `98cf543`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8nk7hm8bjmc73f0sfng` for `98cf543` reached `live`. Production `v51` migration/postcheck passed. Production smoke passed: `/health=ok`, authenticated `/stats=200`, temporary Auditor key `kan108-prod-auditor-20260614194845` listed as role `Auditor`, Auditor `GET`/download succeeded for report `frr_ac4ee214bc051caee783485d5755d34a`, evidence export `cee_cdbddd6037b8483a80ce8127ca7d0a07`, mapping `cem_962e49057e89497aa480b4dc0bb55139`, review package `crp_8c121f821fc98f759db5750329c3338e`, and baseline framework `gitgov_release_governance_baseline_v1`; Auditor `PATCH /compliance/framework-review-reports/{report_id}/review` set `review_status=reviewed` with reviewer provenance, preserved `artifact_hash`, and kept no-claim flags intact; Auditor `POST /compliance/framework-review-reports`, `POST /api-keys`, and `GET /agent-governance/settings` returned `403`; the temporary Auditor key was revoked and all `kan108-prod-auditor-*` keys were inactive after smoke.
- Latest completed implementation ticket is GitHub issue `#374`, `KAN-107: Framework review report auditor workflow`.
- KAN-107 product decision from GPT consultation and local roadmap/repo analysis: after KAN-106, the next slice is manual review workflow over existing Framework Review Reports. Keep it manual-first and Admin-only because the current tenant RBAC model has `Admin`, `Architect`, `Developer`, and `PM`, but no tenant-level `Auditor` role yet. Do not introduce official regulatory mappings, compliance scores, certification claims, signed manifests, PDF/DOCX export, BYOM, MCP, chatbot behavior, OPA/Rego execution, provider mutation, policy mutation, or Agent Governance dependency in this slice.
- KAN-107 implementation status on 2026-06-14: completed by PR `#375`, merged to `main` as `bd4583a`. It adds Supabase migration/postcheck `v50`, review metadata on `compliance_framework_review_reports` (`review_status`, `reviewed_by_user_id`, `reviewed_at`, `review_notes_safe`), Admin-only route `PATCH /compliance/framework-review-reports/{report_id}/review`, Tauri model/client/command support, Desktop Governance Evidence Review manual review controls, audit action `compliance_framework_review_report.reviewed`, architecture/roadmap updates, and docs under `docs/design/framework-review-report-review-workflow-mvp.md` and `docs/reports/framework-review-report-review-workflow-2026-06-14.md`. Review statuses are `needs_review`, `reviewed`, `needs_changes`, and `rejected`; `needs_changes` and `rejected` require a safe note. Review notes are length-limited plain text and reject common HTML/secret token patterns. The workflow updates metadata only: `payload_json_redacted`, `artifact_hash`, source hashes, no-claim flags, and Agent Governance evaluations remain unchanged. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; focused KAN-107/KAN-105 Postgres tests (`2` passed); full backend Postgres tests with `--test-threads=2` (`307` passed); frontend typecheck; focused store test (`34` passed); full frontend tests (`366` passed); frontend lint/build; Tauri check/fmt/clippy/tests (`49` passed); `v50` migration and postcheck through ignored local `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed, including `CI` run `27515604304`, `Release Readiness Gate` run `27515604297`, `Quality Gate Policy Matrix` run `27515604306`, `Secret Scan` run `27515604311`, `Public Naming Guard` run `27515604300`, `Governance Correlation Smoke` run `27515604308`, `Desktop Updater Readiness` run `27515604305`, and `SonarQube Governance` run `27515604310`. Render deploy `dep-d8njkhsvikkc73alv9l0` for `bd4583a` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, `GET /compliance/framework-review-reports?org_name=yohandry10&framework_id=gitgov_release_governance_baseline_v1&limit=10` returned report `frr_ac4ee214bc051caee783485d5755d34a`, `PATCH /compliance/framework-review-reports/frr_ac4ee214bc051caee783485d5755d34a/review` set `review_status=needs_changes` with a safe note, reviewer provenance was present, `artifact_hash` stayed unchanged, list metadata reflected `needs_changes`, no-claim flags stayed intact, and invalid `review_status=approved` returned `400`.
- Latest completed implementation ticket is GitHub issue `#371`, `KAN-106: Framework review report inventory history`.
- KAN-106 product decision from GPT consultation attempt and local roadmap/repo analysis: after KAN-105, do not jump to official regulatory mappings, auditor workflow, signed provenance manifests, BYOM, MCP, chatbot, or PDF reports. First close the recoverability gap: Admins must be able to list previous Framework Review Reports by tenant/framework/mapping/package, inspect metadata without downloading payload JSON, and download a historical server-generated artifact from Desktop. GPT was asked in the existing ChatGPT thread, but the assistant responses were empty after repeated submissions, so the decision was made from the repo state and roadmap.
- KAN-106 implementation status on 2026-06-14: completed by PR `#372`, merged to `main` as `56ec538`. It adds Admin-only `GET /compliance/framework-review-reports` list behavior beside the existing KAN-105 create route, metadata-only list response with `items/count/limit`, safe filters for `framework_id`, `mapping_id`, `review_package_id`, limit clamping to `1..100`, org-scoped DB query ordered newest-first without selecting `payload_json_redacted`, Supabase migration/postcheck `v49` for inventory indexes, Tauri model/client/command support, Desktop Governance Evidence Review `History` load action, recent report cards, selected historical JSON download, store state/actions, tests, and docs under `docs/design/framework-review-report-inventory-history-mvp.md` and `docs/reports/framework-review-report-inventory-history-2026-06-14.md`. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; focused KAN-106/KAN-105 Postgres tests (`2` passed); full backend Postgres tests with `--test-threads=2` (`307` passed); frontend typecheck; focused store test (`34` passed); full frontend tests (`366` passed); frontend lint/build; Tauri check/fmt/clippy/tests (`49` passed); `v49` migration and postcheck through ignored local `DATABASE_URL`; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed, including `CI` run `27514397212`, `Release Readiness Gate` run `27514397217`, `Quality Gate Policy Matrix` run `27514397232`, `Secret Scan` run `27514397218`, `Public Naming Guard` run `27514397237`, `Governance Correlation Smoke` run `27514397226`, `Desktop Updater Readiness` run `27514397220`, and `SonarQube Governance` run `27514397228`. Render deploy `dep-d8nisuv7f7vs73fnnnq0` for `56ec538` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, `GET /compliance/framework-review-reports?org_name=yohandry10&framework_id=gitgov_release_governance_baseline_v1&limit=500` returned `200`, `count=2`, effective `limit=100`, first report `frr_ac4ee214bc051caee783485d5755d34a`, no payload/artifact field in list metadata, historical download returned schema `gitgov_framework_review_report.v1` with `10` controls, and invalid `mapping_id=bad` returned `400`.
- Latest completed implementation ticket before KAN-106 is GitHub issue `#368`, `KAN-105: Framework-specific review report export`.
- KAN-105 product decision from GPT/product review and local roadmap analysis: after KAN-104, do not jump to official regulatory mappings or BYOM. The next safe slice is a JSON-only framework-specific review report that consumes the already-reviewed KAN-99/KAN-100/KAN-101/KAN-103/KAN-104 evidence chain. It remains manual-first and for customer/auditor review only. It does not create official SOC 2, ISO, NIST, PCI, SBS, or LGPD mappings; compliance scores; certification claims; PDF/DOCX export; OPA/Rego execution; provider mutation; Action Center writes; BYOM; MCP; chatbot behavior; LLM summaries; or Agent Governance dependency.
- KAN-105 implementation status on 2026-06-14: completed by PR `#369`, merged to `main` as `84420a7`. Added Supabase migration/postcheck `v48`; backend table `compliance_framework_review_reports`; routes `POST /compliance/framework-review-reports`, `GET /compliance/framework-review-reports/{report_id}`, and `GET /compliance/framework-review-reports/{report_id}/download`; deterministic artifact schema `gitgov_framework_review_report.v1`; admin-only Tauri client/model/commands; Desktop Governance Evidence Review report panel; store actions and tests; docs under `docs/design/framework-specific-review-report-export-mvp.md` and `docs/reports/framework-specific-review-report-export-2026-06-14.md`. Reports bind `mapping_id` and `review_package_id`, verify both sources match, preserve evidence export hash, mapping hash, review package hash, framework owner/source/review provenance, pack hash, controls, evidence refs, missing evidence, audit metadata, and enforce `compliance_claim=false`, `regulatory_claim=false`, `certification=false`, `requires_auditor_review=true`. Customer-owned reports require the current framework pack to remain `reviewed`; rejected/archived/needs-review packs block new reports. Also restored KAN-100 input semantics so unsupported non-customer framework ids such as `soc2_*` fail as `400` instead of falling through to tenant lookup. Local validation passed: backend `cargo check`; focused KAN-105 Postgres tests (`2` passed); focused KAN-100 regression test (`1` passed); full backend Postgres tests with `--test-threads=2` (`307` passed); backend fmt/clippy; frontend typecheck, focused store test, full tests (`366` passed), lint, and build; Tauri check/fmt/clippy/tests (`49` passed); local `v48` migration/postcheck; `git diff --check`; publication guard. PR checks passed. Post-merge `main` checks passed, including `CI` run `27512940457`, `Release Readiness Gate` run `27512940444`, `Secret Scan` run `27512940447`, `Public Naming Guard` run `27512940470`, and `Quality Gate Policy Matrix` run `27512940454`. Render deploy `dep-d8ni0k647okc73f1jv70` for `84420a7` reached `live`. Production `v48` postcheck passed. Production smoke passed: `/health=ok`, authenticated `/stats=200`, Deployment Gate source `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`, export `cee_cdbddd6037b8483a80ce8127ca7d0a07`, mapping `cem_962e49057e89497aa480b4dc0bb55139` with `10` controls, review package `crp_8c121f821fc98f759db5750329c3338e`, report `frr_ac4ee214bc051caee783485d5755d34a`, report hash `sha256:7adb239ac4c00c67064ca39462f7bdce66898a818531b8f813b4cbb6cbee6a54`, downloaded schema `gitgov_framework_review_report.v1`, `10` report controls, source hashes matched, and no-claim flags remained true (`compliance_claim=false`, `regulatory_claim=false`, `certification=false`, `requires_auditor_review=true`).
- Latest completed implementation ticket is GitHub issue `#365`, `KAN-104: Framework pack review and provenance UX`.
- KAN-104 product decision from GPT/product review and local roadmap analysis: after KAN-103, a customer-owned framework pack import must not be treated as tenant-ready until an Admin reviews it. New packs start as `needs_review`; reviewed packs can be mapped; `needs_changes`, `rejected`, `archived`, and unreviewed packs are blocked with explicit `409` codes. This stays manual-first and does not create official regulatory mappings, certification claims, compliance scores, OPA/Rego execution, Policy-as-Code enforcement, provider mutation, Action Center automation, MCP/chatbot behavior, BYOM routing, or Agent Governance dependency.
- KAN-104 implementation status on 2026-06-14: completed by PR `#366`, merged to `main` as `e34433a`. Added Supabase migration/postcheck `v47`; backend `PATCH /compliance/framework-packs/{framework_pack_id}/review`; migrated KAN-103 statuses to `needs_review`/`reviewed`; hid unreviewed customer frameworks from `GET /compliance/control-frameworks`; blocked `POST /compliance/evidence-mappings` and new `POST /compliance/review-packages` for non-reviewed customer packs; added review provenance to package artifacts; added Tauri command/client/model support; added Governance Evidence Review Framework Pack Review UI; corrected frontend import selection so unreviewed imports do not become selected mapping frameworks. Local validation passed: backend `cargo check`; backend `cargo fmt --check`; backend `cargo clippy -- -D warnings`; focused backend `TEST_DATABASE_URL=local Postgres cargo test compliance_framework_packs -- --nocapture` (`3` passed); full backend `TEST_DATABASE_URL=local Postgres cargo test` (`305` passed); v47 migration/postcheck smoke in isolated local Postgres schema migrated legacy statuses to `needs_review` and `reviewed`; frontend `pnpm --dir gitgov typecheck`, lint, full tests (`366` passed), and build (existing large chunk warning); Tauri `cargo check`, `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`, tests (`49` passed), and fmt check; `git diff --check`; publication guard. PR checks passed. Production `v47` migration/postcheck passed. Render deploy `dep-d8ngum19rddc739qhvog` for `e34433a` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, unreviewed customer framework was hidden and mapping returned `framework_pack_not_reviewed`; reviewed framework became listable; mapping `cem_93a966d7e26b4728a5a28f534019a5fc` and review package `crp_e117bebd4154f647be447fbec5fe4ec9` were created with review provenance, hash `sha256:dee159d526ec05cb38e677688d5eceb0c7e1f021fea62584f50816810caace93`, no compliance/regulatory/certification claims; after rejection the old mapping could not produce a new package (`framework_pack_rejected`); smoke packs `cfp_3fb36bb89a583956dc1f1e775654354a` and `cfp_d4d194979ebc0c59fd073cb3884b59ce` were archived.
- Latest completed implementation ticket is GitHub issue `#362`, `KAN-103: Customer-Owned Framework Pack Import`.
- KAN-103 product decision from GPT/product review and local roadmap analysis: implement customer-owned framework pack import after KAN-102, but do not call it regulatory framework import. GitGov lets a customer import its own JSON/YAML control pack to organize KAN-99 evidence exports and KAN-101 review packages. GitGov does not certify customer packs and does not ship official SOC 2, ISO, NIST, PCI, SBS, or LGPD mappings in this slice. Required flags stay `owner_type=customer`, `source=customer_provided`, `compliance_claim=false`, `regulatory_claim=false`, `gitgov_certifies=false`, `official_regulatory_mapping=false`, and `requires_auditor_review=true`.
- KAN-103 implementation status on 2026-06-14: completed by PR `#363`, merged to `main` as `2e9d243`. Backend routes added for `POST /compliance/framework-packs/import`, `GET /compliance/framework-packs`, and `GET /compliance/framework-packs/{framework_pack_id}`; KAN-100 framework list/get became tenant-aware; KAN-100 mapping can use customer frameworks; KAN-101 review packages preserve framework owner/source/pack hash; Supabase migration/postcheck `v46` added; Tauri and Governance Evidence Review UI can list/import/select customer packs. Local validation passed: backend `cargo check`; focused KAN-103 Postgres tests (`2` passed, covering JSON/YAML import, no-claims, reserved ids, unknown evidence types, duplicate controls, oversized packs, secret-like metadata, non-admin denial, agent-key denial, tenant isolation, mapping, review package provenance, and no new Agent Governance evaluations); existing KAN-100/KAN-101 focused Postgres tests (`4` passed); full backend tests with `TEST_DATABASE_URL` (`304` passed); frontend typecheck; focused frontend tests (`37` passed); full frontend tests (`366` passed); frontend lint/build with existing Vite large chunk warning; Tauri `cargo check`, clippy, tests (`49` passed), and fmt check; backend fmt/clippy; v46 migration/postcheck in isolated local PostgreSQL schema seeded with minimal v44 state; `git diff --check`; publication guard; Vite smoke on `/governance/releases` with no console/page errors and expected Desktop-required screen. PR checks passed. Production `v46` migration/postcheck passed. Render deploy `dep-d8nga5km0tmc73basn0g` for `2e9d243` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, Deployment Gate source `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`, JSON customer framework `customer_kan103_runtime_smoke_17814669449_9655a36246a2`, YAML customer framework `customer_kan103_runtime_yaml_178146694497_87ce14558b38`, export `cee_e04f107ee6e04d0891a40129373ce6ef`, mapping `cem_0277f6372a7a4792a1575130f4b80236`, review package `crp_003ae13483485afd55a5cd696c605496`, downloaded artifact hash matched stored hash, `owner_type=customer`, `compliance_claim=false`, `regulatory_claim=false`, `requires_auditor_review=true`, and Agent Governance evaluations stayed `7 -> 7`.
- Latest completed implementation ticket before KAN-103 was GitHub issue `#358`, `KAN-102: Governance Evidence Review UI`.
- KAN-102 product decision from GPT/product review and local roadmap analysis: do not start customer-provided framework pack import yet. First make the completed KAN-99/KAN-100/KAN-101 evidence primitives usable from Desktop Governance. The slice adds a manual-first Governance > Releases UI where an Admin selects a Deployment Gate authorization, generates a KAN-99 Compliance Evidence Export, maps it to the GitGov-owned `gitgov_release_governance_baseline_v1`, creates a KAN-101 Control Mapping Review Package, inspects ids/hashes/no-claim flags/missing evidence, and downloads the server-generated JSON. It is not SOC 2, ISO, NIST, PCI, SBS, LGPD, a compliance score, a certification claim, OPA/Rego execution, Policy-as-Code mutation, provider mutation, MCP, chatbot, BYOM, or required Agent Governance.
- KAN-102 implementation status on 2026-06-14: completed by PR `#359`, merged to `main` as `88cda2a`, plus payload-contract hotfix PR `#360`, merged as `ba655c2`. It adds Tauri compliance DTOs/client methods/commands for existing `/compliance/evidence-exports`, `/compliance/evidence-mappings`, and `/compliance/review-packages` routes; a separate Zustand `compliance` action slice; `ComplianceEvidenceFlowPanel` mounted in Governance > Releases; focused component/store tests; and docs `docs/design/governance-evidence-review-ui-mvp.md` plus `docs/reports/governance-evidence-review-ui-2026-06-14.md`. Local validation passed: `npm --prefix gitgov run typecheck`, focused KAN-102 tests (`35` passed), full frontend tests (`364` passed), `npm --prefix gitgov run lint`, `npm --prefix gitgov run build`, Tauri `cargo check`, Tauri `cargo clippy -- -D warnings`, Tauri `cargo test` (`49` passed), Tauri `cargo fmt --check`, `git diff --check`, and publication guard. PR checks and post-merge checks passed. Browser/Vite smoke for `/governance/releases` loaded the bundle with no page errors and showed the expected `GitGov Desktop required` screen because Governance is a Desktop runtime surface. Render deploy `dep-d8nbnatckfvc73em0vrg` for `ba655c2` reached `live`; `/health=ok`; production smoke reused Deployment Gate authorization `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`, created export `cee_7610ff2db7a44f56875ee2709b486295`, mapping `cem_1e731f2983e4451ea89722c48a27adae`, and review package `crp_6f36f65322b3da03f404ee24edd38855`; downloaded artifact schema was `gitgov_control_review_package.v1`, contained `10` controls, and kept `compliance_claim=false`, `regulatory_claim=false`, `requires_auditor_review=true`, `certification=false`, `agent_governance_required=false`, `policy_mutation=false`, `provider_mutation=false`, and no raw payload.
- Latest completed implementation ticket before KAN-102 was GitHub issue `#355`, `KAN-101: Control Mapping Review Package`.
- KAN-101 product decision from GPT/product review and local roadmap analysis: implement a JSON-only, hashable Control Mapping Review Package over KAN-100 mappings, not customer framework imports yet. It packages source export hash, deterministic mapping hash, framework version, control matrix summary, missing evidence, no-claim flags, and audit metadata for customer/auditor review. It remains manual-first, Admin-only, and creates no Agent Governance dependency or evaluations. It is not SOC 2, ISO, NIST, PCI, SBS, LGPD, a PDF/DOCX, a compliance score, a badge, a certification claim, OPA/Rego execution, Policy-as-Code mutation, provider mutation, MCP, chatbot, or BYOM.
- KAN-101 implementation status on 2026-06-14: completed by PR `#356`, merged to `main` as `beebdca`. Backend routes added: `POST /compliance/review-packages`, `GET /compliance/review-packages/{review_package_id}`, and `GET /compliance/review-packages/{review_package_id}/download`. Migration `supabase_schema_v45.sql` creates `compliance_review_packages`; `supabase_schema_v45_postcheck.sql` validates required columns, indexes, and no-claim constraints. The package id is deterministic/idempotent for the same org, mapping, schema, and mapping hash. Local validation passed: `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, v45 postcheck against temporary PostgreSQL `gitgov-kan101-pg` on `127.0.0.1:55444`, focused `compliance_review_packages` tests (`2` passed), full backend tests (`302` passed), `git diff --check`, and publication guard. PR checks passed. Production `v45` migration/postcheck passed. Render deploy `dep-d8nb3neq1p3s7385sorg` for `beebdca` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, anonymous review-package download returned `401`, source mapping `cem_30553ff8ecd74ad4a06ea5d6ddb0b610`, review package `crp_f01effe8cc716b2353d7ed2078642c20`, create returned `201`, metadata/download returned `200`, repeated create returned the same package id, download hash matched stored `artifact_hash`, source export hash was preserved, package contained `10` controls, `compliance_claim=false`, `regulatory_claim=false`, `requires_auditor_review=true`, `certification=false`, and Agent Governance evaluations stayed `7 -> 7`.
- Latest completed implementation ticket is GitHub issue `#352`, `KAN-100: Evidence-to-Control Mapping MVP`.
- KAN-100 product decision from GPT/product review and local roadmap analysis: implement Evidence-to-Control Mapping now, but do not call it official regulatory compliance mapping. The MVP maps a KAN-99 Compliance Evidence Export to the GitGov-owned `gitgov_release_governance_baseline_v1` catalog. It persists a deterministic matrix of control status, evidence refs, missing evidence, and safe notes. It must keep `compliance_claim=false`, `regulatory_claim=false`, `requires_auditor_review=true`, manual-first behavior, and zero Agent Governance dependency.
- KAN-100 implementation status on 2026-06-14: completed and issue `#352` closed by PR `#353`, merged to `main` as `b3fdf2e`. Backend routes added: `GET /compliance/control-frameworks`, `GET /compliance/control-frameworks/{framework_id}`, `POST /compliance/evidence-mappings`, and `GET /compliance/evidence-mappings/{mapping_id}`. Migration `supabase_schema_v44.sql` creates `compliance_control_frameworks`, `compliance_controls`, `compliance_evidence_mappings`, and `compliance_evidence_mapping_items`, seeds 10 GitGov Release Governance Baseline controls, and hard-checks no compliance/regulatory claims. Local validation passed: `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, v44 postcheck against temporary PostgreSQL `gitgov-kan100-pg` on `127.0.0.1:55443`, focused `compliance_evidence_mappings` tests (`2` passed), sensitive admin route test, full backend tests (`300` passed), `git diff --check`, and publication guard. PR checks passed. Production `v44` migration/postcheck passed. Render deploy `dep-d8nailfaqgkc73c221ug` for `b3fdf2e` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, baseline framework count `1`, control count `10`, source Deployment Gate `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`, export `cee_3df4bb2f2a8a4aa2a5a7613885ad55bf`, mapping `cem_30553ff8ecd74ad4a06ea5d6ddb0b610`, anonymous mapping returned `401`, authenticated mapping create returned `201`, mapping read returned `10` items, export hash verified, `compliance_claim=false`, `regulatory_claim=false`, `requires_auditor_review=true`, and Agent Governance evaluations stayed `7 -> 7`.
- Latest completed implementation ticket is GitHub issue `#349`, `KAN-99: Compliance Evidence Export v1`.
- KAN-99 product decision from GPT/product review and local roadmap analysis: after KAN-98, pause MCP/agent expansion. The next enterprise-safe slice is a manual-first Compliance Evidence Export over existing Deployment Gate authorizations. It packages gate decision, policy checksum/source, readiness, approvals, evidence counts/references, gaps, audit timestamps, and explicit `agent_governance_used=false` into a hashable JSON artifact. It is not a regulatory mapper, not PDF, not a compliance certification claim, not OPA/Rego execution, not provider mutation, and not a new Agent Governance surface.
- KAN-99 implementation status on 2026-06-14: completed and issue `#349` closed by PR `#350`, merged to `main` as `5f84655`. Backend routes added: `POST /compliance/evidence-exports`, `GET /compliance/evidence-exports/{export_id}`, and `GET /compliance/evidence-exports/{export_id}/download`. Migration `supabase_schema_v43.sql` creates `compliance_evidence_exports`; `supabase_schema_v43_postcheck.sql` validates the table, columns, and indexes. Integration tests use real Postgres and verify redaction, stable download hash, Admin-only access, tenant isolation, unsupported scope/format rejection, explicit missing evidence, `llm_decision=false`, `agent_governance_used=false`, and no new `agent_governance_evaluations`. Local validation passed: `cargo check`, `cargo fmt --check`, `cargo clippy -- -D warnings`, sensitive admin route test, focused `compliance_evidence_exports` tests (`2` passed), full backend tests (`298` passed), `git diff --check`, publication guard, and v43 postcheck against temporary PostgreSQL `gitgov-kan99-pg` on `127.0.0.1:55442`. PR checks passed. Production `v43` migration/postcheck passed. Render deploy `dep-d8na49naqgkc73c1porg` for `5f84655` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, latest Deployment Gate source `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`, anonymous compliance export returned `401`, authenticated create returned `201` for export `cee_0043aae0507a4f52a1825774eed10bfb`, metadata/download returned `200`, download hash matched the stored artifact hash, Agent Governance evaluations stayed `7 -> 7`, `agent_governance_used=false`, `compliance_claim=false`, and `framework_mapping=false`.
- Latest completed implementation ticket is GitHub issue `#345`, `KAN-98: Agent read-only governance context API`.
- KAN-98 product decision from GPT/product review attempt and local roadmap analysis: do not start MCP yet. The next enterprise-safe slice is a read-only Agent Governance context API with a separate `agent_governance:read` scope. It prepares future MCP/tool use without shipping MCP, does not authorize execution, does not create formal evaluation rows, and remains denied for agent principals while a tenant is disabled/manual-only.
- KAN-98 implementation status on 2026-06-14: completed and issue `#345` closed. PR `#346` merged to `main` as `0e4ee1c`; production hotfix PR `#347` merged to `main` as `e2b2e60`. KAN-98 adds `GET /agent-governance/context`, `agent_governance:read` scope support for agent keys, route-sensitive agent-key auth for read versus evaluate/dry-run, read-only context over existing branch/policy/pipeline/deployment-gate/risk/activity evidence, explicit response markers `read_only=true`, `will_authorize_execution=false`, and `mcp_surface=false`, and Postgres-backed integration tests for read success, invalid scope, manual-only denial, and read-only key isolation. Local validation passed before PR and after the production hotfix: focused `agent_governance_context` tests (`4` passed), `cargo check`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `git diff --check`, publication guard, and full backend tests (`296` passed). Post-merge `main` checks passed for both PRs. Initial Render production smoke found a production-schema drift: `pipeline_events` has `ingested_at`, while the first KAN-98 query ordered by the integration-test helper `created_at`; PR `#347` corrected the query to `ingested_at`. Render deploy `dep-d8n9dt19rddc739llrb0` for `e2b2e60` reached `live`. Final production smoke passed: `/health=ok`, authenticated `/stats=200`, Agent Governance started and ended as `enabled=false/mode=manual_only`, disabled read context returned `403 agent_governance_disabled`, temporary opt-in read context returned `200` with `read_only=true`, `will_authorize_execution=false`, `mcp_surface=false`, and `principal_type=agent`, read-only key evaluation returned `403 invalid_scope`, smoke evaluation history stayed `total=0`, temporary keys were revoked, and `activeTempCount=0`.
- Latest completed implementation ticket is GitHub issue `#341`, `KAN-97: Agent Key Expiry and Rotation UX`.
- KAN-97 product decision from GPT/product review: do not start MCP yet. The next enterprise-safe slice is agent credential lifecycle hardening because GitGov already has deterministic evaluate, disabled-by-default settings, shared governance decisions, agent-scoped keys, dry-run, and minimal attribution. KAN-97 keeps GitGov manual-first and Agent Governance opt-in; Deployment Gates do not use agent keys.
- KAN-97 implementation status on 2026-06-14: completed and issue `#341` closed. PR `#342` merged to `main` as `58fb41a`, and production-validation docs PR `#343` merged as `75d9285`. It adds default 90-day expiry for new agent keys unless `no_expiry=true` is explicit, derived key lifecycle status (`active`, `expiring_soon`, `expired`, `revoked`, `rotation_pending`, `no_expiry`), `POST /agent-governance/agent-keys/{key_id}/rotate`, old/new key linkage with `rotated_from_key_id` and `replaced_by_key_id`, bounded grace-period rotation, specific `agent_key.denied_expired` and `agent_key.denied_revoked` audit events, Supabase migration `v42`, and docs under `docs/design/agent-key-expiry-rotation-ux-mvp.md` plus `docs/reports/agent-key-expiry-rotation-2026-06-14.md`. Local validation passed: `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `v42_postcheck` against temporary PostgreSQL 16 on `127.0.0.1:55440`, focused Agent Governance tests (`27` passed), full backend tests (`292` passed), `git diff --check`, and publication guard. PR checks and post-merge `main` checks passed for both merge commits. Production migration `v42` was applied and postcheck passed. Render deploy `dep-d8n86cfaqgkc73c0ojt0` for `58fb41a` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, Agent Governance started and ended as `enabled=false/mode=manual_only`, a temporary default-expiring key was created with `status=active`, rotated to a replacement key with old `status=rotation_pending`, replacement evaluation created `agv_8c7a3924c7254417a3db512a233b9f53`, old key dry-run worked during grace, revoked old key returned `401 agent_key_revoked`, a temporary expired key returned `401 agent_key_expired`, temporary keys were revoked, and `activeTempCount=0`.
- KAN-96 product decision from GPT/product review: implement a minimal attribution envelope for optional Agent Governance dry-run/evaluate requests, not a full agent observability system. The slice answers which agent key/tool/session/correlation/external run asked for a decision, whether it was dry-run or formal evaluation, and what deterministic decision GitGov returned. No MCP, chatbot, BYOM, autonomous execution, provider mutation, Deployment Gate behavior change, prompt storage, raw tool traces, source code storage, or Action Center writes are in scope.
- KAN-96 implementation status on 2026-06-14: PR `#339` merged to `main` as `3f24c0b`. It adds optional `attribution` to `POST /agent-governance/dry-run` and `POST /agent-governance/evaluate`, strict 128-character safe string validation, generated `agcorr_` correlation ids, response `attr_` attribution envelopes, formal evaluation persistence columns through Supabase migration `v41`, `GET /agent-governance/evaluations?correlation_id=...`, and dry-run audit metadata without formal evaluation persistence. Local validation passed: backend check/clippy/fmt, v41 postcheck against temporary PostgreSQL 16 on `127.0.0.1:55439`, focused Agent Governance tests (`24` passed), full backend tests (`289` passed), `git diff --check`, and publication guard. Post-merge GitHub workflows passed. Production migration `v41` was applied and postcheck passed. Render deploy `dep-d8n7nb4m0tmc73b5hveg` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, manual-only attributed evaluate returned `403 agent_governance_disabled` with history `total=0`, temporary agent key dry-ran `commit` with `consumer_type=agent_dry_run`, correlation `corr-kan96-prod-dry-1781431652106`, and no formal evaluation rows, formal evaluate created `agv_833b8b31c41947ccaf5c69d153890035` with persisted correlation `corr-kan96-prod-eval-1781431652106`, parent correlation, tool `codex-cli`, and `llm_decision=false`, history filter returned `total=1`, unsafe attribution returned `400`, the temporary key was revoked, and settings were restored to `enabled=false/mode=manual_only`.
- KAN-95 product decision from GPT/product review: implement Agent Governance dry-run as the next small safe slice after KAN-94. Dry-run answers "what would GitGov decide?" without authorizing execution and without persisting an `agent_governance_evaluations` row. It remains tenant opt-in, optional, manual-first, and not required for Deployment Gates or regulated/manual-only customers.
- KAN-95 implementation status on 2026-06-14: PR `#336` merged to `main` as `d63fba3`. It adds `POST /agent-governance/dry-run`, reuses the deterministic Agent Governance policy evaluator, returns `dry_run=true`, `would_persist_evaluation=false`, and `would_authorize_execution=false`, allows KAN-94 agent-scoped keys to call dry-run through the existing `agent_governance:evaluate` scope, and shares agent-key `allowed_actions` enforcement with `POST /agent-governance/evaluate`. Local validation passed: backend check/clippy, v40 postcheck against temporary PostgreSQL 16 on `127.0.0.1:55438`, focused Agent Governance tests (`19` passed), full backend tests (`284` passed), `git diff --check`, and publication guard. Post-merge GitHub workflows passed. Render deploy `dep-d8n79leq1p3s7383pu90` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, disabled dry-run returned `403 agent_governance_disabled` with `would_persist_evaluation=false`, temporary agent key dry-ran `commit` with `decision=allowed`, `principal_type=agent`, matching key id, `llm_decision=false`, and no persisted evaluation rows (`history total=0`), disallowed `change_policy` returned `403 action_not_allowed`, the temporary key was revoked, and settings were restored to `enabled=false/mode=manual_only`.
- Latest completed implementation ticket: GitHub issue `#332`, `KAN-94: Agent-scoped API keys for optional Agent Governance`, closed by PR `#333` and merged to `main` as `aa0a9c9` on 2026-06-14.
- KAN-94 product decision from GPT/product review: implement agent-scoped credentials as an optional Agent Governance hardening slice, not a chatbot, MCP server, BYOM feature, IAM replacement, provider mutation path, Deployment Gate dependency, or default requirement. GitGov remains manual-first. Agent keys are limited to `POST /agent-governance/evaluate`, require tenant opt-in, store only token hashes plus prefix/last-four metadata, return plaintext token only once, audit create/use/deny/revoke/invalid-scope events, and must not create evaluation rows for disabled tenants, revoked/expired keys, invalid scopes, tenant mismatch, or disallowed actions.
- KAN-94 implementation status on 2026-06-14: backend Agent Governance agent-key management routes added (`GET/POST /agent-governance/agent-keys`, `DELETE /agent-governance/agent-keys/{key_id}`), agent-key auth path added for `ggag_` bearer tokens, `agent_governance_agent_keys` table and `agent_governance_evaluations` principal identity columns added through migration `v40`, Agent Governance handlers split into runtime/admin/key modules, and docs added under `docs/design/agent-scoped-api-keys-mvp.md` plus `docs/reports/agent-scoped-api-keys-2026-06-14.md`. Local validation passed: backend fmt/check/clippy, v40 migration/postcheck against temporary PostgreSQL 16 on `127.0.0.1:55437`, focused Agent Governance tests (`15` passed), full backend tests (`280` passed), `git diff --check`, and publication guard. Post-merge checks passed. Production migration `v40` was applied and postcheck passed. Render deploy `dep-d8n6pv77f7vs73fgfta0` for `aa0a9c9` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, manual-only disabled evaluation returned `403 agent_governance_disabled`, temporary agent key created without exposing plaintext token in list responses, temporary opt-in allowed evaluation `agv_356ca31100864046b104e2184eaec0ba` with `principal_type=agent` and `llm_decision=false`, `change_policy` returned `403 action_not_allowed`, revoked token returned `401 revoked_key`, and settings were restored to `enabled=false/mode=manual_only`.
- Previous completed implementation ticket: GitHub issue `#329`, `KAN-93: Shared Governance Decision Model for Deployment Gates`, closed by PR `#330` and merged to `main` as `8a462bd` on 2026-06-14.
- Previous completed implementation ticket: GitHub issue `#326`, `KAN-92: Agent Governance Control Boundary`, closed by PR `#327` and merged to `main` as `104131e` on 2026-06-14.
- KAN-93 product decision from GPT/product review: implement a neutral `shared-governance-decision.v1` model consumed by Deployment Gates and Agent Governance, without coupling Deployment Gates to `/agent-governance/evaluate`. Deployment Gates remain CI/CD-facing/manual-first and emit `consumer_type=deployment_gate`, `actor_type=system`, and `agent_governance_used=false`; Agent Governance remains optional and emits `consumer_type=agent_governance` only when explicitly used. KAN-94 should likely be agent-scoped API credentials, but only as an opt-in tenant feature after KAN-93 is shipped.
- KAN-93 implementation status on 2026-06-14: backend shared decision builder added; Deployment Gate authorizations persist `details.shared_governance_decision`, expose top-level `governance_decision`, and include the same model in admin audit metadata; Agent Governance evaluations include `evaluation.shared_governance_decision`; Desktop/Tauri/frontend models and the Deployment Gate history panel show the shared decision and `agent not used`. No database migration is required. Local validation passed against temporary PostgreSQL 16 on `127.0.0.1:55436`: backend fmt/check/clippy, focused Deployment Gate tests (`10`), focused Agent Governance tests (`10`), full backend tests (`275`), Tauri fmt/check/clippy/tests (`49`), frontend typecheck/lint/build, focused Deployment Gate history test (`1`), full frontend tests (`361`), `git diff --check`, and publication guard. Post-merge main checks passed. Render deploy `dep-d8n66cn7f7vs73fg79sg` for `8a462bd` reached `live`. Production smoke passed: `/health=ok`, authenticated `/stats=200`, Agent Governance remained `enabled=false/mode=manual_only`, KAN-93 release-bound evidence packet generation returned `found=true`, `POST /deployment-gates/authorize` created `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a` with legacy `decision=advisory/approved=true`, shared `consumer_type=deployment_gate`, shared `decision=insufficient_evidence`, shared `agent_governance_used=false`, history preserved `agent_governance_used=false`, and smoke agent `kan93-deployment-gate-smoke` had `0` Agent Governance evaluation rows.
- Previous Agent Governance implementation ticket: GitHub issue `#321`, `KAN-90: Agent Governance Policy API MVP`, closed by PR `#322` and merged to `main` as `f6a3603` on 2026-06-14.
- KAN-92 adds tenant-level `agent_governance_settings`, Admin-only `GET/PUT /agent-governance/settings`, Admin-only `GET /agent-governance/evaluations`, disabled-by-default behavior for `POST /agent-governance/evaluate`, `403 agent_governance_disabled` without persisting evaluation rows when disabled, opt-in/out audit events, denied-attempt audit events, and minimized/redacted persisted request payload. Local validation passed on 2026-06-14: backend fmt/check/clippy, fresh PostgreSQL 16 migration/postcheck for `supabase_schema_v39.sql`, focused `agent_governance` tests (`10` passed), and full backend tests (`275` passed). Production validation passed on 2026-06-14: v39 migration/postcheck passed, Render deploy `dep-d8n5nhu47okc73eqd510` for `104131e` reached `live`, `/health=ok`, authenticated `/stats=200`, anonymous settings read `401`, default settings `enabled=false/mode=manual_only/payload_mode=minimized`, disabled evaluation `403 agent_governance_disabled` with history `total=0`, temporary opt-in allowed evaluation `agv_a8375adeebe640be8d6074883d5e1b71` with `[REDACTED]` secret-like metadata in response/history, and final opt-out restored `enabled=false/mode=manual_only`.
- KAN-90 adds `POST /agent-governance/evaluate`, append-only `agent_governance_evaluations`, deterministic action decisions for `commit`, `push`, `open_pr`, `merge_pr`, `change_policy`, and `deploy`, route-sensitive auth classification, API docs, and real Axum/Postgres integration coverage. Product decision: GitGov is manual-first; agent governance is optional/opt-in, not a chatbot, not a bring-your-own-model requirement, and not a replacement for human approvals. Agents can ask before acting only when a customer chooses to use them; GitGov policy decides; `evaluation.policy.llm_decision=false`.
- KAN-90 local validation on 2026-06-14 passed: `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`, `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`, `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`, local v38 migration/postcheck against temporary Postgres, focused `agent_governance` tests with explicit `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:55434/gitgov` (`7` tests), sensitive path test (`1` test), full backend `cargo test --manifest-path gitgov\gitgov-server\Cargo.toml` with the same real Postgres (`272` tests), `git diff --check`, and `.\scripts\security\publication_guard.ps1`.
- KAN-90 PR and post-merge checks passed: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- KAN-90 production validation on 2026-06-14: production `supabase_schema_v38.sql` was applied manually through ignored `DATABASE_URL`, and `v38_postcheck.sql` returned `PASS` for table, constraints, and indexes. Render deploy `dep-d8n4sj19rddc739je0n0` for `f6a3603` reached `live`. Smoke returned `/health=ok`, authenticated `/stats=200`, anonymous `POST /agent-governance/evaluate` returned `401`, authenticated ticketed `commit` returned `evaluation_id=agv_3962e78980d84ab58a4ccece859226c2`, `decision=allowed`, `allowed=true`, `evaluation.policy.llm_decision=false`, and authenticated protected-branch `push` to `main` returned `evaluation_id=agv_5c6c0d80faa54e7790a02d97a2b12aa8`, `decision=requires_approval`, `requires_approval=true`, `evaluation.policy.llm_decision=false`.
- KAN-89 was documentation/product-state synchronization only: it did not change runtime behavior,
  database schema, provider configuration, or production deployment.
- KAN-80 implements the first concrete Deployment Gates 0.1 slice, not a broad integration wizard: one Admin-managed first-repo setup per org, stable `run_id`, repo/branch selection, provider/module/preset selection, policy/workflow preview acknowledgement, backend-normalized baseline readiness, Action Center gaps, Desktop UI under `Governance > Adoption`, and CTA into advisory gate simulation.
- KAN-80 backend route: `GET/PUT /enterprise/first-governed-repo-setup`. It is Admin-only, org-scoped like the enterprise adoption routes, rejects secret-looking baseline JSON, requires GitHub as selected provider, preserves `run_id` across upserts, and writes `upsert_first_governed_repo_setup` audit entries.
- KAN-80 migration: `gitgov/gitgov-server/supabase/supabase_schema_v32.sql` creates `enterprise_first_governed_repo_setups`.
- KAN-80 documentation: `docs/design/first-governed-repo-setup-mvp.md` and `docs/reports/first-governed-repo-setup-2026-06-13.md`.
- KAN-80 local validation run on 2026-06-13: backend check/clippy/full test (`253` tests), focused KAN-80 test with explicit `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov`, Tauri check/clippy/full test (`49` tests), frontend typecheck/lint/full test (`352` tests)/build, `git diff --check`, and `.\scripts\security\publication_guard.ps1` all passed. Vite still reports the existing large chunk warning while completing the production build.
- KAN-80 production validation on 2026-06-13: PR `#296` merged to `main` as `fae9e69`, all post-merge GitHub checks passed, Render deploy `dep-d8maj2u8bjmc73eakeq0` for `fae9e69` reached `live`, Supabase migration `v32` was applied manually through ignored `DATABASE_URL`, `GET https://gitgov-api.onrender.com/health` returned `200`/`ok`, authenticated `/stats` returned `200`, and authenticated `GET /enterprise/first-governed-repo-setup?org_name=yohandry10` returned `200` with `{"found":false}`.
- Historical implementation ticket after KAN-81: `KAN-82: Platform principals superadmin hardening` on branch `feature/KAN-82-platform-principals`.
- KAN-81 decision: `Platform Founder` is a platform principal outside all tenants (`org_id=null`), not the GitGov internal tenant. `GitGov Internal` remains a normal dogfood tenant; its tenant admins cannot create sibling tenants.
- KAN-81 backend/DB shape on branch `feature/KAN-81-platform-superadmin`: `/me` returns `principal_type` and `requires_workspace_for_tenant_surfaces`; `/platform/tenants` lists/provisions tenants for Platform Founder; `/platform/tenants/{login}/lifecycle` changes tenant lifecycle; `/orgs` create remains compatibility over the same audited platform provisioning semantics; `orgs` is now the tenant catalog with `tenant_type`, `lifecycle_status`, `provisioning_source`, `provisioned_by`, `platform_metadata`, and lifecycle timestamps. Migration: `gitgov/gitgov-server/supabase/supabase_schema_v33.sql`.
- KAN-81 local validation so far: `cargo fmt --manifest-path .\gitgov\gitgov-server\Cargo.toml --check`, `cargo check --manifest-path .\gitgov\gitgov-server\Cargo.toml`, `cargo clippy --manifest-path .\gitgov\gitgov-server\Cargo.toml -- -D warnings`, focused Postgres integration tests `platform_tenant_administration_requires_founder_and_audits_lifecycle`, `create_org_requires_founder_global_admin_key`, and `org_discovery_and_me_return_human_scope`, plus full backend `cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml` (`254` tests) passed with `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov`.
- KAN-81 production validation on 2026-06-13: PR `#299` merged to `main` as `0d2e5e2`; production DB migration `v33` was applied manually through ignored `DATABASE_URL` before merge; postcheck found `8` tenant catalog columns and `3` tenant constraints; post-merge GitHub `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed; Render deploy `dep-d8mc9stckfvc73e5umn0` reached `live`; production `/health` returned `200`, authenticated `/stats` returned `200`, authenticated `/me` returned `principal_type=platform_founder` with `requires_workspace_for_tenant_surfaces=true`, and authenticated `GET /platform/tenants` returned `200` with `21` tenants and lifecycle fields present.
- KAN-82 implementation status: PR `#301` merged to `main` as `11465a0` on 2026-06-13. It adds `platform_principals` through `supabase_schema_v34.sql`, backend auth now resolves Platform Founder from an active/break-glass `platform_principals` row, `/me` includes `platform_principal_id`, Desktop/Tauri types accept the new field, and the old `VITE_FOUNDER_GITHUB_LOGIN`/`VITE_FOUNDER_LOGIN` founder-gating path was removed. Product decision: superadmin/founder authenticates by GitGov API key plus platform principal row, not by GitHub OAuth/device flow; GitHub remains operator/repo identity for tenant workflows.
- KAN-82 production validation on 2026-06-13: `supabase_schema_v34.sql` was applied manually through ignored local `DATABASE_URL`; `v34_postcheck.sql` returned `PASS` for `platform_principals.table`, `platform_principals.constraints`, and `platform_principals.bootstrap_founder`. Post-merge GitHub `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed for `11465a0`. Render deploy `dep-d8md89uq1p3s73fm8ii0` reached `live`; production `/health` returned `ok`, authenticated `/stats` returned `200`, authenticated `/me` returned `client_id=bootstrap-admin`, `role=Admin`, `principal_type=platform_founder`, non-empty `platform_principal_id`, `org_id=null`, and `requires_workspace_for_tenant_surfaces=true`; authenticated `/platform/tenants` returned `21` tenants with lifecycle fields.
- KAN-83 implementation status: PR `#303` merged to `main` as `4dfba5f` on 2026-06-13. It adds the first CI/CD-facing Deployment Gates 0.1 authorization API: `POST /deployment-gates/authorize`, `GET /deployment-gates/authorizations`, Supabase migration `v35` for append-only `deployment_gate_authorizations`, evidence packet binding validation, release-governance evaluator reuse, policy checksum, advisory/blocked/approved decision semantics, break-glass eligibility flag, persisted request/evaluation/details, and admin audit logging. It does not add Desktop history UI, provider mutation, OPA/Rego execution, or default blocking for record-only customers.
- KAN-83 local validation on 2026-06-13: `cargo check --manifest-path .\gitgov\gitgov-server\Cargo.toml`, `cargo fmt --manifest-path .\gitgov\gitgov-server\Cargo.toml --check`, `cargo clippy --manifest-path .\gitgov\gitgov-server\Cargo.toml -- -D warnings`, focused `deployment_gate` tests (`6` passed with real temporary Postgres on host port `55433`, including authorization history, org-scope security, and evidence ticket mismatch rejection), full backend `cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml` (`260` passed against the same temporary Postgres), local `v35` migration/postcheck, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed.
- KAN-83 production validation on 2026-06-13: post-merge GitHub `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed for `4dfba5f`; Render deploy `dep-d8mf606q1p3s73fn44vg` reached `live`; production `supabase_schema_v35.sql` was applied and `v35_postcheck.sql` returned `PASS` for table, decision constraint, and indexes. Production was missing the older dependency table `release_evidence_packets`, so idempotent migration `supabase_schema_v28.sql` was also applied and verified before smoke. Smoke results: `/health=ok`, anonymous `POST /deployment-gates/authorize` returned `401`, authenticated KAN-83 release-bound evidence packet generation returned `found=true`, authenticated `POST /deployment-gates/authorize` created authorization `dga_486236dbd5e34264bebf52ec61db5667` with `decision=advisory`, `approved=true`, `blocking=false`, `would_block=false`, and authenticated `GET /deployment-gates/authorizations` returned `total=1` for that authorization.
- KAN-84 implementation status: active local branch `feature/KAN-84-deployment-gate-history` adds Deployment Gates 0.1 Desktop history plus workflow-template migration. Desktop now has `DeploymentGateHistoryPanel` under `Governance > Releases`, Tauri exposes `cmd_server_list_deployment_gate_authorizations`, the Control Plane store keeps authorization history separate from human release approvals, generated release-governance templates call `POST /deployment-gates/authorize`, and `scripts/control-plane/validate_release_governance_gate.ps1` now creates persisted deployment authorization records instead of only calling the lower-level evaluator.
- KAN-84 local validation on 2026-06-13: focused frontend tests (`63` passed), full frontend tests (`354` passed), `npm --prefix gitgov run typecheck`, `npm --prefix gitgov run lint`, `npm --prefix gitgov run build` (passed with the existing Vite large chunk warning), `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`, `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`, full Tauri `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed), PowerShell parse checks for the updated validator/generator, generated approval-required workflow pack verification for `/deployment-gates/authorize`, invalid-gate JSON shape validation, Vite/browser smoke for `/governance/releases` (`200`, no console errors, unauthenticated web runtime correctly showed Desktop-required gate), `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed.
- KAN-84 production validation on 2026-06-13: PR `#305` merged to `main` as `ad02a35`; post-merge `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed; Render deploy `dep-d8mfnd19rddc7398b640` for `ad02a35` reached `live`; production `/health` returned `ok`; anonymous `GET /deployment-gates/authorizations?org_name=yohandry10&limit=1` returned `401`; authenticated history returned `total=1`, `itemCount=1`, first authorization `dga_486236dbd5e34264bebf52ec61db5667`, `decision=advisory`, and `evaluation` present.
- KAN-85 implementation status: PR `#307` merged to `main` as `047f213` on 2026-06-14. It adds provider-specific Deployment Gate examples for GitHub Actions, Jenkins Pipeline, and GitLab CI under `docs/examples/deployment-gates/`, plus `scripts/control-plane/validate_deployment_gate_provider_examples.ps1`. The examples call `POST /deployment-gates/authorize`, preserve authorization evidence artifacts, read `GITGOV_API_KEY` from provider secret stores, do not call the lower-level evaluator, and do not mutate provider configuration.
- KAN-85 local validation on 2026-06-14: `.\scripts\control-plane\validate_deployment_gate_provider_examples.ps1 -OutputPath out\kan-85-provider-examples-validation.json` passed, verifying required request fields, `/deployment-gates/authorize`, no `/enterprise/release-governance/evaluate` in examples, artifact preservation, `blocking`/`would_block` handling, and no hardcoded bearer/API key pattern; PowerShell parser check for the validator passed; `git diff --check` passed; `.\scripts\security\publication_guard.ps1` passed. PR and post-merge GitHub checks passed, including `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render did not deploy `047f213` because KAN-85 is docs/examples-only outside the backend deploy root; the live backend remains KAN-84 deploy `dep-d8mfnd19rddc7398b640`.
- KAN-86 implementation status: GitHub issue `#309` closed by PR `#310`, merged to `main` as `b280570` on 2026-06-14. It adds the Desktop/admin Environment Policy Matrix for release governance, factors `ReleaseGovernanceEnvironmentPolicyPanel` out of `EnterpriseAdoptionPanel`, adds tested helpers for effective environment rows and override edits, preserves environment overrides when the base release governance mode changes, and documents the UX at `docs/design/environment-policy-ux-mvp.md` plus `docs/reports/environment-policy-ux-2026-06-14.md`. Product decision: production can be stricter than staging through explicit overrides, but `record-only` remains the default and blocking remains customer-selected only.
- KAN-86 validation on 2026-06-14: focused tests `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts src/test/components/ReleaseGovernanceEnvironmentPolicyPanel.test.tsx` passed (`37` tests); `npm --prefix gitgov run typecheck` passed; `npm --prefix gitgov run lint` passed; full frontend `npm --prefix gitgov test -- --run` passed (`360` tests); `npm --prefix gitgov run build` passed with the existing Vite large chunk warning; `git diff --check` passed; and `.\scripts\security\publication_guard.ps1` passed. PR checks passed before merge, and post-merge `main` checks passed for commit `b280570`: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- KAN-87 implementation status: GitHub issue `#312` closed by PR `#313`, merged to `main` as `b415391` on 2026-06-14. It adds audited break-glass authorization for Deployment Gates. `POST /deployment-gates/authorize` now accepts optional `break_glass` only when the evaluated release policy is truly blocking; valid exceptions persist `decision=break_glass`, `approved=true`, original `blocking=true`/`would_block=true`, original blockers, reason, authorizer, optional expiry, request payload, policy checksum, and admin audit metadata. Migration: `gitgov/gitgov-server/supabase/supabase_schema_v36.sql`; postcheck: `gitgov/gitgov-server/supabase/checks/v36_postcheck.sql`. Desktop `Governance > Releases` now shows `break-glass used`, reason, authorizer, expiry, and original blockers.
- KAN-87 validation on 2026-06-14: `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, focused deployment gate tests with `TEST_DATABASE_URL` (`8` tests), full backend tests with `TEST_DATABASE_URL` (`262` tests), `cargo check/clippy/test/fmt --check` for `gitgov/src-tauri` (`49` tests), `npm --prefix gitgov run typecheck`, focused frontend tests (`68` tests), full frontend tests (`361` tests), `npm --prefix gitgov run lint`, `npm --prefix gitgov run build` with the existing Vite large chunk warning, local/production v36 migration and SQL column/constraint verification through ignored `DATABASE_URL`, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. PR checks and post-merge `main` checks passed for `b415391`: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Render deploy `dep-d8n2977aqgkc73bu6780` for `b415391` reached `live`; production smoke returned `/health=ok`, authenticated `/stats=200`, and `GET /deployment-gates/authorizations?org_name=yohandry10&limit=1` returned `total=1` with `break_glass_used` present.
- KAN-88 implementation status: GitHub issue `#315` closed by PR `#316`, merged to `main` as `bd44db1` on 2026-06-14. It hardens KAN-87 by adding pre-approved break-glass routing for Deployment Gates. New routes: `POST /deployment-gates/break-glass-approvals` and `GET /deployment-gates/break-glass-approvals`. New table: `deployment_gate_break_glass_approvals`. `deployment_gate_authorizations` now stores `break_glass_approval_id` and `break_glass_approval_hash`. `POST /deployment-gates/authorize` only accepts `break_glass` when the evaluated policy is blocking and a valid unexpired approval matches the same release id, repository, branch, target SHA, environment, ticket id when supplied, and evidence packet hash. Desktop `Governance > Releases` now shows `pre-approved`, approval id, and approval hash for break-glass records.
- KAN-88 migration: `gitgov/gitgov-server/supabase/supabase_schema_v37.sql`; postcheck: `gitgov/gitgov-server/supabase/checks/v37_postcheck.sql`.
- KAN-88 validation on 2026-06-14: `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, focused deployment gate tests against temporary local Postgres on `127.0.0.1:55433` (`11` tests), full backend tests against the same temporary Postgres (`265` tests), temporary Postgres migration/postcheck applying `supabase_schema.sql`, `v28`, `v35`, `v36`, and `v37` with `v37_postcheck` all `PASS`, `cargo fmt/check/clippy/test` for `gitgov/src-tauri` (`49` tests), `npm --prefix gitgov run typecheck`, focused `DeploymentGateHistoryPanel` test, full frontend tests (`361` tests), `npm --prefix gitgov run lint`, `npm --prefix gitgov run build` with the existing Vite large chunk warning, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. The temporary Postgres container `gitgov-kan88-pg` was removed after validation. PR checks and post-merge `main` checks passed for `bd44db1`: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`. Production `v37` migration/postcheck passed. Render deploy `dep-d8n324u8bjmc73en5qgg` for `bd44db1` reached `live`; production smoke returned `/health=ok`, authenticated `/stats=200`, anonymous `GET /deployment-gates/break-glass-approvals?org_name=yohandry10&limit=1` returned `401`, authenticated `POST /deployment-gates/break-glass-approvals` created `dgbga_8be2e0b2a33741368ab211e7d4b5e77f` from existing release evidence, and authenticated `GET` by approval id returned `total=1`.
- KAN-89 implementation status: GitHub issue `#318` closed by PR `#319`, merged to `main` as `cbe5f95` on 2026-06-14. It updates `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md` so KAN-88 is listed as an implemented Deployment Gates primitive, removes stale wording that treated break-glass approval routing as future work, clarifies the remaining Deployment Gates backlog as provider installation/coverage and advanced routing workflows, and marks `0.2 Agentic Governance Layer` as the next major roadmap block. PR and post-merge checks passed: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- KAN historical planning records were migrated to GitHub Issues on 2026-06-12:
  - `KAN-4` through `KAN-77` were created as closed historical GitHub issues `#217` through `#290`.
  - Labels created/used: `migrated-from-jira`, `historical-record`, `gitgov-recovered`, and `reconstructed-from-github`.
  - GitGov production had `73` Jira `project_tickets` snapshots (`KAN-4` through `KAN-76`) plus GitHub PR evidence; `KAN-77` was reconstructed from GitHub/GitGov evidence without a Jira snapshot.
  - Migration audit artifacts are generated under ignored `out/jira-migration-audit/` (`summary.json`, `migration-inventory.json`, `gitgov-project-tickets.json`, `github-prs.json`, and `github-created-issues.json`).
  - Active follow-up issue: GitHub issue `#291`, title `KAN-78: Record Jira to GitHub Issues migration`.
- Expected branch before new work: `main`; latest validated main commit is `e4bec3f fix(KAN-77): align Render Docker context for policy core (#215)`.
- KAN-77 implementation PR `#214` merged as `0acfd26 security(KAN-77): harden event capture and policy as code (#214)`.
- KAN-77 Render hotfix PR `#215` merged as `e4bec3f fix(KAN-77): align Render Docker context for policy core (#215)`.
- Render deploy `dep-d8lsqf7avr4c73fsemlg` for `0acfd26` failed because the previous Render root/context `gitgov/gitgov-server` excluded the new sibling `gitgov/policy-core` crate, so Cargo could not read `/policy-core/Cargo.toml`.
- Render service `gitgov-api` was updated through the Render API to `rootDir=gitgov`, Docker context `.`, and Dockerfile `gitgov-server/Dockerfile`; local `docker-compose.yml` now uses the same context shape.
- Render deploy `dep-d8lsul8k1i2s73dk1ph0` for `e4bec3f` reached `live` on 2026-06-12, `/health` returned `status=ok`, and authenticated `/stats` returned HTTP `200`.
- Production `supabase_schema_v31.sql` was applied after PR `#216` exposed a `/policy/yohandry10%2FGit-Gov` database error in `Validate quality_gates warn/block matrix`; `v31` now drops/recreates `get_policy_history(UUID, INTEGER)` so the migration is re-runnable when the OUT row changes.
- After applying `v31`, `source_metadata` exists on `policies`, `policy_history`, and `policy_change_requests`; `get_policy_history` exists once; authenticated production `GET /policy/yohandry10%2FGit-Gov` returned HTTP `200`; local rerun of `scripts/jenkins/validate_quality_gate_policy_matrix.ps1` against production passed.
- Post-merge GitHub checks for `e4bec3f` passed, including `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix (Optional)`, `Governance Correlation Smoke (Optional)`, `Desktop Updater Readiness (Optional)`, and `SonarQube Governance (Non-Blocking)`.
- Latest KAN-72 audit baseline: PR `#193` merged as `655478e`, handoff refresh PR `#194` merged as `2ab821e`, and stable wording PR `#195` merged as `0ccef26`.
- Latest completed KAN-24 implementation baseline: `126167f security(KAN-24): product vulnerability review and hardening (#97)`.
- KAN-24 implementation PR: `#97` - `security(KAN-24): product vulnerability review and production hardening`.
- KAN-24 post-merge context refresh PR: `#98` - `docs(KAN-24): record post-merge validation`.
- Recent prior PR: `#96` - `docs(KAN-23): record evidence packet merge validation`.
- Treat commit/PR fields in this file as validated KAN-24 implementation and validation baselines, not an auto-updating source of truth for later docs-only refresh commits; always run `git status --short --branch` and `git log -1 --oneline main` before new work.
- Worktree expectation before new work: clean and aligned with `origin/main`.
- Implementation-status backlog is closed. Remaining items are operational decisions, optional future enhancements, or evidence hygiene.
- Completed ticket chain: `KAN-25` through `KAN-68` (vulnerability review automation, enterprise self-service adoption, release governance, onboarding readiness/remediation, and route auth smoke chains), `KAN-69 - Enterprise Action Center guided UX` (product/UX), and `KAN-70` through `KAN-76` (documentation reality audits and public agent context). Per-ticket titles are in `Recent Ticket Chain` below; per-ticket implementation/validation notes live in `docs/reports/current-context-kan-notes-archive-2026-06-09.md`.
- `KAN-70`, `KAN-71`, `KAN-72`, `KAN-73`, `KAN-74`, and `KAN-75` were documentation/CI hygiene follow-ups. They audited living documentation against actual repository state before returning to `KAN-69`.
- `KAN-75` scope: public web docs, roadmap/context/product-state docs, and systematic cleanup of stale public claims that were not covered by the backend/API, Desktop/dashboard, or workflows/scripts/ops audit phases.
- `KAN-76` scope: publish a sanitized public agent-readable context bridge so external models can understand current product state without force-adding restricted forensic/strategy docs.
- KAN-69 implementation PR: `#204 - product(KAN-69): add guided Action Center workspace`.
- KAN-69 implementation commit on main: `aa7e352 product(KAN-69): add guided action center workspace (#204)`.
- KAN-69 implementation shape: dedicated `/action-center` desktop route, sidebar navigation entry, deterministic `Goal + Evidence + Permission` recommendations, and deep links into existing Control Plane/Workspace surfaces. It is not another panel inside Workspace or Enterprise Adoption.
- KAN-69 verification follow-up PR: `#206 - fix(KAN-69): harden Action Center verification logic`.
- KAN-69 verification follow-up commit on main: `8a55a6d fix(KAN-69): harden action center verification logic (#206)`.
- KAN-69 follow-up verification: `docs/reports/enterprise-action-center-verification-2026-06-07.md` records the product/infrastructure Q/A review. The follow-up fixed release prep so missing or empty Jira coverage remains conservative before Evidence Packet/release decision guidance, and it avoids known-forbidden admin-only adoption-profile/checklist reads for non-admin users.
- KAN-69 Desktop runtime QA is completed and merged to `main` through PR `#209` (`fix/KAN-69-desktop-runtime-qa-maintainability`) and PR `#211` (`fix/KAN-69-control-plane-workspace-auth`); latest main commit `e0c769d`. Report: `docs/reports/kan-69-desktop-runtime-qa-2026-06-07.md`.
- The merged Desktop QA code changes were limited to Action Center mount behavior, Desktop auth/session UX, Workspace pipeline visualizer layout/copy, Control Plane technical connection/configuration UX, Governance information architecture, Control Plane Enterprise Adoption layout, and focused auth/navigation/product-copy tests.
- Desktop QA implementation approach (executed): stabilize startup/auth, preserve Workspace local execution flow, keep heavy evidence refresh explicit, reduce Action Center route mount pressure, move Control Plane connection/configuration into Settings, keep `/control-plane` only as a redirect to `/settings#control-plane`, move operational governance to `/governance/*`, keep `Governance > Evidence` first with no generic Governance Dashboard tab, keep Action Center as the only global `Next Action` owner, and validate without relaunching Desktop unless the user permits runtime interaction.
- Current Control Plane/auth decision: GitHub identifies the Desktop operator; the GitGov API key authorizes Control Plane role/org/evidence. Restore valid local GitHub sessions by default, preserve saved Control Plane config, explain the split in UI, and reserve forced Device Flow on every launch for explicit hardening mode.
- Current Control Plane workspace/auth implementation: Desktop now treats GitHub identity, GitGov API key authorization, and active workspace/tenant as separate product concepts. Scoped API keys get `org_name` from `/me`; global/founder Admin keys with `org_id=null` must validate an active workspace via `/orgs/{login}` before admin tenant surfaces unlock. The active workspace is persisted locally per GitHub login, Control Plane URL, and a non-secret API-key fingerprint. `/orgs` now lists visible workspaces, `/orgs/{login}` validates scope, and `/api-keys?org_name=...` is scoped while unqualified `/api-keys` remains the explicit global Admin catalog.
- Runtime QA finding: Supabase and local backend health were validated as healthy; the observed Action Center freeze is more likely Desktop/Tauri/WebView/client mount pressure than database or backend failure. Opening Action Center must not trigger heavy background refresh automatically; manual Refresh remains the explicit path for heavier evidence refresh.
- Runtime QA product decision: Desktop should reuse a valid local GitHub session by default. GitHub identifies the human operator; the GitGov API key authorizes Control Plane role/org/evidence. The two-step model is acceptable only when it is explained and persisted; it should not force GitHub Device Flow on every app start unless an explicit hardening env flag enables that behavior.
- Runtime QA Control Plane URL finding: GitHub Device Flow can succeed while Step 2 fails if Desktop is forced to `http://127.0.0.1:3000` and no local Control Plane is listening. The fix direction is centralized URL resolution, editable Control Plane URL fields, localhost as fallback only, IPv4/localhost/IPv6 loopback detection for local-target hints, and actionable connection errors instead of raw `Network error ... /health` output.
- Runtime QA Git identity finding: classify as product concept plus data/state. GitHub auth identifies the Desktop operator, while effective `git config user.name/user.email` controls Git CLI/manual commit authorship. The Workspace warning should say "effective Git identity incomplete/not provably aligned", not "cuenta GitGov"; `Ver prueba` writes read-only `git config --get` evidence to the GitGov CLI panel; no automatic `git config` mutation is allowed. The warning should recommend explicit `git config --local user.name/user.email`, not the broader `scripts/setup-dev.ps1` helper, because that script also configures repo hooks. Identity alignment is exact/provable only: login or public name must exactly match `user.name`, or `user.email` must match the public GitHub email or GitHub noreply pattern, including numbered noreply addresses. Commit/Push remain blocked by policy until the effective Git identity is complete and verifiable against the authenticated GitHub user.
- Runtime QA CLI finding: classify as data/state plus performance. Diagnostic CLI proof lines must be visible in the terminal but not treated as executed/audited commands; `emitCliLine` now supports `auditable: false`, and Audit Trail/Pipeline ignore those lines. `cmd_execute_cli` now drains stdout/stderr concurrently to avoid pipe backpressure deadlocks, preserves `command_id` on completion audit metadata, and parses quoted arguments instead of whitespace-splitting safe-mode commands. Native PTY startup kills spawned shells if initialization fails before registration. Legacy shell-session now rejects overlapping structured commands, captures stderr previews, and has a safer PowerShell exit wrapper. Native PTY manual input remains raw terminal I/O, not command-by-command structured audit; future audited-terminal work should use explicit command submission or shell integration, not naive keystroke parsing.
- Runtime QA UI rule: before removing UI or behavior, classify the problem as concept, layout, data/state, performance, or security. If useful content clips or overflows, fix layout/scroll/wrap instead of deleting it. The current Workspace `Gates / Blockers` removal of global `Next Action` is a product/concept decision, not a visual workaround: Action Center is the only global `Next Action` owner, Workspace uses `Next local step`, and Adoption uses `Next onboarding task`.
- Runtime QA Control Plane layout finding: classify as layout/visual. Enterprise Adoption must not keep a long guided checklist and evidence detail rail inside one narrow right column while the left form column ends early. The fix is responsive composition: top configuration/readiness, then full-width checklist/evidence sections, with no useful onboarding or `Next` action content removed.
- Runtime QA Control Plane information architecture finding: classify as product concept plus layout/visual. Control Plane is configuration, not a primary product module. The sidebar no longer shows Control Plane; Settings owns endpoint, API key, role, org scope, and transport state; `/control-plane` remains only as a compatibility redirect to `/settings#control-plane`. Operational governance moved to sidebar route `/governance`, where `/governance` defaults to `Evidence` and there is no `/governance/dashboard` tab. Governance sections are `/governance/evidence`, `/governance/policy`, `/governance/adoption`, `/governance/releases`, and `/governance/copilot`. Former overview contents were redistributed: traceability, pipeline health, GitHub signals, and evidence gaps live in Evidence; release readiness lives in Releases; generic snapshot counters such as active repos/devs/tracked pushes were removed from the primary IA. `DailyActivityWidget` is not mounted because daily commits/pushes are diagnostic telemetry, not a primary product decision surface. `ActionCenterPage` and `GovernancePage` are route-level lazy chunks to reduce main router load.
- Runtime QA Settings layout finding: classify as layout/visual plus product concept. Settings now uses Governance-style tabs instead of a long centered column. Tabs are ordered `Preferences`, `Organization`, `Account`, `Repository`, and `System`. `System` merges the former `Connection` and `Updates` surfaces: Control Plane endpoint/API key/role/scope/transport plus Desktop updater. Account sits next to Organization in the tab order, and System sits last after Repository. No settings capability was removed; `/settings#control-plane` and legacy `/settings#updates` both land on the System tab.
- Runtime QA Organization tab layout finding: classify as layout/visual. The first Settings tab layout pass incorrectly used a two-column parent grid for Organization even though its left admin/API-key stack is much taller than the governance-rules card. That recreated the same empty-right-column defect. Organization now uses a full-width vertical flow; Repository remains the only Settings tab that uses the two-column parent grid when config preview is present.
- Runtime QA Help layout finding: classify as layout/visual plus product concept. Help/FAQ had the same centered-document problem as the old Settings view, and its links still pointed at the old `git-gov.vercel.app` URL. The fix keeps all FAQ sections, removes the narrow `max-w-2xl mx-auto` composition, uses `https://gitgov.cloud` links, adds full-width operational header cards, category navigation, and a responsive FAQ grid (`xl` side rail, `2xl` six-column content grid: first two sections half-width, remaining three sections one-third width) so wide Desktop windows do not leave a final orphan card or dead side space.
- Runtime QA language finding: classify as product concept plus data/state plus text/UI. The language selector persisted and switched i18next correctly, but Settings, Governance, and primary sidebar chrome still used hardcoded labels, so Spanish appeared partial. Local fix expanded `gitgov/src/lib/i18n.ts` and moved first-class Settings/Governance/sidebar labels, descriptions, and status copy onto translation keys. Nested feature panels still need targeted i18n coverage before claiming every deep module string is localized.
- Runtime QA security/business-logic follow-up: classify as security plus data/state plus product concept. Desktop no longer treats `/stats` success as a fallback Admin identity when `/me` fails; role context must come from `/me`. Control Plane API key persistence no longer fails silently: keyring errors keep the session disconnected/degraded with an explicit error. Control Plane URL validation now rejects invalid schemes, embedded credentials, and non-loopback `http://` before persistence. CLI command auditing now redacts URL credentials, bearer/API/token/password/secret values, common GitHub/GitLab/OpenAI-style token prefixes, and stdout/stderr previews before outbox or direct Control Plane ingestion.
- Runtime QA Settings/Governance policy follow-up: classify as product concept. Organization Settings no longer mounts a second governance policy editor; it keeps organization onboarding/team/API-key administration and links to `Governance > Policy` as the single policy owner. Organization admin UI is gated by Control Plane `Admin` role, not local GitHub admin state. `GovernanceRulesPanel` dirty-state tracking now includes forbidden patterns so those changes can be saved when the panel is used.
- Runtime QA Release/Governance performance follow-up: classify as security plus performance plus product concept. Release approvals no longer default to the real `yohandry10/Git-Gov` repository or `KAN-43` when no profile/evidence context exists, and evidence URIs now allow relative API paths or `https://` only. Governance route entry no longer performs the previous heavy role refresh that pulled daily activity plus 500 logs; it loads base stats and defers the smaller log window to `Governance > Evidence`.
- Runtime QA performance follow-up: classify as performance plus data/state. Default governance/event refresh windows are capped at `120` logs instead of `500`, `dailyActivity` is no longer loaded by general dashboard refresh because it has no mounted product consumer, SSE refreshes are batched for `1000 ms`, incremental log refreshes are serialized to avoid overlapping store/API work, and the Workspace pipeline visualizer deduplicates concurrent graph/signal refreshes while limiting Control Plane signal pulls to `50` records per source. Manual explicit heavy refresh still keeps the heavy evidence path available when needed. Validation passed with frontend typecheck, lint, focused store/settings/config tests, full frontend tests (`333` tests), build, and `git diff --check`.
- Runtime QA header refresh follow-up: classify as product concept plus data/state. The global Workspace `Actualizar` button mixed local repository refresh with an interactive Control Plane `checkConnection()` call, so a transient `/me` role/context revalidation could replace the Workspace with the Control Plane access screen before returning. The global button was removed. Header connection checks remain background-only; repo status refresh is handled by route polling and explicit local actions, while manual Control Plane reconnect belongs in Settings/System.
- Local maintenance cleanup: `gitgov/gitgov-server/target_forensic` was inventoried in `docs/reports/target-forensic-cleanup-2026-06-08.md` and removed as a local Rust/Cargo forensic/debug build artifact directory. It contained generated `.rlib`, `.pdb`, `.o`, `.exe`, `.dll`, and incremental cache files, not source, docs, migrations, tests, or runtime configuration.
- Runtime QA validation after Control Plane/Governance/Settings/Help/i18n restructure: after moving Control Plane into Settings, removing the Governance Dashboard tab, deleting the unmounted dashboard-only components, organizing Settings into tabs, making Settings/Governance/sidebar chrome language-reactive, widening Help/FAQ, correcting Organization to full-width flow, merging Connection/Updates into the final System tab, and moving Help links to `gitgov.cloud`, `npm --prefix gitgov run typecheck`, `npm --prefix gitgov run lint`, focused Settings/Governance/i18n/Help layout tests (`17` tests), full frontend tests (`332` tests in `32` files), `npm --prefix gitgov run build`, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. Build still reports the existing Vite `>500 kB` base chunk warning; Action Center and Governance emit separate chunks. Manual Desktop smoke remains pending by design because the active Tauri/Desktop session must not be restarted or relaunched without explicit user instruction.
- Runtime QA documentation/web refresh finding: README, architecture, quickstart, troubleshooting, deployment, public agent context, implementation status, Action Center design docs, GitHub evidence runbook, and public web docs were aligned with the new IA. Public web copy now uses canonical `https://gitgov.cloud`, describes Governance/Action Center instead of the old Admin Dashboard/Control Plane dashboard, and preserves page/component styling while changing only informational copy/URLs.
- Runtime QA operating rule: do not restart, kill, or relaunch the Tauri/Desktop app while the user is manually logged in or validating unless the user explicitly asks for that runtime action.
- Runtime QA maintainability rule: hand-maintained source files should not become giant mixed-responsibility modules. Single Responsibility Principle comes first: split files that mix UI, fetch/state, business rules, data transforms, templates, and types even before they become huge. Practical targets are 300-600 lines for most source files, 800 lines as the normal upper bound, and 1,200 lines only as an exceptional justified ceiling. UI components/pages should usually stay around 150-350 lines and split before 500-600 lines; domain helpers should usually stay around 200-500 lines and below 800; tests can be larger when they cover one coherent module but should normally stay below 800-1,000; type/interface files may grow only while they remain one clear domain contract. When reducing an existing large file, split by responsibility and keep a compatibility facade if existing imports depend on the old path. Generated outputs, lockfiles, vendored artifacts, fixtures, and historical reports are exempt.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/integration_tests.rs` giant backend integration-test file was split into a small facade plus focused modules under `gitgov/gitgov-server/src/integration_tests/` by responsibility: shared helpers, auth, events/admin, policy enforcement, coverage/compliance, and alerts/exports/policy requests. No endpoint behavior was intentionally changed.
- Runtime QA maintainability refactor: the former `gitgov/src/components/control_plane/dashboard-helpers.ts` giant Control Plane helper file was split into a compatibility facade plus focused helper modules under `gitgov/src/components/control_plane/dashboard-helpers/`. Existing imports from `dashboard-helpers.ts` remain valid; the split is organizational only and keeps adoption/profile/workflow/policy/release helper behavior intact.
- Runtime QA maintainability refactor: the former `gitgov/src-tauri/src/control_plane/server.rs` giant Desktop Control Plane client/DTO file was split into a compatibility facade plus `server/models/*` domain DTO modules and `server/client/*` endpoint-group modules. Public import path remains `crate::control_plane::server::*`/`crate::control_plane::*`; no endpoint URL, payload shape, or store/backend behavior was intentionally changed. Validation passed with `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml`, `cargo test --manifest-path gitgov/src-tauri/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`31` tests), `git diff --check` for the split files, no-BOM check, and a public surface comparison confirming `82` public DTO/error types and `46` public client methods before and after.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/db.rs` backend database layer was split by SRP into modules under `gitgov/gitgov-server/src/db/` while preserving the old import path through the `db.rs` module facade. Per the staged migration plan, the original full file is still retained inside `db.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, public surface comparison (`17` public types and `144` public methods unchanged), and full function-name coverage (`180/180`, no missing function names).
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/models.rs` backend model contract file was split by domain into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/models/`. Per the same staged migration plan used for `db.rs`, the original full file is still retained inside `models.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, module/file parity (`32` modules and `32` files), archive integrity (`0` uncommented archive lines), and public surface comparison (`167` public types, `4` public functions, and `1` public const unchanged). The largest live model module is now `tests.rs` at `378` lines; the large `models.rs` facade size is temporary archive evidence only.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/chat_handler.rs` backend chat orchestration file was split into a live include facade plus focused modules under `gitgov/gitgov-server/src/handlers/chat_handler/` for helpers, short-circuit intents, query families, and the public `chat_ask` route handler. Per the staged migration plan, the original full file is still retained inside `chat_handler.rs` as a line-commented archive, partitioned with `PART` markers that map to the new module files; it is intentionally non-compiled and must not be treated as duplicate live code. Do not delete that commented archive until the migration-safe review phase is explicitly closed. Live module validation passed with explicit `rustfmt --edition 2021` over the included module files, `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml`, `git diff --check`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, publication guard, archive exact-match comparison against the original `HEAD` file (`3795` archived lines), archive integrity (`13` `PART` markers, `0` uncommented archive lines), `ChatQuery` dispatch coverage (`31` variants, `0` missing), and public surface comparison confirming only `pub async fn chat_ask` remains public in the split module.
- Runtime QA maintainability refactor: the former `gitgov/src/store/useControlPlaneStore.ts` giant Desktop Control Plane Zustand store was split into a compatibility facade plus focused modules under `gitgov/src/store/useControlPlaneStore/`. The root file still exports the same public path and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` uncommented archive lines. Live modules now separate constants, types, helpers, runtime in-flight guards, initial state, and action slices for connection/auth, dashboard/evidence, enterprise/adoption/releases/export, organization/team/API keys, chat/copilot, and policy/SSE. The largest live module after the second-level split is `types.ts` at `695` lines; action modules are `150-530` lines and the live `store.ts` composer is `19` lines. Validation passed with `npm run typecheck`, `npm run lint`, focused store/config/settings tests (`36` tests), full frontend tests (`333` tests), `npm run build`, `git diff --check`, no-BOM check, archive integrity check, and `.\scripts\security\publication_guard.ps1`. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/main.rs` backend crate-root/server bootstrap file was split into a small crate-root facade plus focused modules under `gitgov/gitgov-server/src/server/`. The root `main.rs` still declares crate modules, calls `server::run().await`, and retains the original full file as a line-commented migration archive with `8` `PART` markers and `0` uncommented archive lines. The archive reconstructs the `HEAD` original exactly (`2188` lines). Live modules now separate env/CLI config, rate limiting, HTTP middleware, distributed SSE listener, job worker, route composition, startup/runtime orchestration, and the moved rate-limit tests. The largest live module is `startup.rs` at `748` lines, followed by `rate_limit.rs` at `450` and `routes.rs` at `411`; `main.rs` size is temporary archive evidence only. Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), route-path parity check against `HEAD`, no-BOM check, `git diff --check`, and exact archive comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/client_ingest_dashboard.rs` backend handler bundle was split into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/handlers/client_ingest_dashboard/`. The root file still exports the same handler names through `include!` and retains the original full source as a line-commented migration archive with `5` `PART` markers and `0` live function declarations. The archive reconstructs the `HEAD` original exactly (`1886` lines). Live modules now separate client event ingest, outbox lease telemetry/acquisition, stats/log/repo lookup caches, dashboard/log/team query handlers, and policy-check helpers/endpoint. Current live module sizes are `policy_check.rs` (`491` lines), `cache.rs` (`460`), `dashboard_queries.rs` (`406`), `ingest.rs` (`338`), and `outbox_lease.rs` (`191`). Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, `git diff --check`, and exact archive comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/gitgov-server/src/handlers/github_webhook.rs` backend GitHub webhook handler bundle was split into a compatibility facade plus focused modules under `gitgov/gitgov-server/src/handlers/github_webhook/`. The root file still exports `handle_github_webhook` and all existing private helpers/tests through `include!`, and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` live declarations. The archive matches the concatenated live modules exactly (`1749` lines). Live modules now separate webhook entry/signature validation, push/create/review processing, generic check/status repository evidence, PR review-comment/issue-comment correlation helpers, PR merge/approval processing plus repo upsert, and existing webhook unit tests. Current live module sizes are `pr_comments.rs` (`452` lines), `pr_events.rs` (`368`), `repo_evidence.rs` (`303`), `push_create_review.rs` (`264`), `entry.rs` (`191`), and `tests.rs` (`171`). Validation passed with `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`, `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`, `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`, full backend `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml` (`193` tests), no-BOM check, `git diff --check`, and live/archive concatenation comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- Runtime QA maintainability refactor: the former `gitgov/src-tauri/src/commands/cli_commands.rs` Desktop CLI command bundle was split into a compatibility facade plus focused modules under `gitgov/src-tauri/src/commands/cli_commands/`. The root file still exports the same `commands::cli_commands::*` surface and retains the original full source as a line-commented migration archive with `6` `PART` markers and `0` live declarations. The archive matches the concatenated live modules exactly (`1785` lines), preserving local Desktop runtime QA changes. Live modules now separate CLI types/managers, parsing/env/redaction/audit helpers, shell-session commands, native-terminal commands, structured command execution, and whitelist/pipeline graph/tests. Current live module sizes are `helpers.rs` (`529` lines), `shell_session.rs` (`384`), `execute.rs` (`285`), `types.rs` (`201`), `native_terminal.rs` (`199`), and `pipeline.rs` (`187`). Validation passed with `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`, `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`, `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`, `cargo test --manifest-path gitgov/src-tauri/Cargo.toml --no-run`, full Tauri `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`31` tests), no-BOM check, `git diff --check`, and live/archive concatenation comparison. Do not delete the commented archive until the migration-safe review phase is explicitly closed.
- KAN-69 local validation: `npm --prefix gitgov run typecheck`, focused Action Center helper tests (`8` tests), full frontend tests (`304` tests in `26` files), `npm --prefix gitgov run lint`, `npm --prefix gitgov run build`, `git diff --check`, and `.\scripts\security\publication_guard.ps1` passed. Browser/Vite smoke for `/action-center` returned HTTP `200`; full authenticated UI validation remains a Tauri/Desktop runtime concern.
- KAN-69 post-merge checks on `main` commit `aa7e352` passed: `CI` run `27086413044`, `Release Readiness Gate` run `27086413043`, `Secret Scan` run `27086413053`, `Public Naming Guard` run `27086413041`, `SonarQube Governance (Non-Blocking)` run `27086413042`, `Quality Gate Policy Matrix (Optional)` run `27086413040`, `Governance Correlation Smoke (Optional)` run `27086413050`, and `Desktop Updater Readiness (Optional)` run `27086413038`.
- KAN-69 verification follow-up post-merge checks on `main` commit `8a55a6d` passed: `CI` run `27100640858`, `Release Readiness Gate` run `27100640831`, `Secret Scan` run `27100640840`, `Public Naming Guard` run `27100640856`, `SonarQube Governance (Non-Blocking)` run `27100640837`, `Quality Gate Policy Matrix (Optional)` run `27100640835`, `Governance Correlation Smoke (Optional)` run `27100640862`, and `Desktop Updater Readiness (Optional)` run `27100640864`.
- Any future branch, commit, and PR title must keep the `KAN-*` traceability ID. New planning records should be opened in GitHub Issues unless Jira Cloud is deliberately reactivated later.

## Local Security Review (uncommitted, 2026-06-09)

A multi-surface security review was run against the backend and several findings were
fixed locally in the working tree (not yet committed; no Jira ID was attached because the
Jira account was temporarily disabled). Validated against a local Postgres (`5434`, since a
native Windows Postgres collides on `5433`) with the full suite green and `clippy -D warnings`
clean. Fixed: traceability coverage now counts only Jira-verified tickets (not pattern-only
matches); multi-tenant org scoping closed on the `integrations.rs` read/evidence/correlation
endpoints, the Jira/Jenkins status aggregates, `append_project_ticket_relations*`, and the
`commit_ticket_correlations` uniqueness (new migration `supabase_schema_v26.sql`); SSE is now
org-scoped per subscriber, admin-gated, and invalidates caches per-org; the governance copilot
was hardened so the LLM cannot forge deterministic-provenance refs and the system prompt has an
explicit prompt-injection guardrail.

### Finding E1 — client-controlled event timestamp (Medium)

- **Issue**: `POST /events` stored `created_at` directly from the client-supplied
  `input.timestamp` (`handlers/client_ingest_dashboard/ingest.rs`) with no bound. A
  server-authoritative `synced_at` exists on `client_events` but no governance query uses it —
  coverage, release readiness, daily activity, `?hours=N` windows and log ordering all filter by
  the client `created_at`. An authenticated client could postdate/backdate events to move them
  in or out of reporting windows or corrupt audit ordering.
- **Fix applied**: ingest now rejects events whose timestamp is more than `5` minutes in the
  future (`event_timestamp_too_far_in_future`, `EVENT_FUTURE_SKEW_MS`). Past timestamps remain
  allowed on purpose — the offline outbox legitimately backfills older events. Tests:
  `client_event_timestamp_in_future_is_rejected` (unit) and
  `events_with_future_timestamp_are_rejected` (integration).
- **Residual / recommended next step**: future-rejection closes postdating but not backdating
  within the past (bounding the past at ingest would break the offline outbox). The complete
  anti-evasion fix is to anchor security-sensitive time windows on the server `synced_at`
  (or `GREATEST(created_at, synced_at)`) instead of the client `created_at`. Tracked as
  follow-up, not yet implemented.
- **Org-invitation identity binding (FIXED)**: `accept` previously let the acceptor override the
  invited identity via `login` and mutate an existing `org_user`. Now the invited identity is
  authoritative: `OrgInvitation::resolved_accept_login()` resolves the login from `invite_login`
  (then the `invite_email` local-part) and never from acceptor input; `accept_org_invitation` no
  longer takes a `requested_login`; and the handler rejects a mismatched acceptor `login` with
  `400 "login does not match the invitation target"` (`accepts_requested_login`). Unit tests in
  `models/tests.rs` cover both helpers, including the spoofing case being rejected.

### Finding W1 — webhook replay via unsigned delivery_id (Low/Medium)

- **Issue**: GitHub/Jira HMAC signs only the request body; `X-GitHub-Delivery` /
  `X-GitHub-Event` are unsigned, sender-controlled headers. Idempotency keyed on
  `github_events.delivery_id` meant a captured, validly-signed payload could be replayed with a
  fresh `delivery_id` and re-injected as duplicate audit evidence (which feeds coverage,
  readiness, and PR-merge correlations). No replay/timestamp window existed. The raw signed
  payloads are also persisted in `webhook_events`.
- **Fix applied**: content-bound idempotency. New migration `supabase_schema_v30.sql` adds
  `webhook_events.payload_sha256` + a unique index. `handlers/github_webhook/entry.rs` now hashes
  the signed material (`SHA256(event_type ‖ raw_body)`) and `store_webhook_event` returns a
  `WebhookIngestDecision`. A content-hash collision is only skipped when the prior occurrence was
  already processed successfully (`processed = TRUE`); a prior delivery whose processing FAILED is
  returned for reprocessing, so a transient failure is not silently lost (retry-safety). A replay
  with a fresh `delivery_id` but the same already-processed signed body collides on the content
  hash and is skipped. Test:
  `webhook_replay_with_fresh_delivery_id_is_deduped_by_content_hash` (integration) covers both the
  retry-safe and the dedup paths. The harness `webhook_events` table was aligned (it was missing
  `signature` / `payload_sha256`).
  - Self-review note: the first cut of this fix skipped processing on ANY content-hash collision,
    which would have permanently dropped an event whose first delivery stored the row but then
    failed processing (GitHub retries would be answered `200 duplicate`). Corrected to the
    processed-aware decision above.

### Finding W2 — Jira webhook stale-replay overwrite (Low)

- **Issue**: `upsert_project_ticket` used `ON CONFLICT (org_id, ticket_id) DO UPDATE` with no
  version guard, so replaying an older Jira webhook overwrote a newer ticket state
  (last-write-wins).
- **Fix applied**: the `DO UPDATE` now carries
  `WHERE project_tickets.updated_at IS NULL OR EXCLUDED.updated_at IS NULL OR EXCLUDED.updated_at >= project_tickets.updated_at`,
  so a strictly-older replay is ignored. Validated directly against the production-shaped schema
  (the harness `project_tickets` is drifted — `project_key NOT NULL`, no `ticket_url`/`title` — so
  `upsert_project_ticket` cannot run against it; verified via `psql`: a stale replay returns
  `INSERT 0 0` and the newer status is preserved).

### Migration numbering note

Local webhook idempotency migration is `supabase_schema_v30.sql`. Migrations `v27`–`v29`
(api-key role integrity, release-approval evidence-packet binding, push-outcome event fidelity)
are separate concurrent local work; `v26` is the `commit_ticket_correlations` org-scoped
uniqueness from the multi-tenant fix.

### Multi-tenant join hardening — loose `org_id IS NULL` pattern (FIXED)

- **Issue**: several SQL joins across the correlation/coverage/noncompliance queries matched org
  with a loose predicate, e.g. `X.org_id = Y.org_id OR X.org_id IS NULL`. A second-pass review
  found 6 instances. None was a cross-tenant breach — the driving table is always strictly scoped
  (`WHERE <pk>.org_id = $N`), so org A never sees org B; the loose branch only let `org_id IS NULL`
  (unowned/legacy) rows bleed into a scoped result, and could over- or under-count.
- **Fix applied**: every instance was tightened to the strict form
  `(X.org_id IS NULL AND Y.org_id IS NULL) OR X.org_id = Y.org_id`. Locations: `db/jira_coverage.rs`
  orphan-ticket join and the `get_commit_pipeline_correlations` lateral; `db/noncompliance_detection.rs`
  in `detect_v2_commit_no_ticket_signals`, `detect_v2_stale_in_progress_signals` (×2), and
  `detect_v2_done_not_deployed_signals`. The remaining coverage/flow joins were already strict from
  concurrent local work.
- **Validation**: a repo-wide grep confirms zero loose column-to-column `org_id IS NULL OR` patterns
  remain. Because the noncompliance orchestrator swallows V2 SQL errors (`Err(e) => warn!`), a green
  suite alone does not prove SQL validity, so the most complex rewritten query was `EXPLAIN`-checked
  against the production-shaped schema and plans correctly. Build + `clippy -D warnings` + full suite
  (`230` tests) green. Post-migration/scoping/E1 work means `org_id IS NULL` data rows should no
  longer be produced, so the change is defense-in-depth with no expected effect on current data.

### KAN-77 Event capture fidelity (local implementation)

- **Scope implemented**: Desktop/Tauri branch and checkout capture now emits only backend-supported
  event types. `cmd_create_branch` no longer writes unsupported `attempt_create_branch` or
  `branch_failed`; failed branch creation is a `create_branch` event with `failed` status, blocked
  branch creation is `blocked_branch`, and successful checkout now emits `checkout_branch` with
  actor, repo/org, branch, HEAD SHA, and `from_branch`/`to_branch` metadata.
- **Remote parser**: `repo_event_context` no longer depends only on `origin`; it prefers the current
  branch upstream remote, then `origin`, then other configured remotes, while still accepting only
  parseable GitHub SSH/HTTPS remote URLs and rejecting ambiguous/non-GitHub remotes.
- **Backend guardrail**: `/events` now rejects evidence-bearing Desktop events that are incomplete:
  `stage_files`, branch/checkout, commit, and push/governance push events must carry
  `repo_full_name` and `branch`; commit/push evidence events must carry `commit_sha`; `stage_files`
  must include at least one file. Non-evidence telemetry such as heartbeat/login is not made
  artificially strict.
- **Desktop native terminal correction**: the Workspace terminal is a core product surface and
  must be operational by default in Desktop. A local hardening pass had accidentally changed
  `GITGOV_ENABLE_NATIVE_TERMINAL` from opt-out to opt-in, leaving the Workspace terminal offline
  unless the variable was set to `true`. Restored the product contract: native PTY is enabled by
  default, `GITGOV_ENABLE_NATIVE_TERMINAL=false` remains an explicit restricted-runtime opt-out,
  and `TerminalPanel` now treats that explicit opt-out as a degraded configuration state instead
  of a repeated red technical error.
- **Tests/validation run locally**: `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml`
  (`230` passed), `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`,
  `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`,
  `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`,
  `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`,
  `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`47` passed),
  `npm --prefix gitgov test -- --run` (`345` passed), `npm --prefix gitgov run typecheck`,
  `npm --prefix gitgov run lint -- --quiet`, `npm --prefix gitgov run build` (existing Vite
  chunk-size warning only), `git diff --check`, and manual Desktop validation confirming the
  Workspace terminal starts as `powershell` at the repo prompt.

## Documentation Intake - 2026-06-12

Session request: read as much repository documentation as practical and preserve the operating
context. Files reviewed included `AGENTS.md`, `README.md`, `CONTRIBUTING.md`,
`docs/CURRENT_CONTEXT.md`, `docs/AGENT_PUBLIC_CONTEXT.md`, `docs/IMPLEMENTATION_STATUS.md`,
`docs/ARCHITECTURE.md`, `docs/QUICKSTART.md`, `docs/DEPLOYMENT.md`,
`docs/TROUBLESHOOTING.md`, `docs/OPERATIONS_ACCESS.md`, `docs/PUBLICATION_POLICY.md`,
`docs/QUALITY_GATE_POLICY_VALIDATION.md`, current Action Center and roadmap design docs,
recent KAN-69/KAN-70/KAN-71/KAN-72/KAN-73/KAN-74/KAN-75/KAN-76 reports, integration-test
harness drift notes, enterprise adoption/GitHub evidence/release governance runbooks,
`gitgov/README.md`, `gitgov/gitgov-server/README.md`, `gitgov-web/README.md`, public web
content headings, and `gitgov-web/CONTENT_ARCHITECTURE_GUIDE.md`.

Key preserved context:

- This worktree already had substantial uncommitted `KAN-77` changes before the documentation
  intake. Treat them as existing local work and do not revert or overwrite them casually.
- Current branch observed during intake: `security/KAN-77-event-capture-fidelity`.
- `main` observed at `e1cba5d security(KAN-77): harden webhook replay idempotency (#213)`.
- Product direction remains consolidation, not another default hardening/report chain. New work
  should improve usability, package existing capabilities into a clearer workflow, fix a real bug,
  close a confirmed security/production risk, or support an explicit customer-selected policy.
- Desktop information architecture remains: `/action-center` owns the global `Next Action`;
  Workspace owns local execution and `Next local step`; Governance owns Evidence, Policy, Adoption,
  Releases, and Copilot; Settings/System owns Control Plane connection, API key, role/scope,
  transport, and updater configuration; `/control-plane` is compatibility redirect only.
- Desktop runtime safety remains non-negotiable: do not restart, kill, or relaunch Tauri/Desktop
  during a user's manual validation session unless the user explicitly asks.
- Publication safety remains non-negotiable: no token values, no real `.env` files, no restricted
  forensic/strategy docs force-added; use `docs/AGENT_PUBLIC_CONTEXT.md` as the public bridge.
- Enterprise onboarding and release-governance tooling is dry-run/report-only by default; mutations
  require explicit reviewed flags such as `-Apply`, and release blocking is customer opt-in.
- The integration-test harness drift report leaves an active durability concern: the backend
  integration harness still relies on a hand-maintained inline schema and CI does not exercise it
  with `TEST_DATABASE_URL`; durable fix is to apply real migrations or add schema parity plus a CI
  Postgres service.
- Some living docs are historical snapshots and may cite older route/test/migration counts. Prefer
  this handoff plus current repo inspection when facts differ from older KAN reports.
- No external-service validation or secret-bearing env-file inspection was performed during this
  documentation intake.

Additional deep intake from the same 2026-06-12 session:

- Public website/docs context: `gitgov-web` is the presentation layer, not the product source of
  truth. Future public copy should follow `docs/IMPLEMENTATION_STATUS.md`,
  `docs/ARCHITECTURE.md`, `README.md`, and actual Desktop/backend behavior before older web copy.
  The commercial category to preserve is "engineering governance with operational evidence".
- Public content risk noted for future cleanup, not changed in this intake: some public docs still
  carry historical wording such as Jira coverage/correlation marked as `Preview` in CI trace
  tables and Desktop commit capture described as including "message". Current security/product
  context is more precise: Jira API plus signed native webhook are operational, and GitGov must not
  imply source content or diff bodies leave the workstation.
- Release governance defaults are a closed product decision: default mode is `record-only`.
  Advisory status is allowed; blocking release gates, approval-required mode, quorum, and
  environment-specific overrides require explicit customer-selected policy. Do not infer blocking
  just because release approval records exist.
- Enterprise onboarding/readiness/remediation/checklist features are evidence and workflow guidance
  surfaces. Readiness reports, remediation plans, checklist tracking, artifact monitors, and trend
  reports must not read secret values, create GitHub variables/secrets, mutate provider settings,
  dispatch workflows, alter branch protection, or make release blocking the default.
- Remote workflow installation and readiness tooling is deliberately review-first: dry-run/plan by
  default, remote PR mutation only with explicit `-Apply`, overwrite only with explicit
  `-Overwrite`, and readiness validators compare workflow hashes/configuration names without
  reading GitHub Actions secret values.
- Product vulnerability review status remains: no critical/high reachable product vulnerability was
  left open after KAN-24. The recurring expected scanner finding is the inactive `sqlx-mysql`/`rsa`
  path classified as not reachable; if MySQL/sqlx-mysql features are enabled later, revisit that
  classification. Website contact/download rate limiting and ecosystem dependency warnings remain
  maintenance/deferred hygiene, not current blockers.
- Restricted local forensic/strategy docs (`docs/ENTERPRISE_READINESS_DECISION.md`,
  `docs/AUDIT_*.md`, `docs/INTEGRATIONS_AUDIT_*.md`) were inspected only as local memory. Do not
  force-add or quote them into public context; extract only sanitized current conclusions into
  tracked docs when needed.
- Older readiness and integration audit notes still support the current direction: enterprise value
  is traceable evidence, risk/readiness outcomes, and deterministic governance. SSO/SCIM, broader
  MCP, and autonomous AI agency remain deal-driven or future work, not default next steps.
- Route auth smoke chains (KAN-61 through KAN-68) are a guardrail family for enterprise route
  authorization and artifact freshness/trends. Treat them as regression evidence; do not add new
  monitor/enforcement chains unless they protect a concrete route/security risk or a selected
  customer policy.
- Policy-as-Code product decision captured in
  `docs/design/policy-as-code-flexible-source-mvp.md`: keep one canonical internal
  `GitGovConfig` model, but let customers choose `control-plane-managed`,
  `repo-policy-as-code`, or `hybrid-advisory` source mode and support TOML/YAML/JSON repo policy
  files. Implementation is merged on `main` through PR `#214`: `gitgov/policy-core` provides shared
  TOML/YAML/JSON parsing, discovery, canonical JSON, checksum, semantic diff, and real Git PR
  validation; backend/Tauri reexport the shared model; `supabase_schema_v31.sql` adds
  `source_metadata`; overrides and policy requests use canonical checksums; merged PR webhooks can
  activate the exact policy blob from GitHub when a token is configured; Governance displays policy
  source and blocks silent direct overrides for `repo-policy-as-code`. OPA/Rego is now supported as
  an optional external adapter, not as the default embedded engine: policy config can define
  `adapters.opa.*`, `enforcement.external_policy`, `effect = advisory|required`,
  `failure_mode = fail-open|fail-closed`, Data API `decision_path`, result mapping, timeout, and
  `token_env_var` by env-var name only. `/policy/check` calls the configured external OPA Data API
  when enabled, sends repo/branch/commit/actor, policy source metadata, and the native GitGov
  result, then returns OPA evidence under `external_decisions`. Required OPA plus
  `external_policy=block` can deny; advisory OPA never blocks. OPA response mapping supports boolean
  `allow`, custom allowed keys, boolean `deny`, and common Rego `deny` collections. An OPA `200`
  response without a mapped boolean decision is treated as adapter failure and obeys
  `fail-open`/`fail-closed`, matching the official Data API behavior where an undefined document can
  return `200` without `result`. Runtime and committed OPA URLs reject inline credentials/token query
  strings, query/fragment suffixes, invalid ports, and non-loopback `http://`; loopback checks parse
  the host so `localhost.example.com` / `127.0.0.1.example.com` are not accepted. Stored policy change
  requests are revalidated and checksum-checked again at approval time before activation. Local
  validation after the second OPA pass: policy-core tests `12` passed, backend OPA adapter tests `10`
  passed including real HTTP mock OPA Data API calls, backend policy-change approval tests `3`
  passed, full backend tests `250` passed, Tauri tests `49` passed, frontend tests `349` passed,
  policy-core/backend clippy `-D warnings` passed, Tauri `cargo check` passed, frontend typecheck/lint
  passed. Targeted `policy_check` integration tests compile the OPA endpoint path but still depend on
  a dedicated `TEST_DATABASE_URL` for non-skipped DB-backed runtime coverage. Remaining work:
  Governance patch/PR proposal UX,
  explicit emergency override UX, periodic drift comparison, Evidence Packet source metadata,
  customer examples/schema docs, controlled GitHub API activation test, persisted OPA decision audit
  history/export, and a real `opa run --server` smoke script. Production packaging was completed by
  PR `#215`, which aligned Render and local Docker context with the new sibling crate layout.
- KAN-77 security/event-fidelity and flexible Policy-as-Code work is no longer local-only. It is
  merged to `main`, production deployed on Render deploy `dep-d8lsul8k1i2s73dk1ph0`, and validated
  with `/health`, authenticated `/stats`, and post-merge GitHub checks.

## Latest Verified GitHub Checks

Latest post-merge validation for handoff baseline commit `126167f` passed:

- `CI` - run `25156959926`
- `Release Readiness Gate` - run `25156959919`
- `Quality Gate Policy Matrix (Optional)` - run `25156959901`
- `Secret Scan` - run `25156959895`
- `SonarQube Governance (Non-Blocking)` - run `25156959902`
- `Public Naming Guard` - run `25156959899`
- `Governance Correlation Smoke (Optional)` - run `25156959914`
- `Desktop Updater Readiness (Optional)` - run `25156959949`

Latest KAN-25 automation baseline:

- Implementation commit: `7c260fe security(KAN-25): automate vulnerability review evidence`.
- PR: `#100` - `security(KAN-25): automate product vulnerability review evidence`.
- Post-merge checks passed:
  - `CI` - run `25157965635`
  - `Release Readiness Gate` - run `25157965664`
  - `Quality Gate Policy Matrix (Optional)` - run `25157965674`
  - `Secret Scan` - run `25157965657`
  - `SonarQube Governance (Non-Blocking)` - run `25157965627`
  - `Public Naming Guard` - run `25157965648`
  - `Governance Correlation Smoke (Optional)` - run `25157965686`
  - `Desktop Updater Readiness (Optional)` - run `25157965670`
- First manual `Product Vulnerability Review` run passed:
  - Run `25157972836`
  - Mode `DependenciesOnly`
  - Artifact `product-vulnerability-review-25157972836`
  - Artifact status: not expired

Latest KAN-26 artifact monitor baseline:

- Implementation commit: `89a234c security(KAN-26): monitor vulnerability review artifacts`.
- PR: `#102` - `security(KAN-26): monitor product vulnerability review artifacts`.
- Post-merge checks passed:
  - `CI` - run `25158430862`
  - `Release Readiness Gate` - run `25158431062`
  - `Quality Gate Policy Matrix (Optional)` - run `25158430899`
  - `Secret Scan` - run `25158430868`
  - `SonarQube Governance (Non-Blocking)` - run `25158430873`
  - `Public Naming Guard` - run `25158430891`
  - `Governance Correlation Smoke (Optional)` - run `25158430896`
  - `Desktop Updater Readiness (Optional)` - run `25158430919`
- First manual `Product Vulnerability Review Artifact Monitor` run passed:
  - Run `25158436168`
  - Artifact `product-vulnerability-review-artifact-monitor`
  - Artifact ID `6727075935`
  - Artifact status: not expired

Latest KAN-27 trend report baseline:

- Implementation commit: `6fd8de8 security(KAN-27): add product vulnerability review trend reporting`.
- PR: `#104` - `security(KAN-27): add product vulnerability review trend reporting`.
- Post-merge checks passed:
  - `CI` - run `25159025219`
  - `Release Readiness Gate` - run `25159025186`
  - `Quality Gate Policy Matrix (Optional)` - run `25159025384`
  - `Secret Scan` - run `25159025195`
  - `SonarQube Governance (Non-Blocking)` - run `25159025371`
  - `Public Naming Guard` - run `25159025481`
  - `Governance Correlation Smoke (Optional)` - run `25159025229`
  - `Desktop Updater Readiness (Optional)` - run `25159025182`
- First manual `Product Vulnerability Review Trend Report` run passed:
  - Run `25159031614`
  - Artifact `product-vulnerability-review-trend-report`
  - Artifact ID `6727320469`
  - Artifact status: not expired

Latest KAN-28 trend enforcement baseline:

- Implementation commit: `7b36cec security(KAN-28): enforce product vulnerability trend baseline`.
- PR: `#106` - `security(KAN-28): enforce product vulnerability trend baseline`.
- Post-merge checks passed:
  - `CI` - run `25160187848`
  - `Release Readiness Gate` - run `25160187829`
  - `Quality Gate Policy Matrix (Optional)` - run `25160187813`
  - `Secret Scan` - run `25160187847`
  - `SonarQube Governance (Non-Blocking)` - run `25160187844`
  - `Public Naming Guard` - run `25160187839`
  - `Governance Correlation Smoke (Optional)` - run `25160187818`
  - `Desktop Updater Readiness (Optional)` - run `25160187859`
- First manual `Product Vulnerability Review Trend Enforcement` run passed:
  - Run `25160194313`
  - Artifact `product-vulnerability-review-trend-enforcement`
  - Artifact ID `6727810243`
  - Artifact status: not expired

Latest KAN-29 enterprise adoption baseline:

- Implementation commit: `bf8e378 product(KAN-29): add enterprise self-service adoption MVP`.
- PR: `#108` - `product(KAN-29): add enterprise self-service adoption MVP`.
- Post-merge checks passed:
  - `CI` - run `25160842461`
  - `Release Readiness Gate` - run `25160842032`
  - `Quality Gate Policy Matrix (Optional)` - run `25160842064`
  - `Secret Scan` - run `25160842081`
  - `SonarQube Governance (Non-Blocking)` - run `25160842041`
  - `Public Naming Guard` - run `25160842023`
  - `Governance Correlation Smoke (Optional)` - run `25160842049`
  - `Desktop Updater Readiness (Optional)` - run `25160842036`

Latest KAN-30 adoption profile dashboard baseline:

- Implementation commit: `0412574 product(KAN-30): add adoption profile dashboard MVP`.
- PR: `#110` - `product(KAN-30): add adoption profile dashboard MVP`.
- Post-merge checks passed:
  - `CI` - run `25161644820`
  - `Release Readiness Gate` - run `25161644879`
  - `Quality Gate Policy Matrix (Optional)` - run `25161644854`
  - `Secret Scan` - run `25161644841`
  - `SonarQube Governance (Non-Blocking)` - run `25161644861`
  - `Public Naming Guard` - run `25161644857`
  - `Governance Correlation Smoke (Optional)` - run `25161644871`
  - `Desktop Updater Readiness (Optional)` - run `25161644824`

Latest KAN-31 adoption profile persistence baseline:

- Implementation commit: `509e2a2 product(KAN-31): persist adoption profiles`.
- PR: `#112` - `product(KAN-31): persist adoption profiles`.
- Post-merge checks passed:
  - `CI` - run `25186881414`
  - `Release Readiness Gate` - run `25186881375`
  - `Quality Gate Policy Matrix (Optional)` - run `25186881361`
  - `Secret Scan` - run `25186881344`
  - `SonarQube Governance (Non-Blocking)` - run `25186881363`
  - `Public Naming Guard` - run `25186881451`
  - `Governance Correlation Smoke (Optional)` - run `25186881376`
  - `Desktop Updater Readiness (Optional)` - run `25186881345`
- Documentation validation PR: `#113` - `docs(KAN-31): record adoption profile validation`.
- Documentation validation commit: `171d43d docs(KAN-31): record adoption profile validation`.
- Post-merge docs refresh checks passed:
  - `CI` - run `25187583892`
  - `Release Readiness Gate` - run `25187583994`
  - `Quality Gate Policy Matrix (Optional)` - run `25187583967`
  - `Secret Scan` - run `25187583907`
  - `SonarQube Governance (Non-Blocking)` - run `25187583895`
  - `Public Naming Guard` - run `25187584004`
  - `Governance Correlation Smoke (Optional)` - run `25187583992`
  - `Desktop Updater Readiness (Optional)` - run `25187583943`
- Production DB migration `v23` was applied on 2026-04-30 using ignored local `DATABASE_URL` without printing credentials.
- `gitgov/gitgov-server/supabase/checks/v23_postcheck.sql` passed:
  - `enterprise_adoption_profiles.table_exists` - `PASS`
  - `enterprise_adoption_profiles.primary_key` - `PASS`
  - `enterprise_adoption_profiles.updated_at_index` - `PASS`
- Production route validation after migration:
  - `GET /health` returned `200`.
  - Anonymous `GET /enterprise/adoption-profile?org_name=yohandry10` returned `401`.
  - Authenticated `GET /enterprise/adoption-profile?org_name=yohandry10` returned `200` with `found=false`.

Latest KAN-38 AI SDK governance copilot baseline:

- Implementation commit: `9742472 product(KAN-38): add AI SDK governance copilot`.
- PR: `#127` - `product(KAN-38): add AI SDK governance copilot`.
- Jira final comment: `10197`.
- Post-merge checks passed:
  - `CI` - run `25194421718`
  - `Release Readiness Gate` - run `25194421743`
  - `Quality Gate Policy Matrix (Optional)` - run `25194421721`
  - `Secret Scan` - run `25194421747`
  - `SonarQube Governance (Non-Blocking)` - run `25194421756`
  - `Public Naming Guard` - run `25194421752`
  - `Governance Correlation Smoke (Optional)` - run `25194421750`
  - `Desktop Updater Readiness (Optional)` - run `25194421717`
- Vercel production deployment `https://git-ih2bzdqq5-trivia1.vercel.app` reached `Ready`.
- Production smoke passed on `https://www.gitgov.cloud/api/copilot/governance` and `https://git-gov.vercel.app/api/copilot/governance` with `success=true`, `mode=fallback`, `4` citations, `4` sources, and `1` expected warning because AI Gateway/OIDC generation was not active.

KAN-24 local validation before PR creation:

- `.\scripts\security\run_product_vulnerability_review.ps1 -Full -OutputDir docs/reports/product-vulnerability-review-2026-04-30 -CommandTimeoutSeconds 1200`
- Result: `20` pass, `1` expected finding, `0` fail.
- Remaining expected finding: backend `cargo audit` reports `rsa` through inactive `sqlx-mysql`; reachability checks showed no active dependency path in the current backend feature graph.

Production validation after Render deploy `dep-d7phm1m8bjmc73fko1lg`:

- Render deployed commit `126167ff1c4ad9756f2e3f78fcb69f9fcf14f2f1` and reached `live` on 2026-04-30.
- `GET https://gitgov-api.onrender.com/health` returned `status=ok`.
- Anonymous `GET /stats` returned `401`.
- Authenticated `GET /stats` returned `200` without printing token values.

## Non-Negotiable Operating Decisions

### Sonar

- SonarCloud is not a valid path for this repository because the current GitHub repository/account is personal, not organizational.
- Do not ask again to use SonarCloud for this repo.
- Do not propose SonarCloud onboarding unless the repository is moved to a GitHub organization.
- Local SonarQube is the selected Sonar runtime.
- Local SonarQube URL: `http://localhost:9000`.
- Sonar project key: `yohandry10_git-gov`.
- GitHub-hosted Sonar scans should skip while `SONAR_HOST_URL=http://localhost:9000`, because hosted runners cannot reach the workstation.
- If GitHub Actions must run a real local Sonar scan, first add and validate a dedicated self-hosted runner using `docs/runbooks/local-sonar-self-hosted-runner.md`.

### Jenkins

- Jenkins authenticated API access is already configured and is the normal agent path.
- Jenkins URL: `http://localhost:8096`.
- Current Jenkins job: `gitgov-demo-pipeline`.
- Jenkins authenticated API access supports inspection, logs, queue state, build history, and authenticated build operations.
- `JENKINS_BUILD_TRIGGER_TOKEN` is only for unauthenticated/manual URL build starts:

```text
{JENKINS_SERVER_URL}/job/{JENKINS_JOB_NAME}/build?token={JENKINS_BUILD_TRIGGER_TOKEN}
```

- Do not ask for the trigger-only token unless the user explicitly wants that unauthenticated/manual URL flow.

### OpenAPI and SDKs

- OpenAPI is the machine-readable API description used by Swagger tools and generated SDKs.
- OpenAPI is not the API itself.
- Normal GitGov API work uses the real backend routes/API.
- `/api-docs` is intentionally a partial schema explorer.
- `docs/ARCHITECTURE.md` plus the backend `main.rs` route table are the operational route source of truth.
- Full OpenAPI annotation is optional product work. Implement it only if generated SDKs or Swagger contract tests become a real requirement.

### Documentation Memory

- After any major access/configuration/deployment/validation change, update `AGENTS.md` and the relevant `docs/` file before finalizing a PR.
- Keep this handoff file current when the project state changes materially.
- Never print or commit token values.

## Access and Tooling

### GitHub

- Repository: `yohandry10/Git-Gov`.
- Default branch: `main`.
- GitHub CLI path: `C:\Users\PC\Tools\gh\bin\gh.exe`.
- `gh` is authenticated as `yohandry10`.
- Branch protection is enabled on `main`.
- Required checks are strict and admin-enforced.
- Traceability policy is active:
  - Branch names must include Jira IDs, except protected/base branches.
  - PR titles must include Jira IDs.
  - Commit messages must include Jira IDs.
  - Local guard: `.\scripts\security\publication_guard.ps1`.

### Render

- Production backend service: `gitgov-api`.
- Production URL: `https://gitgov-api.onrender.com`.
- Service ID: `srv-d7lgtc77f7vs73b38uqg`.
- Render service type: Docker web service.
- Render branch: `main`.
- Render root directory: `gitgov`.
- Render Docker context: `.` within `gitgov`.
- Render Dockerfile path: `gitgov-server/Dockerfile`.
- The Docker build context must include both `gitgov-server` and `policy-core` because the backend depends on `gitgov-policy-core` through a relative Cargo path.
- Render API access is available through ignored local env files as `RENDER_API_KEY`.

### Jira

- Base URL: `https://yohandrychirinos1.atlassian.net`.
- Project key: `KAN`.
- Project name: `GitGov`.
- Current native Jira webhook target:

```text
https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10
```

- Native Jira webhook name: `GitGov signed issue sync`.
- Native Jira webhook is signed with `JIRA_WEBHOOK_SECRET`.
- Use Jira ticket IDs in branches, commits, PR titles, and PR comments.

### Local Env Files

Tokens and secrets are in ignored local env files only:

- `C:\Users\PC\Desktop\GitGov\gitgov\.env`
- `C:\Users\PC\Desktop\GitGov\gitgov\gitgov-server\.env`

Never print values from these files. Treat them as source of truth for local access.

Expected local keys include:

- `GITGOV_API_KEY`
- `GITGOV_URL`
- `RENDER_API_KEY`
- `SONAR_HOST_URL`
- `SONAR_TOKEN`
- `SONAR_PROJECT_KEY`
- `JENKINS_SERVER_URL`
- `JENKINS_USER`
- `JENKINS_API_TOKEN`
- `JENKINS_JOB_NAME`
- `JIRA_BASE_URL`
- `JIRA_EMAIL`
- `JIRA_API_TOKEN`
- `JIRA_PROJECT_KEY`
- `JIRA_WEBHOOK_SECRET`
- `GITHUB_WEBHOOK_SECRET`

## Current Validation Commands

Run these from `C:\Users\PC\Desktop\GitGov`.

Publication and traceability guard:

```powershell
.\scripts\security\publication_guard.ps1
```

KAN-24 product vulnerability review runner:

```powershell
.\scripts\security\run_product_vulnerability_review.ps1 -Full -OutputDir docs/reports/product-vulnerability-review-2026-04-30 -CommandTimeoutSeconds 1200
```

KAN-25 automation workflow:

```text
.github/workflows/product-vulnerability-review.yml
```

Default scheduled mode is `DependenciesOnly`; manual modes are `DependenciesOnly`, `StaticOnly`, `RuntimeSmoke`, and `Full`.

KAN-26 artifact monitor workflow:

```text
.github/workflows/product-vulnerability-review-artifact-monitor.yml
```

It checks latest successful `product-vulnerability-review.yml` runs for artifacts with prefix `product-vulnerability-review-`.

KAN-27 trend report workflow:

```text
.github/workflows/product-vulnerability-review-trend-report.yml
```

It builds Markdown/JSON trend evidence from sanitized `summary.json` files in recent `product-vulnerability-review-*` artifacts.

KAN-28 trend enforcement workflow:

```text
.github/workflows/product-vulnerability-review-trend-enforcement.yml
```

It fails when the latest trend has failures, findings exceed the accepted baseline, findings/failures increase, or the latest successful review run lacks a parseable artifact.

KAN-29 enterprise adoption pack generator:

```powershell
.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-adoption-pack
```

It writes a Markdown/JSON customer adoption pack with providers, modules, policy preset, workflow plan, variable/secret names, and manual setup checklist. It does not read or write secret values.

KAN-33 workflow template generator:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-workflow-templates -Force
```

It writes ignored onboarding output under `out/enterprise-workflow-templates/`: `README.md`, `workflow-template-manifest.json`, and selected `.github/workflows/*.yml` templates. It records variable and secret names only, does not read `.env`, and does not mutate customer repositories.

KAN-35 reviewed workflow installer dry-run:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out\enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out\workflow-install-plan.json
```

Use `-Apply` only after review. Use `-Overwrite` only for reviewed replacements. The installer also supports dashboard JSON packs with `-PackPath`.

KAN-36 provider connection validator:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -ReportOnly -OutputPath out\provider-connections-report-only.json
```

Use strict mode without `-ReportOnly` when every selected provider must be ready. The validator reports sanitized statuses only and does not print secret values.

KAN-40/KAN-42 governance copilot AI mode validator:

```powershell
.\scripts\control-plane\validate_governance_copilot_ai_mode.ps1 -TicketId KAN-39 -ReleaseId KAN-39 -RequireAiMode -OutputPath out\governance-copilot-ai-mode-validation.json
```

Google Gemini is active in production after KAN-41. Use `-RequireAiMode` for normal production validation. Non-strict validation is only for explicit fallback diagnostics.

KAN-31 adoption profile persistence migration postcheck:

```powershell
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/supabase_schema_v23.sql
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/checks/v23_postcheck.sql
```

Do not print the database URL or credentials.
Production `v23` has already been applied; rerun the postcheck only when revalidating or provisioning a new environment.

Provider access smoke test:

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

Jira traceability coverage:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50
```

Jenkins trigger-only dry run:

```powershell
.\scripts\jenkins\validate_trigger_token_flow.ps1
```

Use `-Trigger` only when a real unauthenticated/manual URL build launch is intended.

## Recent Ticket Chain

- `KAN-14`: refreshed local/production operational validation after Docker Desktop and Sonar/Jenkins profiles were up.
- `KAN-15`: added guard that `/api-docs` remains a partial schema explorer.
- `KAN-16`: added provider access validator; latest refresh on 2026-04-28 returned all checks `ok`, readiness `92/100`, pipeline success `98.81%`, Jira coverage `69.88%`, and Sonar pass `98.81%`.
- `KAN-17`: documented local Sonar self-hosted runner path without enabling it.
- `KAN-18`: documented Jenkins trigger-only token flow as optional and dry-run-first.
- `KAN-19`: added Jira traceability coverage validator; latest recorded coverage was `96.67%` (`58/60`) over 720h.
- `KAN-20`: closed implementation backlog semantics; remaining items are operational decisions.
- `KAN-21`: clarified SonarCloud, OpenAPI/SDK, and Jenkins trigger-only defaults.
- `KAN-22`: created this current-context handoff, refreshed it through PR `#89` with baseline commit `c1951c8`, and fixed PowerShell workflow splatting in risk-tier baseline and desktop updater readiness workflows after scheduled/optional job failures.
- `KAN-23`: implemented ticket-scoped Evidence Packets before a Vercel AI SDK copilot. MVP added `GET /evidence/packets/tickets/{ticket_id}`, a Tauri command, dashboard JSON download UI, and docs under `docs/design/evidence-packets-mvp.md`; follow-up PR `#96` recorded production merge validation on `main` commit `a37d489`.
- `KAN-24`: opened Jira issue `KAN-24 - Product vulnerability review and production hardening` and started branch `security/KAN-24-product-vulnerability-review`. Scope covers end-to-end product vulnerability review across code, architecture, runtime, CI/CD, dependencies, and real user surfaces.
- `KAN-25`: opened Jira issue `KAN-25 - Automate product vulnerability review evidence` and started branch `security/KAN-25-product-vulnerability-review-automation`. Scope is operationalizing the KAN-24 runner as a weekly/manual GitHub Actions workflow with sanitized artifacts.
- `KAN-26`: opened Jira issue `KAN-26 - Monitor product vulnerability review artifact freshness` and started branch `security/KAN-26-product-vulnerability-artifact-monitor`. Scope is monitoring the freshness and presence of Product Vulnerability Review artifacts.
- `KAN-27`: opened Jira issue `KAN-27 - Trend product vulnerability review artifacts` and started branch `security/KAN-27-product-vulnerability-review-trend`. Scope is aggregating recent Product Vulnerability Review artifacts into trend evidence so regressions are visible across runs.
- `KAN-28`: opened Jira issue `KAN-28 - Vulnerability trend enforcement gate` and started branch `security/KAN-28-vulnerability-trend-enforcement`. Scope is converting KAN-27 trend evidence into an enforcement workflow and documenting the next two product features: Enterprise Self-Service Adoption and Vercel AI SDK Copilot.
- `KAN-29`: opened Jira issue `KAN-29 - Enterprise self-service adoption MVP` and started branch `product/KAN-29-enterprise-self-service-adoption`. Scope is creating the first reusable adoption pack generator for customer onboarding.
- `KAN-30`: opened Jira issue `KAN-30 - Adoption profile dashboard MVP`, implemented branch `product/KAN-30-adoption-profile-dashboard`, and merged PR `#110` as `0412574`. Scope moved the KAN-29 adoption profile into the admin dashboard with validation and secret-safe JSON export.
- `KAN-31`: opened Jira issue `KAN-31 - Persist adoption profiles for enterprise onboarding`, implemented branch `product/KAN-31-adoption-profile-persistence`, and merged PR `#112` as `509e2a2`. Scope persists the KAN-30 profile per org with admin get/upsert endpoints, backend validation, Supabase migration `v23`, Tauri commands, dashboard save/load, and secret-safe docs. Documentation refresh PR `#113` merged as `171d43d`, and production migration `v23` was applied and validated on 2026-04-30.
- `KAN-32`: opened Jira issue `KAN-32 - Enterprise provider health validation MVP`, implemented branch `product/KAN-32-provider-health-validation`, and merged PR `#115` as `1a16d88`. Scope adds a secret-safe Provider Health section to the Enterprise Adoption dashboard using already-loaded GitGov evidence instead of provider credentials.
- `KAN-33`: opened Jira issue `KAN-33 - Generate customer workflow templates from adoption profile`, implemented branch `product/KAN-33-workflow-template-generation`, and merged PR `#117` as `62b67e5`. Scope converts the KAN-29/KAN-31 adoption profile into reviewed workflow template packs, manifest, README, variables, secret names, and manual install checklist without mutating customer repositories.
- `KAN-34`: opened Jira issue `KAN-34 - Dashboard workflow template pack download`, implemented branch `product/KAN-34-dashboard-workflow-template-pack`, and merged PR `#119` as `31b109d`. Scope exposes workflow template pack generation in the Enterprise Adoption dashboard using the current/persisted profile, while keeping automatic repository mutation out of scope.
- `KAN-35`: opened Jira issue `KAN-35 - Reviewed workflow installation from template pack`, implemented branch `product/KAN-35-reviewed-workflow-installation`, and merged PR `#121` as `c60c486`. Scope installs CLI or dashboard workflow template packs into a local customer repository checkout only after dry-run review and explicit `-Apply`; remote GitHub mutation remains out of scope.
- `KAN-36`: opened Jira issue `KAN-36 - Direct provider connection validation for enterprise onboarding`, implemented branch `product/KAN-36-provider-connection-validation`, and merged PR `#123` as `8c075a4`. Scope validates explicitly provided provider credentials/reachability for GitHub, Jira, Jenkins, SonarQube, Render, and Vercel without printing secrets or mutating provider state.
- `KAN-37`: opened Jira issue `KAN-37 - Formal enterprise release approval MVP`, implemented branch `product/KAN-37-formal-release-approval`, and merged PR `#125` as `d7ae92e`. Scope is append-only formal release approvals with admin-only org scope, evidence packet hash binding, risk acceptance expiration, audit logging, Supabase migration `v24`, and backend validation tests. Production migration `v24` was applied and validated on 2026-04-30; Render deploy `dep-d7ptsvhoagis738cj88g` reached `live`.
- `KAN-38`: implemented `KAN-38 - Vercel AI SDK governance copilot MVP` on branch `product/KAN-38-ai-sdk-copilot`; PR `#127` merged as `9742472`. Scope is the first server-side Next.js AI SDK copilot route over bounded GitGov evidence with citations and fallback when AI Gateway/OIDC is unavailable.
- `KAN-39`: implemented `KAN-39 - Governance copilot dashboard UI MVP` on branch `product/KAN-39-governance-copilot-dashboard`; PR `#129` merged as `eda2f13`. Scope is the first admin dashboard UI for the KAN-38 copilot route, using a secret-safe Tauri proxy command and displaying cited answers, source statuses, and warnings.

## Current Product Roadmap

- Current completed enterprise onboarding foundation: Enterprise Self-Service Adoption MVP (`KAN-29`/`KAN-30`/`KAN-31`/`KAN-32`/`KAN-33`/`KAN-34`/`KAN-35`/`KAN-36`/`KAN-37`).
  - KAN-29 packages the proven GitGov operating model into a reusable adoption pack generator.
  - KAN-30 adds the first dashboard profile builder with provider/module toggles, policy presets, validation, workflow/policy preview, and secret-safe JSON export.
  - KAN-31 persists adoption profiles per org with admin save/load.
  - KAN-32 adds evidence-based provider health validation in the dashboard.
  - KAN-33 generates reviewed workflow template packs from the adoption profile.
  - KAN-34 adds dashboard download for workflow template packs.
  - KAN-35 adds reviewed local workflow installation from CLI or dashboard workflow packs.
  - KAN-36 adds direct provider credential/reachability checks.
  - KAN-37 adds formal release approval persistence with evidence packet hash and risk expiration.
- Current completed Deployment Gates 0.1 slice: `KAN-80` plus `KAN-83` through `KAN-88`, with
  `KAN-93` completed to unify decision evidence across Deployment Gates and optional Agent Governance.
  - KAN-80 adds first governed repo setup.
  - KAN-83 adds the CI/CD-facing deployment authorization API.
  - KAN-84 adds persisted authorization history in Desktop and generated workflow migration.
  - KAN-85 adds GitHub Actions, Jenkins Pipeline, and GitLab CI examples.
  - KAN-86 adds environment policy UX.
  - KAN-87 adds audited break-glass deployment authorization.
  - KAN-88 adds pre-approved break-glass approval routing bound to release evidence.
  - KAN-93 adds `shared-governance-decision.v1` to Deployment Gate records without using agents.
- Current completed AI feature: Vercel AI SDK Copilot.
  - Explain readiness, findings, tickets, pipelines, evidence packets, accepted risks, and blockers in plain language with cited GitGov evidence.
  - KAN-38 implements the first server-side route with `POST /api/copilot/governance`.
  - KAN-39 adds the first admin dashboard surface for that route.
- Completed hardening gate before those larger features: KAN-28 vulnerability trend enforcement.
- Active product block after KAN-90: `0.2 Agentic Governance Layer`. KAN-90 delivered the first deterministic, opt-in agent policy/API slice rather than an LLM-decided control. Manual GitGov workflows remain canonical for banks and regulated customers that prohibit autonomous agents. KAN-92 added the disabled-by-default control boundary. KAN-93 added shared deterministic decision evidence across Deployment Gates and optional Agent Governance. KAN-94 added optional agent-scoped credentials; KAN-95 added dry-run preview; KAN-96 added minimal attribution; KAN-97 hardens agent key expiry/rotation before any MCP surface. Agent Governance remains customer-selected and is not required for manual Deployment Gates.
- Optional later hygiene: remove the residual `rsa` / inactive `sqlx-mysql` dependency finding when upstream resolution or safe dependency cleanup makes that practical.

## Archived Ticket Notes

- Historical per-ticket implementation/validation notes (KAN-24 through KAN-68) were moved verbatim to `docs/reports/current-context-kan-notes-archive-2026-06-09.md` to keep this handoff compact.
- Treat archived notes as evidence snapshots for completed tickets, not as active backlog.

## Latest Workflow Fix Context

- `Risk Tier Baseline Calibration` scheduled run `24999681550` failed on 2026-04-27 because `.github/workflows/risk-tier-baseline-calibration.yml` used array splatting with `"-Param", value` pairs; PowerShell passed those positionally, so `-RepoFullName` reached the `Tier` parameter.
- `.github/workflows/desktop-updater-readiness.yml` used the same pattern and failed inside its optional job when `gitgov/src-tauri/tauri.conf.json` was bound to `TimeoutSeconds`.
- Use hashtable splatting for workflow PowerShell script blocks that call repository scripts with named parameters.
- Local validation for the fix generated a risk-tier baseline report with readiness `92/100`, composite risk `8/100`, and ran desktop updater readiness with endpoint probe skipped, returning the expected optional `WARN` state.
- Manual Risk Tier Baseline runs `25049577630` and `25049782826` on `main` confirmed the calibration step generated a report, then failed artifact upload because `report_path` was not visible to `actions/upload-artifact`; the workflow now uploads the deterministic report path directly.
- Final manual Risk Tier Baseline validation run `25049984199` passed on `main` commit `8e9b043` and uploaded artifact `risk-tier-baseline-25049984199` ID `6682824924`.

## Current Work Classification

KAN-113 is the active implementation slice and is not merged or production-smoked yet. KAN-112, KAN-111, KAN-110, KAN-109, and KAN-108 are complete and production-smoked.

Current work types are:

- Finish KAN-113 validation, PR, merge, deploy, and production smoke.
- Keep compliance/reporting work manual-first unless a customer explicitly opts into agentic features.
- Do not add official regulatory wording, compliance scoring, certification claims, scheduler, or AI summaries to KAN-113.

## Practical Next Steps

When resuming, do this first:

1. Run `git status --short --branch`.
2. Read `AGENTS.md` and this file.
3. If work changes code or docs, create/use a GitHub issue first.
4. Use a traceable branch, commit message, PR title, and issue comment with the `KAN-*` ID.
5. Run `.\scripts\security\publication_guard.ps1` before commit.
6. Push, open PR, wait for required checks, merge only when green.
7. After merge, pull `main`, wait for post-merge checks, and comment the issue with evidence.

## Do Not Reopen Without New Product Decision

- SonarCloud for this personal repo.
- Jenkins trigger-only token for normal agent work.
- Full OpenAPI annotation as a blocker.
- Old EC2/Nginx/systemd deployment path; Render is current production.
- Non-traceable commits or PRs.
