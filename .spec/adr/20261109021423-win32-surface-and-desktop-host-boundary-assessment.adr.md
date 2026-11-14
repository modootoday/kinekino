---
id: adr:kinekino/20261109021423
title: Win32 surface and desktop host boundary assessment
status: active
created: 2026-11-09T02:14:23Z
updated: 2026-11-09T02:14:59Z
rel:
  governs: [crates/kinekino-host/src/player_gui/mod.rs, crates/kinekino-host/src/lib.rs, crates/kinekino-player/src/infrastructure/mpv/parent_surface]
  depends_on: [adr:kinekino/20261031110642, adr:kinekino/20261030190000]
---

# Win32 surface and desktop host boundary assessment

## Context

ADR 20261031110642 left two Windows-only areas for its stage 4: the mpv Win32 surface in
`kinekino-player/src/infrastructure/mpv` and the desktop host body in
`kinekino-host/src/player_gui`. The roadmap step M5-4 asks which parts of these areas are
gated because they really drive Win32 windows (and so need a Windows desktop to check), and
which parts are gated only because they sit next to such code. Nothing may be ungated here
without a Windows run. This record is an assessment; it changes no code.

Inputs, read on the Linux dev host on 2026-11-09 at main `3d5ec6f3`:

- `kinekino-host` and `kinekino-player-ui` are adapter crates in the structure lint class
  table, so their platform `cfg` is allowed and is not counted in the allowlist. The mpv
  surface lives in `kinekino-player`, a domain crate, where it is gated by the
  `native-proposal-window` feature (which pulls the optional native window dependency)
  rather than by `cfg(windows)` lines.
- A throwaway probe in a scratch worktree removed three gates in `kinekino-host` (the
  execution host module, its runner branch, and the job read assembly with its tests) and ran
  `cargo check -p kinekino-host --tests` and the job read tests on Linux. The probe was
  reverted; its results are listed below as observations.

## Findings

| Area | Gate | What it drives | Class |
|---|---|---|---|
| `mpv/parent_surface/win32` (child input, construction guard, owner, preflight, release, window events, visible render-only path) | `native-proposal-window` feature | HWND creation, child window input, window messages | Windows desktop needed |
| `assembly/meari` and `presentation` in the player | `cfg(all(windows, feature = "native-proposal-window"))` / `cfg(windows)` | native proposal and viewer windows over the Win32 surface | Windows desktop needed |
| `player_gui::main` Windows branch | `cfg(windows)` | builds the SDK, starts the local host and runs the native proposal or viewer window; reports the transport as a named pipe | Windows desktop needed |
| `playback_host`, `provider_host` | `cfg(windows)` module gates | the playback workspace and provider jobs, which are WebView2 browser engine adapters (ADR 0009) | Browser engine provider (OS-4), not Win32 surface |
| `native_host`, `review_host` | `cfg(windows)` module gates | `assembly::native_playback` and `assembly::review_playback`, both Windows-gated in the player | Follows the player assembly gates above |
| `execution_host` and its `--serve-execution` runner branch | `cfg(windows)`; Linux returns exit code 3 | only the OS-neutral `local_host::execution_host` | Portable; ungating changes Linux CLI behaviour |
| `player_gui::with_job_reads` and its four tests | `cfg(windows)` / `cfg(all(test, windows))` | read-only job store assembly on the common SDK | Portable; tests need a platform policy in the test harness |

Probe observations (Linux, not a Windows run):

- With the three gates removed, `cargo check -p kinekino-host --tests` succeeds on Linux.
- Of the four job read tests, one passes and three panic because no OS policy is installed
  in the `kinekino-host` unit test process (`OsPolicy not installed: the process entry must
  call kinekino_platform::install()`). Other host unit tests (for example the shutdown tests)
  call that install at their start; these four do not. Whether they pass on Windows today is
  not known from the dev host.

## Decision

1. The Win32 surface (`mpv/parent_surface/win32`), the player `assembly/meari` and
   `presentation` gates, and the Windows branch of `player_gui::main` stay gated. They create
   or drive Win32 windows, so they move only with a Windows desktop check (P8 Windows
   verification), as ADR 20261031110642 stage 4 already says. No Linux-only change may claim
   them as done.
2. `playback_host` and `provider_host` are not Win32 surface work. They wait on the browser
   engine provider decision (roadmap OS-4, step M5-8) and are not part of stage 4.
3. `native_host` and `review_host` follow the player assembly gates they call; they are not
   ungated on their own.
4. Two parts are portable and can be done on Linux, each as its own test-first change:
   - Job read assembly: install the platform policy at the start of these tests, as the
     other host unit tests do, then drop the Windows gate from `with_job_reads` and its
     tests. Behaviour of the Windows desktop process does not change.
   - Execution host: dropping its gate makes `--serve-execution` serve over the OS-neutral
     local host on Unix instead of returning exit code 3. That is a Linux CLI behaviour
     change, so it needs a CLI test for the Unix transport and a short note on the endpoint
     scope before it lands.
5. This assessment does not change the structure allowlist. The host gates are not in it
   because the host crate is an adapter crate, and the mpv surface is gated by a feature. The
   player entries that are in it for this area (`lib.rs`, `assembly/mod.rs`,
   `assembly/meari/mod.rs`, `presentation/mod.rs`) stay until the Windows check in point 1.

## Consequences

- Stage 4 of ADR 20261031110642 is narrowed to the Win32 surface, the player native window
  assembly and presentation, and the Windows branch of the desktop host body.
- The two portable items above are follow-up candidates for the platform cfg cleanup; the
  browser-backed hosts move with OS-4.
- Nothing here has been run on Windows, and the Windows target is not installed on the dev
  host, so no Windows-target compile check was possible for this assessment.
