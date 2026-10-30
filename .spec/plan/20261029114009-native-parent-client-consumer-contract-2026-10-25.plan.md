---
id: plan:kinekino/20261029114009
title: Native parent-client observation consumer contract — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native parent-client observation consumer contract — 2026-10-25

Status: private source-only contract proposal, separate from standalone factory extraction and font work. No new public API implementation, code/Cargo/GUI/OS/media run or guard relaxation. Scope is the existing Meari observation producer → product NativeSurfaceInput seam. Prior ownership HANDOFF remains historical; this document refines its origin gap with currently available producer APIs.

## Observed existing producer, not a missing implementation claim
Meari infrastructure/checked_window.rs::render prepares the observed scene with actual paint offset(insets.left,top), submits the same ticket/renderer target, then passes RendererClientPlacement{viewport,insets,target} to NativeRenderer::commit_surface_paint. infrastructure/engine/surface_slots.rs validates nonzero dimensions, viewport+each inset exactlyequals target, current SurfaceRenderScope/document/DOM/slot generations and actual paint trace before committing. Missing critical resources/stale scope invalidates or rejects; FrameTicket alone is correlation, not transferable permission.

window/surface_slots.rs::NativeSurfaceSlot::parent_client_state reads committed paint placement and validates viewport, original document, renderer weak owner/node, WindowStamp/host geometry and slot generation. with_parent_client borrows the original slot, NativeScopedParent and callback-only NativeCheckedParentClient. parent_client.rs::validate_observation rechecks ClientState::same_frame around GetClientRect/ClientToScreen, compares actual client size to renderer target, retains observed client screen origin and rejects changes during subsequent with_parent. It intentionally does not add insets/scale/zoom a second time. ClientState compares full document/window/frame/slot/geometry/placement/visibility/clip/plane/focus; observation layout_revision is not a substitute authority or required counter equality across independent reads.

Thus raw paint placement/client-origin observation is already present. Do not ask Meari to duplicate it, add a public supported bool/rawHWND/snapshot-authority constructor, or claim current runtime acceptance from source inspection.

## Product seam and remaining missing evidence
Product src/infrastructure/mpv/parent_surface/input.rs::NativeSurfaceInput owns original NativeParentLease+NativeSurfaceSlot. Its crate-private with_checked_parent already invokes slot.with_parent_client and validates matching window/child plus checked.with_parent. win32/child_input.rs::construct_checked_retained and with_disabled_child already use this path to create/verify a centrally retained hidden disabled quarantine child. DisabledChildProof is callback-scoped and revalidates original parent/child; it is neither visible media readiness nor future input permission.

The public inspect()/snapshot-only require_start path still ends in geometry.rs ParentClientOriginUnverified even for caller-constructed supported metadata. Current Meari snapshot producer also sets focus=Unknown; require_rectangular_child_window requires RenderOnly, so it remains blocked earlier for actual snapshots. Logical focus/painted content is not OS child focus policy. The stale a77 comment is not proof that the newer checked-client producer is absent.

Remaining evidence/contract gaps:
1. A product start consumer has not accepted the existing nonforgeable callback-scoped checked-client observation under the original lease/slot, while preserving the snapshot-only hard rejection. Missing is the consumer handoff/ownership and typed error contract, not another origin coordinate DTO.
2. Native visible render-only child admission has no proven OS focus/input/lifecycle observation tied to that same child. Current hidden+disabled proof and logical focusUnknown cannot authorize visible media/RenderOnly. Meari producer observations and product-owned OS child policy must remain distinct; do not write RenderOnly into a snapshot or remove the focus gate.
3. Current ProductFrame remains ViewerNoChild with no trusted child/route/ready activation contract. The SDK-owned parentroute/export capability activation is a separate prerequisite; this proposal cannot turn it on.

## Minimum contract obligations (proposed signatures intentionally not finalized)
Producer owner: Meari keeps renderer/frame/context, original window binding and slot reservation; offers the existing higher-ranked callback visit only. No stored proof DTO, Clone/Send/Sync/serde/raw handle escape. A snapshot may be logged but never used later to start. Existing public with_parent_client/with_parent is the preferred origin API; no new Meari public API is assumed necessary pending maintainer review.

Consumer owner: product NativeSurfaceInput retains original lease and slot and calls with_checked_parent inside an already centrally retained child-owner construction transaction. Validate original window/child pairing, actual successful committed paint and checked-client revalidation immediately before OS use; use physical paint rect/insets exactlyonce. Require supported rectangular visible clip/clear plane independently. No product numeric conversion may substitute another HWND/client, renderer, route epoch, document or generation. UI-thread lease cannot be moved into cleanup worker. Treat renderer/document/window/slot/host geometry epochs as separate typed owners, not equal integers.

