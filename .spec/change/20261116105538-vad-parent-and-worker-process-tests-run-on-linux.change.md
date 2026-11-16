---
id: change:kinekino/20261116105538
title: VAD parent and worker process tests run on Linux
status: active
created: 2026-11-16T10:55:38Z
updated: 2026-11-16T10:55:38Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, change:kinekino/20261116101201, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests/vad_parent_worker.rs]
---

# VAD parent and worker process tests run on Linux

## Change

`vad_parent_worker.rs` and `vad_worker_control_process.rs` lose `#![cfg(windows)]`.

The parent-worker round trip checks that the staging lease denies writers to `stdin.raw` only on hosts with mandatory share modes, because a POSIX open cannot deny writers. Everything else in the round trip is unchanged.

On a Linux dev host:
- 35 parent-worker tests pass;
- 5 worker control tests pass, and its 14 actual-child tests stay ignored.

The structure allowlist drops 2 `r-cfg` entries.

## Migration

None.
