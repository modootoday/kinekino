---
id: adr:kinekino/20261031111316
title: UI mockups and mascot brand icons
status: active
created: 2026-10-31T11:13:16Z
updated: 2026-10-31T11:13:16Z
rel:
  governs: [crates/kinekino-player-ui/examples/ui_mockups.rs, tools/brand-icons/derive.py, crates/kinekino-player-ui/src/player.css, crates/kinekino-player-ui/src/styles.rs, docs/_includes/head-custom.html]
---

# UI mockups and mascot brand icons

## Context

The design drafts show the player in states that are hard to reach by hand: a playlist
open mid-playback, two subtitle lines, the overflow menu, an empty or failing source. To
compare the drafts with the real renderer, every state has to be drawn by meari itself, not
redrawn in an image editor, and without real media or private catalogue names.

The drafts also use a placeholder product name and a play-triangle mark. kinekino already has
a mascot (the cat symbol in `docs/assets/img/favicon.svg`, which the 3D mascot in
`assets/mascot` is built from), but the player and the site only used the triangle and a
single SVG favicon.

## Decision

Mockups are an example binary, `ui_mockups`, in `kinekino-player-ui`. It fills a `View`
with a preset state and hands it to the normal `window::run`, with a still engine that paints
a procedural frame and reports a fixed position, duration and subtitle cue. Nothing in the
library changes to support it, so a mockup is always the product renderer and goes stale the
moment the product does. Run `cargo run -p kinekino-player-ui --example ui_mockups -- --list`
to see the screens and pass `<screen> out.png` to capture one. All text in it is invented
(`샘플 재생목록`, `예시 항목 NN`, `샘플 소스 A`).

All brand files are derived from the one cat symbol by `tools/brand-icons/derive.py`, which
reads the outline and eye coordinates from the SVG, so no new artwork is drawn. It writes a
one-colour mask (`mascot-mark.svg`, eyes cut out) for the CSS mask icons, a full-colour copy
(`mascot-color.svg`), app icon PNGs from 16 to 512 px plus `kinekino.ico` under
`assets/design/app-icon`, and the site's `favicon.ico`, PNG favicons, `apple-touch-icon.png`,
192/512 icons and `site.webmanifest`. Rerun it whenever the symbol changes.

In the player, the title bar mark and the start-screen mark use the mascot mask, and browse
status messages (no sources, source errors) show the colour mascot above the text. The site
head links the new favicon set and manifest.

## Not decided here

The native window and tray icon are set in `window/*`, which is being changed on another
branch; wiring `assets/design/app-icon` there is left for after that lands. Draft screens
with no product counterpart (web browse tabs and ad-block chip, the skip overlay with AI
segments, the AI studio tabs) are not mocked: they would need host features, not states.
