# Policy-as-Code Flexible Source Implementation Report

Date: 2026-06-12

Merge status:

- Implementation PR `#214` merged as `0acfd26 security(KAN-77): harden event capture and policy as
  code (#214)`.
- Render packaging hotfix PR `#215` merged as `e4bec3f fix(KAN-77): align Render Docker context for
  policy core (#215)`.
- Production Render deploy `dep-d8lsul8k1i2s73dk1ph0` reached `live`.
- Render hotfix details are documented in
  `docs/reports/render-policy-core-docker-context-hotfix-2026-06-12.md`.

## Implemented

- Added shared Rust policy core at `gitgov/policy-core`.
- Added TOML/YAML/JSON parsing into one `GitGovConfig` model.
- Added deterministic canonical JSON and SHA-256 policy checksum.
- Added default policy file discovery:
  - `.gitgov/policy.yml`
  - `.gitgov/policy.yaml`
  - `.gitgov/policy.json`
  - `gitgov.toml`
- Added ambiguity rejection when more than one default policy file exists.
- Reexported the shared policy model through backend and Tauri model surfaces.
- Updated Desktop repo validation to recognize any GitGov policy file, not only `gitgov.toml`.
- Added policy source metadata model and DB migration `supabase_schema_v31.sql`.
- Updated backend policy override and policy request checksum generation to use canonical checksums.
- Added `source_metadata` to active policies, policy history, and policy change requests.
- Added `Policy-as-Code Validation` GitHub workflow and `scripts/control-plane/validate_policy_as_code.ps1`.
- Added `gitgov-policy validate` CLI for PR diff validation.
- Added semantic diff detection for risky policy downgrades:
  - enforcement decreases.
  - minimum approval decreases.
  - disabling PR/ticket/signed-commit/force-push rules.
  - removing protected branches.
- Connected merged PR webhook handling to repo Policy-as-Code activation when a GitHub token is configured:
  - detect changed policy file.
  - fetch exact merged policy blob from GitHub Contents API.
  - parse and canonicalize it.
  - activate DB snapshot with repo source metadata.
- Updated Governance policy UI to show policy source.
- Blocked silent Governance override when active policy source is `repo-policy-as-code`.
- Added optional external OPA/Rego adapter configuration to `GitGovConfig`:
  - `adapters.opa.*` for endpoint/connection, decision path, effect, failure mode, timeout, input
    profile, token env var reference, and result mapping.
  - `enforcement.external_policy` for advisory/warn/block integration.
- Added secret-safe validation for OPA config:
  - no inline credentials or token query parameters in committed OPA URLs.
  - `token_env_var` must be an uppercase environment variable name, not a secret value.
  - non-loopback `http://` OPA URLs are rejected; remote OPA should use `https://`.
  - loopback HTTP validation checks the parsed host, not a prefix, so hosts such as
    `localhost.example.com` and `127.0.0.1.example.com` are rejected.
  - OPA base URLs reject query strings, fragments, non-numeric/zero/out-of-range ports, and invalid
    IPv6 authority shapes before runtime.
  - `input_profile` and result-mapping keys cannot be empty.
  - runtime OPA URLs resolved from `GITGOV_OPA_URL` / `GITGOV_OPA_<CONNECTION>_URL` use the same
    safety validation as committed policy config.
- Connected `/policy/check` to an external OPA Data API decision when enabled:
  - GitGov sends repo/branch/commit/actor, policy source metadata, and the native GitGov result.
  - OPA responses are merged as advisory warnings or required external-policy blocks.
  - response mapping supports boolean `allow` / custom `allowed_key`, boolean `deny`, and common Rego
    `deny` collections with messages.
  - OPA `200` responses without a mapped boolean decision are treated as adapter failures and obey
    `fail-open` / `fail-closed`; an undefined OPA document no longer passes silently.
  - failures obey `fail-open` / `fail-closed` and are returned in `external_decisions`.
  - `external_policy` only affects global policy-check enforcement when OPA is enabled and
    `effect = required`; disabled/advisory OPA does not make the native check look blocking.
