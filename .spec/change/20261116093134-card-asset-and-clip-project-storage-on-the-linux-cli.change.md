---
id: change:kinekino/20261116093134
title: "Card, asset and clip project storage on the Linux CLI"
status: active
created: 2026-11-16T09:31:34Z
updated: 2026-11-16T09:31:34Z
rel:
  depends_on: [adr:kinekino/20261108112026, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Card, asset and clip project storage on the Linux CLI

## Change

`crates/kinekino-host/src/runner.rs`: the three rooted project stores open on every OS. The `cfg(not(windows))` refusals ("Rooted card/asset/clip project storage requires Windows") are gone.

`crates/kinekino-player/tests/{card,asset,clip}_project_cli.rs` lose `#![cfg(windows)]`, and the structure allowlist shrinks accordingly:
- 3 `r-cfg` entries removed;
- the `runner.rs` file-lines bucket drops from `>1100` to `>1000`.

Evidence, from a Linux container with `XDG_RUNTIME_DIR` set:
- The ignored normal-CLI cases pass with fresh evidence directories and the CLI pinned by SHA-256.
- Without the runner change, the card and clip cases fail.

## Migration

None. Windows behaviour is unchanged.
