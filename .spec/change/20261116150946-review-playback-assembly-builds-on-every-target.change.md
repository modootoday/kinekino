---
id: change:kinekino/20261116150946
title: Review playback assembly builds on every target
status: active
created: 2026-11-16T15:09:46Z
updated: 2026-11-16T15:09:46Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/src/assembly/mod.rs]
---

# Review playback assembly builds on every target

## Change
- `kinekino_player::assembly::review_playback` (review playback preparation, stdio and transport adapters over the SDK) is no longer gated to Windows. It uses no Windows API: it builds on Linux with no errors or warnings, and its 7 unit tests pass there. Platform `cfg` outside adapter crates goes from 145 to 144 (M5, OS-5).
- `native_playback` stays gated: two of its tests rely on Windows share modes keeping the pinned executable unwritable while open, which POSIX does not give (pinned tool execution on POSIX, owner list in issue 30). `presentation` stays gated: everything inside it is already gated on Windows and the `native-proposal-window` feature.
- The integration suites that call Windows APIs directly (`review_playback_b0` and the other `review_*_actual` suites) are unchanged.

## Migration
- None. Behaviour on Windows is unchanged.
