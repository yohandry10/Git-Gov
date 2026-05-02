# KAN-72 Desktop/Dashboard Documentation Reality Audit

Updated: 2026-05-02

## Summary

KAN-72 is phase 3 of the GitGov documentation reality audit. It checks Desktop React dashboard, Tauri backend, updater configuration, and desktop test-count documentation against the repository state without changing runtime behavior.

## Product Context

- `KAN-69 - Enterprise Action Center guided UX` remains pending product/UX work.
- `KAN-70` completed the first broad documentation cleanup pass.
- `KAN-71` completed the backend/API/schema documentation audit.
- `KAN-72` narrows the audit to `gitgov/src`, `gitgov/src-tauri`, desktop package metadata, and updater configuration.

## Verified Sources

| Area | Source checked | Verified state |
| --- | --- | --- |
| Desktop package | `gitgov/package.json` | React `19.2.0`, Tailwind `4.2.0`, Zustand `5.0.11`, Vite `7.3.1`, Vitest `4.0.18` |
| Desktop frontend files | `gitgov/src` | `99` TypeScript/TSX files |
| Control Plane dashboard modules | `gitgov/src/components/control_plane` | `27` component/helper modules |
| Desktop routes | `gitgov/src/router.tsx` | `/`, `/audit`, `/settings`, `/control-plane`, `/help`, and fallback route |
| Tauri source files | `gitgov/src-tauri/src` | `31` Rust files |
| Tauri commands | `gitgov/src-tauri/src/lib.rs` | `94` registered Tauri commands |
| Frontend tests | `npm test -- --run` from `gitgov` | `25` files, `296` tests |
| Tauri tests | `cargo test -- --list` from `gitgov/src-tauri` | `23` tests, `0` benchmarks |
| Updater config | `gitgov/src-tauri/tauri.conf.json` | `plugins.updater.endpoints` and `plugins.updater.pubkey` are configured |

## Corrections Made

- Root README now records the real desktop test count and removes the stale Cytoscape branch-tree claim.
- `gitgov/README.md` now documents React 19 and the current Control Plane dashboard surface instead of the older basic metrics-only dashboard description.
- `docs/ARCHITECTURE.md` now describes the dashboard as a multi-panel Control Plane surface and records verified Desktop/Tauri counts.
- `docs/QUICKSTART.md` no longer says the next step is configuring an updater server; the remaining task is publishing and validating a signed `latest.json` at the configured endpoint.
- `docs/DEPLOYMENT.md` now distinguishes the current GitHub Releases updater endpoint from the optional S3 + CloudFront distribution guidance.
- `docs/IMPLEMENTATION_STATUS.md`, `docs/CURRENT_CONTEXT.md`, and `AGENTS.md` now track `KAN-72` as the active documentation-only phase and keep `KAN-69` pending.

## Non-Goals

- No runtime code changes.
- No Desktop UI changes.
- No provider mutation.
- No GitHub Actions secret or variable mutation.
- No web public docs audit beyond facts needed for Desktop comparisons.
- No implementation of `KAN-69`; it remains pending guided UX work.
