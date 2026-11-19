---
id: change:kinekino/20261119210434
title: Ten pinned-tool player suites compile and run on Linux
status: active
created: 2026-11-19T21:04:34Z
updated: 2026-11-19T21:04:34Z
rel:
  governs: [crates/kinekino-player/tests]
  depends_on: [adr:kinekino/20261116151137, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, plan:kinekino/20261030013900, change:kinekino/20261116101201, adr:kinekino/20261029164425, adr:kinekino/20261030030000, guide:kinekino/ignored-tests, sot:kinekino/conventions]
---

# Ten pinned-tool player suites compile and run on Linux

## Change

This is slice (d) of `adr:kinekino/20261116151137`. The following suites lose their `#![cfg(windows)]` gate:
- `execution_host_cli`;
- `review_matroska_timeline`, `review_mapping_proof_actual`, `review_playback_b0`, `review_startup_actual` and `review_worker_actual`;
- `timeline_video_production_actual`;
- `historical_provenance_cli`, `sprite_capture_cli` and `sprite_timeline_bundle`.

Two kinds of Windows-only code in the tests move to the existing OS policy port:
- **CPU affinity.** `GetProcessAffinityMask` becomes `os_policy::process().current_affinity()`, as in `review_runtime.rs`. When the port reports no mask, the test uses CPU 0 (`1`).
- **Retained source guard.** `execution_host_cli/receipt_fault.rs` opened the media with `share_mode(0)` and expected a failure. It now opens with `ShareMode::Exclusive` through `os_policy::apply_share_mode`. The failure is asserted only where `has_mandatory_share_modes()` is true, because POSIX sharing is advisory. This is the idiom `model_review.rs` already uses.

Every test in these suites is still `#[ignore]`d. They need prepared media and pinned tools, and several fixtures name Windows tool paths (`mpv.exe`, `ffmpeg.exe`). Running them on POSIX needs POSIX tool pins, which is outside this change. Lifting the gate means a change that breaks them on Linux is now caught by Linux CI and clippy.

Still gated: `local_host`, `native_gui_endpoint`, `jobs`, `download_cli`, `media_download` and `media_download_owned`. They use named pipes, Win32 handles or `CommandExt` directly, and they need the larger ports listed in #30.

## Verification

On x86_64 Linux:
- `cargo test -p kinekino-player` for the ten suites builds and runs them: 0 failed, 32 ignored.
- `cargo clippy -p kinekino-player --all-targets -- -D warnings` passes.
- `cargo fmt --check` and `cargo xtask lint-structure` pass.

For `x86_64-pc-windows-msvc`, `cargo clippy --locked -p kinekino-player --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passes.

## Migration

None. The ignored tests are run as before, as `guide:kinekino/ignored-tests` describes.
