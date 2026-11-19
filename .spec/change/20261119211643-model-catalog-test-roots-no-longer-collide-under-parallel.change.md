---
id: change:kinekino/20261119211643
title: Model catalog test roots no longer collide under parallel tests
status: active
created: 2026-11-19T21:16:43Z
updated: 2026-11-19T21:16:43Z
rel:
  governs: [crates/kinekino-player/tests/model_catalog_cli.rs]
  depends_on: [change:kinekino/20261119210434, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, change:kinekino/20261116101201, change:kinekino/20261116104517, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Model catalog test roots no longer collide under parallel tests

## Change

`model_catalog_cli` named each test's temporary root from the process id and the current time in nanoseconds. The four tests in the binary run in parallel threads and each creates its root first. When two threads read the same clock value, the second `create_dir` fails with `AlreadyExists`.

The root name now also carries a per-process sequence number (`AtomicU64`), so it no longer depends on clock resolution. A new test creates 512 roots from eight threads at once and checks that they are distinct.

## Verification

- **Observed failure.** CI run 44611814111 on #64, job `test (ubuntu-26.04)`, which only changes a document. `mid_hash_cancel_timeout_and_busy_keep_original_file_guards_until_return` panicked at `model_catalog_cli.rs:37` with `Os { code: 17, kind: AlreadyExists }`. The two `main` runs before it passed.
- **Not reproduced here.** The new concurrency test also passed before the fix in this container, three runs out of three, because its clock resolves nanoseconds. It guards the property, not the original timing.
- **After the fix.** `cargo test -p kinekino-player --test model_catalog_cli`, `cargo clippy -p kinekino-player --all-targets -- -D warnings` and `cargo fmt --check` pass.

## Migration

None.
