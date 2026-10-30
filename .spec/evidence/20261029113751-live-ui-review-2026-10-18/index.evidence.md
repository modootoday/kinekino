---
id: evidence:kinekino/20261029113751
title: Live player UI review — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live player UI review — 2026-10-18

The active goal is actual player.exe visual/input/panel review, with the user's later instruction prioritizing consistency with `.spec/artifacts/design` above other work. SDK conversion integration continues alongside it. This file records observed runtime evidence separately from tests and remaining work. No VM or DPI settings are changed.

## Baseline actually inspected

Computer Use (@oai/sky) was initialized using the bundled skill. The UI-target built player.exe was launched without smoke auto-close. Actual window screenshots showed:

- Viewing: black source area, tiny text-only central actions, an oversized multilevel transport, and a centered title unlike the design reference.
- Studio: dense navigation and a generic explanation-oriented preparation layout; source/contract boxes used only a small portion of the editor. Settings repeated its title. Studio title buttons left unused trailing space.
- Mouse selection of subtitle tab and Settings close worked. Early Escape/Down/End did not visibly change state. Later, clicking a TextEdit and typing succeeded, followed by actual Tab movement to the input-format control. This contradicts a blanket keyboard-failure claim.
- Later actual Down navigation showed selected/focused rows diverging. The presentation keyboard handler now locks the appropriate arrow axis to its own widget, avoiding duplicate egui directional focus movement. Infrastructure focus-on-create/hide handling was independently corrected by the main session; this document does not assign all observations to one cause.

## References opened and changes made

Opened local `v2/viewing/native-player-start-v2.png`, `v2/studio/skip-settings.png`, `v2/studio/subtitles.png`, and `v2/studio/convert.png`. The implementation now follows their shared structure: left-aligned brand/title, outlined central start actions, brighter text on chrome/panel tokens, larger navigation rows with selected rail, underline tabs, 180-unit navigation/300-unit inspector, center preview/transport/timeline/table structure, and bottom queue bar. Studio entry proposes 1440×900 within known monitor size and returns to the prior window size. This resizes the application window only.

Functional placeholders are disabled and explicitly preparing; generated sample media, subtitle text, model readiness and progress are not fabricated. Model/job/export views use tables rather than a fake video preview. Unsupported fields remain visible to preserve the approved inspector structure. Full editors and unavailable AI pipelines are not claimed implemented.

## Actual conversion observation

The injected file-backed client is used on a Rust worker thread. In the actual GUI, entered `runtime/ai-spike/source/sintel_ko.srt` and a distinct new output `runtime/verification/sintel-ui-20261018.vtt`. Clicking conversion displayed a genuine completion response: 26 cues, 1572 bytes, utf-8. The generated file was then read from disk; WEBVTT header, Korean text and timestamps were present. This was a real UI conversion, not a screenshot fixture or smoke result. No inference/download was run.

Loss-review fixture `runtime/verification/ui-loss-input.vtt` is explicitly prepared test data, not a real analyzed media result. Actual GUI loss acceptance and final revised layout inspection are still in progress.

## Latest actual key evidence

On a fresh post-infrastructure-focus and presentation-arrow-filter build, before any text entry: clicked the segment navigation row, pressed Down, and observed exactly the subtitle row selected and focused. Pressed End and observed export selected and focused. These observations cover that tested sequence, not every keyboard/accessibility path.

## Remaining verification

Final title alignment/start actions/transport adjustment, conversion inspector layout, actual loss-review acceptance, native dialog cancellation, compact window settings, return-to-viewing size restoration, and regression tests remain to be checked. Native screen-reader behavior, real media playback switching, VM and DPI acceptance are not verified. The goal remains active.
