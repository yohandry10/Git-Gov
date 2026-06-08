# GitGov — Deployment Guide

> Guía unificada: Docker local, Render, AWS EC2 self-hosted, Enterprise (instaladores/GPO) y Desktop Updates.
> Última actualización: 2026-04-25

---

## 1. Docker Local (desarrollo/demo)

Setup Docker local para levantar:
- PostgreSQL (`gitgov-db`)
- GitGov Control Plane Server (`gitgov-server`)
- Jenkins (opcional, perfil `jenkins`)
- Jira Software (opcional, perfil `jira`)

No reemplaza tu app Desktop/Tauri local. La idea es correr el **server** en Docker y seguir usando GitGov Desktop como cliente.

### Requisitos

- Docker Desktop ejecutándose
- Puerto `3001` libre (GitGov server Docker)
- Puerto `5433` libre (Postgres Docker)

### Levantar stack

```bash
# Desde la raíz del repo
docker compose up --build -d

# Ver estado
docker compose ps

# Logs
docker compose logs -f gitgov-server
docker compose logs -f gitgov-db
```

### Jenkins (opcional)

```bash
docker compose --profile jenkins up -d jenkins
docker compose logs -f jenkins
# URL: http://localhost:8096
# Password inicial:
docker exec -it gitgov-jenkins cat /var/jenkins_home/secrets/initialAdminPassword
```

Runbook operativo actual: ver `docs/OPERATIONS_ACCESS.md`.

Estado validado local:

- URL: `http://localhost:8096`
- Usuario API local: `admin`
- Job actual: `gitgov-demo-pipeline`
- Acceso API validado con `JENKINS_API_TOKEN` desde `.env` local ignorado.
- Ultimo build observado: `#30`, `SUCCESS`.

Variables locales esperadas:

```env
JENKINS_SERVER_URL=http://localhost:8096
JENKINS_USER=admin
JENKINS_API_TOKEN=...
JENKINS_JOB_NAME=gitgov-demo-pipeline
```

El trigger URL de Jenkins es independiente del API token:

```text
${JENKINS_SERVER_URL}/job/${JENKINS_JOB_NAME}/build?token=${JENKINS_BUILD_TRIGGER_TOKEN}
```

Usar `JENKINS_BUILD_TRIGGER_TOKEN` solo si el job fue configurado con trigger remoto.

### SonarQube local (opcional)

```bash
docker compose --profile sonar up -d sonarqube-db sonarqube
docker compose logs -f sonarqube
# URL: http://localhost:9000
# Login inicial: admin / admin (cambiar password en primer ingreso)
```

Estado validado local:

- URL: `http://localhost:9000`
- Token local creado: `gitgov-local`
- Expiracion: May 22, 2026
- Project key local: `yohandry10_git-gov`
- Acceso API validado con `SONAR_TOKEN` desde `.env` local ignorado.

Variables locales esperadas:

```env
SONAR_HOST_URL=http://localhost:9000
SONAR_TOKEN=...
SONAR_PROJECT_KEY=yohandry10_git-gov
```

Para usar SonarQube local con Jenkins en Docker:
- `SONAR_HOST_URL=http://host.docker.internal:9000` (o `http://sonarqube:9000` si comparte red compose)
- generar token en `My Account > Security` y cargarlo en Jenkins como `SONAR_TOKEN`
- definir `SONAR_PROJECT_KEY` por repo (ej. `<owner>_<repo>`)

Compatibilidad actual del `Jenkinsfile`:
- si `SONAR_TOKEN` no existe como env var, intenta credencial Jenkins `gitgov-token` (Secret Text).
- si `SONAR_PROJECT_KEY` no existe, lo infiere desde el repo (`owner/repo` -> `owner_repo`).
- la telemetría Sonar se publica en el payload Jenkins con:
  - `stages[].name = quality_gate`
  - `stages[].status = OK|WARN|ERROR|SCAN_FAILED|...`
  - `artifacts[]` con URL de dashboard Sonar cuando está disponible.
- release readiness gate opcional integrado:
  - stage `Release Readiness Gate (Optional)` en `Jenkinsfile`.
  - habilitar con `GITGOV_RELEASE_GATE_ENABLED=true`.
  - perfil por tier: `GITGOV_RELEASE_GATE_TIER=critical|standard|internal`.
  - umbral opcional: `GITGOV_RELEASE_GATE_MIN` (`0` usa target del tier).
  - modo estricto de señales: `GITGOV_RELEASE_GATE_FAIL_MISSING=true`.
  - ventana/volumen: `GITGOV_RELEASE_GATE_HOURS`, `GITGOV_RELEASE_GATE_CORRELATION_LIMIT`.
  - resultado se publica como stage `release_readiness` en telemetría Jenkins (`/integrations/jenkins`).

#### Migración SCM de job Jenkins (repo nuevo)

Si el job sigue mostrando en consola un remoto anterior u otro repositorio legado, corrige el SCM manualmente:

1. Jenkins -> abrir job -> **Configurar**.
2. En **Pipeline > Definition: Pipeline script from SCM**:
   - **SCM**: `Git`
   - **Repository URL**: `https://github.com/<owner>/<repo>.git`
   - **Credentials**: seleccionar token/credencial GitHub (si aplica).
   - **Branches to build**: `*/main`
3. Guardar.
4. Ejecutar **Build Now**.
5. Verificar en consola:
   - `Fetching upstream changes from https://github.com/<owner>/<repo>`
   - que no aparezca referencia a repo legacy.

Verificación automática (Jenkins API):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/check_job_repo.ps1 `
  -JenkinsUrl "http://127.0.0.1:8096" `
  -JobName "gitgov-demo-pipeline" `
  -ExpectedRepoUrl "https://github.com/<owner>/<repo>.git" `
  -Username "<JENKINS_USER>" `
  -ApiTokenOrPassword "<JENKINS_API_TOKEN_OR_PASSWORD>"
```

Smoke check de correlación commit -> pipeline (Control Plane):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/validate_commit_pipeline_correlation.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -CommitSha "<COMMIT_SHA_EXISTENTE_EN_PIPELINE>"
```

Si todavía no existe un pipeline para ese SHA, se puede forzar un evento de pipeline de prueba:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/validate_commit_pipeline_correlation.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -CommitSha "<COMMIT_SHA>" `
  -InjectPipelineIfMissing
```

Automatización en GitHub Actions (opcional, no bloqueante):

- Workflow: `.github/workflows/governance-correlation-smoke.yml`
- Trigger: `push` a `main` + `workflow_dispatch`
- Si faltan `GITGOV_URL` o `GITGOV_API_KEY`, el workflow se salta en `PASS` (skip explícito).
- Si `JENKINS_WEBHOOK_SECRET` está activo en backend, configurar `GITGOV_JENKINS_SECRET` (secret opcional) para que el smoke pueda publicar en `/integrations/jenkins`.

Matrix de policy quality gates en cloud (opcional, no bloqueante):

- Workflow: `.github/workflows/quality-gate-policy-matrix.yml`
- Trigger:
  - `push` a `main`
  - `workflow_dispatch` manual (permite override de SHAs)
- Requisitos:
  - Variable: `GITGOV_URL`
  - Secret: `GITGOV_API_KEY`
- Comportamiento:
  - auto-resuelve SHAs failing/green con `scripts/jenkins/resolve_quality_gate_matrix_commits.ps1`
  - valida `warn/block` con `scripts/jenkins/validate_quality_gate_policy_matrix.ps1`
  - sube artefactos de resolución + reporte markdown por run

#### Calibración semanal de riesgo/readiness por tier

Para cerrar el tuning de score con evidencia operativa real, generar baseline semanal por tier:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/calibrate_risk_tier_baseline.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main" `
  -Tier "standard" `
  -Hours 168
```

