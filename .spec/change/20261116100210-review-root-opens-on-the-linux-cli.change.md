---
id: change:kinekino/20261116100210
title: Review root opens on the Linux CLI
status: active
created: 2026-11-16T10:02:10Z
updated: 2026-11-16T10:02:10Z
rel:
  depends_on: [adr:kinekino/20261108112026, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-host/src/runner.rs]
---

# Review root opens on the Linux CLI

## Change

Recorded in [[adr:kinekino/20261108112026]].

- `kinekino-host` runner: the `--review-root` block is no longer behind `cfg(windows)`, and the Linux refusal is removed.
- `kinekino-player`:
  - New `infrastructure::os_policy::transient_lock_held`. It checks whether the file exists where share modes are mandatory (Windows), and tries a shared lock on POSIX.
  - `model_review/session.rs` uses it instead of `exists()`, and skips `.publish.lock` when listing decisions.
  - Unit test `transient_lock_is_held_only_while_its_handle_lives`.
- Tests:
  - `tests/model_review.rs` and `tests/review_source_cli.rs` lose `#![cfg(windows)]`.
  - The reparse-escape case uses `kinekino_platform::fs::symlink_file`.
  - The writer-denied check runs only with mandatory share modes.
- Structure allowlist: two `r-cfg` entries removed.

## Migration

None. On Linux the review root used to be refused as `unsupported`. On Windows, behavior is unchanged.
