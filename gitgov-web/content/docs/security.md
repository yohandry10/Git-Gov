---
title: Security Policy
description: What GitGov captures, where it is stored, who can access it, and how your data is protected at every layer.
order: 7
category: Operate
---

GitGov is designed with the principle of **least data, maximum protection**. This page details the full security posture of the platform — what enters the system, how it is stored, who can access it, and what technical controls are in place.

---

## What GitGov Captures

GitGov Desktop captures **operational metadata only** — never source code, file contents, diffs, commit messages, secrets, or `.env` values.

| Data Point | Example | Captured? |
|------------|---------|-----------|
| Event type | `commit`, `push`, `stage_files` | Yes |
| Commit SHA | `a3f8c2e` | Yes |
| Branch name | `feat/auth` | Yes |
| Git author | `alice` | Yes |
| Timestamp | ISO 8601 (stored in UTC) | Yes |
| File count | `12` | Yes |
| File paths | `src/main.rs` | Yes (limited to 500) |
| Repo name | `org/repo` | Yes |
| Client version | `0.1.0` | Yes |
| Event status | `success`, `blocked`, `failed` | Yes |
| Block reason | `protected branch` | Yes (when applicable) |
| **Source code** | — | **Never** |
| **File contents** | — | **Never** |
| **Diff content** | — | **Never** |
| **Commit message body** | — | **Never** |
| **Passwords / secrets** | — | **Never** |
| **.env values** | — | **Never** |

> No source code ever leaves the developer workstation. This is an architectural guarantee, not a configuration option.

---

## Where Data Is Stored

### Control Plane Server

Event records are stored in a **PostgreSQL database** (hosted on Supabase in our managed deployment). The deploying organization controls the database instance and its geographic location.

| Layer | Technology | Details |
|-------|-----------|---------|
| **Database** | PostgreSQL 15+ | Supabase-managed or self-hosted |
| **Connection** | TLS-encrypted pooler | Connection string uses `sslmode=require` |
| **Backups** | Supabase daily backups | Or organization-managed backup policy |
| **Region** | Configurable | Organization selects region at deployment |

### Desktop Client

The desktop app stores a **local JSONL outbox** for offline resilience. Events are queued locally and synced to the server when connectivity is restored. A separate **local SQLite database** stores the audit log for local policy verification.

| Local Storage | Purpose | Protection |
|--------------|---------|------------|
| Outbox (JSONL) | Queued events pending sync | Local filesystem, cleared after successful delivery |
| Audit DB (SQLite) | Local audit log for offline policy checks | OS filesystem permissions |
| API key | Server authentication | Stored in OS keyring (never plaintext files) |
| `gitgov.toml` | Policy configuration | Checked into the repository — no secrets |

---

## Encryption

### In Transit

All communication between GitGov Desktop and the Control Plane is protected by **TLS (HTTPS)** in production environments.

- HTTP is only supported for **local development**.
- Production deployments **must** use HTTPS with a valid certificate.
- Webhook payloads from GitHub, Jenkins, and Jira are validated with **HMAC signatures** or dedicated webhook secrets before processing.
- All API requests require a valid `Authorization: Bearer` token — no anonymous access is possible to protected endpoints.

### At Rest

| Component | Encryption at Rest |
|-----------|-------------------|
| PostgreSQL (Supabase) | AES-256 disk encryption (Supabase default) |
| Database backups | Encrypted at rest per Supabase/provider policy |
| API keys in database | Stored as **SHA-256 hashes** — plaintext never persisted after initial issuance |
| Local outbox (JSONL) | Protected by OS filesystem permissions |
| Local audit DB (SQLite) | Protected by OS filesystem permissions |
| OS keyring (API key) | Protected by OS credential storage (Windows DPAPI, macOS Keychain, Linux Secret Service) |

---

## Who Can Access What

GitGov enforces **role-based access control (RBAC)** at the API level. Every request requires a valid `Authorization: Bearer` token.

| Role | Own Events | All Events | Stats/Dashboard | Integrations | API Key Management | Team & Org Management |
|------|-----------|------------|----------------|-------------|-------------------|-----------------------|
| **Developer** | Read | — | — | — | — | — |
| **Architect** | Read | — | — | — | — | — |
| **PM** | Read | — | — | — | — | — |
| **Admin** | Read | Read | Read | Read/Write | Create/Revoke | Full |

### Access Control Details

