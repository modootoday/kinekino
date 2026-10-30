---
id: evidence:kinekino/20261029114110
title: "Normal GUI267r3: ManualRepair mounted, retained frame blocked"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Normal GUI267r3: ManualRepair mounted, retained frame blocked

Exact source `720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137` passed central fmt/type/Clippy/no-run and27 exact tests. This did not establish GUI route activation. Original normal player SHA256 `C5F5F60FFD593645218921CC4A7FCB64FED17CF1C7D3CB16B3BEFD37CE86AD30`,161468928 bytes; original producer22-normal-bins-provenance checked before launch.

## Original run and observations

PID95768, creation2026-10-25T03:34:54.8994645Z, window12849584/Meari Native,1182×792/default main-monitor placement. Same original argv/three roots in [launch owner](../../../../../runtime/coordination/normal-gui-projection267-r3/launch-owner.json). No new build/inference, fullscreen/DPI/VM change, actual9 edit/save/adoption/verdict or input into another app.

Selected actual9-review-r1-gemma270-correction and ManualRepair Correction. Open clicked once03:35:56.197Z. The model raw JSON and schema mismatch text appeared in the actual ManualRepair UI. The prior Adapter mount failure was not observed. Phase trace: Draining → Starting → Inspecting → Inspected → AwaitingArm → AwaitingFrame. No Activating/Active record. Editing/start/save/adoption fields remained disabled as shown by UIA. Raw package/item inputs and read entry controls existed but were not exercised after the stalled-frame boundary; original source/candidate/model/history and three representative tasks are therefore NOT fully GUI accepted.

Same-window UIA showed new Studio navigation and ManualRepair only; old Viewing controls were absent. The screenshot showed a black rectangle approximately window-relative x0–881/y62–344 covering the Studio upper region, plus some unstyled manual buttons. This is consistent with a retained native video child, but physical child bounds were not independently observed; it is not proof of that cause. One transient capture was entirely black while later capture+UIA again showed Studio; that capture alone does not establish an all-black product window.

Evidence [interaction summary](../../../../../runtime/coordination/normal-gui-projection267-r3/interaction-summary.json) retains UIA and timestamps. [Visible Studio capture](../../../../../runtime/coordination/normal-gui-projection267-r3/correction-scrolled-1792899413225.jpg) SHA256 `29DEAE920C3F2287FBCEBC6C6DF814A7321051B5F3C49E49ED97F21DDE80DB43`; earlier black capture correction-open-1792899364847.jpg SHA256 `310C7B089B1CA93CDDD0AC0D4570A2C923209CC4635ECD1EAC6AA1B261419195`. Unique paths were preserved without overwrite.

## Exact final error and cleanup

[stdout](../../../../../runtime/coordination/normal-gui-projection267-r3/stdout.raw) routed_frame_trace.first_functional_error:

`Surface(ParentMediaSurfaceError(Visible(Input(ParentClient(Access(Slot(NotVideo)))))))`

frame_callbacks15, last_scene_generation15, original_engine_retained=true, admission_revoked=true, state_revoked=false. This is the first captured structural error; do not replace it with an assumed missing redraw or ignore NotVideo. Private phase metadata has no mount error because mount already succeeded.

One normal Alt+F4 requested03:37:06.213Z, original PID absence verified. Normal OS exit, WorkersQuiescent(Closed), dispatcher joined/faults0, execution stopped/unacknowledged0, LocalHost pipe/SDK joined and cleanup_complete=true. Cleanup success is separate from route success. No retry/relaunch after failure.

## Source boundary and minimum proposal

Selected viewer_callbacks/lifecycle.rs prepare_route_release validates scope then releases only callbacks/chooser/pump tickets/registry/attachment. It contains no explicit native child hide/engine release/frame consumer retirement. live_entry Draining calls consume_reads and prepare_route_release; after None it enters Starting, original inspection and strict mount, preserving original host/owner/definition.

Selected viewer_native_launch/routed/consumer.rs submitted validates original host/owner/definition/window, then an already activated registration unconditionally refreshes/revalidates its original video checkpoint. Meari's reserve_video/surface_node rejects missing/non-video nodes as NotVideo. A Studio document can share the original owner/definition while no longer exposing the Viewing video. SDK independently identified delivery order: route.observe/pending wake mark precedes consumer.observe, and wake delivery follows it; a consumer error can therefore interrupt completion even if route observation already advanced. AwaitingFrame alone does not prove redraw absence.

Proposed successor boundary: before replacing Viewing video DOM, coordinate the original route's SDK callback drain with explicit native frame-consumer/child surface lifetime handoff. Retain the same original owner/cleanup carrier and verify no old video checkpoint is applied to the Studio document; return-to-Viewing must establish current checked slot authority rather than reuse a stale surface. Exact child/engine retention and cancellation behavior is owned jointly with SDK/native adapter and Meari contracts. No code change is made on this hypothesis yet. Do not waive NotVideo, fake a frame, reuse a ready boolean or suppress original errors.

Applied previously read Computer Use skill for same-window UI observation and normal close; modoo.today defect-triage-evidence/test-fixture-and-mock-build for original evidence/owner preservation and separating frame, callback, visual and cleanup outcomes. Viewing everyday UX private edits were preserved at file boundaries and were not compiled into this run.
