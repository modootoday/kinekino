---
id: change:kinekino/20261116065321
title: CI adds the fail-closed deny-terms job
status: active
created: 2026-11-16T06:53:21Z
updated: 2026-11-16T06:53:21Z
rel:
  depends_on: [adr:kinekino/20261029201516, evidence:kinekino/20261109092655, adr:kinekino/20261116065239]
---

# CI adds the fail-closed deny-terms job

## Change

`ci.yml` gains a `deny-terms` job on pull requests and pushes to `main`. It runs `cargo xtask deny-terms` over the change's commit range against the `KINEKINO_DENY_TERMS` secret and fails when the secret is missing ([[adr:kinekino/20261116065239]]). `.github/rulesets/main-protected.json` adds `deny-terms` to the required checks.

## Migration

A person registers `KINEKINO_DENY_TERMS` (one term per line) as an Actions secret and as a Dependabot secret, then re-imports the ruleset. Local runs: `cargo xtask deny-terms --range origin/main..HEAD --terms-file <list>`.
