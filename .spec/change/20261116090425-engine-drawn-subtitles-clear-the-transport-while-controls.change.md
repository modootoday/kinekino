---
id: change:kinekino/20261116090425
title: Engine-drawn subtitles clear the transport while controls show
status: active
created: 2026-11-16T09:04:25Z
updated: 2026-11-16T09:04:25Z
rel:
  depends_on: [adr:kinekino/20261105101125, sot:kinekino/player-window-controls, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031055110, adr:kinekino/20261103115459, adr:kinekino/20261105092851, adr:kinekino/20261105111300, adr:kinekino/20261105131646, adr:kinekino/20261105141813, adr:kinekino/20261105151827, adr:kinekino/20261105161614, adr:kinekino/20261105182156, adr:kinekino/20261106105936, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Engine-drawn subtitles clear the transport while controls show

## Change

`kinekino-player-ui` controller: `poll_subtitle` places engine-drawn cues. While the engine draws the track and the controls show, it sends `set sub-pos N`, where N = floor(100 × (1 − 56 / osd-height)). Otherwise it sends 100. A value goes out only when it changes, and nothing is sent until the box height is known. `TRANSPORT_HEIGHT_PX` (56) is exported, and a test keeps it equal to `--aw-transport-height` in tokens.css.

## Migration

None. The overlay (text) path is unchanged.