- Policy change request approval now revalidates the stored config and checksum before activation,
  so stale pre-validation requests cannot activate an unsafe OPA config later.

## Validated

- `cargo test --manifest-path gitgov/policy-core/Cargo.toml`
  - includes equivalent TOML/YAML/JSON checksum tests.
  - includes real temporary Git repo validation over `HEAD~1..HEAD`.
  - includes OPA config validation for inline secrets, env-var names, spoofed loopback hosts,
    malformed URLs/ports, empty input profile, and empty result mapping keys.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml config::tests`
  - loads `.gitgov/policy.yml` from a real temp repo path.
  - rejects ambiguous YAML + TOML policy files.
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml policy_override_returns_canonical_checksum_and_source_metadata`
  - exercises real backend router/database path for override source metadata and canonical checksum.
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml policy_change_request_can_be_created_and_approved_by_admin`
  - verifies existing request/approval flow still works after canonical checksum/source metadata changes.
- `.\scripts\control-plane\validate_policy_as_code.ps1 -RepoPath . -BaseRef HEAD -HeadRef HEAD -Json`
  - verifies CLI/script execution path.
- `npm --prefix gitgov test -- --run useControlPlaneStore useRepoStore`
  - verifies repo policy detection and repo-managed override guard.
- `npm --prefix gitgov run typecheck`
  - verifies TypeScript policy source metadata contracts.
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml opa_adapter_tests`
  - verifies OPA response mapping, required blocking behavior, advisory non-blocking behavior,
    common Rego `deny` collections, undefined-result fail-closed behavior, token env-var redaction,
    effective enforcement semantics, runtime env URL validation, and real HTTP calls to a local mock
    OPA Data API server.
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml policy_change_request_tests`
  - verifies approval-time validation for valid requests, invalid stored OPA config, and checksum
    mismatch.
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml policy_check_blocks_when_required_opa_denies`
  - compiles the real `/policy/check` integration path with a local mock OPA server.
  - skipped at runtime unless `TEST_DATABASE_URL` points to a dedicated test DB.
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml policy_check_records_opa_fail_open_without_blocking`
  - compiles the fail-open `/policy/check` path.
  - skipped at runtime unless `TEST_DATABASE_URL` points to a dedicated test DB.
- `cargo clippy --manifest-path gitgov/policy-core/Cargo.toml -- -D warnings`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml`
  - `250` passed in the local suite. DB-gated tests still rely on the local harness behavior /
    `TEST_DATABASE_URL` availability.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
  - `49` passed.
- `npm --prefix gitgov test -- --run`
  - `349` passed.
- `npm --prefix gitgov run lint -- --quiet`
- PR `#214` required GitHub checks passed before merge.
- Post-merge GitHub checks for `0acfd26` passed.
- PR `#215` required GitHub checks passed before merge.
- Post-merge GitHub checks for `e4bec3f` passed.
- Render deploy `dep-d8lsul8k1i2s73dk1ph0` reached `live`.
- Production `GET /health` returned `status=ok`.
- Production authenticated `GET /stats` returned HTTP `200`.
- Production `supabase_schema_v31.sql` was applied and verified.
- Production authenticated `GET /policy/yohandry10%2FGit-Gov` returned HTTP `200`.
- Local rerun of `scripts/jenkins/validate_quality_gate_policy_matrix.ps1` against production
  passed after the `v31` migration was applied.

## Still Pending

- UI flow to create an actual repo patch/PR from Governance edits in `repo-policy-as-code` mode.
- Explicit emergency override UX with reason, ticket, expiration, previous checksum, and active drift banner.
- Drift comparison job that periodically compares repo policy checksum with active Control Plane checksum.
- Evidence Packet export additions for policy source metadata.
- Full webhook activation test with mockable GitHub API base URL or a controlled live GitHub fixture.
- Customer-facing examples/schema docs for all policy formats.
- Persisted OPA decision audit table/export history; current MVP returns decision evidence in
  `/policy/check` responses.
- Real OPA CLI smoke script with `opa run --server`; current backend tests use a local mock OPA HTTP
  server and unit tests for merge semantics.
