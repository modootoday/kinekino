---
id: adr:kinekino/20261031111316
title: UI mockups and mascot brand icons
status: active
created: 2026-10-31T11:13:16Z
updated: 2026-10-31T12:25:00Z
rel:
  governs: [crates/kinekino-player-ui/examples/ui_mockups.rs, tools/brand-icons/derive.py, crates/kinekino-player-ui/src/player.css, crates/kinekino-player-ui/src/styles.rs, docs/_includes/head-custom.html, assets/design/brand-source/kinekino-mark.svg, assets/design/brand-source/kinekino-mark-f_1.6.svg, assets/design/brand-source/kinekino-mark-f_3.3.svg, assets/design/brand-source/combo_scared-v7.png, crates/kinekino-settings-ui/src/styles.rs]
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

The logo and the mascot are one character in two roles. The mascot part-layer guide
(`.spec/guide/mascot-part-layers.guide.md`) says "정지 상태는 모든 weights 0, eyes.scale 0 →
단색 유지": at rest the mark is the plain body with no eyes, inner ears or band, and that rest
state is the logo. The reveal layer brings the eyes and details in, and the app-state combos in
`.spec/guide/mascot-preset-reference.guide.md` (`loading`, `success`, `error`, `notification`,
`offline`, idle) move them. So the logo is shown where nothing is happening, and a situational
mascot only where the screen carries a state.

`tools/brand-icons/derive.py` copies shapes or rasterises frames from the brand files kept
verbatim in `assets/design/brand-source/`; it draws nothing:

- logo, from `kinekino-mark.svg` (the logo's first frame, eyeless): `docs/assets/img/favicon.svg`
  on the logo's dark plate, `assets/design/icons/logo.svg` without the plate, app icon PNGs
  from 16 to 512 px and `kinekino.ico` under `assets/design/app-icon`, and the site's
  `favicon.ico`, PNG favicons, `apple-touch-icon.png`, 192/512 icons and `site.webmanifest`.
  The 16 and 24 px sizes are the same eyeless shape rendered small;
- `mascot-idle.png` from frame `kinekino-mark-f_1.6.svg` (revealed, eyes open);
- `mascot-loading.png` from frame `kinekino-mark-f_3.3.svg` (mid-blink; `loading` is blinks);
- `mascot-error.png`, the last panel (t = 2.0 s) of the 3D render `combo_scared-v7.png`, with
  the render background keyed out. `scared` is the state machine's fatal-error combo; no render
  of the `error` combo's worried eyes exists, so this is the nearest frame.

Rerun it whenever the logo or a frame changes. The settings asset path now declares `.png`
files as images so the 3D render can be served.

| Screen | Shows |
| --- | --- |
| Title bar, start screen | logo |
| Browse, no sources (mockup only; the product shows this on the start screen status) | idle |
| Browse, loading a listing | loading |
| Browse, source error | error |
| Everything else | logo in the title bar only |

The browse status line carries `kk-mood-{idle,loading,error}`, set by the controller.

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

A second correction followed: the first fix used the eyes-open frame as the logo. The logo is
the eyeless first frame; the eyes-open frame is now the idle mascot state only.

## Playlist panel (PotPlayer-style list)

- The panel docks inside the video stage, below the title bar and above the full-width transport, so the window buttons never move (draft `v2/viewing/native-player-playback`).
- Rows show a current-item marker, number, title and duration. The host type is a generic `PlaylistItem` (playable item); `View.playlist` replaces the old site-shaped field names.
- A list filter field and a bottom toolbar (추가 · 삭제 · 정렬 · 반복 · 무작위) are drawn disabled ("아직 연결되지 않음"). Out of scope for now: multi-select, drag reorder, the context menu, collapsing items that have a sub-list, and detaching the panel.

## 작품 탐색 only with a browse plugin

- `View.has_browse_plugin` is true only when the attached catalog lists at least one source; otherwise the start screen offers only 파일 열기. The host knows just this flag, with no plugin names or site shapes. Mockups: `start-no-plugin`, `start-with-plugin`.
