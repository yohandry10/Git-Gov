# Integration Test Harness Schema Drift

Date: 2026-06-09

## Summary

The backend integration test harness (`gitgov/gitgov-server/src/integration_tests/common.rs`)
builds its own **hand-maintained inline schema** (CREATE TABLE / CREATE FUNCTION statements)
instead of applying the production migrations under
`gitgov/gitgov-server/supabase/supabase_schema*.sql`. Over time that inline schema drifted
from production, so several integration tests asserted against a schema/shape that no longer
matches the real handlers and models.

Because the harness skips when `TEST_DATABASE_URL` is not set (`setup_or_skip!`), and CI does
**not** set `TEST_DATABASE_URL`, these tests are silently skipped in CI. The drift therefore
went undetected: `9` integration tests had been failing against a real database and nobody saw
it. They were fixed on 2026-06-09 (`196 passed, 0 failed` against a local Postgres), but the
**root cause — two sources of truth for the schema plus CI skipping the suite — remains.**

## Concrete drift found (2026-06-09)

| Surface | Production reality | Harness had | Effect |
| --- | --- | --- | --- |
| `commit_ticket_correlations` | column `correlation_source` (+ `confidence`, CHECK on source set) | column `source`, no `confidence`, no CHECK | inserts/asserts written for the wrong column name |
| `project_tickets` | includes `ingested_at` | `ingested_at` missing | the orphan-tickets query in `get_ticket_coverage` errored (`column ... does not exist`) |
| `identity_aliases` | `canonical_login`, `alias_login`, `org_id`, composite PK | `primary_login`, `alias_login`, `created_by`, `id`, no `org_id` | `get_combined_events` (behind `/logs` and `/export`) failed with `column ica.org_id does not exist` |
| `get_audit_stats(uuid)` SQL function | exists (real counts) | not defined | `/stats` silently fell back to `AuditStats::default()` (all zeros) |

### Additional drift found during the multi-tenant isolation work (same day)

While fixing the cross-org isolation findings, more `project_tickets` drift surfaced and
was aligned in the harness:

| Column | Production reality | Harness had | Effect |
| --- | --- | --- | --- |
| `related_commits` / `related_prs` | `TEXT[]` | `JSONB` | `append_project_ticket_relations_full` (array ops) would fail against the harness |
| `related_branches` | present (`TEXT[]`) | missing entirely | append of branch relations would fail |
| `commit_ticket_correlations` unique index | now `(org_id, commit_sha, ticket_id)` via `v26` | `UNIQUE(commit_sha, ticket_id)` | harness updated to the org-scoped expression index to mirror production |

The harness still uses `source`/`project_key`/`summary`/`url` where production uses
`correlation_source`/`ticket_url`/`title` (no `project_key`). These remaining name
mismatches are why some production functions (`insert_commit_ticket_correlation`,
`upsert_project_ticket`, `get_project_ticket_by_ticket_id`) still cannot run against the
harness, so those paths are validated against the real Postgres schema instead.

### Stale test data / assertions

Two additional issues were **stale test data / assertions**, not schema drift:

- Several policy tests called `/policy/acme/repo/...` with a raw slash; the route is
  `/policy/{repo_name}` (single segment) and the contract requires URL-encoding (`acme%2Frepo`).
- `golden_path` sent `files` as objects (`{"path":...}`) but the model is `Vec<String>`, and it
  asserted on a `/stats` field `total_events` that does not exist (the real field is
  `client_events.total`).

## Why it matters

- The integration suite is the only layer that exercises real SQL (auth, scoping, combined
  events, exports, policy workflow). When it is skipped, **all of that is unverified in CI**.
- Drift hides real regressions: a handler/model change that breaks a SQL contract passes CI
  because the test that would catch it never runs.
- The inline harness schema is a **second source of truth** that must be hand-synced with every
  migration — an error-prone manual step that already failed silently multiple times.

## Recommended durable fix (not yet done)

1. **Single source of truth for the schema.** Make the harness apply the real
   `supabase/supabase_schema*.sql` migrations into the test schema instead of maintaining an
   inline copy. If full migrations are too heavy per-test, generate the inline schema from the
   migrations and add a parity check that fails when they diverge.
2. **Run the integration suite in CI** against an ephemeral Postgres (e.g. an Actions service
   container) with `TEST_DATABASE_URL` set, so drift fails fast instead of skipping.
3. Until (1)/(2) land, treat the inline harness schema as production-mirroring: any migration
   that adds/renames a column or function used by a handler must update `common.rs` in the same
   change.

## Local validation note

The local Postgres used for validation collides on host port `5433` with a **native Windows
PostgreSQL** instance; the Docker `gitgov-db` was published on `5434` to bypass the collision,
and its `gitgov` role password was aligned to the documented dev value. These are local
environment notes only and do not affect the repository.
