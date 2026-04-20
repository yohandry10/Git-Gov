# Desktop Updater Readiness Report

Generated (UTC): 2026-04-20 09:14:33
Summary: **WARN**
Tauri config: gitgov/src-tauri/tauri.conf.json

## Checks

| Check | Status | Details |
|---|---|---|
| plugins.updater | PASS | Updater block exists. |
| updater.pubkey | PASS | pubkey configured. |
| updater.endpoints | PASS | Configured endpoints: https://github.com/yohandry10/Git-Gov/releases/latest/download/latest.json |
| endpoint syntax: https://github.com/yohandry10/Git-Gov/releases/latest/download/latest.json | PASS | HTTPS endpoint syntax OK. |
| endpoint probe: https://github.com/yohandry10/Git-Gov/releases/latest/download/latest.json | WARN | HTTP 404. Error en el servidor remoto: (404) No se encontró. |

## Warnings

- Endpoint probe returned HTTP 404 for https://github.com/yohandry10/Git-Gov/releases/latest/download/latest.json

## Failures

- none
