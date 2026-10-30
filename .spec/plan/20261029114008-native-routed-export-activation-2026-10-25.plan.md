---
id: plan:kinekino/20261029114008
title: Trusted routed skip export activation — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Trusted routed skip export activation — 2026-10-25

Status: active, source-only proposal at root3fc645572cd3d52ce40388178e7165ecbe06e2b6/Meari5893. No code/Cargo/S/T/GUI/OS execution. Extends native-playback-factory-2026-10-25.md Phase4 and UI's native-parent-client-consumer-contract-2026-10-25.md; does not supersede standalone acceptance or existing router/guard policies. Interface names below are proposed, not implemented support.

## Outcome

A trusted SDK assembly transaction publishes routed skip export only for its exact ready original media child/lease/route and one shared sessions Arc. Existing parent router/binding/issuer/native guard/result registry perform authorization and storage. No public enable boolean, standalone constructor bypass, new engine/guard or result map. Standalone3fc remains unchanged; UI buttons follow routed SDK/CLI acceptance.

## Authority

| Concern | Owner / rule |
| --- | --- |
| Checked parent-client/slot/paint observation | Existing Meari producer; callback-only checked authority, no snapshot/raw-handle substitute |
| Original child registration/ready transaction and partial failure | Product native assembly/UI-thread owner; actual ViewerNoChild is not a registered media child |
| Active original route/issuer preparation and SDK wiring | SDK trusted assembly, using existing ParentEngineRouter/ParentBinding |
| Revision/pass/cancel/guard/results | Existing application/domain/native policies, unchanged |
| Original result-review retention/end | SDK owns original handles; UI/PM must agree explicit review lifetime, not panel collapse |
| Implementation/CPU/actual allocation | PM; this document grants none |

## Evidence

Selected source10 manifest runtime/coordination/native-routed-export-activation-r1/selected-source10.json SHA2392A5B53CD0365CCE4F1E9B3864B3774F221AB359B57879142357EF6B2894DF; source reads only2026-10-25.

ParentEngineRouter::capture_native_export_issuer(&route) checks router identity and binding admission, then captures BoundExportIssuer with explicit Weak router/binding/epoch/scope. It does not itself atomically require router Phase::Active. ParentRouter::activate and request admitted() already own Active/revocation checks. Therefore capture alone is not UI-ready evidence; a separate status read followed by capture also creates a race.

DeferredEpisodeEngine uses contextNone, while ParentEngineRouter uses original Weak router. Do not mix those contexts merely because scope strings match. Direct NativeEpisodeEngine::native_export_issuer requires standalone scope. new_native_export_owner stores the given issuer but leaves export_enabledfalse. These existing closed gates must not be removed or replaced by new_standalone_export_host.

BoundExportIssuer::hold_for_lookup validates original weak router/binding/scope and epoch; its source comment explicitly separates revocation from terminal reads. Weak owners still expire if original strong holders disappear. SkipHostRegistry::begin_close blocks export_owner including list/read, so it is full registry closure, not a result-preserving parent-loss operation. Policy close removes its session but preserves original issuer row, as separately proven by standalone N/V/O. That standalone success is not routed acceptance.

UI contract confirms checkedpaint/client-origin producers already exist. The missing product ready transaction/actual child registration/visible render-only OS observation is distinct; snapshot/focusUnknown paths stay denied. No fresh GUI/OS test was run here.

## Unverified

No routed activation constructor/token, product transaction, result-retention adapter or CLI owned-parent harness is implemented by this note. Actual role/surface ready/parent loss and old-result lifetime after route retirement remain to be tested. Read-owner retention cannot be assumed merely from an issuer value containing Weak references. No GUI support/AI-quality approval or observed in-flight cancellation is claimed.

## Phases

1. **Specify the trusted mint/publish seam.** Prefer a narrow router-owned active-original-route check-and-capture operation, reusing its existing lock/admit/issuer code, rather than a mutable global current-route lookup. Trusted product assembly must already own the original checked parent/slot/registered media child and commit its readiness under the accepted UI transaction. Prepared/not-ready/foreign/revoked routes fail before publication. Parent loss during/after the transaction revokes existing native admission; no success is cached as perpetual OS validity.

   Proposed application contract is a crate-internal, non-Clone/non-Serde prepared routed owner containing only existing opaque NativeExportIssuer and the single Arc<EpisodePlaybackSessions>. Fields/construction are restricted to trusted SDK preparation, no OS/Meari types imported into application. SDK assembly creates the Arc once from the exact original route-backed engine and supplies clones to SDK and registry. A registry consuming constructor may enable its existing private wire capability only from this prepared owner; existing new_native_export_owner remains closed. Exact Rust names/visibility need review before coding. This is composition provenance, not an extra engine guard: every submission still passes existing live router/binding/issuer/guard checks. A boolean/string/engine number or a bare externally supplied issuer is insufficient to prepare it.

   Capability retains existing meaning: skip_exports.available describes configured export backend/owner, not guaranteed active media/admission. General seek_submission staysfalse. UI readiness display is separate from backend authority; no new public ready/enable flag can bypass SDK errors. Revoke may block new submissions while original reads remain available; do not misrepresent available=false as joined cleanup.

