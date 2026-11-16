---
id: change:kinekino/20261116095448
title: Speaker annotation documents open on the Linux CLI
status: active
created: 2026-11-16T09:54:48Z
updated: 2026-11-16T09:54:48Z
rel:
  depends_on: [adr:kinekino/20261108112026, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-host/src/runner.rs]
---

# Speaker annotation documents open on the Linux CLI

## Change

Recorded in [[adr:kinekino/20261108112026]].

- `kinekino-host` runner: `with_speaker_documents(LocalSpeakerFiles)` is no longer behind `cfg(windows)`.
- `crates/kinekino-player/tests/sprite_capture_cli/custody.rs` opens through `infrastructure::os_policy` (share mode, no-follow and directory flags, reparse check, file identity) instead of raw Win32 calls. It maps to the same Windows flags as before.
- `crates/kinekino-player/tests/speaker_annotations_cli.rs`: removed `#![cfg(windows)]`. The writer-denied lease checks run only when `has_mandatory_share_modes()` is true.
- Structure allowlist: one `r-cfg` entry removed.

## Migration

None. On Linux, `speakers.*` used to fail because the adapter was missing; it now works as it does on Windows.