El script calcula:
- `release_readiness` (0-100),
- `composite_risk` (0-100),
- KPIs base (trusted path, blocked push, traceability gap, pipeline failures, sonar failures, unresolved violations),
- brechas contra SLA del tier seleccionado.

Salida:
- Reporte markdown en `docs/reports/risk-tier-baseline-<timestamp>.md`.

Automatización en GitHub Actions:

- Workflow: `.github/workflows/risk-tier-baseline-calibration.yml`
- Trigger:
  - `schedule` semanal: lunes 12:00 UTC
  - `workflow_dispatch` manual (inputs: `tier`, `org_name`, `repo_full_name`, `branch`, `hours`, `correlation_limit`)
- Requisitos:
  - Variable: `GITGOV_URL`
  - Secret: `GITGOV_API_KEY`
- Comportamiento sin configuración:
- El job hace `skip` explícito (no rompe CI) cuando faltan `GITGOV_URL`/`GITGOV_API_KEY`.

Lock y validación SLO por dominio:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/validate_domain_slo_targets.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main" `
  -TargetsPath "ops/slo/domain-slo-targets.json" `
  -OutputDir "docs/reports/domain-slo-validation-local"
```

Salida:
- `docs/reports/domain-slo-validation-<timestamp>/domain-slo-summary.md`
- baseline por dominio: `domain-<name>-baseline.md`

Workflow:
- `.github/workflows/domain-slo-validation.yml`
- Trigger:
  - `schedule` semanal: lunes 12:45 UTC
  - `workflow_dispatch` manual (inputs: `domain_name`, `repo_full_name`, `branch`, `hours`, `correlation_limit`, `fail_on_breach`)
- Requisitos:
  - Variable: `GITGOV_URL`
  - Secret: `GITGOV_API_KEY`
- Fuente de lock:
  - `ops/slo/domain-slo-targets.json` (targets por dominio/tier, con `repo_full_name` y `branch` cuando el dominio debe filtrar evidencia por repo/rama).

#### Gate de release readiness por rama (SQ-10 fase 2)

Validación ejecutable para bloquear/promover release por score de readiness en una rama específica.

Ejecución local:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/validate_release_readiness_gate.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main" `
  -Tier "standard"
```

Modo estricto (falla si falta cualquier señal):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/jenkins/validate_release_readiness_gate.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -ApiKey "<GITGOV_API_KEY>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main" `
  -Tier "critical" `
  -FailOnMissingSignals
```

Automatización GitHub Actions:

- Workflow: `.github/workflows/release-readiness-gate.yml`
- Trigger:
  - `push` a `main`
  - `workflow_dispatch` manual (tier/branch/repo/target/strict)
- Requisitos:
  - Variable: `GITGOV_URL`
  - Secret: `GITGOV_API_KEY`
- Salida:
  - Artifact JSON `release-readiness-gate-<run_id>.json` con score, cobertura de señales, métricas y razones de fallo.

#### Branch protection (checks requeridos en GitHub)

Para evitar merges sin controles activos, aplicar branch protection en `main` con checks requeridos.

Estado verificado por `KAN-73`: la protección real de `main` usa status checks estrictos y requiere exactamente estos checks:

- `Security Guard`
- `Server Clippy + Check`
- `Desktop Rust Clippy`
- `Frontend Lint + Typecheck`
- `Website Lint + Typecheck + Build`
- `Validate quality_gates warn/block matrix`

`Workflow Lint` y `Block internal-assistant markers in branch/commits` siguen ejecutándose en PR/push, pero no son contextos requeridos por branch protection en la regla actual de `main`.

Script automático (usa API de GitHub):

```powershell
$requiredChecks = @(
  "Security Guard",
  "Server Clippy + Check",
  "Desktop Rust Clippy",
  "Frontend Lint + Typecheck",
  "Website Lint + Typecheck + Build",
  "Validate quality_gates warn/block matrix"
)

powershell -ExecutionPolicy Bypass -File scripts/github/set_required_checks.ps1 `
  -GitHubToken "<TOKEN_CON_ADMIN_ON_REPO>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main" `
  -RequiredChecks $requiredChecks
```

Validación rápida:

1. GitHub -> `Settings` -> `Branches` -> regla de `main`.
2. Confirmar:
   - `Require a pull request before merging` activo.
   - `Require status checks to pass before merging` activo con los checks listados.
   - `Do not allow bypassing the above settings` para admins (enforce admins).

Validación automática (API):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_branch_protection.ps1 `
  -GitHubToken "<TOKEN_CON_PERMISOS_REPO_ADMIN_READ>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main"
```

Orquestador único (setup + validación):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/harden_repo_governance.ps1 `
  -GitHubToken "<TOKEN_CON_PERMISOS_REPO_ADMIN>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main" `
  -ApplyBranchProtection
```

Helper para PR (crea PR por API o imprime URL de compare si el token no tiene permiso `pull_requests`):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/create_or_print_pr.ps1 `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Base "main" `
  -Head "feature/governance-hardening" `
  -Title "feat: governance hardening bundle"
```

El orquestador corre preflight de permisos del token al inicio (`check_token_permissions.ps1`).
Si necesitas omitirlo explícitamente: `-SkipTokenPermissionsCheck`.
Si trabajas con un token limitado (sin `Administration` o sin lectura de `Actions secrets/variables`), usa `-BestEffort` para continuar en modo diagnóstico y que el flujo no se detenga.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/harden_repo_governance.ps1 `
  -GitHubToken "<TOKEN_FINE_GRAINED_LIMITADO>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main" `
  -BestEffort
```

Opcional para repositorio con único mantenedor (evita bloqueo de merge por auto-aprobación):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/harden_repo_governance.ps1 `
  -GitHubToken "<TOKEN_CON_PERMISOS_REPO_ADMIN>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main" `
  -ApplyBranchProtection `
  -RequiredApprovals 0
```

### Jira (opcional)

```bash
docker compose --profile jira up -d jira
docker compose logs -f jira
# URL: http://localhost:8095
```

### Qué inicializa automáticamente

Al crear el volumen de Postgres por primera vez, Docker ejecuta:
1. `gitgov/gitgov-server/supabase/supabase_schema.sql`
2. todas las migraciones `gitgov/gitgov-server/supabase/supabase_schema_v*.sql` en orden numérico

Si ya existe el volumen, los scripts **no** se vuelven a ejecutar.

### URLs y credenciales (dev local)

| Recurso | Valor |
|---------|-------|
| Server Docker | `http://127.0.0.1:3001` |
| API Key admin (dev) | `<YOUR_API_KEY>` |
| PostgreSQL host | `localhost:5433` |
| PostgreSQL db/user | `gitgov` / `gitgov` |
| PostgreSQL password | `gitgov_dev_password` |

Nota de runtime local:

- En `docker-compose.yml`, `gitgov-server` debe correr con `GITGOV_ENV=dev`.
- El binario release por defecto asume hardening no-dev; sin ese ajuste exige secretos de producción (por ejemplo `GITHUB_WEBHOOK_SECRET`) y reinicia el contenedor.

### Integrar con Desktop App