2. **Own lifecycle boundaries explicitly.** Before publication, failures retain original lease/slot/child/router/route/native and any partially built registry/SDK; first error and same cleanup attempt/deadline are preserved. !Send parent lease stays on UI thread; cleanup workers receive existing retirement handles, never window authority.

   Policy-panel/session close does not close playback or issuer. Parent loss revokes original binding/native admission immediately and retires/joins through existing NativePlaybackChild; it must not call registry begin_close solely to disable a button if original result reads are still required. Retain existing original router/route/binding and closed engine/scope strong owners for the explicit result-review lifetime while the application host lives. Retirement completing or route replacement must not accidentally destroy those owners. Holding them after revocation does not reopen native permission.

   Result review exposes only original list/peek/consume using saved issuer/origin/reference and original typed client/read handle; no current-route/endpoint retarget, no second result store. A new route gets its own registry/session/issuer context. Old result UI must not receive generic mutation authority to the replacement route. Consume terminal references through existing capacity/lifetime rules; no auto eviction or guessed native counter. When explicit review ends or the full host shuts down, release original read owners after declared cleanup; SourceExpired/NotFound/not_sent is reported honestly. Full SDK/transport shutdown can then begin_close/drain and release original owners. If the current retirement code cannot retain those strong owners, that is an implementation gap, not a promise of surviving reads.

3. **Minimal CPU and CLI gates, later allocations.** Reuse unchanged geometry/issuer/retirement tests; add only focused groups: (a) prepared/foreign/revoked route cannot activate, Active original can, with snapshot/raw/bool construction impossible; (b) partial failure retains exact owners/error/cleanup attempt and one sessions source path; (c) policy close and admission revoke block new work without retargeting original terminal reads, then explicit read-owner end expires them; (d) router-vs-Deferred context, foreign scope/epoch and replacement route reject old submissions while old reads use only original retained owners. Proposed groups are not authored/registered/passed tests.

   After Rust fmt/default+affected native strict/targeted CPU acceptance, a separately scoped CLI/SDK owned-parent harness must consume the real trusted checked-parent transaction and common routed assembly before UI controls. Existing --serve-native is standalone and cannot prove this by accepting a raw HWND flag. Specify the Rust-owned parent fixture/adapter and resource/deadline first; no new engine implementation. Minimum actual scenario is one original routed media instance/manual export/read/consume plus original parent-loss admission revoke/retirement/read-lifetime evidence. Reuse Meari checked-parent OS evidence where unchanged; do not duplicate its geometry suite or standalone N/V/O by habit. GUI later verifies dispatch/display/focus/disabled/cancel presentation, not a substitute for unsupported CLI routing.

   Coordination: UI must confirm original child/ready transaction and explicit result-review owner; SDK must confirm the active mint/registry consumption and partial-failure carrier; Meari confirms existing checked callback/revocation/cleanup guarantees. No public UI property or support label changes before those gates. PM can relay this contract for review; no new cross-chat implementation dispatch is implied.

## Rollback

Source proposal needs no runtime rollback. Future private candidates preserve exact preimages and can be withdrawn without deleting old decisions. Once a child/work is created, preserve original failure/owners and finish the same cleanup; do not spawn replacements to roll back. Revocation cannot undo a dispatched command and is not cancellation success. Actual execution cannot be undone: retain raw/failed evidence, no retry-until-green. Existing standalone CLI remains its independently accepted path throughout.

Applied actually-read modoo.today spec-authoring/plan plan.md091d2f8d92113d9995ab143ec583529c1ef98413017f3c4d8655406d768a9e2d and plan-authoring SKILL.md31459a74d470660d1e32a86f640d49370ec6f0ec61e30468cf39d2a9155e22fb: explicit authority, observations/proposals/unknowns, gates and rollback. QA/fixture skills from factory HANDOFF applied to original read/submit lifetime separation and honest scope. No new code or policy/guard/result implementation.
