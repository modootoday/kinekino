---
id: change:kinekino/20261119213600
title: Local host transport tests run on Linux through the IPC port
status: active
created: 2026-11-19T21:36:00Z
updated: 2026-11-19T21:36:00Z
rel:
  governs: [crates/kinekino-player/tests/local_host.rs]
  depends_on: [adr:kinekino/20261030120000, change:kinekino/20261119210434, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, change:kinekino/20261116101201, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Local host transport tests run on Linux through the IPC port

## Change

The `local_host` suite drops its `#![cfg(windows)]` gate. It is not part of the pinned-tool ADR. Its blocker was the transport, not the tools.

The suite builds raw frames to test how the host handles malformed, oversized, truncated and stalled input. It sends them without going through `EndpointClient`. These frames used to go through a tokio named-pipe client opened at `\\.\pipe\<id>`. They now go through the IPC port of ADR 0007 (`adr:kinekino/20261030120000`): `os_policy::ipc().connect(&endpoint.id())`. The port opens a named pipe on Windows and a Unix socket in the private runtime directory elsewhere. The test no longer derives an OS address itself.

## Verification

- **x86_64 Linux, 9 of 9 pass.** `cargo test -p kinekino-player --test local_host` passed three runs in a row, with `XDG_RUNTIME_DIR` set to a private `0700` directory. The suite covers:
  - malformed, oversized and truncated frames;
  - host drop with a partial client;
  - transport timeout and shutdown;
  - the busy executor;
  - a stalled reader;
  - real CLI parity.
- **Runtime directory.** When `XDG_RUNTIME_DIR` is unset, or the socket path exceeds the `sun_path` limit, the host refuses to start with `PermissionDenied` (`socket_dir`). This is ADR 0007's rule: no `/tmp` fallback. GitHub's Ubuntu runners set `XDG_RUNTIME_DIR`, and other Linux suites such as `manual_skip_cli` already start a `LocalHost` there.
- **Lint and gates.** These pass:
  - `cargo clippy -p kinekino-player --all-targets -- -D warnings`;
  - the same clippy for `x86_64-pc-windows-msvc`;
  - `cargo fmt --check`.

## Migration

None.
