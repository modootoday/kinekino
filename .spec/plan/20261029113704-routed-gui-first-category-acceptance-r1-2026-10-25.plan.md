---
id: plan:kinekino/20261029113704
title: Original GUI owner first-category execution contract
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Original GUI owner first-category execution contract

Source-only SDK review after UI observed union210327B3A56. This document neither allocates CPU/OS/media nor replaces an active input. Native47r2/current graph189 and frozen210 remain unchanged. The source packet remains uncompiled/unexecuted until PM selects an explicit successor and returns the previous slot.

## Inputs and selected ownership

- Existing fixed GUI libtest EXE/PDB and jobs integration test EXE, original Cargo JSON features/target/test provenance and current producer/copy SHA/size/mtime/FileID custody. ROUTED_GUI_FIXTURE_EXE/SHA256 must designate that exact fixed libtest, not player.exe. Existing player-cli artifact and JOB_CLI_EXE/SHA256 retain separate provenance.
- Exact explicit OwnerManifest: schema1; run_id as nonempty ASCII alphanumeric/hyphen nonce <=128 bytes; exclusive_control and record_validation true; fixed GUI executable/digest; fresh/adopted absolute job/catalog/profile paths; pinned explicit mpv path/digest. GUI PID/creation and endpoint are supplied by the original spawned owner/LocalHost event, not adopted from an unrelated process. Configuration is written once from this manifest into fresh fixture evidence.
- JOB_EVIDENCE_DIR uses a fresh exact root; Fixture::named adds routed-gui-first-category and fails on reuse. Its inherited helper hashes/copies the selected CLI into that root; those existing I/O costs must be included in the central actual-run lease, not silently performed in source preparation. No downloads, path discovery, old result substitution or unadmitted large artifact hash/copy.
- PM selects positive TOTAL/CLEANUP/READY/STOP/JOB_EXECUTION/JOB_OBSERVATION/REQUEST millisecond budgets. No historical durations become defaults. Stop allowance must cover original native/execution observation, provider's existing 30-second observation, dispatcher observation and stream/family receipts. These cooperative budgets do not promise preemption of synchronous I/O. Original Deadline is created before fixture setup/launch; every request/final/finish phase constrains rather than renews it.

## Happy path and scope

Parent starts only the existing original OwnedHost/WindowsFamily/Supervisor with fixed child libtest exact ignored test, --quiet --nocapture --test-threads=1 and an explicit environment map. Child invokes production prepare_product_playback once and starts LocalHost on that same configured client. Actual original process identity and named-pipe endpoint must correlate before commands. Child diagnostics::configure(true) precedes assembly; production Ready is emitted only after real activation and original route revalidation.

Bounded complete ReadyV1 and FinalV2 records are checked as whole envelopes, excluding LF. Libtest banners are retained without conferring readiness. Original missing/partial/oversized/wrong-version/diagnostic-failed records cannot pass. A data snapshot, marker/wake or exit0 does not replace individual owner receipts.

The initial chain is airing only: fresh catalog.search, episodes.list and episodes.resolve jobs; original selected catalog rows/title and resolution artifact; first original native playback.open; existing progress/frame/control checks; playback.close and stale ID rejection. Exact successful provider ledger is three completed, zero cancelled/interrupted. No arbitrary selected replacement, GUI previous/next, timer/range, three-category routed reopening, AI inference/quality or visual acceptance is claimed.

After action success/error/panic, parent publishes the same fresh nonce close request atomically. Private observer sends original wake.wake; original UI turn invokes actual route NavigationGuards and original wake.request_close; native child-first lifecycle owns closure. Source trace confirms COMPLETE dispatch calls NativeHostRuntime::after_dispatch even with zero queued callbacks (window.rs COMPLETE branch and native_host/runtime.rs). This is source evidence, not actual OS delivery proof.

Child keeps marker/watcher result separate from product functional final V2, writes through production outcome.write_report, and exits nonzero if fixture causality/diagnostics/cleanup fail. Parent retains raw final record and original process owner, invokes existing finish, and requires child exit0, no forced termination, empty original family, existing raw completion/EOF exports and every functional typed receipt. Observer thread cancellation/join is separate from window-close authority.

## Failure expectations

- Before spawn: invalid inputs fail without inventing an owner or endpoint. After spawn, missing original LocalHost/Ready or request errors preserve original raw evidence and request the same original normal close before finish.
- Request panic/error never becomes a successful category. ScenarioFailure retains original action error/panic and original cleanup/family evidence separately. Early setup/activation failure may yield only legacy retained setup diagnostics; absence of a full typed final remains failure, never an inferred successful receipt.
- Bad/partial/foreign close marker, watcher deadline or original guard refusal cannot pass. Deadline may ask for original cleanup, but records timeout. EOF closes only the supervisor's input channel; EOF is not a GUI close command.
- Final functional success with missing fixture close causality still produces nonzero child exit and fails parent acceptance. Missing secondary overflow is unknown, not false; original secondary AttemptFailure fields stay diagnostic prose, never synthesized ApiError details/counts.
- A retained native/provider/dispatcher owner, pending job, write/flush error, nonempty/unknown family, no EOF, forced stop or unfinished original handle is an incomplete/failed gate. Preserve original owners/failures and return the slot before proposing a successor. No in-gate repair/retry.

## Expansion order

1. Explicit union compiler and headless contract checks, exact ignored-test registration and original fixed artifacts.
2. One actual airing owner run under central OS/browser/mpv lease, original source/producer return and ledger review.
3. Separate fresh original owners for completed and movies using a separately frozen category parameter successor. This establishes three independent first-open paths, not three categories on one GUI endpoint.
4. Common SDK/CLI support and actual tests for same routed parent repeated close/reopen or atomic selected-episode replacement. Only then freeze a same-endpoint three-category GUI caller. No close/open emulation of unsupported replacement.
5. Original adjacent transition/cancel/late-ID regressions with their exact expanded job ledger; then normal player.exe build/launch gate. Libtest production assembly remains distinct from normal product acceptance. UI visual/user-quality review follows the established CLI prerequisites and central allocation.

Source review found parent manifest was previously fs::read then size-check. Separate leaf successor runtime/development/routed-gui-owner-fixture-manifest-bound-r1/source1.json SHA8285C11890AA05E21447885154AB04C90CD401547C3998CBFE573C0EEFC34BEB bounds read to16385 and rejects >16384. Same accepted schema/size, no process behavior change; private fmt/check passed, Cargo0. Frozen source8/210 is preserved. Skills applied: previously read modoo.today code-change-review-verification and required verdict table, plus test-fixture-and-mock-build; exact original ownership/evidence, source versus runtime verdicts and bounded negative contracts.
