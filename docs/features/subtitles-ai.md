---
title: "Subtitles and AI"
description: "Edit subtitles by hand today; speech to text, speakers and translation are planned."
ref: subtitles-ai
nav_order: 12
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: features, cols: 2 }
---

Subtitles are data, not burned-in text. You can convert them, export a range, and fix cues and speakers line by line.

## Today

- **Convert**: SRT and WebVTT with `subtitles.convert`. `status:available`
- **Export a range**: only the cues inside a time range. `status:available`
- **Edit cues and speakers**: by hand, with revisions. `status:partial`
- **Voice activity detection**: find speech regions. `status:partial`

## Planned with AI

- **Speech to text**: whisper-large-v3 in the cloud or whisper.cpp large-v3-turbo locally. `status:planned`
- **Speaker diarization**: off by default; local pyannote community-1 when on. `status:planned`
- **Alignment**: snap cues to speech. `status:planned`
- **Translation and summaries**: per cue, reviewed before saving. `status:planned`
- **OCR and search**: read on-screen text and search all subtitles. `status:planned`
- **ASS and image subtitles**: styled rendering. `status:planned`

> [!NOTE]
> AI results arrive as proposals. Nothing is written to your subtitles until you accept it in review.
