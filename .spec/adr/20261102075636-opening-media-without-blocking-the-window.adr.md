---
id: adr:kinekino/20261102075636
title: Opening media without blocking the window
status: active
created: 2026-11-02T07:56:36Z
updated: 2026-11-02T07:56:36Z
rel:
  governs: [crates/kinekino-player-ui/src/controller/opening.rs, crates/kinekino-libmpv/src/opening.rs]
---

# Opening media without blocking the window

## Context

Opening a file or stream waited on the window's thread until libmpv had a first frame: up to 10 seconds for the source to start and 5 more for the frame. A slow stream froze the window for that long, with no redraw, no Esc and no way to pick something else. Plugin calls had already moved off the UI thread ([ADR](20261031102400-plugin-calls-off-the-ui-thread.adr.md)); the engine open was the remaining wait.

## Decision

- The engine stays on the window's thread and the open becomes step-wise instead. libmpv's render context must not be used from two threads at once, and `loadfile` is already asynchronous inside libmpv, so a worker thread would add locking without removing any wait.
- The `Engine` port has `begin_load` (queue the file, return at once) and `load_ready` (Ok(false) while opening, Ok(true) at the first frame, an error when it failed). Neither may block.
- The libmpv adapter remembers the playlist entry id of the file it queued and only counts start, end and restart events of that entry. Events of a replaced file can no longer end or finish the new open. The 10 s and 5 s limits stay, measured from `begin_open` and from the restart.
- The controller keeps the open in flight and checks it on every window tick while it lasts (instead of every 250 ms poll). Until it ends the view says so where the user is: the start screen's status line with the loading mascot, a notice over a video that is being replaced, or the browse view's loading line.
- A newer choice replaces the open; Esc (after closing an open ⋮ menu) cancels it and stops the engine. The result of a replaced or cancelled open is ignored.
- `LibMpvSurface::open` and `Controller::finish` keep a blocking form for headless callers and tests.

## Consequences

- The window keeps drawing and taking input while a slow source opens.
- Cancelling an open that replaced a playing video leaves no video: the start screen (or the browse view) shows.
- The limits are still fixed; a source that needs longer than 10 s to start still fails.