En la configuración del Control Plane:
- URL: `http://127.0.0.1:3001` (server Docker)
- API Key: `<YOUR_API_KEY>`

> **Golden Path diario (server local nativo):** usar `http://127.0.0.1:3000` para evitar split-brain.

### Reset de base local

```bash
docker compose down -v
docker compose up --build -d
```

### Probar endpoints

```bash
curl http://127.0.0.1:3001/health
curl -H "Authorization: Bearer <YOUR_API_KEY>" http://127.0.0.1:3001/stats
```

### Migraciones adicionales recomendadas (governance/drift v2)

El bootstrap Docker ejecuta automáticamente `supabase_schema.sql` + todas las migraciones `supabase_schema_v*.sql` cuando el volumen se crea desde cero.
Para usar toda la superficie reciente (drift audit + policy requests + timeline compliance), aplicar también migraciones adicionales disponibles en repo:

```bash
# Desde la raíz del repo
for v in 7 8 9 10 11 12 13 18 19 20 21 22; do
  cat "gitgov/gitgov-server/supabase/supabase_schema_v${v}.sql" \
    | docker exec -i gitgov-db psql -U gitgov -d gitgov
done
```

> Nota: `v14..v17` no existen en el árbol actual del repo; no deben incluirse en el loop.

---

## 2. Producción actual: Render + Supabase

### Estado productivo actual (validado)

- Backend primario: Render service `gitgov-api`.
- URL pública: `https://gitgov-api.onrender.com`.
- Render service ID: `srv-d7lgtc77f7vs73b38uqg`.
- Runtime: Docker web service.
- Región: Oregon.
- Rama de deploy: `main`.
- Root directory: `gitgov/gitgov-server`.
- GitHub webhook activo: `https://gitgov-api.onrender.com/webhooks/github` (ID `610772988`).
- Jira webhook nativo activo: `https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10`.
- HTTPS ya lo provee Render para la producción actual.
- Dominio propio y `certbot` solo aplican si se migra a una ruta self-hosted o dominio custom.

### Arquitectura legacy/self-hosted (AWS EC2)

- EC2 Ubuntu 22.04
- Nginx como reverse proxy
- systemd para el backend
- Supabase como PostgreSQL remoto

### Perfil objetivo 250 simultáneos (sin tocar UI)

Topología recomendada:
- **3 instancias** `gitgov-server` (mismo build) detrás de un balanceador L7 (Nginx upstream o ALB).
- URL pública única para Desktop y APIs de Governance/Settings.
- Supabase PostgreSQL compartido (o PostgreSQL 16 self-hosted/RDS).

Contrato operativo:
- No cambiar contratos HTTP de `/events`, `/logs`, `/stats`, `/chat/ask`, `/sse`.
- Mantener Golden Path: Desktop -> `/events` -> DB -> Governance/Action Center.

Ejemplo Nginx upstream (3 nodos backend):

```nginx
upstream gitgov_backend {
    least_conn;
    server 127.0.0.1:3000 max_fails=3 fail_timeout=10s;
    server 127.0.0.1:3002 max_fails=3 fail_timeout=10s;
    server 127.0.0.1:3003 max_fails=3 fail_timeout=10s;
}

server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://gitgov_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        proxy_buffering off; # SSE
        proxy_read_timeout 75s;
    }
}
```

Variables recomendadas por instancia (perfil inicial 250):

```env
GITGOV_DB_MAX_CONNECTIONS=30
GITGOV_DB_MIN_CONNECTIONS=6
GITGOV_DB_ACQUIRE_TIMEOUT_SECS=12
GITGOV_RATE_LIMIT_EVENTS_PER_MIN=1500
GITGOV_RATE_LIMIT_AUDIT_STREAM_PER_MIN=300
GITGOV_RATE_LIMIT_JENKINS_PER_MIN=300
GITGOV_RATE_LIMIT_JIRA_PER_MIN=300
GITGOV_RATE_LIMIT_GITHUB_WEBHOOK_PER_MIN=600
GITGOV_RATE_LIMIT_ORG_INVITATION_PER_MIN=240
GITGOV_RATE_LIMIT_ADMIN_PER_MIN=240
GITGOV_RATE_LIMIT_LOGS_PER_MIN=900
GITGOV_RATE_LIMIT_STATS_PER_MIN=900
GITGOV_RATE_LIMIT_CHAT_PER_MIN=180
GITGOV_CHAT_LLM_MAX_CONCURRENCY=8
GITGOV_CHAT_LLM_QUEUE_TIMEOUT_MS=1200
GITGOV_CHAT_LLM_TIMEOUT_MS=12000
GITGOV_ORG_LOOKUP_CACHE_TTL_MS=30000
GITGOV_REPO_LOOKUP_CACHE_TTL_MS=30000
GITGOV_REPO_UPSERT_MIN_INTERVAL_MS=30000
GITGOV_CACHE_INVALIDATION_MIN_INTERVAL_MS=120
GITGOV_STATS_CACHE_INVALIDATION_MIN_INTERVAL_MS=5000
GITGOV_LOGS_CACHE_INVALIDATION_MIN_INTERVAL_MS=500
GITGOV_CLIENT_SESSION_UPSERT_MIN_INTERVAL_MS=15000
GITGOV_SSE_MAX_CONNECTIONS=120
GITGOV_SSE_DISTRIBUTED_ENABLED=true
GITGOV_SSE_DISTRIBUTED_CHANNEL=gitgov_sse_events
GITGOV_RATE_LIMIT_DISTRIBUTED_DB=true
GITGOV_RATE_LIMIT_DISTRIBUTED_PRUNE_INTERVAL_SECS=60
GITGOV_RATE_LIMIT_DISTRIBUTED_RETENTION_SECS=3600
GITGOV_OUTBOX_SERVER_LEASE_ENABLED=true
GITGOV_OUTBOX_SERVER_LEASE_TTL_MS=2000
```

Notas:
- `GITGOV_SSE_DISTRIBUTED_ENABLED=true` habilita fan-out cross-node en `/sse` vía PostgreSQL `NOTIFY`.
- `GITGOV_RATE_LIMIT_DISTRIBUTED_DB=true` evita inconsistencia de cuotas cuando hay múltiples nodos.
- `GITGOV_ORG_LOOKUP_CACHE_TTL_MS` y `GITGOV_REPO_LOOKUP_CACHE_TTL_MS` reducen round-trips de lookups repetidos en `/events`.
- `GITGOV_REPO_UPSERT_MIN_INTERVAL_MS` mueve alta cardinalidad de `upsert_repo` fuera del camino síncrono de `/events` (debounced background).
- `GITGOV_CACHE_INVALIDATION_MIN_INTERVAL_MS` reduce churn de lock/cache en ráfagas de `/events`.
- `GITGOV_CLIENT_SESSION_UPSERT_MIN_INTERVAL_MS` evita escrituras redundantes a `client_sessions` por cada request.
- Política de degradación: bajo presión se degrada primero `chat` (`429`), preservando `/events` `/logs` `/stats`.

### Preflight de despliegue

Antes de arrancar EC2 o re-aplicar webhooks, valida el entorno con:

```
powershell -ExecutionPolicy Bypass -File scripts/check_deploy_env.ps1
```

