---
title: "Use cases"
description: "What people, scripts and agents do with kinekino, and which parts work today."
ref: use-cases
nav_order: 2
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: features, cols: 2 }
---

Each use case lists the parts it needs. A case is only as ready as its least ready part.

## For viewers and editors

- **Watch a series**: episodes and skip work; resume is planned. `status:partial`
- **Make a short from a scene**: range draft, short render, review. `status:partial`
- **Fix subtitles**: convert, export a range, edit cues and speakers. `status:partial`
- **Caption a talk with speakers**: speech to text and diarization. `status:planned`
- **Translate a video**: translation proposals reviewed per cue. `status:planned`

## For scripts and agents

- **Batch convert subtitles**: loop over files with `subtitles.convert`. `status:available`
- **Grab frames for a thumbnail**: `production.capture_frame` with FFmpeg. `status:partial`
- **Agent-assisted editing**: an agent writes proposals; you review. `status:partial`
- **Work fully offline**: local only mode with local packs. `status:planned`
- **Plug into an MCP client**: `kinekino mcp serve`. `status:planned`
