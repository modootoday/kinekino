---
id: plan:kinekino/20261029114012
title: Visible render-only product child contract — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Visible render-only product child contract — 2026-10-25

Status: source-only proposal, maintainer confirmation pending. Follow-up to native-parent-client-consumer-contract-2026-10-25.md, not a change to hidden quarantine semantics. No source/Cargo/GUI/OS/S/T/index change. SDK standalone factory proceeds independently; Meari font CPU remains undisturbed. Actual product final connection is the goal; passing quarantine alone is not completion.

## Existing owner behavior and authority separation
Product infrastructure/mpv/parent_surface/win32/construction_guard.rs registers a private WndProc with retained original Arc and creates WS_CHILD|WS_DISABLED|WS_CLIPCHILDREN|WS_CLIPSIBLINGS at0,0,0,0 without WS_VISIBLE. owner.rs::observe_hidden checks original callback identity, thread/parent/class/styles and zero client rectangle, and requires disabled/hidden. child_input.rs::with_disabled_child combines existing NativeSurfaceInput::with_checked_parent with original child sameprocess/thread, installed/notdestroying/no callbackerror, !enabled/!visible. DisabledChildProof is read-only callback scope. owner.rs::require_display always returns DisplayAdmissionUnavailable; its old comment about missing borrowed authority does not erase newer checked-client APIs.

Meari NativeCheckedParentClient proves same original successful paint/frame/placement/OSclient readback during callback, not permission to show media, actual presented pixel or focus safety. Current snapshot focusUnknown and snapshot-only product require_start denial remain. A consumer of checked origin must not silently gain visible/input authority. Hidden quarantine and visible render-only must use distinct types/entry points; no hidden flag toggled to reinterpret old proof.

## Required visible evidence (all attached to original owner, no public bool)
| Dimension | Required observation and rejection | Existing reusable seam |
| --- | --- | --- |
| Original identity | Same retained parent lease/slot/window/child generation; exact private HWND callback userdata/class/process/UIthread and actual parent relation; no replacement currentwindow | NativeSurfaceInput::with_checked_parent, preflight::parent/require_thread, original Events/install_events and generation |
| Current frame/geometry | Successful same original paint ticket/scope/document/slot/hostgeometry/placement, physical rect and supported rectangular clip/clear plane; invalidate on changed frame/HMR/resize/slot/revoke | NativeSurfaceSlot::with_parent_client and NativeCheckedParentClient::with_parent; no DTO cache or double scale/inset |
| OS visible placement | Original child at required parent-client rectangle, nonzeroarea, actual visible state/styles plus original parent relation, clipping/stacking compatibility; observation does not claim rendered content/pixel landing | Private Win32 readback on original scoped parent/child; current hidden0area predicate cannot be reused as visible success |
| Input-disabled | Exact original child remains WS_DISABLED and IsWindowEnabled=false before/after placement/show; no tabstop/input/activation policy inferred from parent logical focus | Current InputReadback checks are reusable primitives, but accepts() currently requires hidden; separate typed visible predicate needed |
| Focus noninterference | No SetFocus/foreground request; show/placement uses nonactivating policy; observe original child does not acquire focus/activation during operation and callbacks; preserve current user/parent focus without forcing restoration | Existing checked scope plus private child callback/readback; no current code proves visible child focus safety. External focus change is unknown/interference, not permission to steal or reset focus |
| Revoke | Parent/frame/document revocation blocks further placement/show/native dispatch, and safely removes this original child from visibility per original owner; old proof cannot be replayed | Same original owner lifecycle and checked invalidation; no stale snapshot fallback |
| Destroy ACK | Original generation installed, detach successful, downstream WM_NCDESTROY completed, destroyed/wake flags and class finalization verified; DestroyWindow success/handle absence/Drop alone insufficient | window_events.rs::finish_destroy; Events.complete(generation); release.rs::destroy_then_observe/release_class; preserve firstcallback/classfailure and partial owner |

A visible window bit is not actual paint visibility/presentation acknowledgement, and input-disabled flags alone are not comprehensive focus/input routing evidence. An opaque scoped proof may certify only the observed contract fields. Media engine readiness, seek landing and result ownership remain SDK responsibilities.

