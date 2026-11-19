---
id: change:kinekino/20261119194431
title: Adopt playpilot commit history rules and an advisory History job
status: active
created: 2026-11-19T19:44:31Z
updated: 2026-11-19T19:44:31Z
rel:
  governs: [.agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261119194255, change:kinekino/20261116135248, adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, evidence:kinekino/20261109092655, evidence:kinekino/20261117193255, sot:kinekino/shared-automation-rules]
---

# Adopt playpilot commit history rules and an advisory History job

## Change

This change applies `adr:kinekino/20261119194255`.

- **Rules.** `AGENTS.md` and `.agents/rules/git-and-release.md` adopt playpilot H1–H7 with kinekino's values:
  - merge commit is kept;
  - every commit subject is a Conventional Commits subject;
  - a topic branch holds no merge, WIP or fixup commits at merge time;
  - an agent may rewrite only its own unmerged topic branch, and pushes it with `--force-with-lease=<branch>:<sha>`;
  - auto-merge goes on when a pull request opens, and is turned off and on again around added pushes.
- **Shared rules.** The kinekino profile in `sot:kinekino/shared-automation-rules` gains a "Commit history" row.
- **CI** (`.github/workflows/ci.yml`). A `History` job runs on pull requests. It installs playpilot at commit `0ddc0e1dfd1bac378901817a5fb106935fd5fba1` and runs:

  ```bash
  playpilot history-check --method merge --range "$BASE..$HEAD"
  ```

  The job is not a required check. Making it required is the owner's ruleset change.
- **Verification.** Run locally on this branch, `history-check --method merge --range origin/main..HEAD` reports `1 commit(s) ok`.

## Migration
