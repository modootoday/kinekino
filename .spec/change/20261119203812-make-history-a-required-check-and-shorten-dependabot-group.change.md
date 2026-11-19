---
id: change:kinekino/20261119203812
title: Make History a required check and shorten Dependabot group names
status: active
created: 2026-11-19T20:38:12Z
updated: 2026-11-19T20:38:12Z
rel:
  governs: [.github/rulesets/main-protected.json, .github/dependabot.yml, .agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261119194255, adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, change:kinekino/20261116111105, change:kinekino/20261116135248, change:kinekino/20261119194431, change:kinekino/20261119200443, evidence:kinekino/20261109092655, evidence:kinekino/20261117193255]
---

# Make History a required check and shorten Dependabot group names

## Change

This records the decision in `adr:kinekino/20261119194255` (Record, 2026-11-19).

- **`.github/rulesets/main-protected.json`.** `History` joins the required checks: `lint`, `Public surface`, `Secret scan`, `test (ubuntu-26.04)` and `History`.
- **`.github/dependabot.yml`.** The cargo group is renamed from `cargo-minor` to `cargo`. A comment explains why group names stay short.
- **`.github/workflows/ci.yml`.** Only comments change. `History` is no longer called advisory.
- **Rules.**
  - `AGENTS.md`, `.agents/rules/git-and-release.md` (H6), the shared-rules profile and `sot:kinekino/project-status` list the five required checks.
  - H6 says how a behind branch catches up: by rebase, and with `@dependabot rebase` for Dependabot.

## Verification

- **Every pull request since #58 passed `History`.** These are #58, #59, #61, #62 and #64, in CI runs between 2026-11-19 19:30 and 20:35 UTC. The only other run was cancelled because a newer push replaced it.
- **Past Dependabot pull requests (#1, #39) would have failed.** `playpilot history-check --method merge` over their ranges reports two kinds of finding:
  - merge commits from "Update branch";
  - the 73-character group subject "chore(deps): bump the cargo-minor group across 1 directory with 2 updates".
- **The renamed groups fit the limit.** With the new names, the longest subjects for up to 99 updates are 68 characters (`chore(deps): bump the cargo group …`) and 67 characters (`ci(deps): bump the actions group …`).
- **Checks on this change.** `spec check --changed origin/main` and `playpilot history-check` pass on this branch.


## Migration

The ruleset takes effect only after the owner applies it from their PC (common rule C2.7). Ruleset 27354229 is `main-protected`:

```sh
gh api -X PUT repos/modootoday/kinekino/rulesets/27354229 --input .github/rulesets/main-protected.json
```

Then check that the required contexts now include `History`:

```sh
gh api repos/modootoday/kinekino/rulesets/27354229 --jq '.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context'
```
