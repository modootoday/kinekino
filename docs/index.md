---
title: "kinekino: a video player you can script"
description: "A cross-OS Rust video player and studio. The app, the CLI and AI agents drive it through the same typed commands."
ref: index
nav_order: 0
hero: { c: hero, visual: player-mock, scene: playback }
sections:
  - { c: features, cols: 3 }
  - { c: split, visual: player-mock, scene: timeline }
  - { c: split }
  - { c: features, cols: 3 }
  - { c: platform-list }
  - { c: split, visual: player-mock, scene: settings }
  - { c: timeline }
  - { c: cta }
---

# A video player you can script.

kinekino plays, clips and captions video. Every action is a typed command, so the app, the CLI and AI agents all drive it the same way.

[Download](download.md) [Get started with the CLI](cli.md)

- Rust workspace, 20+ crates
- Typed JSON command schema
- Local first; cloud AI only with consent
- MIT license

## What it does

Badges come from the roadmap plan and the CLI's own `capabilities` output. Nothing here is a promise.

- **[Playback](features/playback.md)**: open, play, seek and switch episodes through a native mpv host. `status:partial`
- **[Subtitles](features/subtitles-ai.md)**: convert SRT/WebVTT, export a range, edit cues and speakers. `status:available`
- **[Clips and snapshots](features/studio.md)**: capture frames, export snapshots, render shorts as jobs. `status:partial`
- **[Review](features/studio.md)**: accept or reject AI proposals on a project before they change anything. `status:partial`
- **[AI subtitles](features/subtitles-ai.md)**: speech to text, speakers and translation through one inference port. `status:planned`
- **[Agents](agents.md)**: the CLI today, an MCP server on the plan. `status:partial`

## Cut clips on a timeline

Mark a range while you watch and turn it into a short, a card or a snapshot. Renders run as durable jobs that you can watch, cancel and retry.

- Range drafts and short renders `status:partial`
- Frame capture to PNG (needs FFmpeg) `status:partial`
- Snapshot export: Windows today, Linux in P4 `status:partial`

[Studio features](features/studio.md)

## Built for agents

Every command takes and returns JSON with a request ID, a revision and an idempotency key, so a script or an AI agent can call it safely and retry it.

```sh
kinekino capabilities --json
kinekino schema subtitles.convert
kinekino --input request.json --json
```

[AI-native surfaces](agents.md)

## What you can do with it

- **Watch a series**: resume where you stopped and skip the intro. `status:partial`
- **Caption a talk**: transcribe, split speakers, then fix lines by hand. `status:planned`
- **Make a short**: mark a range, render it, review it. `status:partial`
- **Automate a library**: batch convert subtitles from a script. `status:available`
- **Let an agent help**: an agent proposes edits, you review them. `status:partial`
- **Work offline**: local models with no upload at all. `status:planned`

[All use cases](use-cases.md)

## One core, every OS

A shared Rust core with OS adapters underneath. Development targets Linux first; Windows is where the current app runs; every OS is checked again at the end (P8).

- **Windows**: the current app and native mpv playback, checked by hand. `status:partial`
- **Linux**: the CLI and local IPC run; playback (P2) and the GUI (P6) are in progress. `status:partial`
- **macOS**: not verified yet; build check in P8. `status:planned`

[Platform details](platforms.md)

## Your media stays yours

Inference runs locally first when you choose it. Cloud models need your consent and stay under a cost budget. Only the needed segments or frames are uploaded, never a whole video.

> [!NOTE]
> The settings page and the inference port are planned (P1b, P6). Their defaults are already decided and listed on the settings page.

[Settings and privacy](settings.md)

## Roadmap

Order approved on 2026-10-30. Cross-OS foundation first.

1. **P0**: Cross-OS foundation
2. **P1**: Core contexts
3. **P1b**: Inference port and cost guard
4. **P2**: Linux playback
5. **P4**: Studio executors
6. **P5**: Subtitles and AI
7. **P6**: GUI, settings, mascot
8. **P9**: MCP server
9. **P8**: Windows and macOS verification

[Full roadmap](roadmap.md)

## Download

There is no public release yet. The cat is still packing. Builds will be listed here per OS.

[See build status](download.md)
