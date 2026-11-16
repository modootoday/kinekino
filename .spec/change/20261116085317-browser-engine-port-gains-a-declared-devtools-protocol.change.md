---
id: change:kinekino/20261116085317
title: Browser engine port gains a declared DevTools protocol session
status: active
created: 2026-11-16T08:53:17Z
updated: 2026-11-16T08:53:17Z
rel:
  depends_on: [adr:kinekino/20261109071720, plan:kinekino/20261031153240, adr:kinekino/20261029143826, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261106032051, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Browser engine port gains a declared DevTools protocol session

## Change

`crates/kinekino-browser/src/engine/port/` gains `Capability::ProtocolSession` and the `ProtocolSession` trait (`protocol.rs`), plus `BrowserEngine::protocol()`, which returns `None` by default. The contract suite (`engine/contract/`) adds a `protocol-session` check. Declared tables in `kind.rs` are unchanged, so every adapter reports the capability as not declared.

## Migration

None. Adapters that later pass DevTools calls through implement `protocol()` and add the capability to their declared table in the same change.
