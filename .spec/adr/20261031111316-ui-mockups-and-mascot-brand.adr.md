---
id: adr:kinekino/20261031111316
title: UI mockups and mascot brand icons
status: active
created: 2026-10-31T11:13:16Z
updated: 2026-10-31T12:25:00Z
rel:
  governs: [crates/kinekino-player-ui/examples/ui_mockups.rs, tools/brand-icons/derive.py, crates/kinekino-player-ui/src/player.css, crates/kinekino-player-ui/src/styles.rs, docs/_includes/head-custom.html, assets/design/brand-source/kinekino-mark-f_1.6.svg]
---

# UI mockups and mascot brand icons

## Context

The design drafts show the player in states that are hard to reach by hand: a playlist
open mid-playback, two subtitle lines, the overflow menu, an empty or failing source. To
compare the drafts with the real renderer, every state has to be drawn by meari itself, not
redrawn in an image editor, and without real media or private catalogue names.

The drafts also use a placeholder product name and a play-triangle mark. kinekino already has
a finished mascot: the orange cat with a V-notch top, dark-orange inner ears, oval black eyes
and a light band along the bottom edge, drawn as the animated logo and rigged as the 3D model
in `assets/mascot`. The player and the site only used the triangle and an older SVG favicon.

## Decision

Mockups are an example binary, `ui_mockups`, in `kinekino-player-ui`. It fills a `View`
with a preset state and hands it to the normal `window::run`, with a still engine that paints
a procedural frame and reports a fixed position, duration and subtitle cue. Nothing in the
library changes to support it, so a mockup is always the product renderer and goes stale the
moment the product does. Run `cargo run -p kinekino-player-ui --example ui_mockups -- --list`
to see the screens and pass `<screen> out.png` to capture one. All text in it is invented
(`샘플 재생목록`, `예시 항목 NN`, `샘플 소스 A`).

All brand files are derived from one source, `assets/design/brand-source/kinekino-mark-f_1.6.svg`:
a verbatim copy of the eyes-open front frame (t = 1.6 s) of the mascot's animated logo,
the frame the 3D model's front view matches. `tools/brand-icons/derive.py` reads the body,
inner-ear, lower-band, tail-tip and eye shapes and colours from that file and recombines them,
so no new artwork is drawn. It writes:

- `docs/assets/img/favicon.svg`: full-colour mark on the logo's dark rounded plate;
- `assets/design/icons/mascot-color.svg`: full-colour mark without plate;
- `assets/design/icons/mascot-small.svg`: the same shapes for 16 and 24 px, without the tail
  tip and with ears and eyes scaled up around their own centres so they stay visible;
- `assets/design/icons/mascot-mark.svg`: a one-colour mask with eyes and inner ears cut out,
  for places that need a monochrome mark;
- app icon PNGs from 16 to 512 px plus `kinekino.ico` under `assets/design/app-icon`, and the
  site's `favicon.ico`, PNG favicons, `apple-touch-icon.png`, 192/512 icons and
  `site.webmanifest`.

Rerun it whenever the logo changes.

In the player, the title bar shows `mascot-small.svg`, the start screen shows `mascot-color.svg`
(both in colour, replacing the earlier mask), and browse
status messages (no sources, source errors) show the colour mascot above the text. The site
head links the new favicon set and manifest.

## Not decided here

The native window and tray icon are set in `window/*`, which is being changed on another
branch; wiring `assets/design/app-icon` there is left for after that lands. Draft screens
with no product counterpart (web browse tabs and ad-block chip, the skip overlay with AI
segments, the AI studio tabs) are not mocked: they would need host features, not states.

## Correction (2026-10-31)

The first version of this change derived every icon from the previous
`docs/assets/img/favicon.svg`, an early cat silhouette that is a different design from the
finished mascot (no V-notch body, no inner ears, no highlight band). It was taken as the
mascot without comparing it with
the logo frames. The derivation now reads the logo frame above, the old favicon was replaced
by the derived one, and every icon, favicon and mockup capture was regenerated and checked
by eye at 16, 24 and 32 px.
