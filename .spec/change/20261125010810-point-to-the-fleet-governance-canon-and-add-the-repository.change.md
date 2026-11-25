---
id: change:kinekino/20261125010810
title: Point to the fleet governance canon and add the repository permission layer
status: active
created: 2026-11-25T01:08:10Z
updated: 2026-11-25T01:08:10Z
rel:
  governs: [.claude/settings.json, AGENTS.md, .agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261119194255, adr:kinekino/20261116062843, change:kinekino/20261120032422, change:kinekino/20261116135248, change:kinekino/20261119194431, change:kinekino/20261119195313, change:kinekino/20261119200443, change:kinekino/20261119203812, sot:kinekino/shared-automation-rules]
---

# Point to the fleet governance canon and add the repository permission layer

## Change

- **Governance pointer.** `AGENTS.md` now names playpilot `.agents/rules/governance.md` (autonomy levels A0, A1 and A2, the unattended loop, escalation) and `.agents/rules/permissions.md` as the fleet canon. This repository's rules win wherever they are stricter. The private configuration repository is called only "the fleet configuration" on the public surface.
- **Repository permission layer.** `.claude/settings.json` gains a `permissions` block that follows `permissions.md`.
  - **`allow`** lists only the A0 routine commands this repository uses:
    - topic-branch git: status, diff, log, show, fetch, switch, rebase onto `origin/main`, add and commit with explicit paths, and `git push -u origin claude/…`;
    - `cargo fmt`, `check`, `clippy`, `test`, and the `xtask` checks;
    - read-only `spec` commands;
    - the `playpilot` mail, sync, history-check and render subcommands;
    - `gh pr create`, `gh pr comment`, `gh pr merge --auto`, `gh issue create` and `gh issue comment`.

    There are no broad patterns.
  - **`deny`** lists:
    - A2 commands: `gh secret`, `gh release`, `gh pr review`, `gh pr merge --admin`, `gh repo edit/delete`, ruleset commands and API writes, and `cargo publish`/`npm publish`;
    - pushes to `main` and tag pushes, and `git tag`;
    - `git add .`, `git add -A` and `git add --all`;
    - credential files.
  - **`ask`** is empty, because this repository has no deploy or Worker secret commands.
- **Guard hook.** Unchanged. `playpilot render settings` keeps the other keys, and it still reports the file as up to date.
- **Rules.** `.agents/rules/git-and-release.md` H6 describes the permission layer and its levels: adding to `deny` is A0, and adding to `allow` or `ask` is A1.

## Verification

- **Rendering.** `playpilot render settings --project kinekino` over the fleet configuration reports `.claude/settings.json` as up to date with the new block. `playpilot sync --check` reports 2 files up to date.
- **JSON.** The file parses: 28 allow rules and 24 deny rules.
- **Guard.** The guard hook still runs on every Bash command in this session.

## Migration

None.
