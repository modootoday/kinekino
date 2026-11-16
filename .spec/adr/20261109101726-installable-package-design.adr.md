---
id: adr:kinekino/20261109101726
title: Installable package design
status: active
created: 2026-11-09T10:17:26Z
updated: 2026-11-16T09:00:00Z
rel:
  governs: [.github/scripts/package.sh]
  depends_on: [adr:kinekino/20261029201516, adr:kinekino/20261109071720]
---

# Installable package design

## Context

The roadmap step M6-3 (CI-2) asks for the design of an installable package: its layout, its
manifest, how it is checked, which systems it supports and how a user rolls back. It is a
design only. Signing, store submission, publishing, tags and releases are outside this step,
and no workflow or script changes here. ADR 20261029201516 (ADR 0010, item 5) deferred
installers and signing; today the release workflow ships only archives and checksums.

Facts read on the Linux dev host on 2026-11-09 at main `be02cb7f`:

- `.github/workflows/release.yml` builds five targets (x86_64 and aarch64 Linux on
  ubuntu-22.04, so glibc 2.35; x86_64 and aarch64 macOS; x86_64 Windows MSVC).
  `.github/scripts/package.sh` copies `kinekino`, `player` and `kinekino-vad-worker` with
  `LICENSE`, `README.md` and `THIRD_PARTY.md` into `kinekino-<target>.tar.gz` (`.zip` on
  Windows) plus a `.sha256`. Nothing is signed.
- Playback loads libmpv at run time from `KINEKINO_LIBMPV`, else the platform soname
  (`libmpv.so.2`, `libmpv-2.dll`, `libmpv.2.dylib`), and never bundles it
  (`kinekino-libmpv`, guide `mpv-install`). The mpv binary comes from `KINEKINO_MPV` or `PATH`;
  the verified mpv is 0.40.0 and the command format needs 0.38 or later.
- The browser engine is installed Google Chrome, overridable with `KINEKINO_CHROME_PATH`
  (ADR 20261109071720).
- Helper tools such as ffmpeg are looked up beside the running executable, then on `PATH`
  (`frame_capture::find_tool`).
- Per-user files already live outside the program directory: player preferences, resume and
  skip ranges under `XDG_CONFIG_HOME`/`XDG_STATE_HOME` (or `LOCALAPPDATA`/`APPDATA`), and
  plugins under `KINEKINO_PLUGIN_DIR`, else the per-user data directory `kinekino/plugins`.
- The font cache is the exception: `noto_fonts::cache_root` uses `MEARI_FONT_CACHE`, else the
  nearest `runtime/fonts` above the working directory or the executable, else `runtime/fonts`
  beside the executable. `kinekino fonts fetch` downloads the pinned fonts into that cache.
  Installed under a system prefix, the last fallback points into a read-only program
  directory.
- The speech worker's native runtime has no product release record yet (`native-release`
  SOT: `PRODUCT_RELEASE=None`), so no model or native inference runtime is a release payload
  today.

## Decision

1. **One payload list.** A package contains exactly the release archive's payload: the three
   binaries and the three notice files. The list is declared once in the repository and both
   `package.sh` and every installer format read it. A package carries no libmpv, mpv, ffmpeg,
   Chrome, fonts, models or plugins.
2. **First format: Linux `.deb`, x86_64 and aarch64.** It is built from the same release
   binaries as the archive (no second build). Layout: the three binaries in
   `/usr/lib/kinekino/`, one link `/usr/bin/kinekino` to the CLI, notices in
   `/usr/share/doc/kinekino/`. `player` and the worker are not linked into `/usr/bin` (the
   name `player` is too generic for a shared directory); sibling lookup still works because the
   running executable resolves to its real path.
3. **Dependencies are declared, not bundled.** `Depends: libmpv2 (>= 0.38)`;
   `Recommends: mpv, ffmpeg`. Chrome is not a dependency (it is not a distribution package); the
   browser features need it installed separately. The supported floor follows the build host:
   glibc 2.35 or later (Ubuntu 22.04, Debian 12 and newer).
