# GitGov Control Plane Server

Centralized audit and policy server for GitGov desktop clients. Uses PostgreSQL (Supabase in production, local PG16+ for development).

## Features

### Core
- **GitHub Webhooks**: Receives push/create events as source of truth
- **Client Events**: Batch telemetry from desktop clients with idempotency
- **Append-Only Audit**: Events cannot be modified or deleted
- **Row Level Security**: PostgreSQL RLS policies for multi-tenant access

### Compliance (V1.0)
- **Correlation Engine**: Correlates client events with GitHub events by commit_sha
- **Bypass Detection**: Detects pushes without GitGov client events
- **Confidence Scoring**: NOT binary - uses `high`, `medium`, `low` confidence levels
- **Noncompliance Signals**: Evidence-based signals, not accusations
- **Policy Versioning**: Automatic history of all policy changes
- **Export with Hash**: JSON/CSV exports with SHA256 content hash

## Quick Start

### 1. Setup PostgreSQL

1. Install PostgreSQL 16+ (or use Docker: `docker compose up -d gitgov-db`)
2. Create a database: `CREATE DATABASE gitgov;`
3. Run schema base: `psql -d gitgov -f supabase/supabase_schema.sql`
4. Run all migrations in order: `supabase_schema_v2.sql` through `supabase_schema_v25.sql`

#### 1.1 Create a Limited Database User (recommended for production)

```sql
CREATE ROLE gitgov_server WITH LOGIN PASSWORD 'your-strong-password';
GRANT CONNECT ON DATABASE gitgov TO gitgov_server;
GRANT USAGE ON SCHEMA public TO gitgov_server;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO gitgov_server;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO gitgov_server;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO gitgov_server;
```

**Update your `.env`:**

```env
DATABASE_URL=postgresql://gitgov_server:YOUR_PASSWORD@localhost:5432/gitgov
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=postgresql://gitgov_server:YOUR_PASSWORD@localhost:5432/gitgov
GITGOV_JWT_SECRET=your-secure-secret-key
GITGOV_SERVER_ADDR=0.0.0.0:3000
GITHUB_WEBHOOK_SECRET=your-webhook-secret
```

### 3. Run Server

```bash
cargo run
```

## Job Queue Architecture

### Overview

The job queue provides **backpressure control** for webhook processing. Instead of processing detection synchronously, webhooks enqueue jobs that are processed by a background worker.

### Key Features

| Feature | Implementation | Purpose |
|---------|---------------|---------|
| **Atomic Claim** | `FOR UPDATE SKIP LOCKED` | No race conditions between workers |
| **Deduplication** | Partial unique index on `(org_id, job_type)` | Only 1 pending/running job per org+type |
| **Exponential Backoff** | `30s * 2^attempts`, capped at 1 hour | Prevents retry storms |
| **Dead-Letter Queue** | `status='dead'` after max_attempts | Failed jobs for manual inspection |
| **Stale Job Reset** | TTL of 5 minutes, safe reset | Recover from worker crashes |
| **Structured Logging** | `job_id`, `org_id`, `duration_ms` | Observability |

### Job States

```
pending -> running -> completed
                  \-> failed (with retry)
                  \-> dead (max attempts exceeded)
```

### Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `max_attempts` | 10 | Max retries before dead-letter |
| `TTL` | 5 minutes | Time before job considered stale |
| `Poll Interval` | 5 seconds | How often worker checks for jobs |
| `Backoff Base` | 30 seconds | Base for exponential backoff |
| `Backoff Max` | 1 hour | Maximum backoff delay |

### Monitoring Jobs

```bash
# Get job metrics
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/jobs/metrics

# Response:
{
  "worker_id": "worker-12345",
  "metrics": {
    "pending": 2,
    "running": 1,
    "completed_today": 45,
    "failed_today": 0,
    "dead": 1,
    "stale_running": 0,
    "avg_duration_ms": 1523,
    "oldest_pending_seconds": 12
  }
}

# List dead jobs
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/jobs/dead

# Retry a dead job
curl -X POST -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/jobs/{job_id}/retry
```

### Troubleshooting

#### Stuck Jobs

If jobs are stuck in `running` state:

1. Check if worker process is alive
2. Wait for TTL (5 minutes) for auto-recovery
3. Manually inspect: `SELECT * FROM jobs WHERE status = 'running';`

#### Dead Jobs

Jobs that exceeded max attempts:

1. Check `last_error` column for failure reason
2. Fix underlying issue
3. Retry via API: `POST /jobs/{id}/retry`

