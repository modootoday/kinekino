---
id: adr:kinekino/20261116062843
title: "Pull request gate for main: CI on every PR, merge only after the Linux checks pass"
status: active
created: 2026-11-16T06:28:43Z
updated: 2026-11-16T06:29:08Z
rel:
  governs: [.github]
  depends_on: [adr:kinekino/20261029201516]
---

# Pull request gate for main: CI on every PR, merge only after the Linux checks pass

## Context

- ADR 0010 ([[adr:kinekino/20261029201516]]) made CI manual-only, as in specfold, and left push or pull request triggers as a user decision ("a one-line change").
- 2026-11-16 the user decided that kinekino maintenance and development run through pull requests: work on a `claude/` (or other topic) branch, open a pull request to `main`, merge only after the gate passes. This replaces the direct push to `main` that the roadmap's scheduled-work rule allowed ([[plan:kinekino/20261031153240]]) and the direct push allowed earlier the same day in one session. The decision was made together with a separate design session for unattended maintenance across the user's projects.
- Every job in `ci.yml` was guarded by a `workflow_dispatch` input (`if: inputs.linux`). A `pull_request` run has no inputs, so adding the trigger alone would skip every Linux job, and GitHub counts a skipped required check as passed: pull requests would merge with nothing checked.
- Measured cost (2026-11-16, 4 cores, cold cache): clippy about 3 min, test build about 6 min, tests about 2.5 min. Remote runs on `ubuntu-26.04` and `ubuntu-26.04-arm` were green on `main` `46496b3` (run 43597176229).

## Decision

1. `ci.yml` runs on `pull_request` to `main` and stays runnable by hand (`workflow_dispatch`). `pull_request_target` is not used (ADR 0010 item 2 stands); fork pull requests run without secrets, and CI uses none.
2. The Linux jobs (`lint`, `test`, `site`) use `if: github.event_name == 'pull_request' || inputs.linux`, so they always run on a pull request and keep the input on a manual run. No workflow-level `paths` filter: a filtered-out run leaves required checks pending.
3. `windows-gate` and `test-macos` stay manual and advisory (`continue-on-error`); they are off on pull requests and are never required checks. `window-smoke.yml` and `release.yml` keep their triggers.
4. Required status checks for `main` (repository settings, set by a person): `lint` and `test (ubuntu-26.04)`. `test (ubuntu-26.04-arm)` and `site` run on every pull request but are not required at first; make them required once their run time is acceptable.
5. Branch rule for `main` (repository settings, set by a person): require a pull request before merging, require the checks above to pass and the branch to be up to date, no force push, no deletion. Agents do not approve or merge pull requests on their own; a person merges, or a future rule says otherwise.
6. Merge method: merge commit, matching the existing `Merge branch '…'` history, so commit hashes quoted in plan records stay reachable.
7. Dependabot pull requests go through the same gate and are merged by a person (`.github/dependabot.yml`). Updates past an `=` pinned version are reviewed by hand and never merged unattended.
8. The roadmap's scheduled-work rule changes from "merge into local main, then push main" to "push the topic branch, open a pull request, merge after the gate passes"; tag, release and publication rules are unchanged.

## Consequences

- Every pull request costs roughly 30 to 40 runner minutes (lint, two test jobs, site); this is an estimate from the timings above, not a measurement. `rust-cache` saves only on `main` (`save-if`), and `main` no longer runs CI on its own, so pull requests restore the last cache a manual `main` run saved. If cold builds become the norm, add a scheduled or post-merge run on `main` in a separate change.
- The gate becomes real only after a person sets the branch rule and required checks in GitHub Settings → Rules (or Branches); until then a pull request can still be merged without passing checks. This change cannot set it.
- Verification so far: the workflow parses and the job conditions were read back from the parsed file. The pull request that adds this decision is the first `pull_request` run and is the evidence that the Linux jobs run instead of skipping. actionlint was not run in this session.
- Not checked: repository visibility and billing plan, and whether the `test (…)` check names show exactly as written once the rule is created.
- skill: modoo.today skills search for CI pull request gating | 적용 없음: the closest matches cover skill-repository approval and headless agent CI, not this workflow change.
