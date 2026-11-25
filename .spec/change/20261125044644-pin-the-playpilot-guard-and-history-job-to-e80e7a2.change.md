---
id: change:kinekino/20261125044644
title: Pin the playpilot guard and History job to e80e7a2
status: active
created: 2026-11-25T04:46:44Z
updated: 2026-11-25T04:46:44Z
rel:
  governs: [.claude/settings.json, .github/workflows/ci.yml, .agents/rules/git-and-release.md]
  depends_on: [change:kinekino/20261120032422, change:kinekino/20261125010810, adr:kinekino/20261119194255, adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, change:kinekino/20261116135248, change:kinekino/20261119194431, change:kinekino/20261119195313, change:kinekino/20261119200443, change:kinekino/20261119203812, evidence:kinekino/20261109092655, evidence:kinekino/20261117193255, sot:kinekino/shared-automation-rules]
---

# Pin the playpilot guard and History job to e80e7a2

## Change

The fleet configuration moved its playpilot pin from `fde4784` to `e80e7a2`. `playpilot sync --check` then reported `.claude/settings.json` as stale.

- **Guard hook.** `playpilot sync --project kinekino` re-rendered `.claude/settings.json`. Only the hook's pinned revision changed. The permissions block is untouched.
- **History job.** `.github/workflows/ci.yml` installs the same revision, so CI and the guard run one playpilot.
- **Rules.** `.agents/rules/git-and-release.md` H6 names the new pin.

## Verification

- After the sync, `playpilot sync --check` reports 2 files up to date.
- playpilot `e80e7a2` builds with `cargo install --locked`.
- Its `history-check --method merge` passes on the last merged topic range.

## Migration

None. On its next command, the guard reinstalls the pinned revision.
