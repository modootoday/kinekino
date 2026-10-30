---
id: evidence:kinekino/20261029114120
title: Native Rust presentation implementation — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native Rust presentation implementation — 2026-10-18

Scope: `src/presentation/**`. This records observed implementation and its verification; it does not replace or edit the accepted screen contracts.

Reference: [UI contract](../../player/presentation/sot-020-ui-contract.md), [viewing](../../player/presentation/viewing/viewing.md), [shared field rules](../../player/presentation/shared/field-rules.md), [consistency](../../player/presentation/shared/consistency.md).

## Implemented

- Connected the existing Rust icon and undecorated title-bar WIP to the actual viewing shell. Window drag, maximize/restore, minimize and close use egui viewport commands, not new platform API calls.
- Display product name comes from the shared `domain::product::PRODUCT` identity. The title truncates with a full-title tooltip.
- Viewing controls use one Rust icon component with accessible button labels, hover/selected/disabled/focus treatment, minimum 32-unit targets and a 40-unit primary playback target. Playback is disabled without a usable source. Existing mpv/WebView adapters and episode transition intent remain in use.
- Seek stays above the controls, unknown duration is `--:--`, volume uses integer values and an icon popup in compact space. Episode panel defaults to 260 units with rows at least 40 units and full-title tooltips.
- `Ctrl+O` opens local media, `Ctrl+L` opens catalogue navigation. Keyboard shortcuts yield while egui requests text input. Esc closes the studio settings drawer before leaving the shell and closes episode panels before fullscreen exit.
- `studio/catalog.rs` owns screen presentation descriptors. `studio/mod.rs` owns navigation and shared preparation rendering, separately from player transport, window chrome and episode UI. Twelve workspaces share selected state, subtab selection, source summary, inspector, action area and queue bar. Subtab choice is retained in memory per workspace.
- Inspector is a 300-unit panel at width >=1280 and a settings window below it; navigation becomes a 48-unit icon rail below 1180. Actual DPI behavior was not tested.
- Studio entry hides native/browser child surfaces through their existing adapters; it does not seek, restart playback or modify pause intent. Return restores the prior browser visibility intent. Runtime interaction of this transition remains unverified.
- Production actions, draft saving, model downloads and queue execution are disabled with reasons. Model/job state is explicitly unqueried, without invented job counts, progress, installed status or available models. `절약` is identified as a design default, not an observed scheduler state.

## Verification

Commands use Cargo concurrency `-j2` and target directory `runtime/parallel-build/ui`. Formatting is checked with `cargo fmt --check`.

Final test rerun passed: 27 library tests, 2 CLI unit tests and 2 CLI integration tests (31 total). Two presentation tests cover all twelve workspaces rendering at widths 760/980/1180/1280/1440 and settings-first Esc behavior. These are headless egui logic/render construction tests, not pixel, DPI or accessibility acceptance.

`cargo fmt --check` passed. `cargo clippy -j2 --target-dir runtime/parallel-build/ui --all-targets -- -D warnings` passed. Icon stroke widths explicitly use f32 literals as requested by the SDK integration session.

Built `runtime/parallel-build/ui/debug/player.exe --smoke-test` exited 0. Startup observation: `page_finished=true`, `requests=71`, `blocked_requests=0`, WebView engine `155.0.4283.62`. The test app closed itself after its existing 20-second startup probe. This verifies native app/WebView startup, not video playback or manual studio interaction.

## Open acceptance and integration

- Existing PlayerApp still talks to infrastructure adapters; the new SDK currently reports playback/production/model/job commands unsupported. Typed application playback commands/observations and production capability/document ports must replace those calls before shared GUI/CLI functional parity can be claimed. Requests are in `runtime/coordination/ui.md` and `ui-dependencies.md`.
- Icons remain the existing Rust vector implementation, with a few common semantic additions. They are not claimed to be Phosphor Regular assets. Compact editing screens currently share a generic edit icon; distinct semantic artwork remains pending.
- These are preparation screens, not implemented timeline editors, subtitle/speaker editors, canvas tools or production pipelines. No mock progress, results, media PTS or project revisions are fabricated.
- Rendering density, exact 32/56-unit viewing frame targets, popover placement, keyboard focus order, screen-reader tree, actual native/WebView interactions and original-media playback require manual runtime acceptance. Mute and preference persistence remain pending.
- No Pages changes, VM work, DPI changes, model downloads/inference, long media tests or other-application termination were performed.

## Explicit goal follow-up

An explicit /goal was registered on this chat without a token budget. Presentation remains the owned code scope. The later follow-up adds:

- Distinct Rust vector icons for segments, subtitles, subjects, clips, frames, cards, layers, projection, models, queue and export. Expanded and compact navigation use the same component and mappings. Icon canvas is 18 units inside a minimum 32-unit target. This is still not a claim of Phosphor artwork compliance.
- Accessible selected metadata and button labels for common icons, pressed-state feedback, keyboard focus outline and style-derived button text. Long navigation text clips to its button; labels remain in tooltips and widget metadata.
- Stable workspace/subtab IDs and focused Up/Down or Left/Right plus Home/End selection, with scroll-to-focused-item. Studio queue and fixed inspector contents register after central tab controls so panel allocation order does not put queue controls ahead of navigation in Tab traversal. Floating settings still requires runtime focus review.
- Esc handling runs before the focused-widget shortcut guard: popup first, studio settings/shell or episode panel next, then fullscreen. Space and editing keys continue to yield to focused controls.
- A separate `sdk_status.rs` queries actual SDK version and capability responses for the inspector. It uses the default unavailable-storage SDK runtime; file-backed subtitle conversion is being implemented by the SDK owner and is not exposed as a working GUI control. Application client injection and input/output subtitle dialog adapters are requested in coordination documents.
- Additional executable egui tests verify Space activation of a focused icon, its emitted accessible name and minimum target, and actual focused navigation key events with boundary handling. These are synthetic egui input tests, not native mouse/keyboard or screen-reader verification.

The previous startup and render-construction results remain limited to their stated scope. Final follow-up: cargo fmt --check passed; cargo test -j2 --target-dir runtime/parallel-build/ui passed 29 library + 2 CLI unit + 2 CLI integration tests (33 total); cargo clippy -j2 --target-dir runtime/parallel-build/ui --all-targets -- -D warnings passed. A fresh built-player startup smoke exited 0, page_finished=true, requests=70, engine=155.0.4283.62. No video playback or manual studio manipulation was tested by this startup probe.

SDK source now supports `subtitles.convert` only when file storage is injected. The GUI's default unavailable-storage SDK truthfully keeps that operation unsupported; file-client injection/dialog requirements are recorded for the SDK/assembly owner. The current dependency configuration also does not enable eframe/accesskit; a dependency request is recorded. Widget-info emission alone is not a native accessibility bridge.

Manual visual acceptance, real media transition interaction, native accessibility bridging and screen-reader behavior remain unverified. Presentation code review, implemented corrections, scoped automated validation and integration handoff are complete for this goal; production use-case assembly and manual acceptance remain explicit future work.
