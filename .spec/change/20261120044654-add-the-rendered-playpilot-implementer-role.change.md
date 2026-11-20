---
id: change:kinekino/20261120044654
title: Add the rendered playpilot implementer role
status: active
created: 2026-11-20T04:46:54Z
updated: 2026-11-20T04:46:54Z
rel:
  governs: [.agents/agents/playpilot-implementer.md]
  depends_on: [sot:kinekino/shared-automation-rules]
---

# Add the rendered playpilot implementer role

## Change

`.agents/agents/playpilot-implementer.md` is added. `playpilot sync --project kinekino` wrote it from the fleet configuration (`playpilot render roles`, playpilot `fde4784`). It is the role that fleet session prompts name. It reaches `.claude/agents/` through the existing link, and its header says to edit the fleet configuration instead of this file.

The role is looser than this repository on one point. It says to turn auto-merge on, while `needs-human` pull requests here wait for measurement first. Its first line defers to AGENTS.md where AGENTS.md is stricter, so the repository rules still win.

## Verification

`playpilot sync --check --project kinekino` reports both files up to date.

## Migration
