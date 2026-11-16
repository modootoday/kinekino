---
id: change:kinekino/20261116105620
title: Review media preparation tests run on Linux
status: active
created: 2026-11-16T10:56:20Z
updated: 2026-11-16T10:56:20Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, change:kinekino/20261116101201, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests/review_media_preparation.rs]
---

# Review media preparation tests run on Linux

## Change

`review_media_preparation.rs` and `review_matroska_input.rs` lose `#![cfg(windows)]`. Their note said the review source, model review and timeline evidence infrastructure was Windows-only, but that infrastructure now builds on Linux.

On a Linux dev host:
- `review_media_preparation`: 4 tests pass; 1 actual-package test stays ignored.
- `review_matroska_input` builds; its only test reads a prepared actual package and stays ignored.

The structure allowlist drops 2 `r-cfg` entries.

## Migration

None.
