---
id: change:kinekino/20261116095002
title: Catalog root opens on the Linux CLI
status: active
created: 2026-11-16T09:50:02Z
updated: 2026-11-16T09:50:02Z
rel:
  depends_on: [adr:kinekino/20261108112026, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-host/src/runner.rs]
---

# Catalog root opens on the Linux CLI

## Change

Recorded in [[adr:kinekino/20261108112026]].

- `kinekino-host` runner: removed the `cfg(windows)` gate on `--catalog-root` and the Linux refusal.
- `crates/kinekino-player/tests/catalog.rs`: removed `#![cfg(windows)]` and the Windows-only `CREATE_NO_WINDOW` spawn flag. The test now runs on Linux with the site-neutral fixture plugin.
- Structure allowlist: one `r-cfg` entry removed.

## Migration

None. On Linux, `--catalog-root` used to be refused as `unsupported`; it now opens the experimental catalog as it does on Windows.
