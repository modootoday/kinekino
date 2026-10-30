---
title: "Playback"
description: "Native mpv playback with episodes, resume and skip, driven by typed commands."
ref: playback
nav_order: 10
hero: { c: hero, visual: player-mock, scene: playback }
sections:
  - { c: features, cols: 2 }
  - { c: prose }
---

kinekino hosts mpv natively and wraps every control in a typed command: `playback.open`, `playback.seek`, `playback.set_volume`. The GUI buttons, the CLI and an agent send the same command.

## What works

- **Open, play, pause, seek, volume, mute**: through the mpv host. `status:partial`
- **Episodes**: next and previous episode, autoplay countdown. `status:partial`
- **Skip intro**: manual skip markers. `status:partial`
- **Resume and preferences**: remember position, audio and subtitle track. `status:planned`
- **Browser sources**: catalog and episode pages. `status:partial`

## Per OS

Playback was checked by hand on Windows. The Linux mpv host is step P2 of the roadmap; macOS is checked in P8.

> [!IMPORTANT]
> No public build exists yet, so you cannot try playback outside the development tree.
