---
id: change:kinekino/20261116092109
title: Chrome adapters declare the DevTools protocol session
status: active
created: 2026-11-16T09:21:09Z
updated: 2026-11-16T09:21:09Z
rel:
  depends_on: [adr:kinekino/20261109071720, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029143826, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261106032051, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Chrome adapters declare the DevTools protocol session

## Change

`kinekino-browser`:
- New `chromium/cdp/protocol.rs`, which holds:
  - `RawCommand`: a run-time method name with JSON params and a JSON result;
  - session lookup;
  - typed listeners that forward the `FORWARDED` events.
- `CdpEngine` implements `ProtocolSession`, and `SystemChromeEngine` delegates to it.
- `ChromeCdp` and `SystemChrome` now declare `ProtocolSession`.

Tests:
- The contract unit test now expects exactly those two kinds to declare it.
- `tests/cdp_engine.rs` asserts `protocol-session Pass` and adds a headless-Chrome check of calls and events.

## Migration

None. Engine users that do not call `protocol()` see no change.
