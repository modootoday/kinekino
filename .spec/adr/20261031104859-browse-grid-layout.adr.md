---
id: adr:kinekino/20261031104859
title: browse grid layout
status: active
created: 2026-10-31T10:48:59Z
updated: 2026-10-31T10:48:59Z
rel:
  governs: [crates/kinekino-player-ui/src/source.rs, crates/kinekino-player-ui/src/player.css]
---

# Browse grid layout

## Context

The 작품 탐색 view only showed listings as a single-column list. That is fine for episode
lists, but a catalogue of series is easier to scan when several entries fit side by side,
and the implementation matrix tracked "browse views other than lists" as missing.

## Decision

The browse heading gets a layout button (`browseLayout`) next to the back button. It
toggles a `browse_grid` flag on the view, and the rows container switches between the
`kk-list` and `kk-grid` classes. The grid is plain CSS grid with three columns and tiles a
third of the panel height, so no new layout code is needed in the host. The choice is a
viewer preference, not a property of a listing, so it is kept when the user opens a
series, goes back, searches or loads more rows; it is not persisted across launches yet.

Tiles carry the title only. `SourceItem` has a title, a kind and a reference, nothing else,
and showing thumbnails would need a new wire field plus an image-fetch policy that stays
inside the plugin's declared media hosts. That is a separate decision, deferred on purpose
so this change does not touch the plugin protocol or golden files.

## Consequences

The host stays generic: it lays out whatever items the plugin returns and never guesses
artwork. A later thumbnail ADR can add an optional field and fill the tiles without
changing this toggle.
