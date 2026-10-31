---
id: adr:kinekino/20261031110642
title: IPC migration, allowlist reduction and Windows dependency scope
status: active
created: 2026-10-31T11:06:42Z
updated: 2026-10-31T11:06:42Z
rel:
  governs: [xtask/structure-allowlist.txt, crates/kinekino-player-ui/src/window/edges.rs]
---

# IPC migration, allowlist reduction and Windows dependency scope

## Context

The status matrix listed three open items without a design document: the "P2 IPC
migration", allowlist reduction, and an upgrade to windows/windows-core 0.62.2 and
webview2-com 0.39.1. This ADR fixes what each of them actually means today, does the part
that can be done and checked on the Linux box, and names the stages that remain.

## Findings

**Windows crates.** The root `Cargo.toml` has pinned `windows = "=0.62.2"`,
`windows-core = "=0.62.2"` and `webview2-com = "0.39.1"` since the workspace was set up
(27306aec), `Cargo.lock` resolves exactly those versions, and on 2026-10-31 they are also the
newest stable releases on crates.io. There is nothing to upgrade. The matrix entry described
the target state, which is already the current state. A future bump is an ordinary
dependency update and does not need this ADR.

**IPC.** ADR 0007 and its follow-ups already moved the local IPC to the OS-neutral
`kinekino_core::platform::ipc` port: the Unix socket and Windows named-pipe adapters (P2-3),
`local_host` without its `cfg(windows)` gate (P2-4) and the mpv addresses (P2-5). What is
left is not transport code but the Windows-only *callers*: the desktop host body in
`kinekino-host/src/player_gui` and the mpv parent-surface, review and session-engine modules
in `kinekino-player/src/infrastructure/mpv`, which are still gated because they drive Win32
windows, not because of IPC. Removing those gates is part of the platform-cfg cleanup
(roadmap #36) and needs Windows runtime checks.

**Windows build.** A Windows-target clippy run
(`cargo clippy --target x86_64-pc-windows-msvc --workspace --exclude kinekino-platform-unix
--all-targets -- -D warnings`) showed that `kinekino-player-ui` no longer compiled for
Windows: the X11 edge-resize fallback in `window/edges.rs` sat in an ungated `impl` block, so
it clashed with the non-X11 stubs and used X11-only fields. This blocks any Windows work, so
it is fixed here first.

## Decision

Stage 1 (this change):

- Put the X11 edge-resize methods in their own `cfg(all(unix, not(target_os = "macos")))`
  impl block, and mark the pure geometry helpers that only the X11 path and the unit tests
  use as allowed dead code elsewhere. The Windows-target clippy run above now passes.
- Drop the 12 structure-allowlist entries whose files were deleted with the old egui
  presentation layer (`cargo xtask lint-structure` reported them as fixed). The allowlist goes
  from 361 to 349 entries with no new violations.

Later stages, each its own branch and merge:

1. Stage 2: split the over-long files in the allowlist's `file-lines` group (38 entries)
   where the split is mechanical and covered by Linux tests.
2. Stage 3: move the `r-cfg` call sites (80 files) behind platform ports, starting with the
   ones that only pick a path or a process API, and keep Windows-target clippy green.
3. Stage 4: ungate the mpv Win32 surface and the desktop host body. This needs the Windows
   desktop to check, so it stays with the P8 Windows verification.

`kinekino-platform-unix` is excluded from the Windows-target run because it is a Unix
adapter crate; making the workspace build without that flag is roadmap item #38.

## Consequences

Windows compile checks are possible on the Linux box again and are part of the gate for the
later stages. Nothing here has been run on Windows itself.
