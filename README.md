# GitGov

**Git Governance Control Plane** - Sistema de gobernanza de Git con auditoría centralizada.

## Estado del Proyecto

**✅ Funcional** - El pipeline Desktop → Server → Dashboard está operativo.

## Inicio Rápido

```bash
# 1. Control Plane Server
cd gitgov/gitgov-server
cp .env.example .env
# Editar .env con credenciales de PostgreSQL
cargo run

# 2. Desktop App
cd gitgov
npm install
npm run tauri dev

# 3. Web App pública (opcional)
cd ../gitgov-web
pnpm install
pnpm dev
```

Ver [QUICKSTART.md](./docs/QUICKSTART.md) para guía completa.

## Componentes

| Componente | Tecnología | Ubicación |
|------------|------------|-----------|
| Desktop App | Tauri v2 + React | `gitgov/` |
| Control Plane Server | Axum + Rust | `gitgov/gitgov-server/` |
| Web App pública | Next.js 15.5 (App Router) | `gitgov-web/` |
| Database | PostgreSQL (Supabase) | Render + Supabase (producción) |

## Funcionalidades

- ✅ Dashboard principal con commits y pushes
- ✅ Control Plane conectado
- ✅ Pipeline de eventos E2E
- ✅ Autenticación GitHub OAuth
- ✅ Outbox offline con reintentos
- ✅ Auditoría centralizada
- ✅ Sitio público (marketing/docs/download) en Next.js 15.5
- ✅ Chat de gobernanza en dashboard desktop (`/chat/ask`) para consultas tipo:
  - Operacionales: quién hizo commits/pushes, rangos de fechas, actividad por usuario
  - Riesgo/calidad: pushes bloqueados/sin ticket, quality gates no verdes, tickets en riesgo
  - Readiness: resumen de release-readiness y ranking de repos/ramas con fallos
  - Acceso: perfil de usuario y estado de clave (sin exponer secretos)
  - Roles permitidos: `Admin`, `Architect`, `PM`
- ✅ Editor de políticas en dashboard desktop para definir ramas y reglas (guardado en Control Plane)
- ✅ SSE (Server-Sent Events) para actualizaciones en tiempo real con fallback a polling
- ✅ Enterprise adoption profiles — self-service onboarding enterprise
- ✅ Enterprise release approvals — aprobaciones formales con evidence packets
- ✅ Enterprise onboarding checklist — checklist guiado con tracking persistente
- ✅ Governance copilot (AI mode) — copilot de gobernanza con Vercel AI SDK
- ✅ Release governance evaluator con enforcement gate configurable
- ✅ Evidence packets auditables por ticket
- ✅ Policy drift detection y auditoría
- ✅ GDPR: erase/export de datos de usuario
- ✅ Compliance signals y detección automática
- ✅ Branch tree visual (Cytoscape.js)
- ✅ Métricas Prometheus (`/metrics`)
- ✅ 32 GitHub Actions workflows (CI, security, governance, monitoring, trends)
- ✅ 193 server tests + 25 frontend test files

> Nota: métricas de quality gate/readiness dependen de tener telemetría Jenkins/Jira/Sonar configurada.

### Alcance del Dashboard Desktop

- El botón de chat en el dashboard sí está implementado y conectado al backend (`/chat/ask`).
- El flujo de reglas es **manual/asistido** desde `Policy Editor` (UI + API de políticas).
- No existe, hoy, un conversor automático de "diagrama de arquitectura/repos/ramas" a ramas/reglas Git.

## Documentación

| Documento | Propósito |
|-----------|-----------|
| [QUICKSTART.md](./docs/QUICKSTART.md) | Guía de inicio (5 min) |
| [ARCHITECTURE.md](./docs/ARCHITECTURE.md) | Arquitectura del sistema |
| [IMPLEMENTATION_STATUS.md](./docs/IMPLEMENTATION_STATUS.md) | Estado técnico actual y próximos pasos |
| [OPERATIONS_ACCESS.md](./docs/OPERATIONS_ACCESS.md) | Runbook de accesos operativos GitHub/Render/SonarQube/Jenkins |
| [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) | Solución de problemas |
| [PUBLICATION_POLICY.md](./docs/PUBLICATION_POLICY.md) | Qué documentación puede publicarse (y cuál no) |
| [CURRENT_CONTEXT.md](./docs/CURRENT_CONTEXT.md) | Estado actual compacto y handoff operativo |
| [DEPLOYMENT.md](./docs/DEPLOYMENT.md) | Guía de despliegue |
| [ENTERPRISE_READINESS_DECISION.md](./docs/ENTERPRISE_READINESS_DECISION.md) | Decisión de readiness enterprise |

## Scripts de Prueba

```bash
# E2E flow test
cd gitgov/gitgov-server/tests
./e2e_flow_test.sh

# Stress test
./stress_test.sh
```

## Licencia

MIT

