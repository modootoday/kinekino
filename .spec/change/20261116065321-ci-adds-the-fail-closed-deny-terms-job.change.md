---
id: change:kinekino/20261116065321
title: CI adds the fail-closed Public surface check
status: active
created: 2026-11-16T06:53:21Z
updated: 2026-11-16T07:20:00Z
rel:
  depends_on: [adr:kinekino/20261029201516, evidence:kinekino/20261109092655, adr:kinekino/20261116065239]
---

# CI adds the fail-closed Public surface check

## Change

New workflow `public-surface.yml` adds the check `Public surface` on pull requests (including title and body edits) and pushes to `main`. It runs `cargo xtask deny-terms` over the change's commit range and the pull request text against the `KINEKINO_DENY_TERMS` secret, rejects non-`users.noreply.github.com` `Co-authored-by` trailers in new commits, and fails when the secret is missing ([[adr:kinekino/20261116065239]]). `ci.yml` is unchanged apart from a pointer comment. `.github/rulesets/main-protected.json` adds `Public surface` to the required checks.

## Migration

A person registers `KINEKINO_DENY_TERMS` (one term per line) as an Actions secret and as a Dependabot secret, then re-imports the ruleset. Local runs: `cargo xtask deny-terms --range origin/main..HEAD --terms-file <list>`.
