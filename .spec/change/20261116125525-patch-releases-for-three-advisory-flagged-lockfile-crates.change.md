---
id: change:kinekino/20261116125525
title: Patch releases for three advisory-flagged lockfile crates
status: active
created: 2026-11-16T12:55:25Z
updated: 2026-11-16T12:55:25Z
rel:
  governs: [Cargo.lock]
---

# Patch releases for three advisory-flagged lockfile crates

## Change

`Cargo.lock` moves three transitive crates to patch releases inside their existing semver ranges. No manifest or source changes.

| Crate | From | To | Advisory |
|---|---|---|---|
| `rustix` (1.x) | 1.1.5 | 1.1.6 | RUSTSEC-2026-0486 |
| `rustls-webpki` | 0.103.15 | 0.103.17 | RUSTSEC-2026-0382 |
| `webpki-roots` | 1.0.9 | 1.0.12 | RUSTSEC-2026-0381 |

The lockfile was matched against the RustSec advisory database. One more match stays open: `glib` 0.18.5 (RUSTSEC-2024-0429, fixed in 0.20). It comes in through `wry` → `webkit2gtk` → `gtk` 0.18, so it moves only with a major `wry`/`gtk` upgrade, and that is the owner's decision.

Checked on Linux: fmt, clippy `-D warnings`, the structure lint and the full workspace tests (2088 passed). The four targets that need a local mpv/libmpv fail here, as they did before the change.

## Migration
