---
id: evidence:kinekino/20261116135502
title: glib 0.18 advisory is lockfile-only; no wry release leaves gtk 0.18
status: active
created: 2026-11-16T13:55:02Z
updated: 2026-11-19T20:40:00Z
rel:
  governs: [Cargo.lock]
---

# glib 0.18 advisory is lockfile-only; no wry release leaves gtk 0.18

## What was measured

Advisory: RUSTSEC-2024-0429, `glib` 0.18.5. It is fixed in `glib` 0.20.

**Where it comes from.** `Cargo.lock` holds `glib` 0.18.5 through `wry` → `webkit2gtk` =2.0.2 → `gtk` 0.18. `wry` is declared only under `[target.'cfg(windows)'.dependencies]` (`kinekino-player`, `kinekino-browser` feature `webview2`), and `wry` pulls `gtk`/`webkit2gtk` only for Linux and BSD targets. Cargo resolves the lockfile for every target, so the crate is listed there but never compiled.

**Measured on 2026-11-16.** `cargo tree -i glib@0.18.5 -e all --target <t>` prints nothing for each of:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-apple-darwin`

With `--target all` it prints the `wry` path above.

**Upgrade path (crates.io, 2026-11-16).**
- The newest `wry` is 0.57.2. It requires `gtk ^0.18`, `webkit2gtk =2.0.2` and `javascriptcore-rs =1.1.2`.
- The newest `webkit2gtk` is 2.0.2. It requires `gtk`/`glib`/`gio` `^0.18`.
- `gtk` 0.19.0, published 2026-09-08 and built on `glib` 0.22, exists, but no `webkit2gtk` or `wry` release uses it yet.

So no `wry` or `gtk` upgrade can remove `glib` 0.18 today. A `[patch]` override would replace third-party bindings, and the agent does not do that.

## Recheck 2026-11-19

`cargo-audit` 0.23.0 still lists `glib` 0.18.5 (RUSTSEC-2024-0429) as its only unsound finding. It reports no vulnerabilities.

**Per target.** `cargo tree -i glib@0.18.5 -e all --target <t>` still prints nothing for the four targets above.

**Upgrade path (crates.io, 2026-11-19).**
- The newest `wry` is 0.57.3. It still requires `gtk ^0.18`, `webkit2gtk =2.0.2` and `javascriptcore-rs =1.1.2`.
- `webkit2gtk` 3.0.0 was published on 2026-11-19. It is built on `gtk` 0.19, `glib` 0.22, `javascriptcore-rs` 2 and `soup3` 0.9.
- So the first half of the recheck condition below is now met. The second half is not: no `wry` release uses `webkit2gtk` 3 yet.

The conclusion is unchanged: no binary is affected, and there is no upgrade to take.

## How to read it

- **No binary is affected.** The advisory reaches no binary this workspace builds for its supported targets, so no code change is needed now.
- **Recheck** when `webkit2gtk` publishes a release on `gtk` ≥ 0.19 (`glib` ≥ 0.20) and `wry` adopts it. Then raise `wry` and check that the lockfile no longer lists `glib` 0.18.
- **If the Linux build ever enables `wry`, this changes.** That would happen by moving it out of the `cfg(windows)` sections, and the advisory would then apply.
- **Dependabot alert.** Dismissing it as "vulnerable code is not actually used", with a link to this note, is an owner action.
