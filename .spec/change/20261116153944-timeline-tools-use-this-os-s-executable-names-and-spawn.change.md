---
id: change:kinekino/20261116153944
title: Timeline tools use this OS's executable names and spawn their verified program
status: active
created: 2026-11-16T15:39:44Z
updated: 2026-11-16T15:39:44Z
rel:
  depends_on: [adr:kinekino/20261116151137, change:kinekino/20261116153442, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/src/infrastructure/timeline_evidence/production/tool.rs, crates/kinekino-player/src/infrastructure/timeline_evidence/production/process.rs, crates/kinekino-player/src/infrastructure/frame_capture/tools.rs]
---

# Timeline tools use this OS's executable names and spawn their verified program

## Change
- Slice (c) of [[adr:kinekino/20261116151137]] for the timeline evidence tools. `PinnedTimelineProbe`, `PinnedTimelineAudioDecoder` and `PinnedTimelineCaptureDecoder` used to accept only `ffprobe.exe`/`ffmpeg.exe`. They now take the tool stem plus this OS's executable suffix (`std::env::consts::EXE_SUFFIX`), so a Unix host can pin `ffprobe`/`ffmpeg`.
- Each pinned timeline tool now holds the program from `OsPolicy::pinned_program`. Spawns run that program, which on Linux is the verified handle. Evidence still records the verified path.
- The error mapping is shared with the frame-capture tools (`frame_capture::tools::pinned_program`).
- New test: an installed `ffprobe` is pinned and run, and the wrong tool name is refused. The test skips when no `ffprobe` is on `PATH`.

## Migration
- None. On Windows the program equals the verified path, and the accepted names are unchanged.
