---
title: "Platforms"
description: "One Rust core with OS adapters. Status per OS, from the roadmap plan."
ref: platforms
nav_order: 5
hero: { c: prose }
sections:
  - { c: platform-list }
  - { c: prose }
---

The core, the commands and the CLI are the same everywhere. Only adapters (mpv host, IPC, credential store, GUI shell) differ per OS.

## Status per OS

- **Windows**: current app (Meari UI), native mpv playback checked by hand, named pipe IPC, snapshot export. Final check in P8. `status:partial`
- **Linux**: development base. CLI and Unix socket IPC run; mpv playback is P2, snapshot P4, GUI P6. `status:partial`
- **macOS**: not built or verified yet; build check in P8. `status:planned`

## How it is checked

Linux is verified first in development. Windows and macOS are verified again at the end of the roadmap (P8), so a badge only turns to available after that check.

CI runs only by hand. Linux (lint, tests, site build) decides pass or fail; Windows (compile check) and macOS (tests) are optional per run and advisory, so their failures are reported but do not fail the run.

CI cannot replace a check on a real machine for:

- **Windows**: native window creation, resize, fullscreen and DPI scaling need a desktop session; runners have none.
- **mpv**: decoding and audio/video output with real media and hardware decoding are not exercised; tests that need mpv are `#[ignore]`.
- **Input**: keyboard, mouse, IME and drag-and-drop come from a real user session, not from the runner.
- **GL**: OpenGL contexts and rendering need a GPU and display; runners fall back to no or software rendering.

These are checked on each OS by hand before a badge changes.