Validity ends on original document/HMR replacement, renderer loss, slot/node removal/reissue, parent revocation, frame/geometry/placement change, critical resources, or failed OS readback. Reacquire only through the same original owners; never cache success or silently target current replacement. Preserve NativeParentClientError Access/UnsupportedPlatform/OsReadback/PlacementMismatch and original SurfaceInputError Identity/geometry/visibility/focus/Revoked/Busy/NotReady; no fallbackzero or generic success conversion. Snapshot-only require_start must continue denying arbitrary fabricated observations. A new consumer path must be explicitly distinguishable from that old denied path; it is not deletion of guards.

Cleanup: retain NativeSurfaceInput/child owner before any OS creation; failure carriers keep original lease/slot/created child and original error. On revoke block new admission first; original full CleanupRequest stamp remains usable through with_cleanup_parent. OS Destroy ACK/class finalization and original router retirement joins remain separate. DisabledChildProof/checked parent read is not DestroyACK, media success or joined status. Do not revoke result read authority merely because a policy panel closes.

Visible render-only observation must be separately specified/validated by existing product child owner and Meari lifecycle maintainers: exact original child class/process/thread/parent relation, visibility/input-disabled/focus noninterference requirements, callback/Destroy behavior and revocation ordering. The present packet does not define a new engine guard or grant visible child startup. SDK trusted routed export activation remains closed until its own accepted contract; standalone factory Phase1 proceeds independently.

## Existing tests to reuse, source names only (not newly run)
Meari engine surface_slots: target_includes_each_safe_area_edge_once; empty_or_overflowing_placement_is_rejected.
Meari window parent_client: same_frame_accepts_negative_origin_without_adding_paint_offset_again; stale_frame_geometry_host_slot_and_renderer_placement_stop_before_os_read; original_access_errors_are_preserved_without_readback_or_fallback; readback_failure_and_resize_do_not_reuse_an_old_success; scope_or_os_changes_during_readback_are_rejected. Existing compile_fail opaque constructor/Clone/Send/Sync/serde/borrow-escape contracts remain.
Meari window surface_slots: actual_observed_scene_is_unchanged_and_safe_area_offset_is_applied_once; checked_client_revoke_hmr_and_document_busy_never_enter_consumer; actual_rounded_and_unknown_url_clips_never_become_rectangular_success; actual_hoisted_overlay_blocks_the_child_plane_and_focus_stays_logical; revoked_slot_blocks_new_use_but_forwards_original_full_cleanup_request.
Product parent_surface: even_freely_constructed_supported_metadata_cannot_attest_parent_client_origin; hidden_unknown_and_zero_area_never_authorize_start; observed_physical_coordinates_are_checked_without_extra_dpi_or_zoom_multiplication; checked_parent_failure_retains_original_error_source. Product child_input: every_readback_requirement_is_fail_closed; late_destroy_ack_requests_explicit_class_finalization_without_os_reissue.

## New exact tests proposed only after contract approval
- checked_start_consumer_requires_original_callback_not_snapshot: proposed product consumer has no snapshot/raw/bool bypass, foreign lease/child denied before OS call; snapshot-only guard unchanged.
- checked_start_consumer_invalidates_on_original_frame_or_parent_change: revalidation fails between acquisition/use, no creation side effect; preserve original typed error/owners.
- checked_start_consumer_retains_original_cleanup_authority_after_revoke: original stamp and created-owner failure retained; no startup via cleanup path, no new route selected.
These are proposed names, not authored/registered/passed. Avoid duplicating Meari geometry tests. A later separately allocated single original-window observation of the chosen visible render-only contract is needed; no claim that three fake tests prove OS focus or paint visibility. Exact OS test name/deadline/resource belongs to the later accepted child integration, not invented now.

## Completion and coordination
Source contract complete when Meari confirms current checked API guarantees, product/SDK agree same original lease-slot-child transaction and typed errors, snapshot guard stays closed, visible render-only prerequisite and trusted route activation remain explicitly gated. Implementation complete only with accepted exact changes, reused/new test results, original-owner cleanup evidence and separate actual observation; no source-only success label. No GUI quality/design acceptance implied.

Sent Meari a noninterrupting follow-up queue message while fontCPU remains sole owner; no new implementation or CPU demand. SDK standalone extraction is unaffected. Read/apply existing full modoo.today defect-triage-evidence body: current producer fact versus missing consumer/runtime evidence versus proposed contract, no defect inference or new repeated test. All paths selected/narrow, no broad scan or D large I/O.
