---
id: change:kinekino/20261116094309
title: Protocol session reaches auto-attached child sessions
status: active
created: 2026-11-16T09:43:09Z
updated: 2026-11-16T09:43:09Z
rel:
  depends_on: [adr:kinekino/20261109071720, adr:kinekino/20261029143826, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261105201855, adr:kinekino/20261106022549, adr:kinekino/20261106032051, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-browser/src/engine/chromium/cdp]
---

# Protocol session reaches auto-attached child sessions

## Change

The decision is recorded in [[adr:kinekino/20261109071720]].

`kinekino-browser` (`chromium/cdp`):
- New `connection.rs` gives the protocol session its own DevTools connection. It:
  - attaches to the page target;
  - enables `Page` and `Network`;
  - sends calls with a 30-second timeout.
- `protocol.rs` now holds only message routing (`Routes`):
  - replies settle calls;
  - page-session events carry no session id;
  - events from auto-attached children (nested ones too) carry their id;
  - detached children stop taking calls;
  - a closed connection fails every call.
- Removed: `RawCommand`, the typed listeners and `FORWARDED`.
- Workspace dependency: `async-tungstenite` `=0.32.1`, already in the lockfile through chromiumoxide.

Tests:
- Routing unit tests.
- New headless Chrome check in `tests/cdp_engine.rs`. A dedicated worker is auto-attached, takes calls and reports `Runtime.executionContextCreated` under its session id. Before this change, the call to the worker failed.

## Migration

`kinekino_browser::engine::chromium::cdp::FORWARDED` is removed. Callers now get all events from the page session and from attached children, not just a fixed list.
