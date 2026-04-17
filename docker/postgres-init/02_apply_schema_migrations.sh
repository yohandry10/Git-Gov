#!/bin/sh
set -eu

echo "[gitgov-db-init] Applying supabase schema migrations (v*.sql)..."

if [ ! -d /workspace/supabase_migrations ]; then
  echo "[gitgov-db-init] No /workspace/supabase_migrations directory found; skipping."
  exit 0
fi

find /workspace/supabase_migrations -maxdepth 1 -type f -name 'supabase_schema_v*.sql' \
  | sort -V \
  | while IFS= read -r migration; do
      echo "[gitgov-db-init] -> $(basename "$migration")"
      psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "$migration"
    done

