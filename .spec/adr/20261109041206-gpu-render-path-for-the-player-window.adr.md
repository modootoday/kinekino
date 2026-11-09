---
id: adr:kinekino/20261109041206
title: GPU render path for the player window
status: active
created: 2026-11-09T04:12:06Z
updated: 2026-11-09T04:13:34Z
rel:
  governs: [crates/kinekino-player-ui/src/texture.rs]
  depends_on: [adr:kinekino/20261031020000, adr:kinekino/20261029113523]
---

# GPU render path for the player window

## Context

The player window is drawn by Meari on every OS (ADR 20261031020000). Today the whole window
is rasterized on the CPU: the prepared native driver paints through Meari's CPU renderer, and
each video frame reaches the page as an RGBA8 buffer that libmpv's software render API
(`kinekino-libmpv`, `mpv_render_context_create(sw)`) writes through the `VideoSurface` port
and `VideoTexture` uploads as the `<video>` raster image. ADR 20261031020000 already records
that high-resolution fullscreen is expensive on this path and leaves a GPU path for later.
The roadmap step M5-6 (UI-8) asks for the port, error and platform scope of that path as a
decision record only; no GPU code and no mockup work belong to this step.

Facts read on the Linux dev host on 2026-11-09 at main `e50c4ccb`:

- The workspace depends on `meari`, `meari-native`, `meari-dom` and `meari-shell` 0.2.0 from
  crates.io. Meari 0.3.0 adds an opt-in GPU window renderer behind the `window-gpu` feature
  (Vello on wgpu), chosen with `WindowRendererChoice::{Cpu, Gpu, GpuAllowSoftware}`. Its CPU
  renderer stays the default and it has no automatic CPU fallback. Meari 0.3.0, with that
  feature, is on crates.io (published 2026-11-08); kinekino has not moved to it.
- Meari's GPU renderer keeps one submission in flight, releases it on GPU completion rather
  than on present return, merges window resizes to the latest size and stops with an error
  after a completion timeout. It exposes timing metrics and a resize probe tool.
- The dev host has no hardware GPU adapter; wgpu only offers a software adapter (llvmpipe),
  which Meari's `Gpu` choice rejects. No GPU timing exists for kinekino.
- ADR 0018 (20261029113523, draft) planned a GPU canvas on the old egui Glow context. egui and
  eframe are no longer in the workspace, so its Glow premise does not apply to this window.

## Decision

The player window gains a GPU path in two separate stages, and neither stage hides a failure
behind a silent CPU fallback.

1. **Window compositing first.** The first stage only selects Meari's GPU window renderer for
   the player window. It is adopted only through a published Meari release that carries
   `window-gpu` (0.3.0 or later), as its own dependency upgrade change with the usual gates;
   kinekino does not take Meari from a path or git dependency for it.
   The video keeps arriving through the existing `VideoSurface` RGBA port, so libmpv, the
   controller and the CLI do not change in this stage.
2. **Renderer choice is a presentation option.** The choice lives in `kinekino-player-ui`
   (an adapter crate) as a value with three cases mapped one-to-one to Meari's choice: CPU
   (default), GPU, and GPU allowing a software adapter (diagnostic only). It is set by the
   user (a player setting and a `kinekino watch` option), never guessed from the machine, and
   the chosen and actual backend are reported in the `--smoke` report. No domain or
   application type names a GPU API.
3. **Errors.** When the user asks for GPU and window preparation finds no usable adapter, the
   player stops before showing the window with an `Unavailable` port error that names the
   cause and says that the CPU renderer is the way to open the window. A device loss or a GPU
   completion timeout after the window is up ends the window with an error; the player does
   not switch to the CPU renderer mid-run. A software adapter counts as GPU only under the
   diagnostic choice and is reported as such.
4. **UI thread.** The window loop never waits without bound for GPU completion. At most the
   in-flight limit of Meari's renderer (one submission today) is outstanding; while it is
   busy the loop keeps handling input and skips the frame. Resizes are merged to the latest
   size, and `VideoTexture` keeps sizing the video to the latest laid-out box only. Wait code
   is not just deleted: buffers handed to the GPU stay alive until the GPU reports
   completion.
5. **Platform scope.** Linux (X11 and Wayland) and Windows are in scope through the same code
   path, with backend selection left to wgpu inside Meari; kinekino adds no platform `cfg`
   for it. macOS is out of scope. Hardware checks happen on a real GPU per OS; the dev host
   result (software adapter only) is not evidence for either OS.
6. **Video on the GPU is a later, separate port.** Handing decoded video to the GPU without
   the RGBA copy (libmpv's GL render API or a shared texture) is a second stage with its own
   port and decision record. `VideoSurface` is not widened for it here.
7. **Measurement.** The GPU path counts as better only when it shortens the delay from a
   window resize to the first presented frame at the new size, and keeps video frame pacing
   during playback, measured on the same machine, media and window sizes against the CPU
   renderer. Renderer names and GPU utilisation are not success measures. Values that cannot
   be measured stay marked as unmeasured.

## Alternatives rejected

| Alternative | Why not |
|---|---|
| Automatic CPU fallback when the GPU is missing | It hides which renderer is drawing, so a slow window looks like a GPU result; Meari itself refuses this. |
| GPU video texture first (skip window compositing) | It needs a new port, libmpv GL interop and per-OS context sharing, while the measured cost today is the whole-window raster; it is the second stage. |
| Render the player outside Meari (a separate GL or wgpu window for video) | It duplicates window, focus and capture lifetimes and breaks the rule that all UI is Meari. |
| Take Meari's GPU renderer from a path or git dependency | The workspace pins published Meari crates; a local or unpublished dependency would break reproducible builds. |
| Revive ADR 0018's Glow canvas | egui and its Glow context are gone from the workspace. |

## Consequences

- Nothing changes until the Meari upgrade to a release with `window-gpu` lands; the CPU
  path stays the default afterwards, so CPU cost at fullscreen remains for users who do not
  opt in.
- Asking for GPU on a machine without a usable adapter fails visibly instead of opening a
  window; the message must point to the CPU option.
- The `wgpu` and Vello dependencies grow the player build when the feature is on; whether it
  is on by default in release builds is decided with the release, not here.
- The video still costs one CPU RGBA copy per frame in the first stage.
- Adoption needs a hardware GPU run per OS for the resize-delay comparison; the dev host
  cannot provide it.

## Reversal conditions

- Reconsider stage order if measurements on a real GPU show the RGBA video copy, not the
  window raster, dominates the resize delay or fullscreen frame pacing.
- Reconsider the no-fallback rule if users on supported OSes regularly lack a usable adapter
  and an explicit, reported fallback is requested.
- Reconsider the Meari dependency rule if Meari stops publishing the GPU renderer.
