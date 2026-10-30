---
title: "Features"
description: "Everything kinekino is designed to do, with an honest status for each part."
ref: features
nav_order: 1
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: prose }
---

Each badge is checked against the roadmap plan and the code. `status:available` means it works in the current build, `status:partial` means some of it works or only on one OS, `status:planned` means it is designed but not built.

## Feature areas

- **[Playback](features/playback.md)**: mpv playback, episodes, resume and skip. `status:partial`
- **[Studio](features/studio.md)**: timeline ranges, clips, snapshots, review and jobs. `status:partial`
- **[Subtitles and AI](features/subtitles-ai.md)**: subtitle editing, speech to text, speakers, translation. `status:partial`
- **[Local and cloud inference](features/inference.md)**: one port for local and cloud models, with consent and a cost guard. `status:planned`
- **[AI agents](agents.md)**: CLI, request JSON and an MCP server. `status:partial`
- **[Settings and privacy](settings.md)**: one settings page for inference, playback and agents. `status:planned`

## All features

| Feature | Status | Notes |
|---|---|---|
| Discovery: `version`, `capabilities`, `schema` | `status:available` | CLI |
| Subtitle convert and range export (SRT/WebVTT) | `status:available` | CLI |
| Frame capture to PNG | `status:partial` | Local mp4/mkv/webm, needs FFmpeg |
| Snapshot export | `status:partial` | Windows today, Linux in P4 |
| Playback via mpv | `status:partial` | Windows checked by hand, Linux in P2 |
| Episode navigation, skip intro | `status:partial` | Manual skip; detection later |
| Resume and preferences | `status:planned` | P2 |
| Review of media and AI proposals | `status:partial` | |
| Projects, durable jobs | `status:partial` | Revisions and idempotency keys |
| Cards, shorts, assets | `status:partial` | Executors finish in P4 |
| Cue and speaker editing | `status:partial` | By hand |
| ASS and image subtitles | `status:planned` | P5 |
| Speech to text, speaker diarization, alignment | `status:planned` | P1b, P5 |
| Translation and summaries | `status:planned` | P5 |
| OCR and subtitle search | `status:planned` | P5 |
| Local model packs | `status:planned` | P5 |
| Inference consent and cost guard | `status:planned` | P1b |
| Settings page | `status:planned` | P6; schema decided |
| MCP server | `status:planned` | P9 |
| Meari UI (Windows) | `status:partial` | settings window on every OS (P6); player Linux GUI waits on mpv (P2) |
| Provenance of edits | `status:partial` | |
