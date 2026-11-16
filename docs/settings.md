---
title: "Settings and privacy"
description: "One settings page, also scriptable with kinekino settings. The settings window is checked on Linux; Windows and macOS are not checked yet."
ref: settings
nav_order: 21
hero: { c: split, visual: player-mock, scene: settings }
sections:
  - { c: prose }
  - { c: prose }
  - { c: prose }
---

Settings live on one page, and the same keys are readable and writable with `kinekino settings`. `kinekino settings gui [YYYY-MM]` opens the page in a native window drawn by Meari, the same code on every OS, with the product's design tokens, icons and the pinned Noto Sans fonts (Latin, Korean, Japanese) that `kinekino fonts fetch` downloads and verifies. The window has been checked on Linux (X11); Windows and macOS have not been checked yet. Edits apply all-or-nothing, and the spending-limit dialog appears near the cap. `kinekino settings page|apply` stays for scripts. If the credential store cannot answer, only the token row shows the store error; the rest of the page still loads.

## Inference

| Key | Default |
|---|---|
| Mode | cloud; or local first, local only |
| Budget total, per call, per month | $5.00, $0.50, $10.00 |
| Confirm ratio | 80% |
| Consent scope | project; or task, always (can be withdrawn) |
| Speaker diarization | off; or local |
| Local model packs | install, remove and verify as jobs |

## Agents

| Key | Default |
|---|---|
| MCP enabled | off |
| Allowed tools | read only |
| Confirm destructive calls | on |

## Privacy

- Only the segments or frames a task needs are uploaded, never a whole video.
- A token is kept in memory for the current session only. The OS credential store adapters (Linux Secret Service first) are built but not wired into the app yet. Settings show only whether a token is set.
- Settings load in this order: defaults, file, environment, command line.

> [!TIP]
> Choose local only if nothing should leave your computer. AI features then use local model packs only.
