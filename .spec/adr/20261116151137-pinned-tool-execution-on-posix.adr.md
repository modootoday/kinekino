---
id: adr:kinekino/20261116151137
title: Pinned tool execution on POSIX
status: active
created: 2026-11-16T15:11:37Z
updated: 2026-11-16T15:11:37Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029164425, adr:kinekino/20261030030000]
  governs: [crates/kinekino-player/src/infrastructure/frame_capture]
---

# Pinned tool execution on POSIX

## Context

Pinned tools (ffmpeg, ffprobe, mpv and the other native helpers) are configured as an absolute
local path plus a SHA-256. `frame_capture::tools::open` walks the path through
`immutable_storage_paths::root`/`file`, hashes the opened file, and keeps the directory and
file handles as guards for the adapter lifetime. The launcher then starts the tool by path
(`frame_capture/process/owned/launch.rs`, which only requires the path to be absolute).

- **Windows.** The guards are opened with share modes that deny write and delete, so while they
  are held the bytes at that path cannot change. Starting the tool by path runs the bytes that
  were verified.
- **POSIX.** Sharing is advisory (`has_mandatory_share_modes() == false`). A held handle keeps the
  original inode readable, but another process can still overwrite the file in place or rename a
  new file over the name. Starting by path may then run bytes that were never verified. This is
  why the suites that run pinned tools end to end (`execution_host_cli`,
  `historical_provenance_cli`, `review_matroska_timeline`, `timeline_video_production_actual`,
  `sprite_*`) and the `native_playback` assembly remain Windows-only. Issue 30 listed this as an
  owner decision. On 2026-11-16 the owner delegated the remaining decisions to the agent sessions
  ("decide and proceed unattended"), so this ADR records the agent's decision.

## Decision

1. **Run the verified handle, not the name, where the OS allows it.** On Linux the launcher starts
   the tool through `/proc/self/fd/<n>`, where `<n>` is the verified file guard. Path resolution
   happens before close-on-exec descriptors are closed, so the kernel executes exactly the
   inode that was hashed. A replaced or renamed file at the configured path has no effect. Pinned
   tools must be native executables, not `#!` scripts, because a script's interpreter would
   reopen the closed descriptor. The launcher refuses a file that starts with `#!`.
2. **Elsewhere on POSIX (macOS, BSD), constrain who can change the path.** Before every launch,
   the file and each ancestor directory must be owned by the current user or root, and must not
   be writable by group or others. A directory that is writable by others but has the sticky bit
   is also refused for the tool's own parent. After that check, the file is re-hashed through
   its guard and its identity (device and inode from `fstat` on the guard, `stat` on the path) is
   compared just before spawning. Under these conditions only the user or root can swap the bytes,
   which is the same trust boundary as the process itself.
3. **Linux applies the ownership rule too.** It costs one `stat` per ancestor and keeps a
   world-writable directory from being configured as a tool location on any OS.
4. **Windows is unchanged.** Share-mode guards stay the mechanism there.
5. **Capability, not cfg, in shared code.** The choice is exposed through the platform port: a new
   `PlatformHost` capability for handle execution, plus an adapter function returning the program
   path to spawn for a guard. Player code calls the port; no `cfg` is added outside the adapter
   crates. Tests that today assert Windows share-mode behaviour assert per capability instead,
   following `frame_capture::tools` tests (`has_mandatory_share_modes`).
6. **Errors.** A failed ownership or mode check is `PermissionDenied` with the offending path
   class (file, parent or ancestor), never the full private path. A failed identity or hash check
   before launch is `SourceExpired`, as for the first verification.

## Consequences

- The Windows-only gates on the pinned-tool suites and on `native_playback` can be lifted one
  suite at a time once the launcher uses the port. Each slice keeps its Windows behaviour and must
  pass Linux CI and the Windows gate check.
- Tools under a world-writable or shared-group location stop working on POSIX until they are moved
  to an owner-only directory. This is intended.
- macOS keeps a small window between the identity check and `posix_spawn`. It is limited to
  processes running as the same user or root, which can already change the process itself.
- Slices, in order:
  - (a) the port capability and the Unix adapter, with contract tests;
  - (b) the frame-capture launcher uses it;
  - (c) the timeline, clip-encoder and sidecar launchers;
  - (d) the suites and `native_playback` drop their Windows gates where they pass on Linux.
- Security-sensitive: every slice carries `needs-human` and merges only after measurement.