El script verifica `GITGOV_SERVER_ADDR`, `GITGOV_API_KEY`, `GITHUB_WEBHOOK_SECRET`, `SUPABASE_URL` y al menos un par válido de credenciales AWS/Accesskey. Devuelve `PASS` (exit 0) si todo está listo y `FAIL` (exit 1) cuando falta algo, detallando las claves faltantes. Después de rotar credenciales actualiza `gitgov/gitgov-server/.env` y vuelve a ejecutarlo antes de lanzar `cargo run`.

### Perfil productivo validado (single-node, 2026-03-15)

Configuración actualmente validada en EC2 `t3.small` con PostgreSQL local:

```env
DATABASE_URL=postgresql://gitgov:<password>@127.0.0.1:5432/gitgov
GITGOV_DB_MAX_CONNECTIONS=60
GITGOV_DB_MIN_CONNECTIONS=10
GITGOV_RATE_LIMIT_EVENTS_PER_MIN=60000
GITGOV_RATE_LIMIT_AUDIT_STREAM_PER_MIN=12000
GITGOV_RATE_LIMIT_JENKINS_PER_MIN=12000
GITGOV_RATE_LIMIT_JIRA_PER_MIN=12000
GITGOV_RATE_LIMIT_GITHUB_WEBHOOK_PER_MIN=24000
GITGOV_RATE_LIMIT_ORG_INVITATION_PER_MIN=6000
GITGOV_RATE_LIMIT_ADMIN_PER_MIN=3000
GITGOV_RATE_LIMIT_LOGS_PER_MIN=6000
GITGOV_RATE_LIMIT_STATS_PER_MIN=6000
GITGOV_RATE_LIMIT_CHAT_PER_MIN=6000
GITGOV_STATS_CACHE_TTL_MS=15000
GITGOV_LOGS_CACHE_TTL_MS=10000
GITGOV_AUTH_CACHE_TTL_SECS=120
GITGOV_CACHE_INVALIDATION_MIN_INTERVAL_MS=5000
GITGOV_STATS_CACHE_INVALIDATION_MIN_INTERVAL_MS=15000
GITGOV_LOGS_CACHE_INVALIDATION_MIN_INTERVAL_MS=500
GITGOV_CORS_ALLOW_ANY=true
```

Resultados certificados con esa configuración:
- Stress (`think_ms=120`): pasa `250` usuarios simultáneos en pruebas de 60s.
- Realista (`think_ms=2000`): pasa `100` usuarios simultáneos con amplio margen.

### Decisiones operativas

- **DB**: Supabase (PostgreSQL) en producción. EC2 + PG16 local es ruta legacy/self-hosted validada.
- **No subir Desktop a AWS**: Tauri se distribuye como instalador.
- **Render**: ruta actual para el backend productivo.
- **EC2 + Nginx + systemd**: ruta legacy/self-hosted documentada para migraciones o instalaciones propias.
- **Webhooks**: ya están activos en Render; si se migra a self-hosted, mover targets a la nueva URL pública HTTPS.

### Estado legacy/self-hosted validado (2026-03-15)

- EC2 creada y accesible por SSH
- Elastic IP asignada
- Security Group: `22` (IP operador), `80`, `443`
- `gitgov-server` corriendo como systemd
- Nginx proxy hacia `127.0.0.1:3000`
- Endpoints validados: `/health`, `/stats` con Bearer
- Fuente de despliegue activa en EC2: `/home/ubuntu/GitGov-deploy` (alineada a `origin/main`)
- Repo legacy archivado para evitar drift operativo: `/home/ubuntu/GitGov-legacy-20260315-074028`

### URLs legacy de ejemplo (self-hosted)

- Público (HTTP): `http://<ec2-public-host>`
- Health: `http://<ec2-public-host>/health`

### Estructura en EC2

| Path | Propósito |
|------|-----------|
| `/opt/gitgov/bin/gitgov-server` | Binario |
| `/opt/gitgov/config/gitgov-server.env` | Variables de entorno |
| `/etc/systemd/system/gitgov-server.service` | Servicio systemd |
| `/etc/nginx/sites-available/gitgov` | Nginx site |

### Variables de entorno requeridas

Archivo: `/opt/gitgov/config/gitgov-server.env`

- `DATABASE_URL` — PostgreSQL (local `127.0.0.1` o remoto con SSL según topología)
- `GITGOV_JWT_SECRET`
- `GITGOV_ALLOW_INSECURE_JWT_FALLBACK=false` (recomendado; solo usar `true` en dev/test local)
- `GITGOV_API_KEY`
- `GITGOV_SERVER_ADDR=0.0.0.0:3000`
- `RUST_LOG=info`
- `GITHUB_WEBHOOK_SECRET`
- `JENKINS_WEBHOOK_SECRET` (opcional)
- `JIRA_WEBHOOK_SECRET` (opcional)
- `GITGOV_ALERT_WEBHOOK_URL` (opcional, alertas genéricas)
- `GITGOV_DRIFT_ALERT_WEBHOOK_URLS` (opcional, webhooks dedicados para drift crítico)
- `GITGOV_POLICY_CHECK_BLOCK_SCOPES` (opcional, CSV `org:branch_glob`, activa `409` en `/policy/check` cuando `allowed=false`)

> Permisos recomendados del archivo: `root:gitgov` + `640`. No guardar en Git.

### Operación

```bash
# Backend
sudo systemctl status gitgov-server --no-pager
sudo systemctl restart gitgov-server
sudo journalctl -u gitgov-server -f

# Nginx
sudo systemctl status nginx --no-pager
sudo nginx -t
sudo systemctl restart nginx
```

### Validación rápida

```bash
# Desde EC2
curl http://127.0.0.1:3000/health
curl http://127.0.0.1/health

# Desde equipo local
curl http://<ec2-public-host>/health
curl -H "Authorization: Bearer <API_KEY>" http://<ec2-public-host>/stats
```

### Orden de validación post-deploy

1. Smoke tests: `/health`, `/stats` (Bearer), logs del servicio
2. Golden Path Desktop: stage → commit → push → logs/commits
3. Jenkins: `/integrations/jenkins` + Pipeline Health
4. GitHub/Jira webhooks: validar deliveries contra Render o contra la URL pública HTTPS self-hosted configurada

### Gate de capacidad 250 simultáneos (runtime)

Precondición:
- Gate 0 en verde (`make smoke` + checklist Golden Path).

Gate 1 (single-node hardening):

```bash
cd gitgov/gitgov-server
python tests/perf_baseline_control_plane.py --server-url http://127.0.0.1:3000 --out-json tests/artifacts/perf_gate1.json
python tests/chat_capacity_test.py --server-url http://127.0.0.1:3000 --scenario mixed --out-json tests/artifacts/chat_gate1.json
```

Gate 2 (3 nodos + limiter distribuido + SSE distribuido):

```bash
cd gitgov/gitgov-server
make capacity-mixed \
  SERVER_URL=http://127.0.0.1:3000 \
  API_KEYS_FILE=tests/api_keys.txt \
  CAPACITY_USERS=250 \
  CAPACITY_DURATION_SEC=1200 \
  CAPACITY_OUT=tests/artifacts/capacity_mixed_250_gate2.json

make capacity-soak \
  SERVER_URL=http://127.0.0.1:3000 \
  API_KEYS_FILE=tests/api_keys.txt \
  CAPACITY_USERS=250 \
  CAPACITY_SOAK_DURATION_SEC=3600 \
  CAPACITY_SOAK_OUT=tests/artifacts/capacity_mixed_250_soak.json
```

Criterios de salida obligatorios:
- Core (`/events`, `/logs`, `/stats`):
  - `401 = 0`
  - `5xx = 0`
  - `429 < 2%`
  - `p95 < 800ms`
  - `p99 < 1500ms`
