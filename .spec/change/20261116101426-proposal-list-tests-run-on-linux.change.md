---
id: change:kinekino/20261116101426
title: Proposal list tests run on Linux
status: active
created: 2026-11-16T10:14:26Z
updated: 2026-11-16T10:14:26Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, adr:kinekino/20261106022549, adr:kinekino/20261109121745, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests/proposal_list.rs]
---

# Proposal list tests run on Linux

## Change

`kinekino-platform` `fs` gains `symlink_dir`, next to `symlink_file`, for link-rejection fixtures: Unix `symlink`, Windows `symlink_dir`.

`crates/kinekino-player/tests/proposal_list.rs` loses `#![cfg(windows)]` and makes its reparse fixture through the new port. On a Linux dev host:
- 17 tests pass by default;
- the ignored reparse-denial test also passes when run with `--include-ignored`;
- the `actual_cli_*` tests stay ignored, because they need an allocated evidence root.

The structure allowlist drops 1 `r-cfg` entry.

## Migration

None.
