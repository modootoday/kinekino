---
id: change:kinekino/20261116103742
title: CLI discovery inventory tests run on Linux
status: active
created: 2026-11-16T10:37:42Z
updated: 2026-11-16T10:37:42Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests/cli_discovery_inventory.rs]
---

# CLI discovery inventory tests run on Linux

## Change

`crates/kinekino-player/tests/cli_discovery_inventory.rs` loses `#![cfg(windows)]`. Its configured review discovery shards needed `--review-root`, which now opens on the Linux CLI. On a Linux dev host all 6 tests pass. The structure allowlist drops 1 `r-cfg` entry.

## Migration

None.
