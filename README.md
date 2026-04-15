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

## Documentación

| Documento | Propósito |
|-----------|-----------|
| [QUICKSTART.md](./docs/QUICKSTART.md) | Guía de inicio (5 min) |
| [ARCHITECTURE.md](./docs/ARCHITECTURE.md) | Arquitectura del sistema |
| [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) | Solución de problemas |

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