- **Developers** can only see their own event records (`GET /logs` is scoped by `user_login`). They cannot see other developers' data, statistics, or integration information.
- **Admins** have full visibility: all events, statistics, dashboard, integrations, compliance signals, API key management, team overview, and organization settings.
- There is **no superuser bypass** — the Admin role is the highest privilege level, and it is still subject to authentication.
- API keys are hashed (SHA-256) before storage. The server never stores or logs plaintext keys.
- **Organization scoping** — all data is scoped to the organization. An admin of one organization cannot see another organization's data.

### Role Assignment

Roles are assigned at API key creation time or when a member is provisioned into an organization. Available roles:

- **Admin** — Full access to all endpoints, integrations, team management, and API key operations.
- **Architect** — Currently same as Developer; reserved for future granular permissions.
- **Developer** — Read access to own events only.
- **PM** — Currently same as Developer; reserved for future reporting-focused access.

---

## Audit Trail Integrity

Event records are **append-only**. The system is architecturally designed to prevent tampering:

- **No UPDATE** — audit event records cannot be modified via the API.
- **No DELETE** — audit event records cannot be deleted via the API.
- **Database triggers** — PostgreSQL triggers enforce append-only at the database level (not just the API layer).
- **Deduplication** — each event carries a unique `event_uuid`; duplicates are rejected and reported back to the client.
- **Export logging** — every data export (`POST /export`) is itself logged as an audit event, creating an immutable chain of custody.
- **Retention enforcement** — audit data has a minimum retention floor of **1,825 days (5 years)**, configurable upward by the organization.

This design supports compliance frameworks including **SOC 2**, **ISO 27001**, and **PCI-DSS** audit trail requirements.

---

## Authentication Security

| Mechanism | Details |
|-----------|---------|
| **API Key hashing** | SHA-256 — server computes hash of the bearer token and looks up by `key_hash` |
| **Key storage (desktop)** | OS keyring — Windows DPAPI, macOS Keychain, Linux Secret Service |
| **Key storage (server)** | Hashed column in PostgreSQL — plaintext is never persisted |
| **Key lifecycle** | Keys can be created, listed, and revoked. Revocation takes effect immediately. |
| **JWT signing** | `GITGOV_JWT_SECRET` — must be a strong, unique secret in production |
| **Webhook validation** | GitHub: HMAC-SHA256 with `X-Hub-Signature-256`; Jenkins: `x-gitgov-jenkins-secret`; Jira: `x-gitgov-jira-secret` |
| **Rate limiting** | Per-route configurable rate limits to prevent abuse |
| **Invitation tokens** | Hashed (SHA-256) before storage; expire after a configurable period |

### Rate Limit Defaults

| Route | Default Limit |
|-------|--------------|
| Event ingestion (`/events`) | 240 req/min |
| Audit stream (`/audit-stream/github`) | 60 req/min |
| Jenkins integration | 120 req/min |
| Jira integration | 120 req/min |
| Admin endpoints (logs, stats, dashboard) | 60 req/min |

All limits are configurable via environment variables.

---

## Network Security

- The Control Plane listens on a configurable address and port (set via `GITGOV_SERVER_ADDR` environment variable).
- Local development uses loopback-only binding to prevent accidental network exposure.
- Production deployments should be placed behind a **reverse proxy** (e.g., Nginx, Caddy) with TLS termination.
- CORS and request body size limits are enforced per integration endpoint.
- Maximum request body sizes are configurable per integration (Jenkins, Jira, audit stream) to prevent abuse.

---

## Organization & Data Isolation

- All data is **scoped by organization** — events, logs, integrations, team members, and API keys belong to a specific org.
- Cross-org access is architecturally impossible: the auth middleware enforces org scoping on every request.
- Organizations are created by an admin and members are added via **direct provisioning** or **invitation tokens**.
- Invitation tokens are hashed before storage and automatically expire after the configured period.
- When a member is disabled, their API keys stop working immediately on the next request.

---

## What GitGov Does NOT Do

This section is critical for setting accurate expectations:

| GitGov Does NOT... | Explanation |
|-------------------|-------------|
| **Read your source code** | Only metadata (SHA, branch, author, timestamp, file count) is captured. |
| **Analyze code quality** | No static analysis, linting, or code review functionality. |
| **Monitor keystrokes or screen** | GitGov only observes Git operations, not developer behavior. |
| **Make HR decisions** | Signals are advisory observations, not disciplinary determinations. |
| **Replace CI/CD** | GitGov traces CI pipelines but does not run builds, tests, or deployments. |
| **Enforce branch protection** | GitGov detects policy violations; it does not block Git operations. |
| **Store passwords or secrets** | API keys are hashed; no passwords are ever collected. |
| **Access private repositories** | GitGov does not clone, fetch, or read repository content. |
| **Profile individual productivity** | There are no "lines of code" or "commit frequency" performance scores. |
| **Sell or share your data** | Data belongs to the deploying organization. GitGov has no data monetization. |
| **Collect telemetry about your machine** | No hardware, network, or OS profiling beyond what Git metadata contains. |

