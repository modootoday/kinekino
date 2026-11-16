---
id: change:kinekino/20261116062956
title: CI runs on pull requests to main
status: active
created: 2026-11-16T06:29:56Z
updated: 2026-11-16T06:29:56Z
rel:
  depends_on: [evidence:kinekino/20261109092655]
---

# CI runs on pull requests to main

## Change

`ci.yml` gains `pull_request` and `push` triggers for `main` (ADR 20261116062843). The Linux jobs (`lint`, `test`, `site`) now run on those events or when the `linux` input is set on a manual run; before, `if: inputs.linux` would have skipped them. Job steps and commands are unchanged, so the local reproduction recorded in evidence 20261109092655 still uses the same commands. Windows and macOS jobs stay manual.

## Migration

None for local runs. A person applies `.github/rulesets/main-protected.json` in GitHub (required checks `lint` and `test (ubuntu-26.04)`).
