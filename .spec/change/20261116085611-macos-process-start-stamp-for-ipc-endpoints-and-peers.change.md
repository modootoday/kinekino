---
id: change:kinekino/20261116085611
title: macOS process start stamp for IPC endpoints and peers
status: active
created: 2026-11-16T08:56:11Z
updated: 2026-11-16T08:56:11Z
rel:
  depends_on: [adr:kinekino/20261109121745, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# macOS process start stamp for IPC endpoints and peers

## Change

`crates/kinekino-platform-unix/src/ipc/mod.rs`: `process_start` now dispatches per OS. Linux reads `/proc/<pid>/stat` as before. macOS uses `proc_pidinfo(PROC_PIDTBSDINFO)` (`pbi_start_tvsec`, `pbi_start_tvusec`, in microseconds, at least 1). Other Unix systems keep returning `Unsupported`. `ipc/tests.rs` adds a stability and missing-pid test that runs on Linux and macOS.

## Migration

None on Linux. On macOS, IPC endpoints and peer checks no longer fail at the start stamp.