#### High Pending Count

Many pending jobs:

1. Check worker logs for errors
2. Verify database connectivity
3. Check if jobs are being claimed: `SELECT COUNT(*) FROM jobs WHERE status = 'running';`

### SQL Queries for Debugging

```sql
-- View all jobs for an org
SELECT id, job_type, status, attempts, last_error, created_at
FROM jobs WHERE org_id = '...' ORDER BY created_at DESC;

-- Check for orphaned jobs
SELECT * FROM jobs WHERE status = 'running' AND locked_at < NOW() - INTERVAL '10 minutes';

-- Manual stale reset
SELECT reset_stale_jobs_safe(5);

-- View dead-letter queue
SELECT id, job_type, attempts, last_error, created_at
FROM jobs WHERE status = 'dead' ORDER BY created_at DESC;

-- Job throughput
SELECT 
    status,
    COUNT(*) as count,
    AVG(duration_ms) as avg_duration_ms
FROM jobs
WHERE created_at > NOW() - INTERVAL '24 hours'
GROUP BY status;
```

## Cursor-Based Incremental Processing

### Why `ingested_at` Instead of `created_at`

The `detect_noncompliance_signals` function uses **server-side ingestion time** (`ingested_at`) as the cursor, not event creation time (`created_at`).

**Problem with `created_at`:**
- Events can arrive late (retries, backlogs, network delays)
- `created_at` reflects when the event happened in GitHub
- A cursor on `created_at` would skip late-arriving events

**Solution with `ingested_at`:**
- Set at INSERT time by the database
- Never modified
- Guarantees processing order

```
GitHub Event Time:     10:00  (created_at)
Arrives at Server:     10:05  (ingested_at)
Cursor processes:      10:05  (using ingested_at)
```

### Schema

```sql
-- Added in supabase_schema_v2.sql
ALTER TABLE github_events ADD COLUMN ingested_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE client_events ADD COLUMN ingested_at TIMESTAMPTZ DEFAULT NOW();

-- Cursor stored in org_processing_state
last_ingested_at TIMESTAMPTZ,
last_processed_event_id UUID
```

## Append-Only Guarantee

### Tables with Append-Only Triggers

| Table | Trigger | Allowed Updates |
|-------|---------|-----------------|
| `github_events` | `BEFORE UPDATE OR DELETE` | None |
| `client_events` | `BEFORE UPDATE OR DELETE` | None |
| `violations` | Limited UPDATE | Only `resolved` fields |
| `noncompliance_signals` | `BEFORE UPDATE OR DELETE` | None |
| `governance_events` | `BEFORE UPDATE OR DELETE` | None |
| `signal_decisions` | `BEFORE UPDATE OR DELETE` | None |
| `policy_history` | `BEFORE UPDATE OR DELETE` | None |

### Jobs Table (NOT Append-Only)

The `jobs` table allows state transitions but restricts which columns can be updated:

- **Immutable**: `id`, `org_id`, `job_type`, `created_at`, `payload`
- **Mutable**: `status`, `locked_at`, `locked_by`, `attempts`, `last_error`, `next_run_at`

## Endpoints

### Health

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Simple health check |
| GET | `/health/detailed` | Detailed health with DB latency, uptime, pending events |

### Webhooks & Events

| Method | Path | Description |
|--------|------|-------------|
| POST | `/webhooks/github` | GitHub webhook receiver (push, create) |
| POST | `/events` | Client events (batch with idempotency) |

### Queries

| Method | Path | Description |
|--------|------|-------------|
| GET | `/logs` | Query combined events (github + client) |
| GET | `/stats` | Statistics |
| GET | `/dashboard` | Dashboard data |

### Compliance

| Method | Path | Description |
|--------|------|-------------|
| GET | `/compliance/{org_name}` | Compliance dashboard (signals, correlation rate) |
| GET | `/signals` | List noncompliance signals with filters |
| POST | `/signals/:id` | Update signal status (investigate/dismiss) |
| POST | `/signals/detect/:org` | Trigger bypass detection |

### Policy

| Method | Path | Description |
|--------|------|-------------|
| GET | `/policy/:repo` | Get policy for repo |
| PUT | `/policy/:repo` | Save policy for repo |
| GET | `/policy/:repo/history` | Policy change history |

### Export

| Method | Path | Description |
|--------|------|-------------|
| POST | `/export` | Export events (JSON/CSV) with SHA256 hash |

