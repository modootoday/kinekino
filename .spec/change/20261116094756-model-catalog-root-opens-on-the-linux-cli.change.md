---
id: change:kinekino/20261116094756
title: Model catalog root opens on the Linux CLI
status: active
created: 2026-11-16T09:47:56Z
updated: 2026-11-16T09:47:56Z
rel:
  depends_on: [adr:kinekino/20261108112026, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-host/src/runner.rs]
---

# Model catalog root opens on the Linux CLI

## Change

Recorded in [[adr:kinekino/20261108112026]].

- `kinekino-host` runner: removed the `cfg(windows)` gate on `--model-root` and the Linux refusal. Every OS now opens `LocalModelCatalog`.
- New `crates/kinekino-player/tests/model_catalog_root_cli.rs` (no `cfg`). It runs `models.list`, `models.inspect` and `models.verify` through the normal CLI against a synthetic catalog and compares each with the typed SDK. It also checks that a stale catalog pin is refused and that the stored files are left unchanged.

## Migration

None. On Linux, `--model-root` used to be refused as `unsupported`; it now works as it does on Windows.