- Chat (`/chat/ask`):
  - `5xx = 0`
  - puede degradar en `429` sin afectar SLO del core.

Rollout sugerido:
1. Canary 10% por 30 min.
2. 50% por 60 min.
3. 100% si se cumplen SLO/gates.

Rollback inmediato:
- Volver a 1 nodo backend.
- Desactivar limiter distribuido:
  - `GITGOV_RATE_LIMIT_DISTRIBUTED_DB=false`
- Mantener URL canónica de Desktop (`http://127.0.0.1:3000` en local).

### Runbook post-deploy (governance v2)

Validar en este orden para confirmar que policy workflow, drift audit y export de compliance siguen operativos:

```bash
# 1) Health + stats (admin key)
curl http://127.0.0.1:3000/health
curl -H "Authorization: Bearer <ADMIN_API_KEY>" http://127.0.0.1:3000/stats

# 2) Crear policy change request (developer o admin)
# repo path debe ir URL-encoded (ej: <owner>%2F<repo>)
curl -X POST "http://127.0.0.1:3000/policy/<repo_full_name_urlencoded>/requests" \
  -H "Authorization: Bearer <DEV_OR_ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"config":{"branches":{"protected":["main"],"patterns":["feat/*"]},"rules":{"require_pull_request":true},"enforcement":{"pull_requests":"warn","commits":"off","branches":"warn","traceability":"off","quality_gates":"warn"}},"reason":"post-deploy check"}'

# 3) Aprobar/rechazar request (admin)
curl -X POST "http://127.0.0.1:3000/policy/requests/<REQUEST_ID>/approve" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"note":"post-deploy approval check"}'

# 4) Ingesta drift snapshot crítica (auth)
curl -X POST "http://127.0.0.1:3000/policy/drift-events" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"action":"drift_snapshot","repo_name":"<owner>/<repo>","result":"observed","metadata":{"drift_count":2,"critical_count":1}}'

# 5) Export compliance v2 (admin)
curl -X POST "http://127.0.0.1:3000/export" \
  -H "Authorization: Bearer <ADMIN_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"export_type":"events_csv"}'
```

Resultado esperado:
- Paso 2: `{"accepted":true,"status":"pending",...}`
- Paso 3: `{"status":"approved",...}` o `{"status":"rejected",...}`
- Paso 4: `{"accepted":true,...}`
- Paso 5: CSV con filas `policy_drift` y `policy_change_request` cuando existen datos.

### Estado de dominio/HTTPS/webhooks

- Producción actual en Render no tiene pendiente dominio/HTTPS para operar: `https://gitgov-api.onrender.com` está activo.
- GitHub webhook ya está configurado contra `https://gitgov-api.onrender.com/webhooks/github`.
- Jira webhook nativo ya está configurado contra `https://gitgov-api.onrender.com/webhooks/jira?org_name=yohandry10`.
- Dominio custom, A record, Nginx y `certbot` solo son necesarios si se decide mover producción a self-hosted o exponer un dominio propio.
- Si se cambia la URL pública, actualizar los targets de GitHub/Jira, alinear secretos HMAC y ejecutar la validación pública de esta sección.

### Validación automática de dominio/HTTPS/webhooks

Script de preflight público:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/deploy/validate_public_infra.ps1 `
  -BaseUrl "https://<tu-dominio>" `
  -ApiKey "<GITGOV_API_KEY_ADMIN>" `
  -ExpectedIp "<ELASTIC_IP_OPCIONAL>" `
  -OutputPath "docs/reports/public-infra-validation-<fecha>.md"
```

Checks incluidos:
- resolución DNS del host
- certificado TLS (expiración y handshake)
- `GET /health`
- `GET /stats` autenticado (si se pasa `-ApiKey`)
- reachability de rutas de integración:
  - `/webhooks/github`
  - `/integrations/jenkins`
  - `/integrations/jira`

Resultado:
- `PASS`: listo para continuar con hardening productivo
- `WARN`: hay observaciones no bloqueantes
- `FAIL`: hay fallo crítico (DNS/HTTPS/health/rutas)

### Bundle unificado de readiness

Ejecuta todas las validaciones clave en un solo comando (infra pública, updater, matrix quality gates, baseline tier y prechecks GitHub):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/deploy/run_enterprise_readiness_bundle.ps1 `
  -GitGovUrl "http://127.0.0.1:3001" `
  -PublicBaseUrl "https://<tu-dominio>" `
  -RepoFullName "<owner>/<repo>" `
  -Branch "main"
```

Salida:
- carpeta `docs/reports/readiness-bundle-<timestamp>/`
- reporte principal `readiness-bundle-summary.md`

### Reporte ejecutivo de evidencia GitHub

Genera un artefacto Markdown independiente con la misma cobertura ejecutiva GitHub usada por el dashboard y el paquete de export:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_report.ps1 `
  -GitGovUrl "https://<gitgov-api-host>" `
  -ApiKey "<GITGOV_API_KEY_ADMIN>" `
  -OrgName "<org>" `
  -OutputPath "docs/reports/github-evidence-executive-report-<fecha>.md"
```

Para validar el render del reporte sin tokens:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_report.ps1 `
  -StatsJsonPath "path/to/stats-fixture.json" `
  -OutputPath "docs/reports/github-evidence-executive-report-fixture.md"
```

El reporte calcula `Completo`, `Parcial` o `Sin evidencia` sobre cuatro familias:

- PR lifecycle: `pull_request`
- Reviews: `pull_request_review`
- Comentarios PR: `pull_request_review_comment` + `issue_comment`
- Checks/status: `check_run` + `check_suite` + `status`

Workflow opcional:

- `.github/workflows/github-evidence-report.yml`
- corre lunes 13:23 UTC + `workflow_dispatch`
- requiere `GITGOV_URL` como Actions variable y `GITGOV_API_KEY` como Actions secret
- sube artifact `github-evidence-executive-report`
- salta sin fallar si falta configuración

Runbook operativo completo: `docs/runbooks/github-evidence-operations.md`.

Monitor operativo del artifact:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/validate_github_evidence_report_artifact.ps1 `
  -Repository "owner/repo" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxAgeHours 192 `
  -OutputPath "out/github-evidence-artifact-monitor.json"
```

- requiere `GITHUB_TOKEN` con lectura de Actions
- valida el último run exitoso del workflow de reporte
- falla si el artifact no existe, está expirado o supera la edad máxima
- `.github/workflows/github-evidence-artifact-monitor.yml` corre martes 14:07 UTC + `workflow_dispatch`
- sube artifact `github-evidence-artifact-monitor` con el resumen JSON del monitoreo

Trend histórico de artifacts:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_trend_report.ps1 `
  -Repository "owner/repo" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxReports 12 `
  -OutputMarkdownPath "out/github-evidence-trend-report.md" `
  -OutputJsonPath "out/github-evidence-trend-report.json"
