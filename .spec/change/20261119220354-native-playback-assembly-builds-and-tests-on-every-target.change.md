---
id: change:kinekino/20261119220354
title: Native playback assembly builds and tests on every target
status: active
created: 2026-11-19T22:03:54Z
updated: 2026-11-19T22:03:54Z
rel:
  governs: [crates/kinekino-player/src/assembly/native_playback]
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261116151137, guide:kinekino/ignored-tests, plan:kinekino/20261029151512, plan:kinekino/20261030013900, plan:kinekino/20261031153240, sot:kinekino/conventions]
---

# Native playback assembly builds and tests on every target

## Change

`assembly::native_playback` is no longer Windows-only. The M5-3 follow-up of 2026-11-16 kept it gated for one reason: two of its standalone tests assert that a held pinned executable refuses a writer open. That is true on Windows because of mandatory share modes. POSIX sharing is advisory.

The two assertions now use the existing test oracle `os_policy::guarded_mutation`:

- on Windows, the writer open must fail;
- on POSIX, it succeeds, and the guard is the identity and digest re-check from the pinned-tool ADR (`adr:kinekino/20261116151137`).

The module itself uses no Windows API. Its routed bootstrap under the `native-proposal-window` feature still needs the Windows-only adapters, and that feature does not build on Linux either before or after this change.

## Verification

- **x86_64 Linux.** `cargo test -p kinekino-player --lib -- native_playback parent_router`: 35 of 35 pass. Before the oracle change, the same run failed the 2 share-mode assertions.
- **Lint.** These pass:
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo clippy -p kinekino-player --all-targets --target x86_64-pc-windows-msvc -- -D warnings`;
  - `cargo fmt --check`.
- **Structure.** `cargo xtask lint-structure` reports 0 new violations, the same as `main`.

## Migration

None.
