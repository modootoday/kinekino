---
id: change:kinekino/20261116093643
title: Proposal storage on the Linux CLI
status: active
created: 2026-11-16T09:36:43Z
updated: 2026-11-16T09:36:43Z
rel:
  depends_on: [adr:kinekino/20261108112026, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Proposal storage on the Linux CLI

## Change

`crates/kinekino-host/src/runner.rs`:
- `--proposal-root` opens on every OS.
- Off Windows, a configured historical verifier is refused as `unsupported`. A runner unit test covers this; it failed before the guard was added.

Five proposal test files lose `#![cfg(windows)]`:
- `proposals.rs`
- `capture_mutation.rs`
- `projects.rs`
- `cue_edit_cli.rs`
- `accepted_subtitle_export_cli.rs`

`projects.rs` now uses temp-dir locators. The structure allowlist drops 5 `r-cfg` entries. `proposals.rs` and `cue_edit_cli.rs` fail without the runner change.

## Migration

None on Windows. On Linux, proposal commands work with `--proposal-root`; historical verification stays unavailable.