### Evidence Packets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/evidence/packets/tickets/:ticket_id` | Build a ticket-scoped audit evidence packet with SHA256 content hash |

### Integrations

| Method | Path | Description |
|--------|------|-------------|
| POST | `/integrations/jenkins` | Ingest Jenkins pipeline events |
| GET | `/integrations/jenkins/status` | Jenkins integration health check |
| GET | `/integrations/jenkins/correlations` | Commit↔pipeline correlations |
| GET | `/integrations/correlations/v2` | Unified ticket↔commit↔pipeline view |
| POST | `/integrations/jira` | Ingest Jira issues (admin/manual) |
| POST | `/webhooks/jira` | Signed Jira webhooks (HMAC) |
| GET | `/integrations/jira/status` | Jira integration health check |
| GET | `/integrations/jira/tickets/:id` | Jira ticket detail |
| POST | `/integrations/jira/correlate` | Batch commit↔ticket correlation |
| GET | `/integrations/jira/ticket-coverage` | Ticket coverage metrics |

### Enterprise

| Method | Path | Description |
|--------|------|-------------|
| GET/PUT | `/enterprise/adoption-profile` | Get/upsert enterprise adoption profile |
| GET/PUT | `/enterprise/onboarding-checklist-tracking` | Get/upsert onboarding checklist tracking |
| GET/POST | `/enterprise/release-approvals` | List/create formal release approvals |
| GET | `/enterprise/release-governance/evaluate` | Evaluate release governance policy |

### Real-Time

| Method | Path | Description |
|--------|------|-------------|
| GET | `/sse` | Server-Sent Events stream |

### Organization Management

| Method | Path | Description |
|--------|------|-------------|
| POST | `/orgs` | Create organization |
| GET/POST | `/org-users` | List/create org users |
| PATCH | `/org-users/:id/status` | Update org user status |
| POST | `/org-users/:id/api-key` | Create API key for org user |
| GET/POST | `/org-invitations` | List/create invitations |
| POST | `/org-invitations/:id/resend` | Resend invitation |
| POST | `/org-invitations/:id/revoke` | Revoke invitation |
| GET | `/org-invitations/preview/:token` | Preview invitation (public) |
| POST | `/org-invitations/accept` | Accept invitation (public) |

### Chat & AI

| Method | Path | Description |
|--------|------|-------------|
| POST | `/chat/ask` | Conversational governance chat (Admin/Architect/PM) |
| POST | `/feature-requests` | Create feature request from bot |

### GDPR

| Method | Path | Description |
|--------|------|-------------|
| POST | `/users/:login/erase` | Erase user data |
| GET | `/users/:login/export` | Export user data |
| GET | `/clients` | List client sessions |
| GET/POST | `/identities/aliases` | List/create identity aliases |

### Audit & Observability

| Method | Path | Description |
|--------|------|-------------|
| GET | `/stats/daily` | Daily activity |
| GET | `/team/overview` | Team overview |
| GET | `/team/repos` | Team repositories |
| GET | `/pr-merges` | List PR merges |
| GET | `/admin-audit-log` | Admin audit log |
| POST/GET | `/cli/commands` | Ingest/list CLI commands |
| POST/GET | `/policy/drift-events` | Ingest/list policy drift events |
| GET | `/metrics` | Prometheus metrics (public) |
| GET | `/me` | Current user info |

### Admin

| Method | Path | Description |
|--------|------|-------------|
| GET/POST | `/api-keys` | List/create API keys |
| POST | `/api-keys/:id/revoke` | Revoke API key |
| GET | `/jobs/metrics` | Job queue metrics |
| GET | `/jobs/dead` | List dead-letter jobs |
| POST | `/jobs/:id/retry` | Retry dead job |
| POST | `/outbox/lease` | Acquire outbox flush lease |
| GET | `/outbox/lease/metrics` | Outbox lease metrics |

## Correlation & Bypass Detection

### How It Works

1. Desktop sends `client_event` with `event_uuid` before/during push
2. GitHub sends webhook with `delivery_id` after push
3. Server correlates by `commit_sha + actor_login + branch`
4. If no client_event found → creates noncompliance signal

### Confidence Levels (NOT Binary)

| Level | Condition | Signal Type |
|-------|-----------|-------------|
| **High** | GitHub push with NO client event, empty outbox | `untrusted_path` |
| **Low** | GitHub push with NO client event, pending outbox events | `missing_telemetry` |

### Language (Evidence-Based)

The system uses evidence-based language, NOT accusations:

