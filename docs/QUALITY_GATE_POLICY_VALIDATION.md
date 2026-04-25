# Quality Gate Policy Validation Runbook

Updated: 2026-04-24

## Objective

Validate end-to-end behavior of `enforcement.quality_gates` in `/policy/check`:

- `warn`: pipeline continues, warnings are returned.
- `block`: policy denies when Sonar gate is not green.
- failed quality gate emits governance evidence (signal) and optional alert webhook.

This runbook is for real environments (GitHub Actions/Jenkins + Control Plane).

## Preconditions

1. SonarQube is configured. For this repo the supported runtime is local SonarQube, not SonarCloud:
   - `SONAR_TOKEN`
   - `SONAR_HOST_URL`
   - `SONAR_PROJECT_KEY`
   - `GITGOV_URL`
   - `GITGOV_API_KEY`
   - GitHub-hosted runners cannot reach `localhost:9000`; use Jenkins/local validation or a self-hosted runner when SonarQube local must run in CI.
2. Sonar telemetry is reaching GitGov via `/integrations/jenkins`.
3. Jenkins uses the current `Jenkinsfile`:
   - `Sonar Scan (Optional)` enabled when `SONAR_TOKEN` + `SONAR_PROJECT_KEY` exist.
   - If `SONAR_TOKEN` env is absent, credential fallback uses Jenkins Secret Text id `gitgov-token`.
   - `Policy Check (Advisory)` parses JSON response from `/policy/check`.
4. You have an admin API key for policy override/check.
5. (Optional) `GITGOV_ALERT_WEBHOOK_URL` configured if you want alert delivery validation.
6. Use URL-encoded repo path for policy endpoints:
   - repo full name: `<owner>/<repo>`
   - encoded path segment: `<owner>%2F<repo>`

## 1) Set `quality_gates=warn`

```bash
curl -sS -X PUT "http://127.0.0.1:3001/policy/<repo_full_name_urlencoded>/override" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "branches": { "patterns": ["feat/*","fix/*"], "protected": ["main"] },
    "groups": {},
    "admins": [],
    "rules": {
      "require_pull_request": true,
      "min_approvals": 1,
      "require_conventional_commits": true,
      "require_signed_commits": false,
      "max_files_per_commit": null,
      "require_linked_ticket": false,
      "block_force_push": true,
      "forbidden_patterns": []
    },
    "checklist": { "confirm": [], "auto_check": [] },
    "enforcement": {
      "pull_requests": "warn",
      "commits": "warn",
      "branches": "warn",
      "traceability": "off",
      "quality_gates": "warn"
    }
  }'
```

## 2) Run policy check on commit with failed Sonar gate

```bash
curl -sS -X POST "http://127.0.0.1:3001/policy/check" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "repo": "<owner>/<repo>",
    "branch": "main",
    "commit": "<commit_sha_with_failed_sonar>",
    "user_login": "jenkins"
  }'
```

Expected:

- `"allowed": true`
- `"advisory": true`
- `warnings` includes quality gate message.
- `violations` contains `rule = "quality_gate_green"`.

## 3) Switch to `quality_gates=block`

Set the same policy but:

- `"quality_gates": "block"`

Run `/policy/check` again with same failing commit.

Expected:

- `"allowed": false`
- `"advisory": false` (if any enforcement level is `block`)
- `reasons` includes quality gate message.

## 3.1) Governed exception for temporary quality-gate downgrade

When you need a temporary downgrade (`block -> warn` or `warn -> off`), use governed payload:

```bash
curl -sS -X PUT "http://127.0.0.1:3001/policy/<repo_full_name_urlencoded>/override" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "enforcement": {
        "quality_gates": "warn"
      }
    },
    "quality_gate_exception": {
      "reason": "Hotfix release window",
      "ticket_id": "OPS-777",
      "approved_by": "security-admin",
      "expires_at": 1760000000000
    }
  }'
```

Rules:

- Downgrade without active exception is rejected (`400`).
- `quality_gate_exception.reason` is required.
- `quality_gate_exception.expires_at` must be future and <= 30 days.
- Exception metadata is persisted in policy + admin audit log (`policy_override` metadata).

Behavior in `policy/check` while exception is active:

- Non-green quality gate is marked as violation with `enforcement = "override"`.
- Response stays allowed (`allowed=true`) and includes warning `allowed by active quality gate exception`.
- Standard quality-gate failure signal/alert is not emitted for that overridden evaluation.

## 4) Validate green commit

Run `/policy/check` with a commit that has Sonar `success`.

Expected:

- `"allowed": true`
- No quality gate violation.

## 5) Jenkins behavior verification

In current `Jenkinsfile`:

- Sonar stage captures quality gate status (`OK/WARN/ERROR`) per commit when configured.
- warnings are logged,
- advisory deny continues only when `GITGOV_STRICT=false`,
- non-advisory deny fails the build.

Quick check:

1. Trigger one build with `quality_gates=warn` and failed gate evidence.
2. Trigger one build with `quality_gates=block` and failed gate evidence.

Expected:

- First build continues (unless strict mode true).
- Second build fails in policy stage.

## 6) Signal and alert verification (SQ-06 phase 1)

After running `/policy/check` against a failing quality gate commit:

Expected:

- A `noncompliance_signal` is created with:
  - `signal_type = "policy_violation"`
  - `evidence.rule = "quality_gate_green"`
  - `evidence` includes repo, commit, job, gate status, enforcement.
- Duplicate spam is avoided for same commit/repo (dedup window 24h).

Admin query example:

```bash
curl -sS "http://127.0.0.1:3001/signals?signal_type=policy_violation&limit=20&offset=0" \
  -H "Authorization: Bearer <ADMIN_API_KEY>"
```

If `GITGOV_ALERT_WEBHOOK_URL` is configured:

- Expect one alert payload with message `Quality Gate no verde` including actor/repo/branch/commit/job/status/enforcement.

## Automated matrix runner

Use the script to validate `warn/block` end-to-end and automatically restore policy afterward:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/validate_quality_gate_policy_matrix.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<ADMIN_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main" `
  -FailingCommitSha "<commit_sha_with_failed_quality_gate>" `
  -GreenCommitSha "<commit_sha_with_green_quality_gate>"
```

Outputs:

- markdown evidence report under `docs/reports/quality-gate-policy-matrix-<timestamp>.md`
- non-zero exit when any matrix assertion fails
- original policy restored unless `-LeavePolicyAsIs` is explicitly set

Resolver automático de SHAs (sin pasar commits manuales):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/resolve_quality_gate_matrix_commits.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<ADMIN_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main"
```

El resolver toma:
- commit failing desde correlaciones Sonar (`pipeline.status` no verde) o fallback por señales `policy_violation` (`quality_gate_green`),
- commit green desde correlaciones Sonar (`pipeline.status=success`).

GitHub Actions (cloud, no bloqueante):

- Workflow: `.github/workflows/quality-gate-policy-matrix.yml`
- Trigger: `push/main` + `workflow_dispatch`
- Precheck: auto-skip si faltan `GITGOV_URL` / `GITGOV_API_KEY`
- Artefactos: reporte de matrix + resolución de SHAs por run

## GitHub-hosted Matrix Status (2026-04-24)

The quality gate policy matrix is validated in GitHub-hosted CI against GitGov telemetry. It does not require GitHub-hosted runners to reach local SonarQube directly.

- Workflow file is published on `main`:
  - `.github/workflows/quality-gate-policy-matrix.yml`
- Repository Actions config is present:
  - variable `GITGOV_URL`
  - secret `GITGOV_API_KEY`
- Required check is protected on `main`:
  - `Validate quality_gates warn/block matrix`
- Latest PR validation passed after the runbook update:
  - PR `#14`
  - workflow run `24918516902`
  - job `72975366893`

## Validated Local Evidence

### 2026-04-23 (latest)

Automated run evidence:

- `docs/reports/quality-gate-policy-matrix-auto-local-2026-04-23.md`
- `docs/reports/quality-gate-matrix-commit-resolution-auto-local-2026-04-23.json`

Validated against local Docker stack (`gitgov-server` on `:3001`) with repo
`yohandry10/Git-Gov`:

- Failing Sonar commit: `fd3fb268dc4c34aad9f01aec5e8da3f69017be74`
- Green Sonar commit: `3a5ddde5c616706e52b5c0ed2ff4e587c6863870`

Observed results:

- `quality_gates=warn` + failing commit:
  - `allowed=true`
  - `advisory=true`
  - `violations` includes `quality_gate_green` with `enforcement=warn`
- `quality_gates=block` + failing commit:
  - `allowed=false`
  - `advisory=false`
  - `reasons` includes non-green quality gate message
- `quality_gates=block` + green commit:
  - `allowed=true`
  - no `quality_gate_green` violation

### 2026-04-20 (baseline)

Automated run evidence:

- `docs/reports/quality-gate-policy-matrix-local-2026-04-20.md`

Validated against local Docker stack (`gitgov-server` on `:3001`) with real commits from repo
`<owner>/<repo>`:

- Failing Sonar commit: `fd3fb268dc4c34aad9f01aec5e8da3f69017be74`
- Green Sonar evidence commit: `edca03409724c0c4ed1d49b59f1607c557ca1108` (manual Sonar Jenkins event ingested with `job_name` containing `sonar`)

Observed results:

- `quality_gates=warn` + failing commit:
  - `allowed=true`
  - `advisory=true`
  - `violations` includes `quality_gate_green` with `enforcement=warn`
- `quality_gates=block` + failing commit:
  - `allowed=false`
  - `advisory=false`
  - `reasons` includes non-green quality gate message
- `quality_gates=block` + green-evidence commit:
  - `allowed=true`
  - no `quality_gate_green` violation
- Signal verification:
  - `policy_violation` signal created with `evidence.rule=quality_gate_green`
  - includes repo, commit, job name, gate status, enforcement

## Troubleshooting

- If response says quality check skipped: ensure `commit` is present and that Sonar telemetry was ingested for that SHA.
- If no Sonar rows are visible in dashboard: verify workflow variables/secrets and `/integrations/jenkins` auth.
- If Jenkins fails on transport: verify `GITGOV_URL`, API key, and network access from Jenkins node.
- If signal is missing: verify policy check hit a non-green gate and inspect server logs for `Failed to persist quality gate policy signal`.
- If local Docker backend keeps restarting after rebuilding server: set `GITGOV_ENV=dev` in `docker-compose.yml` (or provide required prod hardening secrets, including `GITHUB_WEBHOOK_SECRET`).