```

- requiere `GITHUB_TOKEN` con lectura de Actions
- descarga artifacts recientes del workflow de reporte
- parsea `Status`, `Coverage` y `Missing signals` de cada Markdown
- genera Markdown + JSON para comparar cobertura entre runs
- `.github/workflows/github-evidence-trend-report.yml` corre martes 14:17 UTC + `workflow_dispatch`
- sube artifact `github-evidence-trend-report`
- La cadencia semanal recomendada y criterios de escalamiento están en `docs/runbooks/github-evidence-operations.md`.

Workflow cloud (manual + semanal):
- `.github/workflows/enterprise-readiness-bundle.yml`
- corre lunes 12:30 UTC + `workflow_dispatch`
- genera artifact `enterprise-readiness-bundle-<run_id>` con:
  - `readiness-bundle-summary.md`
  - `public-infra.md`
  - `desktop-updater.md`
  - `quality-gate-matrix*.{md,json}` (si hay `GITGOV_API_KEY`)
  - `risk-tier-baseline-standard.md` (si hay `GITGOV_API_KEY`)
  - `github-token-permissions.json` / `github-ci-config-precheck.txt` (si hay PAT)

Variables/secrets recomendados para el workflow:
- Variable requerida: `GITGOV_URL`
- Variable opcional: `GITGOV_PUBLIC_BASE_URL`
- Secret opcional: `GITGOV_API_KEY` (habilita matrix+baseline)
- Secret opcional: `GITHUB_PERSONAL_ACCESS_TOKEN` (precheck cloud estricto/best-effort)

### Nota de seguridad

Si una API key fue compartida en chat/capturas, **rotarla**:
1. Generar nueva key
2. Actualizar `GITGOV_API_KEY` en EC2
3. Reiniciar `gitgov-server`
4. Actualizar Desktop/Jenkins

---

## 3. Enterprise Desktop Deployment

### Prerequisites

- Network access to Control Plane server (HTTP/HTTPS, default port 3000)
- API key issued by GitGov admin
- Platform requirements:
  - Windows 10/11 x64 (+ .NET Framework 4.7.2+)
  - macOS 12+ (Apple Silicon / Intel)
  - Linux x64 (glibc-based distro)

### Installer Options

| Format | File | Use case |
|--------|------|----------|
| NSIS (`.exe`) | `GitGov_x.x.x_x64-setup.exe` | Silent install via GPO / Intune / SCCM |
| MSI (`.msi`) | `GitGov_x.x.x_x64_en-US.msi` | Group Policy Software Installation |

Both installers are code-signed. Verify SHA256 hashes from the release page.

### Silent Installation (NSIS)

```
GitGov_x.x.x_x64-setup.exe /S /D=C:\Program Files\GitGov
```

| Flag | Description |
|------|-------------|
| `/S` | Silent mode — no UI |
| `/D=<path>` | Installation directory (must be last, no quotes) |

Uninstall:
```
"C:\Program Files\GitGov\Uninstall GitGov.exe" /S
```

### MSI via Group Policy

```
msiexec /i GitGov_x.x.x_x64_en-US.msi /quiet /norestart INSTALLDIR="C:\Program Files\GitGov"
```

Assign to GPO: Computer Configuration > Software Settings > Software Installation.

### Microsoft Intune

1. Package with `IntuneWinAppUtil.exe`:
   ```
   IntuneWinAppUtil.exe -c . -s GitGov_x.x.x_x64-setup.exe -o ./output
   ```
2. In Intune > Apps > Windows > Add Win32:
   - **Install:** `GitGov_x.x.x_x64-setup.exe /S`
   - **Uninstall:** `"C:\Program Files\GitGov\Uninstall GitGov.exe" /S`
   - **Detection:** File exists `C:\Program Files\GitGov\GitGov.exe`
   - **Return codes:** 0 = success, 1641/3010 = success (reboot)

### Pre-configuring Server Connection

Set machine-wide environment variables:

| Variable | Example | Description |
|----------|---------|-------------|
| `GITGOV_SERVER_URL` | `http://127.0.0.1:3000` | Control Plane URL |
| `GITGOV_API_KEY` | `57f1ed59-...` | API key from admin |

**Via Group Policy:**
```
Computer Configuration > Preferences > Windows Settings > Environment
```

**Via PowerShell (Intune):**
```powershell
[System.Environment]::SetEnvironmentVariable("GITGOV_SERVER_URL", "http://127.0.0.1:3000", "Machine")
[System.Environment]::SetEnvironmentVariable("GITGOV_API_KEY", "your-api-key-here", "Machine")
```

**Via SCCM:**
```
cmd.exe /c setx GITGOV_SERVER_URL "http://127.0.0.1:3000" /M
cmd.exe /c setx GITGOV_API_KEY "your-api-key-here" /M
```

> Fallback: the app also reads from `%APPDATA%\..\Local\gitgov\.env`.

### Verifying Installation

```powershell
Test-Path "C:\Program Files\GitGov\GitGov.exe"
(Get-Item "C:\Program Files\GitGov\GitGov.exe").VersionInfo.ProductVersion
[System.Environment]::GetEnvironmentVariable("GITGOV_SERVER_URL", "Machine")
```

### SHA256 Hash Verification

```powershell
Get-FileHash .\GitGov_x.x.x_x64-setup.exe -Algorithm SHA256
```

Generate `.sha256` file:
```powershell
.\scripts\generate_sha256.ps1 -InstallerPath ".\gitgov\src-tauri\target\release\bundle\nsis\GitGov_x.x.x_x64-setup.exe"
```

Upload both `.exe` and `.sha256` as GitHub Release assets. Set hash in Vercel:
```
NEXT_PUBLIC_DESKTOP_DOWNLOAD_CHECKSUM=sha256:<hex>
```

### Code Signing

Verify signature:
```powershell
Get-AuthenticodeSignature .\GitGov_x.x.x_x64-setup.exe | Select-Object -Property Status, SignerCertificate
```

**CI secrets required for signed releases:**

| Secret | Description |
|--------|-------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded `.pfx` blob |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for `.pfx` |
| `WINDOWS_CERTIFICATE_THUMBPRINT` | Cert thumbprint for Tauri signing |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri updater signing private key |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for updater key |

**CI builds** (`build-signed.yml`) on `v*` tag pushes:
- Windows: NSIS + MSI + `.sha256`
- macOS: DMG + `.sha256`
- Linux: AppImage + DEB + `.sha256`

### SonarQube Governance (opcional, no bloqueante)

Workflow agregado:
- `.github/workflows/sonar-governance.yml`

Comportamiento:
- Ejecuta scan SonarQube + quality gate en `push/main`, `pull_request/main` y `workflow_dispatch` solo si el runner puede alcanzar `SONAR_HOST_URL`.
- Es **no bloqueante** (`continue-on-error: true`): no corta el flujo principal de CI.
- Si hay `GITGOV_URL` + `GITGOV_API_KEY`, publica resultado de quality gate como evento en `/integrations/jenkins`.
- En esta instalación, SonarCloud no se usa porque la cuenta GitHub es personal y no permite el onboarding requerido. La ruta oficial es SonarQube local.

Configurar en GitHub (repo settings):

| Tipo | Nombre | Uso |
|------|--------|-----|
| Secret | `SONAR_TOKEN` | Token de SonarQube local cuando el runner pueda alcanzar el host |
| Secret | `GITGOV_API_KEY` | API key admin para publicar telemetría a GitGov |
| Secret (opcional) | `GITGOV_JENKINS_SECRET` | Header `x-gitgov-jenkins-secret` si está habilitado |
| Variable | `SONAR_PROJECT_KEY` | Project key en Sonar |
| Variable | `SONAR_HOST_URL` | Host SonarQube alcanzable por el runner |
| Variable (opcional) | `GITGOV_URL` | URL base del Control Plane (`https://...`) |

