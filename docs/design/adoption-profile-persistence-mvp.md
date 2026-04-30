# Adoption Profile Persistence MVP

Updated: 2026-04-30

Ticket: `KAN-31`

## Goal

Make the `KAN-30` Enterprise Adoption dashboard profile durable per organization.

Before this MVP, the dashboard could build and download a secret-safe adoption pack, but the profile lived only in the current browser session. KAN-31 lets an admin save and reload the customer adoption profile from the GitGov Control Plane.

## API

Authenticated admin routes:

- `GET /enterprise/adoption-profile?org_name={org}`
  - returns `{ found, profile }`.
  - global admin keys must provide `org_name`.
  - org-scoped keys can only read their own org.
- `PUT /enterprise/adoption-profile`
  - body: `{ "org_name": "...", "profile": { ... } }`.
  - returns the saved profile record.
  - writes an admin audit entry without storing secret values in the audit metadata.

## Data Model

Migration: `gitgov/gitgov-server/supabase/supabase_schema_v23.sql`.

Table: `enterprise_adoption_profiles`.

- `org_id`: primary key and foreign key to `orgs(id)`.
- `profile`: validated JSONB adoption profile.
- `updated_by`: admin client id that last saved the profile.
- `created_at` / `updated_at`: server timestamps.

## Validation

The backend validates the same core rules as the UI:

- profile must be a JSON object.
- profile size is capped at `32 KiB`.
- customer name is required.
- repository must look like `owner/repo`.
- default branch is required.
- policy preset must be `audit-only`, `moderate`, or `strict`.
- provider IDs must be known GitGov adoption providers.
- module IDs must be known GitGov adoption modules.
- traceability requires a Jira project key.
- Jira project key must be uppercase letters/numbers.
- at least one provider and one module must be selected.

## UI

The Enterprise Adoption panel now:

- loads the saved org profile when the dashboard opens.
- saves the current profile through the Tauri Control Plane client.
- keeps JSON export available for sharing/adoption pack handoff.
- shows saved timestamp and save/load errors.

## Safety

The persisted profile stores configuration intent only.

It does not store:

- API keys.
- provider tokens.
- webhook secrets.
- `.env` values.
- generated secret values.

Secret handling remains name-only, matching the `KAN-29` adoption pack generator and `KAN-30` JSON export.

## Non-Goals

- No direct provider credential validation yet. `KAN-32` adds a dashboard evidence-based provider health MVP.
- No automatic GitHub workflow installation yet.
- No formal enterprise release approval engine yet.
- No Vercel AI SDK Copilot yet.

## Next Product Steps

1. Provider validation dashboard: implemented as a secret-safe evidence MVP by `KAN-32`; direct provider reachability checks remain future work.
2. Workflow template generation: CLI generation implemented by `KAN-33`; dashboard download implemented by `KAN-34`; reviewed installation remains future work.
3. Formal release approval: approvers, risk acceptance, expiration, evidence binding, and release packet.
4. Vercel AI SDK Copilot: explain adoption readiness, blockers, evidence packets, and security findings in plain language with cited GitGov evidence.
