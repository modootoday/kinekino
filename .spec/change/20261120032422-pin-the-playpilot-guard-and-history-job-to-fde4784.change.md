---
id: change:kinekino/20261120032422
title: Pin the playpilot guard and History job to fde4784
status: active
created: 2026-11-20T03:24:22Z
updated: 2026-11-20T03:24:22Z
rel:
  governs: [.claude/settings.json, .github/workflows/ci.yml, .agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261119194255, change:kinekino/20261119200443, change:kinekino/20261119203812, change:kinekino/20261119195313, change:kinekino/20261119194431, adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, change:kinekino/20261116135248, evidence:kinekino/20261109092655, evidence:kinekino/20261117193255, sot:kinekino/shared-automation-rules]
---

# Pin the playpilot guard and History job to fde4784

## Change

This follows `change:kinekino/20261119200443`. The guard hook and the `History` job move from playpilot commit `4ed7daee4b5bfa4a3b0a89e7f5e2281690efec48` to `fde47840d02fe81976d60d668f7fa08ee0211d24`. That is the commit the fleet configuration's `playpilot.lock` records.

- **Guard hook.** `.claude/settings.json` is no longer written by hand. It is the output of `playpilot render settings --project kinekino` over the fleet configuration. The rendered hook differs in three ways:
  - It records the installed commit in `share/playpilot.rev` next to the CLI. When that file differs from the pin, it reinstalls with `--force`. So raising the pin also upgrades an existing install; before, the hook skipped installation whenever any CLI was present.
  - It passes `PLAYPILOT_PROTECTED_BRANCHES=main` inside the hook command instead of through the settings `env` block.
  - It still fails closed when no CLI can be found or installed.
- **`History` job.** It installs the same commit. Between the two commits, playpilot's history-check code is unchanged. The guard only got stricter: it refuses pushing a tag by its bare name, and it checks `arks` commands like `git`. The check is not weakened.
- **Rules.** `.agents/rules/git-and-release.md` H6 describes the rendered hook and the automatic reinstall.

## Verification

- **Rendered hook, run by hand on the dev host.**
  - On its first call, it reinstalled `fde4784` and wrote `share/playpilot.rev`.
  - `git status` and `git push -u origin claude/x`: allowed (exit 0).
  - `git push origin main`: refused (exit 2, "`main` is protected").
- **Diff reviewed.** `git log 4ed7dae..fde4784` in playpilot shows no change to the history-check code.
- **CI.** The pull request's own `History` job runs on the new commit.

## Migration

None. An existing install upgrades on the next guarded command.
