---
id: evidence:kinekino/20261029114008
title: Original Ready timeout — source review
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Original Ready timeout — source review

SDK actualfirstAiring attempt1 on original B590FABC350DFEB33507CB080538A829D4A09CE78638B5304C1574AEB736F806. This UI session read original raw outputs and selected producer/controller/emitter sources only; Cargo/GUI/OS/media/retry0, no source repair or pin change.

## Observed original evidence

Gate %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-routed-airing-actual-r1-gate. Original child raw stdout at evidence/product-routed-airing-actual-r1/routed-gui-airing-first/owned/stdout.raw is2232B. LocalHost-ready emitted at851ms, no Ready record, then FinalV2 after original private close and cleanup; original child exited0. SDK caller failed101 after Ready wait, jobs0/site/media not reached. SDK confirmed parser did not discard a Ready record: no new complete record existed between LocalHost and close. Latest protected344 SHA4940B0EC22B5077CC4276F5AEDECAEE133EB87FF62ABB6D498C110478519E8BA, slot-return SHA prefix3B5E980C. SDK owns actual execution evidence; UI did not rerun it.

Final native/Execution/endpoint/provider/dispatcher cleanup completes, diagnostic_write_error null. These prove original close behavior only. Final observations=1 is ViewerCleanupEvidence polling count (viewer_cleanup.rs), not Native notice/frame count.

## Confirmed source boundaries

Selected child CC2741B9 configures diagnostics(true), prepares original product playback, starts same-client LocalHost and calls original run. UI selected consumer E9921D73 calls Ready capture only after original checkpoint.activate and pending.activate success. Capture validates lease/window/current frame, reacquires actual with_ready_route, then serializes and writes once after checked borrows end. Encoding/output failures are retained in ReadyEmitter.snapshot.diagnostic_error and consumed by Final getter. No fake Ready or timeout extension is appropriate.

RoutedFrameObservation currently contains ready and diagnostic_error only. consumer first_error, registration_failure, activation_failure and revocation_error remain separately retained and are absent from that snapshot. Thus null diagnostic_write_error does not establish that activation or Ready capture was reached; original functional failures may be unreported in Final.

## Hypothesis requiring actual stage evidence

First successful FrameSubmitted registration returns Ok(true) immediately; activation occurs only on a later FrameSubmitted. delivery.rs converts true to original NativeWindowWake::wake(COMPLETE). Native observe_dispatch requests redraw only for receipt.redraw, and subsequent Application.update updates A11y only. There may be no guaranteed second frame after registration when the host returns default effects. UI and SDK independently identified this source chain. This is not an actual root-cause verdict: no retained stage/counter proves even first FrameSubmitted or registration was reached. Meari was asked to review the original rendering/wake guarantee.

Before another centrally allocated actual run, narrowly expose original frame/registration/activation/revalidation milestones and retained functional failures separately from serialization/I/O diagnostics; retain original identities, error priority, ownership and cleanup. Do not weaken Frame/Ready validation, emit from a fixture, infer readiness from cleanup, add a new SDK, or retry blindly. Any follow-up source must be a separately pinned successor; B590/3AD, private automatic UX and timer/range packets remain distinct.

## Meari cross-review

Meari independently read the exact selected B590 consumer/controller/ViewerHost plus canonical delivery/Native paths. Registration notify changes neither projection.dirty nor HostEffects.redraw. Completion consumption can return default effects when neither projection nor chooser changes. FrameReflection may separately OR a receiver redraw, but that is conditional and not registration-driven; ChildWindowHost delegates inner effects. COMPLETE is consumed by its own event branch and does not pass through the renderer event handler; update only refreshes A11y. Thus no guaranteed registration-notify→second-frame connection exists in these sources. Other dirty/receiver/probe/external redraw causes remain possible. This strengthens the source-contract finding, not an actual phase/root-cause verdict. Meari performed no edits/Cargo/GUI. PM received exact selected source locations and this boundary.

Previously-read modoo.today defect-triage-evidence and test-fixture-and-mock-build apply: preserve original bytes, distinguish source hypothesis from runtime observation, retain actual owner provenance, and do not count synthetic/headless/cleanup evidence as actual Ready/site/media success.
