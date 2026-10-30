---
id: plan:kinekino/20261029113748
title: ManualRepair explicit projection successor
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ManualRepair explicit projection successor

Assigned by PM after the independent state-props audit. Current262/9DF and source3 remain frozen. This is the exact-contract and minimum patch proposal; no implementation, build or GUI claim is made here.

## Source ownership

UI session owns the dedicated ManualRepair projection, callback adaptation and contract regression. SDK retains application/domain state transitions, source validation, save/CAS/adoption, and review verdict/history policy. Meari retains strict module/NodeSlot/callback and retained-frame contracts. No parser-derived retain, unknown-key drop or permissive component props.

Predecessor for the follow-up must be the centrally selected262/9DF manifest, with its exact full hash recorded by central supply owner before freezing the successor. Do not silently replace the current gate or use the earlier mount261 hash as the new executable baseline. Source3 and the audit are design inputs, not a substitute for that predecessor.

## Exact ManualRepair component contract

The maintained key/type/signature inventory and current source/verification status live in [ManualRepair contract](../../sots/player/presentation/studio/manual-repair-projection-contract.md). This plan does not duplicate the inventory. It includes the full output-review evidence/retry/reviewer contract implemented in source10, beyond the historical template subset.

## State/value and binding sources

Identity, immutable raw response/import error, original source options and task flags come from the existing stored snapshot/display. sourceChoice and explicit selected source come from the original manual source selection; no implicit first source. Before a working draft exists, all edit fields are present as empty strings and their Text callbacks bind the existing UnboundText behavior. onSourceChoice keeps DocumentChoice; onStartManual remains the existing ReviewAction::BeginManualRepair.

After the explicit BeginManualRepair action, resolve only the one item the existing API created, using the original ReviewItem/TextField input tokens. Construct front/back/title/workingText directly from those display fields and bind QueueBinding::Text to their original tokens. Unused task fields stay empty/disabled. Do not parse raw model text or reconstruct source/adoption policy. onSave/onAccept/onRefresh remain existing ReviewAction callbacks over the original ReviewIntents owner.

Busy, refresh, dirty, pending operation and editable task flags use the existing ReviewSession/panel transport facts. Required state coverage is no draft; chosen source; invalid choice; explicit working draft; edited dirty field; pending save; save error/CAS stale; stored current revision after response/refresh; explicit adoption/pending/result; raw/output review pending/error/verdict/history. API failure and stale states remain visible and disabled as appropriate. User verdict is never synthesized.

## Minimal module patch

1. `repair_panel.rs` becomes the small public preparation façade over dedicated `repair_panel/projection.rs` and `repair_panel/bindings.rs`. The projection module explicitly builds the manual scalar/list/state map from the facts above. The bindings module creates the original queue/node, explicit field-token bindings and actions; it does not call prepare_panel/prepare_correction or inherit their props. Public signatures remain stable to minimize callers.
2. Preserve existing raw_review_panel and output_review_panel projectors. Initial/bootstrap and bound/reprojected ManualRepair use the same base builder; their live review state is projected by existing session owners. Add no SDK/client/application policy.
3. Extract existing ReviewFeedback formatting into a small rendered-facts value (revision, result, errors) without changing its formatted strings. Existing typed `apply` continues writing the same keys. A ManualRepair-specific application writes only resultLabel and combines the existing revision/result text there so the current stored/output revision, confirmed save/adoption, errors and pending facts remain visible through a declared field. No temporary typed props map or subsequent key deletion. This avoids losing stored revision after refresh while retaining the current TSX contract.
4. In window_bridge's ManualRepair branch choose the manual feedback application after local-error and transport-pending facts are included. Other typed branches keep existing apply behavior. Existing entry gate, raw false path, strict NodeSlot, callback/export ownership and retained receipt/proof remain unchanged.

## Contract regression scope

Extend the existing synthetic ReviewSession projection fixture for fast per-state exact key/type/binding assertions, and compose the actual component through the original strict NodeSlot. Add a missing-required-key negative and unknown-extra-key negative. No declaration-based production filtering; tests may compare independently declared contracts.

Add a same-original-SDK scratch regression that advances explicit manual source/start/edit/save/current revision/adoption and raw/output review states through existing APIs, then renders the resulting UI projections. Reuse established SDK/storage fixtures and actual responses; assertions target display/binding/key conformity, not a second save/CAS policy suite. Isolate state cases so a history fixture does not assert AI quality approval. Do not write actual9 or invoke external inference.

Preserve original prepared Open/current inspection/release initial-mount regression and existing raw sibling/tampered negatives. The original live fixture ends with the gate unobserved; do not enable editor inputs through a fake frame. Separate projection composition coverage from live GUI event/observed-route acceptance. Central Cargo allocation follows the completed separate source packet; fmt/test/clippy results must name its exact predecessor and hash.

## Status

Exact declaration/state audit and implementation are handed off in [source10 packet](../../../runtime/development/product-manual-repair-projection-r1/HANDOFF.md), SHA256 8A88587A8E3EA8F9E85C3DE4B3B7991E7EAF408D49E58E00EC0DCA598E8D4C3D. Original projection267 failed its first type check on IntentPins; that is historical failure evidence. Latest pinsfix-inclusive267r2 source `39E0FC7E023906C1E42934647493ABFD3FC3D699EFC0D2D755D00A6E8127A880` passed central fmt/type(53.410s)/strict Clippy(46.921s), per SDK execution report relayed by PM. Root jobs no-run passed (252.340s); exact12 passed, then the13th ManualRepair graph callback expectation failed;14 were not run. The new strict three-task state and original three-task mount tests passed. A separate callback expectation successor D5F40468E2FC1071785C9E2523363395B2331A28A33DE207E1816A92D2FCB5EE adds exactly the three declared/consumed callbacks while retaining full equality; central rerun is pending. No successor GUI acceptance or user AI quality approval. See the maintained contract for explicit required/extra versus unimplemented wrong-type/foreign-node negative coverage; source review and type success do not fill those gaps.

Applied previously read modoo.today software-qa/defect-triage-evidence and software-qa/test-fixture-and-mock-build to retain strict negatives and reproduce the original task/state without duplicating SDK policy tests; Rust ownership guidance preserves the original queue, input tokens and SDK owner.
