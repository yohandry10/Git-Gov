# Target Forensic Cleanup

Date: 2026-06-08

Classification: performance plus local maintenance.

## Scope

Local directory removed:

- `gitgov/gitgov-server/target_forensic`

Resolved path before deletion:

- `C:\Users\PC\Desktop\GitGov\gitgov\gitgov-server\target_forensic`

## Findings

The directory was not a standard source module. It contained Rust/Cargo build artifacts and debug outputs from a previous forensic/debug build snapshot:

- `3,637` files
- `2,562.07 MB`
- `421` files larger than `1 MB`

Largest artifact classes:

- `.rlib`: `312` files, `1,010.08 MB`
- `.pdb`: `96` files, `603.34 MB`
- `.o`: `542` files, `331.05 MB`
- `.bin`: `3` files, `140.04 MB`
- `.exe`: `78` files, `100.69 MB`
- `.dll`: `18` files, `47.51 MB`

Largest individual files observed:

- `debug/deps/gitgov_server.pdb`: `188.66 MB`
- `debug/gitgov_server.pdb`: `188.66 MB`
- `debug/incremental/.../dep-graph.bin`: `89.86 MB`
- `debug/deps/libsqlx_postgres-980c3b9c000bfc35.rlib`: `75.35 MB`
- `debug/deps/libmetrics_exporter_prometheus-281cb7f38f8f1ca1.rlib`: `62.55 MB`

## Decision

Delete the local directory after recording this inventory. The contents are build/debug artifacts, not product source, docs, migrations, tests, or runtime configuration.

This cleanup does not restart Desktop, does not alter the active manual session, does not touch secrets, and does not change application behavior.
