---
id: change:kinekino/20261116073143
title: Settings as files with a read-only Settings drift check
status: active
created: 2026-11-16T07:31:43Z
updated: 2026-11-16T07:31:43Z
rel:
  depends_on: [adr:kinekino/20261029201516, adr:kinekino/20261116062843, sot:kinekino/shared-automation-rules]
---

# Settings as files with a read-only Settings drift check

## Change

Repository settings become files (shared automation rules v1.2 C2.6–C2.11, user decision 2026-11-16): `.github/labels.json` (bug, needs-owner, needs-human, dependencies) and `.github/settings/repository.json` (delete head branches on merge, merge commits only, auto-merge off, secret scanning and push protection on), next to the existing `.github/rulesets/main-protected.json`.

`cargo xtask settings-drift --repo OWNER/NAME [--issue]` (module `xtask/src/settings_drift/`) reads the live rulesets, labels and repository settings with `gh api`, compares only the keys the files state (array items paired by `type`, `name` or `context`), and reports each difference. With `--issue` it opens or updates one `needs-owner` issue. The workflow `settings-drift.yml` (job `Settings drift`) runs it weekly and on a push to `main` that changes the settings files, with `GITHUB_TOKEN` (`issues: write`). Nothing is applied from GitHub Actions and no admin token exists.

Merging moves to Phase 2 (owner instruction 2026-11-16): `repository.json` allows auto-merge, and `AGENTS.md`, `.agents/rules/git-and-release.md` and `routine-operations.md` say that the agent that opens a pull request turns on GitHub auto-merge (merge commit) unless a changed path is human-gated, judged from `git diff --name-only`, and otherwise adds `needs-human`. Agents still never approve or merge directly.

## Migration

The owner applies the files from their own computer after this merges (or tells a session on that computer to): the ruleset by name (`gh api -X PUT repos/modootoday/kinekino/rulesets/27354229 --input .github/rulesets/main-protected.json`), the labels, and the repository settings. A local run on 2026-11-16 against the live repository found these differences, all expected until then: the ruleset lacks `Public surface` and `Secret scan`; labels `needs-owner` and `needs-human` are missing; squash and rebase merges are allowed; head branches are not deleted on merge. Whether `GITHUB_TOKEN` can read ruleset details in Actions is not yet confirmed; the first run shows it.
