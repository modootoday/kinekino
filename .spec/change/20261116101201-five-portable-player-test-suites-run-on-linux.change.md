---
id: change:kinekino/20261116101201
title: Five portable player test suites run on Linux
status: active
created: 2026-11-16T10:12:01Z
updated: 2026-11-16T10:12:01Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261031110642, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/tests]
---

# Five portable player test suites run on Linux

## Change

These five integration test files had a file-wide `#![cfg(windows)]`, but they use no Windows API and the product paths they exercise already run on every OS. The gate is removed, so Linux CI now runs them:
- `manual_skip_cli.rs`: 4 tests
- `ffprobe_observation.rs`: 8 tests (5 real-media tests stay `#[ignore]`)
- `frame_snapshot_pipeline.rs`: 11 tests (1 ignored)
- `vad_product_cli.rs`: 8 tests (3 ignored)
- `vad_jobs.rs`: 40 tests (1 ignored)

They all passed on a Linux dev host without changes to product code. The structure allowlist drops 5 `r-cfg` entries.

Still Windows-only, and why:
- `jobs.rs`, `proposal_list.rs`, `download_cli.rs` and `sprite_timeline_bundle.rs` use `std::os::windows` handles.
- `model_catalog_cli.rs`, `cli_discovery_inventory.rs` and `historical_provenance_cli.rs` have no test that runs on Linux.

## Migration

None.
