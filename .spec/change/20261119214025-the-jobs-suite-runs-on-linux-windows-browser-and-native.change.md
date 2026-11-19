---
id: change:kinekino/20261119214025
title: The jobs suite runs on Linux; Windows browser and native window parts stay gated
status: active
created: 2026-11-19T21:40:25Z
updated: 2026-11-19T21:40:25Z
rel:
  governs: [crates/kinekino-player/tests/jobs.rs, crates/kinekino-player/tests/jobs]
  depends_on: [change:kinekino/20261119210434, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261105211630, change:kinekino/20261116101201, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# The jobs suite runs on Linux; Windows browser and native window parts stay gated

## Change

The `jobs` integration binary drops its file-level `#![cfg(windows)]`. Only the parts that call Windows-only product code or Win32 stay gated.

- **Still gated, browser provider.** `provider`, `synthetic_browser`, `observation_guard` and `episode_capture` need product `infrastructure::browser_provider`, which is Windows-only. `card_bundle` keeps its feature gate and is now also gated to Windows, like the product's `card_bundle_jobs`.
- **Still gated, native window (ADR 0009).** `native_episode`. The parts of `playback_owner` and `playback_observation` that only it uses move into it, as `native_episode/owner.rs` (`NativeOwner`) and `native_episode/observation.rs` (`observe_until`, `wait_for_progress`, `capture_frame`). The shared files stay free of platform `cfg`, as `cargo xtask lint-structure` (rule r-cfg) requires.
- **Moved.** The job-observation helpers `terminal`, `terminal_until` and `try_terminal_until` move from `provider.rs` into a new portable `job_observation.rs`, with the `observation_trace` module they use. `asset_build`, `clip_render`, `retaining_consumer`, `series` and `same_endpoint` import them from there.
- **Ported.** Child processes no longer call `creation_flags(CREATE_NO_WINDOW)` directly. They use `os_policy::hide_console_window`, which does nothing on POSIX.

**Test fix.** `same_endpoint::evidence::provider_provenance_preserves_urls_and_raw_refs_without_extending_admission` failed on its first real run, which was on Linux. The test expected a media-host player document to be admitted. But an integration binary sees the deny-by-default host policy, because the `cfg(test)` fixture applies only to the library's own unit tests. On Windows the suite was only compiled, never run, so the failure did not show. The test now installs a plugin-shaped policy with `admitted_hosts_from_plugin`: the catalog host is navigable and only the media host plays. With that policy the frame host stays rejected, as the test asserts. No other running test in the binary depends on admitted hosts.

## Verification

On x86_64 Linux:
- Before the fix, `cargo test -p kinekino-player --test jobs` gave 41 passed and 1 failed: the evidence test, at `left: false, right: true`.
- After the fix, it gives 42 passed and 20 ignored, three runs out of three.
- `cargo clippy -p kinekino-player --all-targets -- -D warnings`, `cargo fmt --check` and `cargo xtask lint-structure` (0 new violations) pass.

For `x86_64-pc-windows-msvc`, `cargo clippy --locked -p kinekino-player --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passes.

## Migration

None.
