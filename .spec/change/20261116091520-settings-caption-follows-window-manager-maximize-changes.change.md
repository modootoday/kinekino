---
id: change:kinekino/20261116091520
title: Settings caption follows window-manager maximize changes
status: active
created: 2026-11-16T09:15:20Z
updated: 2026-11-16T09:15:20Z
rel:
  depends_on: [adr:kinekino/20261105191533, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Settings caption follows window-manager maximize changes

## Change

`kinekino-settings-ui`: a new `MaximizeFollow` (`src/maximize.rs`, exported with `MAXIMIZE_SETTLE`) decides the caption's maximize state. The window records caption requests in it. On `SurfaceResized` and `Moved` events it reads `is_window_maximized` back and syncs the caption icon and label with a full prop commit.

## Migration

None.
