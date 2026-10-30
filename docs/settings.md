---
title: "Settings and privacy"
description: "One settings page, also scriptable with kinekino settings. The settings window ships in P6 on Windows, Linux and macOS."
ref: settings
nav_order: 21
hero: { c: split, visual: player-mock, scene: settings }
sections:
  - { c: prose }
  - { c: prose }
  - { c: prose }
---

Settings live on one page, and the same keys are readable and writable with `kinekino settings`. `kinekino settings gui [YYYY-MM]` opens the page in a native window drawn by Meari, the same code on Windows, Linux and macOS, with the product's design tokens, icons and Pretendard font. Edits apply all-or-nothing, and the spending-limit dialog appears near the cap. `kinekino settings page|apply` stays for scripts. If the credential store cannot answer, only the token row shows the store error; the rest of the page still loads.

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
- Tokens go to the OS credential store. Settings show only whether one is set.
- Settings load in this order: defaults, file, environment, command line.

> [!TIP]
> Choose local only if nothing should leave your computer. AI features then use local model packs only.
