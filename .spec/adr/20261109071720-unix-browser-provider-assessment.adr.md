---
id: adr:kinekino/20261109071720
title: Unix browser provider assessment
status: active
created: 2026-11-09T07:17:20Z
updated: 2026-11-16T10:20:00Z
rel:
  governs: [crates/kinekino-player/src/application/browser_provider]
  depends_on: [adr:kinekino/20261029143826, adr:kinekino/20261109021423]
---

# Unix browser provider assessment

## Context

The roadmap step M5-8 (OS-4, plan item #28) asks whether the browser provider can run on
Unix: which existing engine adapters it would use, what runtime they need, and what counts as
supported. It is an assessment only; no adapter code belongs to this step. ADR 20261109021423
(M5-4) moved `playback_host` and `provider_host` here because they depend on the browser
engine, not on the Win32 surface.

Facts read on the Linux dev host on 2026-11-09 at main `e665ffe7`:

- The provider jobs implement `BrowserProviderPort`
  (`kinekino-player/src/application/browser_provider/ports.rs`). Its only implementation,
  `kinekino-player/src/infrastructure/browser_provider/`, is compiled only on Windows
  (`#[cfg(windows)] pub mod browser_provider`). It drives WebView2 through `wry` and
  `webview2-com` on a COM apartment with a hidden Win32 parent window (`window.rs`), and reads
  the page through raw DevTools protocol calls and events: `Network.enable`,
  `Network.requestWillBeSent`, `Network.requestWillBeSentExtraInfo`,
  `Network.responseReceived`, `Network.loadingFinished`, `Network.loadingFailed`,
  `Network.getResponseBody`, `Page.enable`, `Page.getFrameTree`, `Page.frameAttached`,
  `Page.frameNavigated`, `Page.frameDetached`, `Target.setAutoAttach`,
  `Target.attachedToTarget`, `Target.detachedFromTarget` and
  `Runtime.runIfWaitingForDebugger`.
- `kinekino-host` compiles `playback_host` and `provider_host` only on Windows; on Linux the
  provider and playback modes do not exist.
- `kinekino-browser` already has the engine port of ADR 0002 (20261029143826) with four
  adapters: `Cef` and `SystemChrome` (installed Chrome embedded over X11, controlled over the
  DevTools protocol) as the Linux X11 default and fallback, `ChromeCdp` (headless or separate
  window) for the CLI and checks, and `WebView2` on Windows. Selection is by declared
  capabilities and runtime presence (`engine/select/`).
- The engine port (`BrowserEngine`) offers navigate, eval, script injection, an Allow/Block
  request interceptor, a small event list, audio mute and PNG capture. It has no way to
  observe network responses, read response bodies, follow frames or auto-attach to child
  targets. The provider needs all of these, so the port as it stands cannot carry the
  provider on any engine.
- CEF declares no request interception and no frame capture in its current adapter, and has
  no DevTools protocol pass-through in the adapter.
- Runtime on this host: Google Chrome 156.0.8078.47 at `/usr/bin/google-chrome`, an X11
  display (`DISPLAY` set), no Wayland display, `KINEKINO_CEF_DIR` unset (no CEF runtime), and
  no WebKitGTK or GTK development package.

## Decision

1. **No new engine.** The Unix provider runs on the existing Chromium adapters of ADR 0002.
   WebKitGTK (through `wry` on Linux) is not adopted: ADR 0001 and ADR 0002 already set
   Chromium as the only engine family because the provider reads Chromium DevTools protocol
   traffic and needs H.264, AAC and Widevine. No other engine is added for this step.
2. **The engine port grows a protocol session, as a declared capability.** The first code
   slice adds a `ProtocolSession` capability to `kinekino-browser`'s port: send a DevTools
   method with JSON parameters, receive events by name, and auto-attach to child targets. An
   adapter declares it only if it really passes calls through; an adapter that does not
   returns `Unsupported`, as for every other capability. The provider's protocol observer,
   network worker, frame tracking and media observer move behind this capability instead of
   calling WebView2 directly.
3. **Which engine serves the provider on Linux.** The provider asks the selector for
   `ProtocolSession`, `RequestInterception`, `ScriptInjection` and `Eval`. On Linux X11 that
   selects the installed Chrome adapter (`SystemChrome`, or `ChromeCdp` where no window is
   needed). CEF stays the Linux default for browser pages that need none of these, and is
   chosen for the provider only after its adapter declares and passes those capabilities.
   When no engine is present with the required capabilities, provider jobs fail with
   `Unsupported` and the selector's missing-capability list. There is no automatic fallback
   to another engine family and no silent downgrade to fewer capabilities.
4. **The Windows provider keeps working unchanged.** The WebView2 code is not rewritten in
   this decision. After the port gains the protocol session, the WebView2 adapter declares it
   through its existing DevTools calls; moving the provider onto the port is a later slice
   and needs a Windows run before it is called done.
5. **Hosts follow the provider.** `provider_host` and `playback_host` keep their
   `cfg(windows)` gates until a provider implementation exists on Unix and a Linux run of the
   provider host passes its tests. Removing a gate without that run is not allowed.
6. **What "supported" means.** Linux X11 with an installed Google Chrome is the supported
   Unix target, and it counts as done only after a provider contract test runs against the
   real Chrome adapter on Linux. Linux with CEF counts only after a run with a CEF runtime;
   this host has none, and downloading one still needs the user's approval (ADR 0001). Native
   Wayland and macOS are unsupported, as in ADR 0002, and report `Unsupported` rather than a
   partial provider.

## Alternatives rejected

| Alternative | Why not |
|---|---|
| WebKitGTK through `wry` on Linux | Not Chromium: no DevTools protocol traffic for the provider, different codec and DRM support; ADR 0001 and ADR 0002 rejected it. |
| Let the provider call chromiumoxide directly on Linux | It would bypass the engine port and its capability selection, and give the provider a second, Linux-only code path. |
| Implement the provider on CEF first | The current CEF adapter declares no interception and no protocol pass-through, and no CEF runtime is available to run it here. |
| Ungate `provider_host` now and return errors at run time | The host would claim a mode that cannot do its job on Linux; the gate stays until a real implementation passes. |
| Fall back to a headless engine when the embedded one is missing | It changes what the user sees and which cookies the page uses without telling them; selection fails instead. |

## Consequences

- No behaviour changes in this step; on Linux the provider and playback host modes still do
  not exist.
- Follow-up slices, each test-first: the `ProtocolSession` capability with a fake-engine
  contract test and an `Unsupported` default; the Chrome adapters declaring it, checked with
  the existing headless contract run on Linux; the provider's protocol, network, frame and
  media observers moved behind the capability with an in-process fake; a Linux provider
  implementation on the Chrome adapter; then the provider host ungated on Linux after its
  run. The WebView2 adapter declaring the capability needs a Windows machine.
- Linux users need Google Chrome installed for provider jobs; the error names the missing
  engine and capabilities.

## Reversal conditions

- Reconsider CEF as the provider engine on Linux if its adapter gains protocol pass-through
  and interception and passes the same contract run with a CEF runtime.
- Reconsider native Wayland if an embedding path (for example off-screen rendering) is
  adopted for the player window.
- Reconsider the engine family if the provider stops depending on DevTools protocol traffic.

## Record

- 2026-11-09: M5-8 (OS-4) assessed the Unix browser provider without code changes.
- 2026-11-16: first follow-up slice done. `kinekino-browser`'s engine port has `Capability::ProtocolSession` (`protocol-session`) and a `ProtocolSession` trait: `call(session, method, params)` to the page or an attached child session, `drain_protocol_events` and `set_auto_attach`. `BrowserEngine::protocol()` returns `None` by default. The common contract suite checks that a declared protocol session answers `Runtime.evaluate` and toggles auto-attach, and that an undeclared one is absent; in-memory fakes cover pass, missing accessor and not declared. No adapter declares it yet, and a test keeps that so until an adapter passes calls through (next slice: the Chrome adapters, checked with the headless contract run on Linux).
- 2026-11-16: The Chrome adapters (`ChromeCdp`, and `SystemChrome` through it) now declare `protocol-session`. `call` sends any method by name with JSON params to the page session, or to an attached child target chromiumoxide knows; an unknown session is an error. Events are forwarded for a fixed set, because chromiumoxide only has typed listeners: `Network.requestWillBeSent`, `responseReceived`, `loadingFinished`, `loadingFailed`, `Page.frameAttached`, `frameDetached`, `frameNavigated`, `Target.attachedToTarget`, `detachedFromTarget`. Page-session events carry no child id. Forwarding child-session events still remains. `set_auto_attach` sends `Target.setAutoAttach` (flatten, paused on start while on). Checked against headless Google Chrome on Linux: the shared contract reports `protocol-session Pass`, and a fixture run sees `Network.responseReceived` for the page and `Page.frameNavigated`. The provider and playback hosts stay gated.
