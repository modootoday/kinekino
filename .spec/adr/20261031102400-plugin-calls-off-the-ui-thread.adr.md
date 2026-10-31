---
id: adr:kinekino/20261031102400
title: Plugin calls off the UI thread
status: active
created: 2026-10-31T10:24:00Z
updated: 2026-10-31T10:24:00Z
rel:
  governs: [crates/kinekino-player-ui/src/controller/jobs.rs]
---

# Plugin calls off the UI thread

## Context

Every source plugin call starts a process and waits for its answer. The controller made those calls from the window's event loop, so a slow site froze the window: no redraw, no Esc, no close.

## Decision

- `browse` and `resolve` run on a short-lived worker thread per call (`player-ui/src/controller/jobs.rs`). The answer comes back through a channel. The controller takes it in `settle`, which the window's poll tick calls. `Catalog` is now `Send + Sync`.
- Only one call is in flight. A new browse, back or play request replaces it; a late answer to the replaced request is dropped because its receiver is gone.
- While waiting, the browse view's status line says "불러오는 중…". The shown listing stays until the answer arrives; a refusal keeps it and says why.
- Trail changes (push, page, back) apply only when a listing arrives, so a failed call never leaves the trail out of step with the screen.
- Local files still load at once: no plugin is involved.
- Headless callers and tests wait with `Controller::finish`.

## Consequences

- A hung plugin no longer hangs the player; its thread ends when the plugin host's own timeout ends the process.
- Answers are picked up at the window's poll interval, so a fast answer shows a frame or two later than before.
- The synthetic plugin takes `KINEKINO_FAKE_DELAY_MS` to make the waiting visible on a desktop.