## Proposed typed adapter seam (names provisional, no implementation)
A product-internal VisibleRenderOnlyChild owner, UI-only/nonClone, retains original NativeSurfaceInput (lease+slot), original class/window callback owner, generation and pending cleanup. Construct inert owner before any OS call and centrally retain it across reentrancy/failure. Its callback visit, conceptually with_visible_render_only(input,visit), yields a private lifetime-bound proof after checked frame/parent and current child observations; no public constructor, Clone/Send/Sync/serde or raw-handle getter. verify() rechecks inside original callback immediately before use. Do not let physical rect/snapshot or an enable boolean manufacture it.

The existing snapshot require_rectangular_child_window requires RenderOnly, while producer reports Unknown. Maintainers must agree how the new trusted combination of paint geometry eligibility and actual child render-only evidence is represented without rewriting Unknown to RenderOnly, skipping old gates or modifying generic snapshot admission to accept untrusted input. Possible internal split/composition is a design question, not approved guard relaxation. Until that contract is accepted, both visible admission and parentroute export activation remain closed.

Error carrier preserves original SurfaceInputError/NativeParentClientError plus precise child creation/style/parent/visibility/input/focus/callback/destroy/class failure stage and original OS error. Partial create/show/revoke/destroy failure returns the original owner and firsterror; no auto replacement window/engine/retry. No OS API in presentation/domain; concrete adapter belongs infrastructure and is assembled under existing Meari lifecycle. Cleanup uses original with_cleanup_parent full request even after node removal, independently joins existing router retirement; DestroyACK and engine join are separate outcomes.

## Exact Rust file / test proposal, conditional on maintainer reply
Minimal adapter-only candidate, not full product wiring:
1. src/infrastructure/mpv/parent_surface/win32/visible_render_only.rs (new private UI-owner and scoped typed readback/proof; reuse original checked-parent and Win32 ownership primitives, no engine/guard/result code).
2. src/infrastructure/mpv/parent_surface/win32/visible_render_only/tests.rs (new pure contract/failure/ownership tests).
3. src/infrastructure/mpv/parent_surface/win32/mod.rs (private module registration only).

If shared construction/destruction primitives cannot be reused without changing HiddenSurfaceProbe semantics, stop and request an explicit extraction scope; do not silently broaden to owner/construction/window_events files. Hidden owner/tests remain unchanged by default. No Meari public code change is assumed until maintainer confirms existing API limitations. Separate subsequent product integration would require explicitly allocated original child registration/Viewer generic owner and SDK routed activation contracts; these three files alone cannot mark feature complete or enable production media.

Existing tests reuse: parent-client sameframe/stale/readback-error/scopechange and compile-fail borrow restrictions; surface-slot actual insets/clip/overlay/focuslogical/revoke; product original_generation_requires_install_and_destroy_observation, destroy_failure_never_observes_or_completes_and_return_success_is_not_ack, failed_detach_keeps_callback_reference_and_refuses_complete, successful_detach_reclaims_one_arc_after_downstream_and_tolerates_reentry, class_release_failure_retains_retry_and_success_is_once. They remain prior evidence; none rerun now.

Proposed new exact pure tests (not authored/registered/passed): visible_readback_rejects_enabled_foreign_or_destroying_child; visible_proof_never_escapes_original_checked_frame; nonactivating_transition_does_not_certify_external_focus_change; frame_revocation_blocks_new_visibility_and_retains_original_cleanup; visible_partial_creation_preserves_first_error_and_owner. Do not mirror OS implementations with fake constants; use these only for typed decision/ownership edge cases. A later allocated actual original-window test must verify visible placement + inputdisabled/focus noninterference + revocation/DestroyACK under original budgets, then real ProductFrame child registration and same-engine tests. No current OS test/deadline is invented as executed evidence.

## Maintainer coordination and completion gate
Meari received two noninterrupting follow-up queue messages identifying existing checked producer and focusUnknown/hiddenproof limitations, explicitly preserving fontCPU. Await confirmation of existing API guarantee and any already-owned visible contract before fixing exact implementation scope. No reply is treated as agreement. SDK standalone extraction is independent and was informed of the earlier producer refinement.

Contract ready when Meari acknowledges origin/geometry guarantee and remaining visible/focus boundary, SDK/product agree original child transaction and routed activation/retirement, and exact file/test scope is accepted. Implementation complete only after technical tests and original visible product child lifecycle/engine connection are verified; quarantine-only pass is insufficient. User AI-quality/visual approval follows prepared actual product review and is not a gate substituted for unfinished development.

Applied already-read full modoo.today defect-triage-evidence: distinguish observed hidden/style/callback source from proposed visible semantics and runtime evidence; keep firstfailure/owner, no success inference from WndProc return or snapshot. Narrow selected source reads, no D HDD broadscan or artifact work.
