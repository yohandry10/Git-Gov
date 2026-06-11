-- v30: Content-bound idempotency for inbound webhooks (replay defense).
--
-- GitHub/Jira HMAC signatures cover only the request body, not the
-- `X-GitHub-Delivery` / `X-GitHub-Event` headers. Idempotency keyed on the
-- attacker-controllable `delivery_id` lets a captured, validly-signed payload be
-- replayed with a fresh delivery_id and re-injected as duplicate audit evidence.
--
-- This adds a hash of the signed material (event_type + raw body) so the entry
-- point can dedup replays regardless of the delivery_id. NULL is used for rows
-- predating this migration; a UNIQUE index treats NULLs as distinct, so existing
-- rows never collide.

ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS payload_sha256 TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_webhook_events_payload_sha256
    ON webhook_events (payload_sha256);