---

## Data Retention & Right to Erasure

- **Audit retention** — Minimum 1,825 days (5 years) for audit event data. Configurable upward by the organization.
- **Session retention** — Operational session data has a shorter, separately configurable retention period.
- **Right to erasure** — GitGov supports anonymization/deletion of a specific developer's data within an organization scope, compliant with GDPR/LOPD. The erasure endpoint returns 404 for non-existent users (privacy-preserving: indistinguishable from "user not found").
- **Export** — `POST /export` allows authorized users to export events in machine-readable format. Each export is logged as an audit event.

---

## Incident Response

If a security vulnerability is discovered in GitGov:

1. Report it to **security@gitgov.io** with a detailed description.
2. Do not disclose publicly until a fix is available.
3. We aim to acknowledge reports within **48 hours** and provide a remediation timeline within **7 business days**.

---

## Non-Compliance Signals

The Control Plane can generate **signals** — automated flags that indicate a potential deviation from configured governance policies. Examples:

- A `successful_push` to a branch listed as `protected` in `gitgov.toml` by a user not in `admins` or an authorized group.
- A commit outside configured operational hours (if a time-window policy is defined).
- A push without a corresponding Jira ticket reference (if ticket-coverage policy is active).

### Critical: Advisory Nature

> [!IMPORTANT]
> **Signals are computational observations. They are not legal conclusions, HR determinations, or findings of misconduct.**

A signal means: _"a configured rule was triggered based on available metadata."_ It does not establish:

- **Intent** — the developer may have had a legitimate reason.
- **Negligence** — configuration errors or clock skew can produce false positives.
- **Fault** — the signal has no knowledge of context beyond the captured metadata.

GitGov provides **no warranty** — express or implied — as to the accuracy, completeness, or fitness of signal data for any employment, disciplinary, or legal purpose.

---

## Liability Boundaries

### GitGov's responsibility ends at the signal.

| Boundary | GitGov's Role | Deploying Organization's Role |
|----------|--------------|-------------------------------|
| **Data capture** | Captures metadata per the schema above | Must inform developers that monitoring is active |
| **Signal generation** | Flags policy deviations based on configured rules | Responsible for policy configuration accuracy |
| **Signal interpretation** | None — signals carry no judgement | Responsible for human review before any action |
| **Decisions based on signals** | None — GitGov makes no decisions | Assumes full legal responsibility for HR/disciplinary actions |
| **False positives** | Provides tooling to review and dismiss signals | Must not act on unreviewed signals |
| **Data subject rights** | Provides export endpoint (`POST /export`) | Acts as data controller; handles individual DSAR requests |

### Deploying organizations must:

1. **Inform employees** that Git operational metadata is captured before deployment.
2. **Establish legal basis** (legitimate interests, legal obligation, or contract) for processing under applicable data protection law (GDPR Art. 6).
3. **Configure policies accurately** — a misconfigured `gitgov.toml` will produce incorrect signals.
4. **Require human review** before taking any action based on a signal.
5. **Comply with local labor law** — in many jurisdictions, employee monitoring requires works council consultation, individual notice, or regulatory approval.

---

## GDPR Reference

For EU-based deployments:

| GDPR Concept | Implementation |
|-------------|---------------|
| **Data controller** | The deploying organization |
| **Data processor** | GitGov (software + operators) |
| **Legal basis** | Art. 6(1)(b) contract, 6(1)(c) legal obligation, or 6(1)(f) legitimate interests |
| **Data minimization** | Only operational metadata — no content, no diffs |
| **Right of access** | `GET /logs?user_login={user}` scoped to own data for Developer role |
| **Portability** | `POST /export` — full JSON export of event records |
| **Erasure** | Subject to the organization's audit trail retention obligations |

---

## Related

- [**Privacy Policy**](/privacy) — Full legal terms for end-users.
- [**Governance & Policies**](/docs/governance) — Configuring `gitgov.toml` rules.
- [**FAQ**](/docs/faq) — Common questions about data, security, and compliance.
- [**Governance & Policies**](/docs/governance) — Configuring `gitgov.toml` rules.
- [**FAQ**](/docs/faq) — Common questions about data, security, and compliance.
