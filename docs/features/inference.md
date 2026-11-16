---
title: "Local and cloud inference"
description: "One inference port for local and cloud models, with consent per scope and a cost guard."
ref: inference
nav_order: 13
hero: { c: split, visual: player-mock, scene: settings }
sections:
  - { c: features, cols: 3 }
  - { c: prose }
---

All AI features call one inference port. You pick where models run; kinekino asks before anything leaves your machine and stops before a budget is exceeded. The port, consent and cost guard are built and tested against a mock server; no real model has been called yet (P7 waits for approval). `status:partial`

## Modes

- **Cloud** (default): hosted models, fastest to start.
- **Local first**: local models, cloud only when a local one is missing.
- **Local only**: nothing is uploaded.

## Guards

| Guard | Default |
|---|---|
| Mode | Cloud (change to local first or local only) |
| Cost budget per call | $0.50 |
| Cost budget in total / per month | $5.00 / $10.00 |
| Confirm before spending | at 80% of a budget |
| Consent scope | per project (task, project or always) |
| Upload | only needed segments or frames, never a whole video |
| Network timeouts | 10 s connect, 120 s request, 3 retries |
| Token | kept for the current session only for now (OS credential store not wired yet); settings show status only |

> [!WARNING]
> Cloud models may charge you. The cost guard stops a job before it goes over budget, not after.
