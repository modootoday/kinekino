---
id: adr:kinekino/20261109121745
title: macOS support and build feasibility assessment
status: active
created: 2026-11-09T12:17:45Z
updated: 2026-11-16T09:30:00Z
rel:
  governs: [crates/kinekino-platform, crates/kinekino-platform-unix]
  depends_on: [adr:kinekino/20261109071720, adr:kinekino/20261109101726]
---

# macOS support and build feasibility assessment

## Context

The roadmap step M6-6 (OS-3) asks whether macOS can be supported and built, judged from the
Linux dev host. It is an assessment only: no macOS build, real-window check or test run is
possible here, and this record does not claim one. Signing, notarization, store submission and
release stay out of scope.

Facts read on the Linux dev host on 2026-11-09 at main `f45b0f74`:

- **Adapter selection.** `kinekino-platform` picks the OS adapter with `cfg(windows)` /
  `cfg(not(windows))`, so a macOS build uses `kinekino-platform-unix`. The core target model
  already names macOS (`OsId::MacOs`, `"macos"`).
- **Process start stamp.** `kinekino-platform-unix::ipc::process_start` reads
  `/proc/<pid>/stat` and returns `Unsupported` on any OS other than Linux. `UnixIpc` calls it in
  `fresh_endpoint` and when it identifies a peer, so on macOS no IPC endpoint can be created and
  no peer can be verified. Every feature that talks to a worker process over this transport
  therefore fails on macOS, and the unit test `process_start(std::process::id()).unwrap() > 0`
  would fail in the advisory macOS CI job.
- **Process ownership.** The POSIX adapter already reports `cleanup_on_owner_death` and
  `affinity_budget` as `Unsupported` outside Linux (no `PR_SET_PDEATHSIG`, no CPU affinity);
  process groups and `RLIMIT_AS` remain `Advisory`. This is an honest report, not a gap to hide.
- **Socket paths.** The IPC adapter already refuses socket paths of 104 bytes or more, the
  BSD/macOS `sun_path` limit.
- **Credential store.** The OS store adapter is `SecretServiceCredentialStore`, compiled only on
  Linux. macOS has no Keychain adapter, so only the in-process session store exists there and a
  saved token would not survive a restart.
- **Player window.** The window is Meari's native driver (winit + softbuffer), one code path for
  every OS. The X11 presence check, window-edge handling and the X11 window-action fallback are
  compiled only for `all(unix, not(target_os = "macos"))`; on macOS window actions go through
  Meari's action port with no fallback. The Linux tray (StatusNotifierItem over `zbus`) is
  Linux-only, so macOS has no tray or menu-bar item.
- **Playback.** `kinekino-libmpv` already loads `libmpv.2.dylib` on macOS, and the CI macOS job
  takes it from Homebrew (`brew install mpv ffmpeg`, `KINEKINO_LIBMPV`).
- **Browser provider.** ADR 20261109071720 already leaves macOS unsupported for the browser
  provider; the embedded system Chrome adapter is Linux-only and the provider/playback host
  modes are Windows-only.
- **State paths.** Resume and skip-range files go to `HOME/.local/state/kinekino/` when `HOME`
  is set, so on macOS they land in that XDG-style folder rather than under `Library/`.
- **CI and release.** `ci.yml` has an advisory `test-macos` job (macos-15 and macos-15-intel,
  run by hand) that has never run remotely (evidence 20261109092655). `release.yml` builds
  `kinekino-cli` and `kinekino-app` for `x86_64-apple-darwin` and `aarch64-apple-darwin`; no
  release has run. The public site lists macOS as `status:planned` with no build.
- **Cross-check from Linux.** With the Rust standard library for `aarch64-apple-darwin` added to the
  pinned toolchain, `cargo check --locked --target aarch64-apple-darwin --workspace --lib --bins`
  finishes with no errors and no warnings. This is a type check only: nothing was linked, no
  Apple SDK was used, test targets were not checked, and no code ran on macOS.

## Decision

1. **macOS stays unsupported and `status:planned`.** Nothing in this step changes a public
   badge, and no document may call macOS supported, built or verified until the checks in
   point 4 have real records.
2. **macOS goes through the POSIX adapter, not a new platform crate.** The gaps are a few
   OS-specific functions inside `kinekino-platform-unix`, not a different model. A separate
   `kinekino-platform-macos` crate is rejected while the differences stay this small.
3. **The first code slice is the process start stamp.** Implement `process_start` on macOS with
   the kernel process-info call for the start time (`proc_pidinfo` with `PROC_PIDTBSDINFO`, or
   `sysctl` `KERN_PROC_PID`), keeping the stamp non-zero and stable for the life of the
   process, and gate the Linux `/proc` reader to Linux. Until it lands, IPC on macOS fails with
   the existing `Unsupported` error rather than a fake stamp. This is the only known blocker
   for the worker processes, so it comes before any macOS CI run is worth reading.
4. **What counts as macOS support, in order.** (a) the advisory `test-macos` job passes on both
   runners in a hand-started run; (b) a real-window smoke on a Mac: open, play, pause, seek,
   resize, fullscreen and close through Meari's window port; (c) the CLI commands that need no
   display. Each needs its own record; a Linux cross-check is not a substitute for any of them.
5. **Accepted gaps for a first macOS build.** No tray or menu-bar item, no Keychain adapter
   (the session store is used and the settings page must say a saved token will not persist),
   no owner-death cleanup or CPU affinity (already reported `Unsupported`), no browser provider,
   and state files under `HOME/.local/state/kinekino/`. Each is a later slice, not a reason to
   block the first build.
6. **Packaging stays as ADR 20261109101726 set it.** No DMG, signing or notarization until the
   libmpv supply decision and signing are settled.

Rejected:

- Declaring macOS supported because the code compiles for the target: compiling says nothing
  about libmpv, the window or IPC at run time.
- Returning a constant or zero process start stamp on macOS to make IPC work: the stamp guards
  against a reused process id, and the core model already refuses zero.
- Building a Keychain adapter or a menu-bar item first: neither blocks playback.

## Consequences

- The next macOS code work is one small slice in `kinekino-platform-unix` with its own unit
  test, runnable on Linux only as a compile check; its behaviour needs the macOS CI job.
- The public settings page says the settings window ships "on Windows, Linux and macOS"; that
  is a design statement, and the public surface check should keep it from reading as a verified
  macOS release.
- M6-6 is recorded as an assessment. macOS build and real-machine checks wait for a Mac or a
  hand-started CI run, so M6 cannot close on macOS evidence from this step.

Would reverse this: a macOS CI run or Mac check that shows a blocker outside the POSIX adapter
(for example in Meari's window driver or libmpv loading), which would call for a separate
platform crate or a different window path.

## Record

- 2026-11-16: decision 3 implemented. `kinekino-platform-unix::ipc::process_start` reads `/proc` on Linux only, asks the kernel on macOS (`proc_pidinfo` with `PROC_PIDTBSDINFO`, start time in microseconds since the epoch, at least 1), and returns `Unsupported` on other Unix systems. A new test checks the stamp is non-zero and stable for this process, non-zero for a child, and refused for a missing pid. It ran on Linux; for `aarch64-apple-darwin` the crate passes clippy with `-D warnings` (type-check only). Running it on macOS waits for a hand-started `test-macos` CI run.