4. **The program directory is never written.** Before the first package is built, the font
   cache's last fallback moves to the per-user cache directory (`XDG_CACHE_HOME/kinekino/fonts`
   or `~/.cache/kinekino/fonts` on Linux, the platform equivalent elsewhere), with a test that a
   read-only executable directory is not written. The development rule (nearest
   `runtime/fonts`) stays first.
5. **Checks.** The package version comes from `[workspace.package] version` through
   `workspace-version.sh`, as the archive's does. A check unpacks the `.deb` into a scratch
   directory (no installation on the build host), compares the file list with the payload list,
   and runs one CLI command that needs no display from the unpacked path (for example
   `kinekino plugins list` against an empty plugin directory), expecting exit status 0; slice
   (c) fixes the exact command. A package that fails
   any check is not uploaded.
6. **Upgrade and rollback.** An upgrade replaces only the program files. Removing or purging
   the package leaves per-user configuration, state, plugins and caches in place. Rollback is
   installing the previous version's package. A release that changes the format of a per-user
   file must still read the previous format, or its notes must say that rolling back across it
   is not supported.
7. **Windows and macOS layouts are written down but not built.** Windows: a per-user MSI
   installing into `%LOCALAPPDATA%\Programs\kinekino\` (no administrator rights). macOS: an
   app bundle with the three binaries in `Contents/MacOS/`, shipped in a DMG. Neither is built
   until two things exist: a decision on how libmpv reaches those systems (there is no package
   manager dependency to declare) and signing (Windows Artifact Signing, Apple notarization;
   ADR 0010 secret names). An unsigned installer that cannot play is not shipped.

## Alternatives rejected

- **AppImage or Flatpak first.** Both expect a self-contained runtime, so libmpv and ffmpeg
  would have to be bundled, which changes the licence notices and the size; a Flatpak sandbox
  also stands between the player and installed Chrome and the owned helper processes.
- **Bundling libmpv in the `.deb`.** The distribution already ships it with security updates;
  a private copy would freeze it.
- **A system-wide Windows MSI.** It needs administrator rights for a per-user application.
- **A separate packaging build.** Rebuilding for the installer would let the package and the
  archive differ for the same version.

## Consequences

- Follow-up slices, in order: (a) per-user font cache fallback with its test; (b) the single
  payload list read by `package.sh`, with a check that the archive matches it; (c) a `.deb`
  builder and its unpack check in the release workflow's dry-run path. Each runs the usual
  gates; none adds signing, a package repository, publishing, a tag or a release.
- A `.deb` that installs cleanly still needs Chrome and, for some features, mpv and ffmpeg;
  the README must say so when the package exists.
- Windows and macOS packages stay unbuilt; the archives remain their only download.

## Reversal conditions

- Reconsider bundling if a decision is made to ship libmpv with the Windows or macOS build;
  that decision opens the MSI and DMG stages and their notice changes.
- Reconsider AppImage or Flatpak if a self-contained runtime with libmpv becomes acceptable.
- Reconsider the dependency on `libmpv2` if supported distributions drop that package name.

## Record

- 2026-11-09: M6-3 (CI-2) designed the installable package without code or workflow changes.
- 2026-11-14: slice (a) done. `kinekino_core::platform::font_cache` decides the font cache root for both the player window and `kinekino fonts fetch`: `MEARI_FONT_CACHE`, then the nearest existing `runtime/fonts` above the working directory or the executable, then the per-user cache (`XDG_CACHE_HOME`, `LOCALAPPDATA`, `~/Library/Caches` on macOS, `~/.cache`); it never creates a directory beside the executable. Unit tests cover each step, including an installed binary under `/usr/lib/kinekino`. Before this, `fonts fetch` wrote under the working directory while the player fell back beside the executable.
- 2026-11-16: slice (b) done. `.github/scripts/package-payload.txt` is the one payload list (`bin NAME` for release binaries, `.exe` added on Windows; `file NAME` for repository files). `package.sh` copies exactly those files, and the release workflow runs `cargo xtask package-check --target T --archive A` after packaging, which lists the archive (zip read in Rust, tar.gz through `tar -tzf`) and fails on a missing or an unexpected file. Checked locally with a Linux tar.gz and an Info-ZIP zip; the first check on the release runners is the next release dry run.
