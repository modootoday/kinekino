---
id: change:kinekino/20261116153442
title: Pinned frame and clip tools spawn their verified program
status: active
created: 2026-11-16T15:34:42Z
updated: 2026-11-16T15:34:42Z
rel:
  depends_on: [adr:kinekino/20261116151137, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261109121745, change:kinekino/20261116151826, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/src/infrastructure/frame_capture, crates/kinekino-platform-unix/src/process/ownership]
---

# Pinned frame and clip tools spawn their verified program

## Change
Slices (b) and (c) of [[adr:kinekino/20261116151137]] for the tools opened through `frame_capture::tools::open`:
- **`tools::open`** returns a `PinnedTool`:
  - `path` is the verified local path, for evidence and messages;
  - `program` is what to spawn, from `OsPolicy::pinned_program`;
  - `guards` keep the handle open.

  Port errors map to `PermissionDenied`, `SourceExpired` (the path no longer names the verified file) or `InvalidArgument`.
- **Spawning:** frame capture (ffmpeg, ffprobe), the clip encoder, clip-source probing and clip output verification now spawn `program`. Frame-capture capability evidence still records the verified paths. `FfmpegFrameCapture::pinned` is no longer Windows-only.
- **Unix spawner** (`process/ownership/spawn.rs`): it no longer canonicalizes a verified-handle program. Resolving `/proc/self/fd/<n>` would name the file at its path again.
- **Tests (Linux):**
  - the ownership port runs the verified `true` after `false` was renamed over its path (this test failed with `NotFound` before the spawner change);
  - handle programs are recognised;
  - `tools::open` keeps the verified path for evidence;
  - pinned frame capture with the installed ffmpeg and ffprobe is available and reports the verified paths.

## Migration
- **POSIX hosts:** a pinned tool in a directory that another user can write is now refused (`PermissionDenied`). Move it to an owner-only directory.
- **Windows:** unchanged; programs are the configured paths.
