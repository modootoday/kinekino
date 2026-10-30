---
id: evidence:kinekino/20261029114025
title: "ManualRepair state projection: independent source audit"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ManualRepair state projection: independent source audit

2026-10-25, PM assigned source-only follow-up. Current262/9DF gate and source3 remain frozen. No source changes, Cargo, GUI, actual9 writes, inference or new SDK storage regressions in this audit. Findings below concern UI projection contracts, not user AI quality approval.

## Selected sources and boundary

The inherited mount261 supply identifies repair_panel.rs from ai-review-retained-recovery-r1 (6570113E3BD5F95DEFD45E4B4E38548784681E289AAFCC112EBAEF2312B4A6B4), manual-repair.tsx from the same packet (EA77EA6AB77A25B26D9FF065FE7121042A39B62CAEC3F8503ED72A8524C77A56), panel_model.rs from ai-review-repair-ui-r1, and window_bridge.rs from product-raw-review-reopen-r1 (2F7CD788CFFBD595A00C0C41B46623F4521354A0EE7515945528B7F423533522). Source3 overlays repair_panel.rs as documented in its HANDOFF. correction_panel.rs and review_feedback.rs are canonical inherited sources. This audit follows those exact producers plus source3 cleanup; it does not assert a new executable observation or independently hash the entire262/9DF supply.

ManualRepair has raw_review=false, so compose_editor_slot passes the complete props to the strict declared component. All task-specific editing remains in the original ReviewSession/application SDK; the UI mode remains ManualRepair after a draft is created. Field values change while the component declaration stays fixed.

## Exact undeclared keys after source3 cleanup

prepare_repair_panel uses prepare_panel for Card/Highlight working drafts and prepare_correction for Correction working drafts. It then removes the five previously identified typed-only keys. The remaining inherited props are still not the ManualRepair declaration:

| Producer branch | Remaining undeclared keys before common bridge feedback |
| --- | --- |
| No draft | None identified from this branch after source3 cleanup; initial mount requires central execution |
| Card working draft | sourceIdentityLabel, sourceRefs, qualityLabel, selectedCardLabel |
| Highlight working draft | sourceIdentityLabel, sourceRefs, qualityLabel, selectedCandidateLabel, rangeLabel |
| Correction working draft | sourceSha256, documentRevision, errorText, sourceIdentityLabel, qualityLabel, selectedItemLabel, sourceRefs, inputDisabled, saveDisabled, acceptDisabled, refreshDisabled, sourceCountLabel, selectedSourceText |

These are unconditional keys in their respective typed producers, not inferred from values or historical metadata. All are absent from the selected ManualRepair Props declaration. The branch fills missing front/back/title/workingText and installs ManualRepair callbacks; those actions do not remove the listed extras. Strict exact-props rejection remains appropriate.

## A second boundary reintroduces revisionLabel

Selected window_bridge.rs approximately639–642 creates ReviewFeedback::from_review, adds last_view_error and transport pending, then invokes feedback.apply on the prepared props after prepare_repair_panel returns. ReviewFeedback::apply unconditionally inserts revisionLabel and resultLabel. ManualRepair declares resultLabel but does not declare revisionLabel. This defeats the source3 local revisionLabel cleanup on every subsequent bridge projection, including no-draft source selection, local validation errors and review requests. The later raw/output-review projection adds its own already-declared families; it does not remove revisionLabel.

Do not conclude that initial mount success verifies this boundary. The original source3 fixture stops after the synchronous mount receipt/proof, with observed=false and the admission gate closed; it deliberately does not exercise editing through an unobserved live route.

## State consequences

| State / trigger | Source contract finding |
| --- | --- |
| NeedsRepair initial no draft | Initial mount covered by source3 pending centralized fixture; common bridge re-projection not yet exercised |
| Explicit source choice / invalid source choice | No draft may remain; common bridge feedback adds undeclared revisionLabel. Local error should remain resultLabel, not become a new props family |
| BeginManualRepair | Working draft is explicitly created in ReviewSession. The selected task's 4/5/13 extras appear, plus bridge revisionLabel |
| Text change / dirty / pending save | Same working-draft producer and extras. Explicit task disabled flags still carry input state; SDK save/CAS correctness is outside this audit |
| Save response / new stored revision / refresh | No component mode switch was found; persisted draft projects through the same working branch. A stored revision alone does not cure its key mismatch |
| Explicit adoption / adoption response | Same ManualRepair projection boundary; no claim about successful adoption or stored output is made |
| Raw/output verdict, history, error, pending/retry | Review panels retain their declared prefixed keys, but common bridge feedback still reintroduces revisionLabel. Actual user quality approval remains a separate GUI action |

## Minimal successor proposal, not mixed into current gate

1. Give ManualRepair its own final props construction from its explicit component contract and existing ReviewSession display/field bindings, rather than inheriting the complete typed-editor props. Reuse the typed binding/selection logic where necessary, but copy only explicitly named ManualRepair values at this producer boundary. Do not filter arbitrary props against a dynamically parsed declaration or relax NodeSlot validation.
2. Make the bridge feedback application component-aware for ManualRepair: preserve resultLabel/local errors/pending facts without adding the typed-editor-only revisionLabel. Keep shared ReviewFeedback behavior for Card/Highlight/Correction unchanged. The component identity should determine the presentation projection, with no SDK policy copy or new state transition.
3. Preserve raw/output prefixed projections, callback ownership, original inspection/release, receipt/proof and observed-route admission. Do not use synthetic frame acceptance or bypass the live gate to test editing.

## Focused validation impact

Existing `presentation::web_ui_adapter::proposals::manual_repair::tests::manual_panel_bootstrap_and_bound_projection_match_without_implicit_draft` already checks initial/bootstrap equality, explicit source choice, BeginManualRepair and pending disabled flags for all three tasks. It does not check exact declaration key equality or strict node-slot composition, so its passing assertions cannot clear this defect.

Extend that projection fixture, or add a sibling exact-contract fixture, to compose the actual ManualRepair component through the existing strict document/FrameProjection at no draft, selected source, working draft, dirty, pending, saved revision, adopted output and review status/error/history states. Reuse existing synthetic SDK fixtures and storage responses; do not rewrite save/CAS domain tests. Include an unknown extra-key negative to prove the exact guard remains active. New fixture name to reserve: `manual_repair_state_projections_match_exact_component_contract` (proposed, not implemented or run).

Keep original headless `assembly::meari::viewer_host::live_entry::tests::prepared_manual_repair_mounts_all_tasks_with_exact_props_and_original_inspection` for initial selection/current inspection/release/mount ownership. Its scope must remain initial mount, not editable route acceptance. Keep strict sibling/tampered declaration regressions. Functional GUI event integration after the original retained frame is actually observed remains central allocation work; user quality approval is not a development test fixture.

Applied previously read modoo.today software-qa/defect-triage-evidence and software-qa/test-fixture-and-mock-build: compare the actual flavor across transitions, preserve strict negative checks, distinguish source proof from executable evidence and avoid duplicate SDK storage tests. Local rust-pro ownership/error preservation guidance remains applicable. No new tool or non-Rust validation implementation was introduced.