Notas:
- Si faltan `SONAR_TOKEN`, `SONAR_HOST_URL` o `SONAR_PROJECT_KEY`, el job se omite automáticamente.
- Si falta `GITGOV_URL` o `GITGOV_API_KEY`, se hace scan pero se omite publicación a GitGov.
- SonarQube local:
  - levantar `docker compose --profile sonar up -d sonarqube-db sonarqube`
  - generar token local en SonarQube (`My Account > Security`)
  - setear `SONAR_HOST_URL` al host local alcanzable desde runner (`http://host.docker.internal:9000` en Jenkins Docker local)
  - para GitHub-hosted runners, no usar `localhost`; esos runners no pueden alcanzar tu SonarQube local. Mantener el workflow no bloqueante o usar self-hosted runner.

Jenkins (`Jenkinsfile`) también soporta Sonar en modo opcional/no bloqueante:
- stage `Sonar Scan (Optional)` bootstrappea `sonar-scanner` si no existe en el agente.
- consulta CE task + `quality gate` vía API Sonar y guarda estado en telemetría.
- publica stage `quality_gate` en `/integrations/jenkins` junto con artifact `sonar_dashboard` cuando aplica.
- si `GITGOV_STRICT=true`, errores de Sonar/telemetría escalan a fallo de build.
- stage `Release Readiness Gate (Optional)` calcula score por `repo+branch+tier` con datos de Jira/Jenkins/Sonar en Control Plane.
- en `GITGOV_STRICT=true`, un gate fallido (`readiness_below_target`, sin señales, etc.) falla el build.
- en modo no estricto, el gate registra `WARN` y continúa (telemetría conserva `reasons/warnings`).

Preflight de configuración CI del repo (GitHub Actions):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_ci_repo_config.ps1 `
  -GitHubToken "<TOKEN_CON_PERMISOS_REPO/ACTIONS_READ>" `
  -Owner "<owner>" `
  -Repo "<repo>"
```

### Cierre cloud CI (strict mode)

Para validar configuracion GitHub Actions en modo estricto, el PAT debe tener visibilidad de:

- `secrets=read`
- `actions_variables=read`
- `administration=read` (si también validarás branch protection)

Validación de permisos del PAT:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_token_permissions.ps1 `
  -GitHubToken "<TOKEN>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main"
```

Validación estricta de configuración CI (sin best-effort):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_ci_repo_config.ps1 `
  -GitHubToken "<TOKEN>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -RequireGitGovTelemetry
```

Resultado esperado para considerar cierre GitHub Actions:

- `PASS` en `check_token_permissions.ps1` (sin `FORBIDDEN`).
- `PASS` en `check_ci_repo_config.ps1` (sin `UNKNOWN`).
- Un run de `.github/workflows/sonar-governance.yml` con skip explicito cuando el runner no pueda alcanzar SonarQube local, o scan activo solo si hay runner alcanzable.

Estado GitHub Actions actual (2026-04-24):

- `GITGOV_API_KEY` esta configurado como secret de repositorio.
- `GITGOV_URL=https://gitgov-api.onrender.com` esta configurado como variable de repositorio.
- `SONAR_HOST_URL=http://localhost:9000` esta configurado como variable de repositorio para señalar SonarQube local.
- SonarCloud no es objetivo. SonarQube local es la ruta soportada para esta cuenta; GitHub-hosted scan se salta cuando el host es local.
- La matriz `quality_gates=warn/block` ya paso en GitHub-hosted CI y el check requerido esta protegido en `main`.

Nota:
- `-Owner` y `-Repo` ahora son opcionales en scripts de `scripts/github/*`; si no se pasan, se auto-resuelven desde `GITHUB_REPOSITORY` o `git remote origin`.

Modo best-effort cuando el token es limitado (no bloquea por `403` en secrets/variables):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_ci_repo_config.ps1 `
  -GitHubToken "<TOKEN_FINE_GRAINED_LIMITADO>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -AllowMissingSonar `
  -NoFailOnForbidden
```

Diagnóstico rápido de permisos del token:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_token_permissions.ps1 `
  -GitHubToken "<TOKEN_FINE_GRAINED>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main"
```

Si devuelve `403`, revisa la columna `Accepted permissions hint` para habilitar exactamente ese permiso en el token.

Modo máquina (JSON + no fallo en permisos parciales):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_token_permissions.ps1 `
  -GitHubToken "<TOKEN_FINE_GRAINED>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -Branch "main" `
  -EmitJson `
  -NoFailOnForbidden `
  -Quiet
```

Resultado esperado:
- `PASS` si secrets/variables requeridos para el modo elegido están presentes.
- `PASS (best-effort)` si usas `-NoFailOnForbidden` y el token no puede leer secrets/variables (se muestra `UNKNOWN` en lugar de cortar flujo).
- Modo base (scan Sonar): requiere `SONAR_TOKEN` + `SONAR_HOST_URL` + `SONAR_PROJECT_KEY`.
- `-AllowMissingSonar`: permite operar sin Sonar (marca Sonar como opcional).
- `-RequireGitGovTelemetry`: exige `GITGOV_API_KEY` + `GITGOV_URL` para publicación de telemetría.
- Los scripts aceptan token por `-GitHubToken` o por entorno (`GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_PAT`, `GITHUB_PERSONAL_ACCESS_TOKEN`).
- Si no hay token en entorno, intentan resolverlo desde `gitgov/gitgov-server/.env` (`GITHUB_PERSONAL_ACCESS_TOKEN`).
- Para token fine-grained, habilitar permisos mínimos:
  - `Repository permissions > Secrets`: `Read` (o `Read and write`)
  - `Repository permissions > Actions variables`: `Read` (o `Read and write`)
  - `Repository permissions > Administration`: `Read` (y `Read and write` si aplicarás branch protection)

Bootstrap de variables CI (sin tocar secrets):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/github/bootstrap_ci_variables.ps1 `
  -GitHubToken "<TOKEN_CON_PERMISOS_REPO_ACTIONS_WRITE>" `
  -Owner "<owner>" `
  -Repo "<repo>" `
  -SonarProjectKey "<owner>_<repo>"
```

Opcional:
- `-SonarHostUrl "http://host.docker.internal:9000"` cuando Jenkins corre en Docker y SonarQube en host/local compose.
- `-GitGovUrl "https://<tu-control-plane>"`
- `FAIL` con lista concreta de faltantes si aún falta configuración.

**Local signed build:**
```powershell
.\scripts\build_signed_windows.ps1 -RepoRoot . -PfxPath "C:\secrets\gitgov-codesign.pfx" -PfxPassword "<password>"
```

### Firewall / Proxy

| Destination | Port | Protocol | Purpose |
|-------------|------|----------|---------|
| Control Plane server | 3000 (or configured) | HTTP/HTTPS | Events + dashboard |
| `downloads.gitgov.com` | 443 | HTTPS | Auto-update checks |

If using a proxy, set `HTTP_PROXY` / `HTTPS_PROXY` environment variables.

### Offboarding a Developer

1. **Revoke API key** from dashboard (immediate effect — 401 on next sync)
2. **Uninstall** via Intune/SCCM/GPO
3. Audit history remains intact and immutable

### Compliance Export

1. Open **Control Plane** tab in Desktop
2. Connect with Admin API key
3. **Export Historial de Auditoría** → select range → Exportar JSON
4. Creates immutable log entry in `export_logs` table

---

## 4. Desktop Updates (Tauri Updater)

