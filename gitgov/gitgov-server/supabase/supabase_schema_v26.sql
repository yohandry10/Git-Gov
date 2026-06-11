-- v26: Scope commit_ticket_correlations uniqueness per organization.
--
-- The original unique index `idx_commit_ticket_unique(commit_sha, ticket_id)`
-- was global across all tenants. When the same commit SHA exists in more than
-- one organization (forks, shared commits, or colliding ticket ids such as
-- `KAN-4` reused per org), the first org to insert a correlation would block any
-- other org from recording its own correlation via `ON CONFLICT DO NOTHING`.
--
-- This migration makes uniqueness `(org_id, commit_sha, ticket_id)`. NULL org_id
-- rows (global correlations) are folded onto a fixed sentinel so they still
-- dedupe with each other instead of being treated as always-distinct.

DROP INDEX IF EXISTS idx_commit_ticket_unique;

CREATE UNIQUE INDEX IF NOT EXISTS idx_commit_ticket_unique_org
    ON commit_ticket_correlations (
        COALESCE(org_id, '00000000-0000-0000-0000-000000000000'::uuid),
        commit_sha,
        ticket_id
    );
