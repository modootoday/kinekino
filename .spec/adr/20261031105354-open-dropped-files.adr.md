---
id: adr:kinekino/20261031105354
title: open dropped files
status: active
created: 2026-10-31T10:53:54Z
updated: 2026-10-31T10:53:54Z
rel:
  governs: [crates/kinekino-player-ui/src/window/drop.rs]
---

# Open dropped files

## Context

The player could open a local file only through the Open dialog (Ctrl+O or the menu).
Dragging a video from a file manager onto the window did nothing, which the
implementation matrix tracked as missing. On the pinned winit 0.31 beta there is no
`DroppedFile` event any more; drops arrive through the asynchronous data-transfer API.

## Decision

The window accepts a drag as a copy when it enters, asks for the `text/uri-list` data when
it is dropped, and, when the list arrives, opens the first entry that is a regular file
through the same `Controller::open` path the dialog uses. That keeps one behaviour for both
ways of opening: the folder of the file becomes the playlist and a playlist that no longer
fits the mounted markup triggers the usual remount. The shared "after open" step is one
`App::opened` helper so the dialog and the drop cannot drift apart.

Only local files are handled. Folders, several files at once and dropped URLs are ignored on
purpose: a URL would bypass the plugin and its declared media hosts, and multi-file drops
need a decision about playlist order that nobody has asked for yet.

## Consequences

The code lives in its own `window/drop.rs` so `window/mod.rs` stays under the structure
limit. Platforms whose winit backend does not support data transfer simply never accept the
drag; the dialog still works there.