Actualizaciones in-app usando `tauri-plugin-updater` con full updates (sin deltas). La configuración trackeada actual apunta a GitHub Releases; S3 + CloudFront queda como guía para distribución custom/self-hosted.

### Estado actual (implementado)

- `tauri-plugin-updater` integrado en Desktop
- UI en `Configuración > Actualizaciones Desktop`
- `Buscar actualizaciones` manual
- Auto-check al iniciar (throttling ~6h)
- Changelog simple (campo `body` del manifest)
- Fallback de descarga manual
- Soporte de canales `stable` / `beta` en runtime
- Telemetría local de updater (checks, descargas, installs, errores)
- Reintento de descarga desde UI
- Enforcement de actualización obligatoria por metadata de release:
  - `min_supported_version`
  - `force_update` / `critical_update`
- `gitgov/src-tauri/tauri.conf.json` ya define `plugins.updater.endpoints` y `plugins.updater.pubkey`

### Requisito para producción

El updater requiere publicar un `latest.json` firmado en el endpoint configurado y firmar cada update con la clave privada correspondiente al `plugins.updater.pubkey` trackeado. Si el endpoint devuelve `404` o el manifest no cumple shape/firma, el validador queda en `WARN`/`FAIL` según modo.

### Arquitectura custom/self-hosted (AWS)

- **S3**: almacenar artefactos y manifests
- **CloudFront**: servir con HTTPS y CDN
- Canales: `stable` y `beta`

```
s3://gitgov-downloads/desktop/
  stable/
    latest.json
    GitGov_0.1.1_x64-setup.exe
    GitGov_0.1.1_x64-setup.exe.sig
  beta/
    latest.json
    ...
```

CloudFront URL de ejemplo: `https://downloads.gitgov.com/desktop/stable/latest.json`

### Configuración `tauri.conf.json`

La configuración actual del repo usa GitHub Releases:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/yohandry10/Git-Gov/releases/latest/download/latest.json"
      ],
      "pubkey": "<public-updater-key>"
    }
  }
}
```

Ejemplo para reemplazarlo por distribución custom:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://downloads.gitgov.com/desktop/stable/latest.json"
      ],
      "pubkey": "TU_PUBLIC_KEY_DEL_UPDATER"
    }
  }
}
```

> Ver snippet listo: `docs/examples/desktop-updater/tauri.updater.config.snippet.json`

### Claves de firma del updater

El updater usa un par de claves asimétricas:
- **Clave privada (secreta)**: firma cada update. Solo en máquina de release o CI secrets. Nunca se commitea.
- **Clave pública**: en `tauri.conf.json`. Verifica firma antes de instalar. No es secreta.

> Esto NO es lo mismo que code signing de Windows. Son dos firmas distintas. Usar **ambas** en producción.

### Generar claves (una sola vez)

```powershell
npx tauri signer generate --ci -p "TU_PASSWORD_FUERTE" --write-keys .\secrets\tauri-updater.key
```

Copiar la clave pública a `tauri.conf.json` → `plugins.updater.pubkey`.

### Firmar instalador

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = ".\secrets\tauri-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "TU_PASSWORD"
npx tauri signer sign .\src-tauri\target\release\bundle\nsis\GitGov_0.1.1_x64-setup.exe
```

### Release flow

1. Incrementar versión en `tauri.conf.json`
2. Build release (`tauri build`)
3. Generar firma (`.sig`)
4. Crear/actualizar `latest.json`
5. Subir `.exe`, `.sig` y `latest.json` a S3
6. Invalidar CloudFront (si aplica)
7. Probar desde versión anterior

### Scripts helper

```powershell
# Generar manifest
.\scripts\release\desktop-updater\New-TauriUpdaterManifest.ps1 `
  -Version "0.1.1" `
  -Url "https://downloads.gitgov.com/desktop/stable/GitGov_0.1.1_x64-setup.exe" `
  -Signature "FIRMA" `
  -Notes "Changelog" `
  -MinSupportedVersion "0.1.0" `
  -ForceUpdate `
  -ForceUpdateReason "Security hotfix CVE-xxxx" `
  -CriticalUpdate `
  -OutputPath ".\release\desktop\stable\latest.json"

# Publicar a S3
.\scripts\release\desktop-updater\Publish-DesktopUpdateAws.ps1 `
  -ExePath ".\src-tauri\target\release\bundle\nsis\GitGov_0.1.1_x64-setup.exe" `
  -SigPath ".\release\desktop\stable\GitGov_0.1.1_x64-setup.exe.sig" `
  -ManifestPath ".\release\desktop\stable\latest.json" `
  -Bucket "gitgov-downloads" `
  -Channel "stable" `
  -CloudFrontDistributionId "E123ABC456DEF"

# Generar snippet de config
.\scripts\release\desktop-updater\New-TauriUpdaterConfigSnippet.ps1 `
  -Channel "stable" `
  -BaseUrl "https://downloads.gitgov.com/desktop" `
  -PubKey "PUBLIC_KEY" `
  -OutputPath ".\release\desktop\tauri.updater.stable.json"
```

### Disable auto-updates (air-gapped)

Block `downloads.gitgov.com` at the firewall. The app continues functioning; only update notifications are suppressed.

### Troubleshooting

| Síntoma | Causa | Solución |
|---------|-------|----------|
| "Updater no configurado" | Falta `plugins.updater`, `endpoints` o `pubkey` en `tauri.conf.json` | Configurar los campos |
| "No se pudo verificar/instalar" | URL inaccesible, firma incorrecta o pubkey mal | Verificar URL, signature y pubkey |
| Usuario no ve notificación | Throttling ~6h o no está en Desktop | Probar `Buscar actualizaciones` manual |

### Validación automática de readiness del updater

```powershell
powershell -ExecutionPolicy Bypass -File scripts/deploy/validate_desktop_updater_readiness.ps1 `
  -TauriConfigPath "gitgov/src-tauri/tauri.conf.json" `
  -OutputPath "docs/reports/desktop-updater-readiness-<fecha>.md"
```

El validador verifica:
- `plugins.updater` presente
- `updater.endpoints` y `updater.pubkey` configurados
- sintaxis HTTPS de endpoints
- probe real de `latest.json` (HTTP + shape del manifest `version/platforms`)
- metadata de enforcement (`min_supported_version`, `force_update`, `force_update_reason`)

Si devuelve `WARN` por `404` en `latest.json`, falta publicar assets/manifest en el endpoint configurado.

Automatización GitHub Actions (opcional, no bloqueante):

- Workflow: `.github/workflows/desktop-updater-readiness.yml`
- Trigger: `push/main` + `workflow_dispatch`
- Artifact: `desktop-updater-readiness-<run_id>.md`
- Inputs manuales:
  - `probe_endpoint=true` para validar `latest.json` en endpoint real
  - `fail_on_warnings=true` para tratar `WARN` como `FAIL`

### Fases cerradas

- **Fase 2:** Canales beta/stable, telemetría de updater y reintento de descarga (implementado).
- **Fase 3:** enforcement de `min_supported_version` y forced updates críticos desde metadata firmada del `latest.json` (implementado).

---

## Support

- Documentation: `docs/` directory
- Issues: https://github.com/<owner>/<repo>/issues
- Health check: `GET http://<server>:3000/health`

---

*Documento consolidado de: DEPLOY_EC2_SUPABASE.md, DOCKER.md, ENTERPRISE_DEPLOY.md, DESKTOP_UPDATES.md*
*Fecha de consolidación: 2026-03-14*

