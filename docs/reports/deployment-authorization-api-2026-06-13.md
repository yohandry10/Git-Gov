# KAN-83 Deployment Authorization API Report

Date: 2026-06-13

## Scope

KAN-83 implements the Deployment Gates 0.1 authorization API slice.

Implemented:

- `POST /deployment-gates/authorize`.
- `GET /deployment-gates/authorizations`.
- append-only `deployment_gate_authorizations` persistence.
- Supabase migration `supabase_schema_v35.sql`.
- Supabase postcheck `checks/v35_postcheck.sql`.
- release-governance evaluator reuse.
- evidence packet binding validation before authorization.
- advisory history when first governed repo setup is missing or incomplete.
- blocking decision only when explicit release governance policy blocks.

Not implemented:

- Desktop history UI.
- provider-specific remote deployment mutation.
- OPA/Rego execution.
- automatic branch protection or workflow secret setup.

## Product Decision

Deployment authorization is now a first-class API contract instead of a script-only wrapper around the evaluator.

CI/CD callers can ask GitGov:

```text
Can this exact release, SHA, environment, and evidence packet deploy?
```

GitGov answers with:

- `approved`;
- `decision`;
- `reason`;
- `blocking`;
- `would_block`;
- `blocked_by`;
- `warnings`;
- `policy_checksum`;
- persisted authorization history.

## Validation

PR:

```text
#303 merged to main as 4dfba5f
```

Local backend checks run:

```text
cargo check --manifest-path .\gitgov\gitgov-server\Cargo.toml
cargo fmt --manifest-path .\gitgov\gitgov-server\Cargo.toml --check
cargo clippy --manifest-path .\gitgov\gitgov-server\Cargo.toml -- -D warnings
```

Result: passed.

Migration validation run against a temporary Postgres 16 container on host port `55433`:

```text
psql --dbname=postgresql://gitgov:<redacted>@127.0.0.1:55433/gitgov --file=.\gitgov\gitgov-server\supabase\supabase_schema_v35.sql
psql --dbname=postgresql://gitgov:<redacted>@127.0.0.1:55433/gitgov --file=.\gitgov\gitgov-server\supabase\checks\v35_postcheck.sql
```

Result: postcheck returned `PASS` for table, decision constraint, and indexes.

Focused tests run with the same real Postgres integration database:

```text
$env:TEST_DATABASE_URL='postgresql://gitgov:<redacted>@127.0.0.1:55433/gitgov'
cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml deployment_gate -- --nocapture
```

Result: `6 passed`.

Full backend tests run with the same real Postgres integration database:

```text
$env:TEST_DATABASE_URL='postgresql://gitgov:<redacted>@127.0.0.1:55433/gitgov'
cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml
```

Result: `260 passed`.

Publication checks:

```text
git diff --check
.\scripts\security\publication_guard.ps1
```

Result: passed.

## Production Validation

Post-merge checks for `4dfba5f` passed:

- `CI`
- `Release Readiness Gate`
- `Secret Scan`
- `Public Naming Guard`
- `Quality Gate Policy Matrix`
- `Governance Correlation Smoke`
- `Desktop Updater Readiness`
- `SonarQube Governance`

Render:

```text
dep-d8mf606q1p3s73fn44vg reached live
```

Production DB:

- Applied `supabase_schema_v35.sql`.
- `v35_postcheck.sql` returned `PASS` for table, decision constraint, and indexes.
- Production was missing the older release evidence dependency table, so idempotent `supabase_schema_v28.sql` was applied and `release_evidence_packets` was verified before endpoint smoke.

Production smoke:

```text
GET /health => ok
anonymous POST /deployment-gates/authorize => 401
GET /evidence/packets/tickets/KAN-83?... => found=true
POST /deployment-gates/authorize => decision=advisory, approved=true, blocking=false, would_block=false
GET /deployment-gates/authorizations?authorization_id=dga_486236dbd5e34264bebf52ec61db5667 => total=1
```

The integration tests generate a real release-bound evidence packet through the existing evidence packet endpoint before calling the new deployment authorization endpoint.

## Test Coverage Notes

Covered:

- missing first governed repo setup produces advisory authorization and persists history;
- blocking release governance policy without approval produces `decision=blocked`;
- persisted history returns the original authorization and normalized request payload;
- Developer keys cannot authorize deploys and scoped Admin keys cannot write another tenant's authorization history;
- provided `ticket_id` must match the release-bound evidence packet ticket;
- evidence packet binding remains enforced;
- existing release approval and release governance binding behavior still passes.

Remaining follow-up:

- provider-specific deploy examples for Jenkins, GitHub Actions, GitLab CI, and other deployers;
- break-glass workflow design and authorization evidence;
- advanced environment policy workflows beyond the Desktop/admin matrix.
