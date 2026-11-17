---
id: evidence:kinekino/20261117193255
title: CI build cache size per lockfile generation
status: active
created: 2026-11-17T19:32:55Z
updated: 2026-11-17T20:50:00Z
rel:
  governs: [.github/workflows/ci.yml]
---

# CI build cache size per lockfile generation

## What was measured

**Configuration** (`.github/workflows/ci.yml`, `window-smoke.yml`, `release.yml` at `main` 98536a8).
- Every job uses `Swatinem/rust-cache` v2.9.3 with `save-if: github.ref == 'refs/heads/main'`. Pull requests only restore the cache from `main` and never save one, so pull requests add no per-branch caches.
- Defaults that apply:
  - each job gets its own key (no `shared-key`);
  - `cache-on-failure` is off;
  - `cache-all-crates` is off. Before saving, the registry is pruned to the crates that are in use.
  - Workspace crates are removed from `target`, so only dependency builds are stored.
- `window-smoke.yml` saves under `key: window-smoke`, and only on `main`.
- `release.yml` uses no cache, so a tag build starts clean.
- `.cargo/config.toml` sets `incremental = false`. `[profile.dev.package."*"]` sets `debug = false`, and `dev`/`test` use `debug = "line-tables-only"`.

**Sizes, from the rust-cache save logs.** The run is `main` CI 43884346967, the merge of #38. Its lockfile changed, so every job saved a new cache.

| Job | Saved cache (zstd) |
|---|---|
| `test (ubuntu-26.04)` | 824,821,499 B (≈ 825 MB) |
| `test (ubuntu-26.04-arm)` | 814,803,564 B (≈ 815 MB) |
| `lint` | 388,727,368 B (≈ 389 MB) |
| **One lockfile generation, Linux gate** | **≈ 2.03 GB** |

**No re-save without a key change.** The later `main` merges (#46–#52) did not change the lockfile or the toolchain. In run 43904561960, the merge of #52, the rust-cache post step took 0 s in all three jobs: the restored key was an exact hit and nothing was saved. The cache therefore grows only when `Cargo.lock` or `rust-toolchain.toml` changes.

**Not measured here.** The cloud session cannot call the repository's Actions cache usage API, because its proxy blocks it. Both of the following come from the owner's computer survey:
- the total size and number of cache entries;
- the sizes of the advisory `windows-gate`, `test-macos` and `window-smoke` caches, which are saved only by manual runs on `main`.

## How to read it

- **The repository limit is 10 GB.** GitHub evicts the oldest entries beyond that, and also any entry not used for 7 days.
- **How many generations fit.** One Linux generation is about 2 GB, so about four generations fit next to the advisory caches. Only the newest generation is restored, and older generations age out after 7 days without use. The Linux gate on its own therefore stays well under the limit.
- **No workflow change is needed for the Linux gate.**
  - Sharing one key across `lint` and `test` would not save space. Clippy metadata and test builds are different artifacts, so a shared cache would carry both.
  - `cache-on-failure` and `cache-all-crates` are already at the smaller setting.
- **Revisit when one of these happens:**
  - the survey shows total usage above about 8 GB;
  - a single job's cache passes 1.5 GB;
  - lockfile changes come more often than weekly, for example a batch of Dependabot merges.

  Then delete superseded `main` keys by hand, or drop the arm test cache first. The arm cache restores in about 15 s, and losing it costs only a cold arm build.
- **The owner's computer has a separate cache.** The shared target root (`../.cargo-target/kinekino`), toolchains and temporary clones are cleaned from the survey numbers. The commands go on the survey issue, and the owner's computer session runs them.

## Survey result (issue #54, 2026-11-17)

**Actions caches.** 18 entries, 9,680 MB, all on `refs/heads/main`:

| Key prefix | Entries | MB |
|---|---:|---:|
| `v0-rust-test-Linux-x64-*` | 6 | 4,264 |
| `v0-rust-test-Linux-arm64-*` | 5 | 3,570 |
| `v0-rust-lint-Linux-x64-*` | 5 | 1,777 |
| `setup-ruby-bundler-*` (docs) | 2 | 61 |

Two revisit triggers fired. Usage is above 8 GB, and 5–6 lockfile generations were saved within one day, during the Dependabot merges of 2026-11-16. No single job is above 1.5 GB, and no advisory cache was present. Each entry is 340–790 MB, which matches the per-job sizes above.

**Decision: prune once, and do not change the workflow.**
- Only the newest entry per prefix is restored. The superseded ones (about 7.8 GB) are deleted once, by hand, with the commands on #54.
- Hitting the limit is not harmful. GitHub evicts the least recently used entries first, which are exactly the superseded generations, and it drops any entry unused for 7 days. The newest generation, the one in use, is never the one evicted.
- An automatic prune step would need a CI token with `actions: write`. That is more permission than the problem warrants, so it is not added.
- Dropping the arm cache stays the next step if usage stays near the limit outside Dependabot bursts.

**Owner's computer (Windows).**
- The leftovers from the 2026-11-16 Windows run are the shared target root (5.14 GB), the temporary clones (0.46 GB) and `runtime/build-cache` (0.23 GB). They are deleted.
- The `stable-x86_64-pc-windows-gnu` toolchain (1.59 GB) is not used by this repository. Removal is listed on #54 as optional because the toolchain is shared with other projects.
- `target-dir` is relative to the clone (`../.cargo-target/kinekino`). A clone under `%TEMP%` therefore builds on C:, so later Windows runs clone on D:.
