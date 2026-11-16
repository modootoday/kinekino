---
id: change:kinekino/20261116104517
title: Model catalog CLI tests run on Linux
status: active
created: 2026-11-16T10:45:17Z
updated: 2026-11-16T10:45:17Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests/model_catalog_cli.rs]
---

# Model catalog CLI tests run on Linux

## Change

`crates/kinekino-player/tests/model_catalog_cli.rs` loses `#![cfg(windows)]`. It needed `--model-root` and the portable file custody guard, and both now work on Linux.

The mid-hash test checks that the original-file guards still deny writers only on hosts with mandatory share modes; a POSIX open cannot deny writers. It still checks on every OS that the guards are released when verification returns.

On a Linux dev host all 4 tests pass. The structure allowlist drops 1 `r-cfg` entry.

## Migration

None.
