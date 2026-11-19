---
id: change:kinekino/20261119180237
title: WebView2 adapter declares the DevTools protocol session
status: active
created: 2026-11-19T18:02:37Z
updated: 2026-11-19T18:02:37Z
rel:
  governs: [crates/kinekino-browser/src/engine/webview2]
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261105201855, adr:kinekino/20261106032051, adr:kinekino/20261109071720, change:kinekino/20261116125525, evidence:kinekino/20261116135502, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# WebView2 adapter declares the DevTools protocol session

## Change

This is the next follow-up slice of `adr:kinekino/20261109071720`, in the order recorded there on 2026-11-19. The WebView2 adapter (`kinekino-browser`, feature `webview2`, Windows only) now declares `protocol-session`, so its row in the capability table of `adr:kinekino/20261029143826` (ADR 0002) gains it.

- **How calls run (`engine/webview2/protocol.rs`).**
  - A call goes to the page, or to an attached child session through `ICoreWebView2_11::CallDevToolsProtocolMethodForSession`.
  - WebView2 completes calls on the WebView's STA thread. The synchronous port `call` waits there while pumping that thread's messages with `webview2_com::wait_with_pump`.
  - A `WM_QUIT` that arrives during the wait ends the call with an error.
- **Which events are forwarded.**
  - One receiver is registered per event name: the set the provider's own observer reads, plus `Runtime.executionContextCreated`.
  - Events are buffered until drained. Each carries the child's session id, or none for the page.
  - An event whose parameters are larger than 128 KiB, or are not JSON, is dropped.
  - The receivers are removed when the engine is dropped.
- **Auto-attach.** `set_auto_attach` sends `Target.setAutoAttach` with flatten, and pauses new children on start while auto-attach is on. This is the same as the Chrome adapter.
- **Constructor.** `WebView2Engine::from_view` now returns `EngineResult<Self>`, because registering the receivers can fail. No product path constructs this engine yet.
- **Windows demo.** The new example `demo_engine_webview2` opens a Win32 window, runs the shared contract, and checks that an auto-attached dedicated worker takes a call on its child session.
- **Contract test.** It now expects `protocol-session` from the Chrome adapters and from WebView2.

The browser provider still uses its own observer. Moving it onto the port is the next slice, after the Windows run below passes.

## Verification

- **Linux.** On x86_64 Linux:
  - `cargo fmt --check`: passes.
  - `cargo clippy --locked --all-targets -- -D warnings`: passes.
  - `cargo xtask lint-structure`: 0 new violations.
  - `cargo test -p kinekino-browser -p kinekino-app`: passes. The declaration test failed before the capability change.
- **Windows build.** This passes, built from Linux:

  ```bash
  cargo clippy --locked --target x86_64-pc-windows-msvc -p kinekino-browser --features webview2 --all-targets -- -D warnings
  ```

  It covers the adapter, the protocol module and the demo.
- **Not verified yet.** A run against a real WebView2 is pending, on Windows only. The command is below; the expected result is no `Fail` lines and `child-session Pass`:

  ```bash
  cargo run --locked -p kinekino-browser --features webview2 --example demo_engine_webview2
  ```

## Migration
