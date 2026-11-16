---
id: change:kinekino/20261116151826
title: Pinned tool program path comes from the platform port
status: active
created: 2026-11-16T15:18:26Z
updated: 2026-11-16T15:18:26Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261106022549, adr:kinekino/20261109121745, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-platform-unix/src/fs, crates/kinekino-core/src/platform]
---

# Pinned tool program path comes from the platform port

## Change
Slice (a) of the pinned-tool-execution decision (ADR 20261116151137):
- `PlatformHost::has_handle_execution` (Linux true; Windows and other POSIX false) and `OsPolicy::pinned_program(verified, path)` are added to the core platform port and routed through `kinekino-platform`.
- **Unix adapter** (`kinekino-platform-unix/src/fs/pinned.rs`):
  - Refuses a relative path or a `#!` script (`InvalidInput`).
  - Refuses a file or parent directory that is not owned by the user or root, or that is writable by group or others. Refuses such an ancestor directory too unless it is sticky (`PermissionDenied`).
  - Refuses a path that no longer names the verified inode (`InvalidData`).
  - Returns `/proc/self/fd/<n>` on Linux and the canonical path elsewhere.
- **Windows adapter:** returns the absolute path unchanged, since share-mode guards keep the bytes fixed.
- **Tests (Linux):** the verified file runs, a renamed-over path still runs the verified inode, and the mode, ancestor, script and relative-path cases are refused.
- No launcher uses the port yet; that is slice (b).

## Migration
- None. No caller changes behaviour in this slice.
