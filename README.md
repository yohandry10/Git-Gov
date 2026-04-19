# GitGov

**Git Governance Control Plane** - Sistema de gobernanza de Git con auditoría centralizada.

## Estado del Proyecto

**✅ Funcional** - El pipeline Desktop → Server → Dashboard está operativo.

## Inicio Rápido

```bash
# 1. Control Plane Server
cd gitgov/gitgov-server
cp .env.example .env
# Editar .env con credenciales de Supabase
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
| Database | PostgreSQL (Supabase) | Supabase Cloud |

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
| [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) | Solución de problemas |
| [PUBLICATION_POLICY.md](./docs/PUBLICATION_POLICY.md) | Qué documentación puede publicarse (y cuál no) |

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

