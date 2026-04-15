#!/bin/bash
# governance smoke — multi-session workflow for policy change requests
set -euo pipefail

SERVER_URL="${SERVER_URL:-http://127.0.0.1:3000}"
ADMIN_API_KEY="${ADMIN_API_KEY:-}"
DEV_API_KEY_A="${DEV_API_KEY_A:-}"
DEV_API_KEY_B="${DEV_API_KEY_B:-}"
POLICY_REPO="${POLICY_REPO:-acme/repo}"

check_var() {
  if [ -z "${1:-}" ]; then
    echo "❌ $2"
    exit 1
  fi
}

check_var "$ADMIN_API_KEY" "Missing ADMIN_API_KEY."
check_var "$DEV_API_KEY_A" "Missing DEV_API_KEY_A."
check_var "$DEV_API_KEY_B" "Missing DEV_API_KEY_B."

POLICY_REPO_PATH="${POLICY_REPO//\//%2F}"

PASS=0
FAILED=0

pass() { echo "✅ $1"; PASS=$((PASS + 1)); }
fail() { echo "❌ $1"; FAILED=$((FAILED + 1)); }

extract_field() {
  local field="$1"
  python - <<'PY' "$field"
import json
import sys

payload = json.load(sys.stdin)
print(payload.get(sys.argv[1], ""))
PY
}

contains_request() {
  local bearer="$1"
  local request_id="$2"
  local status="$3"
  local url="$SERVER_URL/policy/$POLICY_REPO_PATH/requests"
  if [ -n "$status" ]; then
    url="${url}?status=${status}"
  fi
  local response
  response=$(curl -sSf -H "Authorization: Bearer $bearer" "$url")
  set +e
  printf '%s' "$response" | python - <<'PY' "$request_id"
import json
import sys

needle = sys.argv[1]
data = json.load(sys.stdin)
for row in data.get("requests", []):
    if row.get("request_id") == needle:
        sys.exit(0)
sys.exit(1)
PY
  local rc=$?
  set -e
  return $rc
}

create_request() {
  local key="$1"
  local label="$2"
  local payload="{\"config\":{},\"reason\":\"governance-smoke-${label}\"}"
  local response
  response=$(curl -sSf -X POST "$SERVER_URL/policy/$POLICY_REPO_PATH/requests" \
    -H "Authorization: Bearer $key" \
    -H "Content-Type: application/json" \
    -d "$payload")

  local request_id
  request_id=$(printf '%s' "$response" | extract_field request_id)
  if [ -z "$request_id" ]; then
    fail "Failed to create policy request for $label."
    exit 1
  fi
  local status
  status=$(printf '%s' "$response" | extract_field status)
  pass "Created request $request_id ($status) as $label."
  echo "$request_id"
}

approve_request() {
  local request_id="$1"
  curl -sSf -X POST "$SERVER_URL/policy/requests/$request_id/approve" \
    -H "Authorization: Bearer $ADMIN_API_KEY" \
    -H "Content-Type: application/json" \
    -d "{}" \
    >/dev/null
  pass "Admin approved request $request_id."
}

list_check() {
  local key="$1"
  local request_id="$2"
  local status="$3"
  local label="$4"
  if contains_request "$key" "$request_id" "$status"; then
    pass "$label sees $request_id (status filter=${status:-none})."
  else
    fail "$label did not see $request_id with status filter=${status:-none}."
  fi
}

echo "========================================"
echo "Governance multi-session smoke"
echo "Server: $SERVER_URL"
echo "Repo:   $POLICY_REPO"
echo "========================================"

REQ_A=$(create_request "$DEV_API_KEY_A" "dev-A")
REQ_B=$(create_request "$DEV_API_KEY_B" "dev-B")

list_check "$DEV_API_KEY_A" "$REQ_A" "" "Developer A (all statuses)"
list_check "$DEV_API_KEY_B" "$REQ_B" "" "Developer B (all statuses)"

list_check "$ADMIN_API_KEY" "$REQ_A" "" "Admin (pending view for A)"
list_check "$ADMIN_API_KEY" "$REQ_B" "" "Admin (pending view for B)"

approve_request "$REQ_A"

list_check "$ADMIN_API_KEY" "$REQ_A" "approved" "Admin (approved filter)"

echo ""
echo "Summary: PASS=$PASS FAIL=$FAILED"
if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
exit 0
