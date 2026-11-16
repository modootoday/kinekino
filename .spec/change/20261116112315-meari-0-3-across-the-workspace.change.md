---
id: change:kinekino/20261116112315
title: Meari 0.3 across the workspace
status: active
created: 2026-11-16T11:23:15Z
updated: 2026-11-16T11:23:15Z
rel:
  depends_on: [adr:kinekino/20261106022549]
  governs: [Cargo.toml]
---

# Meari 0.3 across the workspace

## Change

The workspace moves the Meari crates from 0.2.0 to 0.3.0 together:
- `meari`
- `meari-native`
- `meari-dom` (imported as `blitz-dom`)
- `meari-shell` (imported as `blitz-shell`)

Bumping only `meari` or only `meari-native` (Dependabot) left 0.2 `meari-dom`/`meari-shell` in the graph. `kinekino-player-ui` then failed to build, because it mixed two `blitz_dom` versions (`SpecialElementData`, `ScrollBehavior`).

`meari-native` 0.3.0 still pins `rfd =0.17.2` and `winit =0.31.0-beta.3`. The workspace pins the same `winit`. Its own `rfd` is `=0.18.0` (already on `main`), so the lockfile carries both `rfd` 0.17.2 (through `meari-native`) and 0.18.0 (the workspace crates), as it did with `meari-native` 0.2.

No source changes were needed.

## Migration

None for callers. The lockfile now holds a single 0.3 generation of every Meari crate.