- ✓ `untrusted_path` - Direct push detected
- ✓ `missing_telemetry` - Incomplete data, outbox pending
- ✓ `noncompliance signal` - Generic term
- ✗ `bypass detected` - Too accusatory for automated detection
- ✗ `violation` - Requires manual confirmation

## Security

- **HMAC Signature**: Webhooks are validated using `X-Hub-Signature-256`
- **API Keys**: Desktop clients authenticate with API keys
- **RLS**: PostgreSQL Row Level Security restricts data access by user/role
- **Append-Only**: Audit events cannot be tampered with
- **Export Hash**: Every export has SHA256 for verification
- **Bootstrap Security**: API keys printed only with explicit flag or TTY

### API Key Authentication

**Important:** The server expects `Authorization: Bearer {api_key}` header, NOT `X-API-Key`.

**Flow:**
1. Client sends: `Authorization: Bearer <YOUR_API_KEY>`
2. Server hashes: `SHA256("57f1ed59-...")` → `abc123...`
3. Server queries: `SELECT * FROM api_keys WHERE key_hash = 'abc123...'`
4. If found → authentication successful

**Example:**
```bash
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/stats
```

**Common Pitfall:**
```bash
# ❌ WRONG - Will return 401 Unauthorized
curl -H "X-API-Key: $API_KEY" http://127.0.0.1:3000/stats

# ✅ CORRECT - Use Authorization: Bearer
curl -H "Authorization: Bearer $API_KEY" http://127.0.0.1:3000/stats
```

### Bootstrap Key Security

The bootstrap admin key is only printed when:
1. Running with `--print-bootstrap-key` flag, OR
2. Running in an interactive terminal (TTY)

In Docker/Kubernetes environments (no TTY), the key is never printed to logs:

```bash
# Interactive (TTY) - key printed to console
cargo run

# Docker (no TTY) - key NOT printed
docker run gitgov-server

# Explicit flag - key always printed
docker run gitgov-server --print-bootstrap-key
```

## Testing

### E2E Flow Test

Verifica el pipeline completo de eventos:

```bash
cd gitgov/gitgov-server/tests
chmod +x e2e_flow_test.sh
SERVER_URL=http://127.0.0.1:3000 API_KEY=your-key ./e2e_flow_test.sh
```

**Tests incluidos:**
1. Health check del servidor
2. Autenticación con `Authorization: Bearer`
3. Rechazo de `X-API-Key` (header incorrecto)
4. Envío de evento cliente
5. Verificación en logs
6. Obtención de estadísticas
7. Query de eventos combinados

### Stress Tests

```bash
# Run stress test suite
cd tests
chmod +x stress_test.sh
./stress_test.sh

# With API key for full testing
SERVER_URL=http://127.0.0.1:3000 API_KEY=your-key ./stress_test.sh
```

### Manual Testing

```bash
# Health check
curl http://127.0.0.1:3000/health

# Simulate webhook
curl -X POST http://127.0.0.1:3000/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -H "X-GitHub-Delivery: test-123" \
  -d '{
    "ref": "refs/heads/main",
    "before": "abc123",
    "after": "def456",
    "repository": {
      "id": 123,
      "name": "repo",
      "full_name": "org/repo",
      "private": false,
      "owner": {"id": 1, "login": "org"},
      "organization": {"id": 1, "login": "org"}
    },
    "sender": {"id": 1, "login": "developer"},
    "commits": [{"id": "def456", "message": "test"}]
  }'

# Check job metrics
curl -H "Authorization: Bearer $API_KEY" \
  http://127.0.0.1:3000/jobs/metrics
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   GitHub        │────▶│  Webhook        │
│   (Webhooks)    │     │  POST /webhooks │
└─────────────────┘     └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │   Job Queue     │
                        │   (Postgres)    │
                        └────────┬────────┘
                                 │
┌─────────────────┐              │
│   Desktop App   │──────┐       │
│   (Outbox)      │      │       │
└─────────────────┘      │       │
                         ▼       ▼
                  ┌─────────────────┐
                  │  POST /events   │
                  │  (Batch)        │
                  └────────┬────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │   PostgreSQL    │
                  │   (Supabase /   │
                  │    local PG16)  │
                  └────────┬────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │ Job Worker      │
                  │ (background)    │
                  │                 │
                  │ ▶ claim_job     │
                  │ ▶ detect_signals│
                  │ ▶ complete_job  │
                  └─────────────────┘
```

## License

MIT
