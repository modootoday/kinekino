---
id: plan:kinekino/20261029113710
title: "Non-AI OP/ED skip: independent implementation proposal"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Non-AI OP/ED skip: independent implementation proposal

코어 gate 마감: [실제 검증](../../sots/verification/application/manual-skip-core-2026-10-20.md). frozen r3 9C1BE6…F15FE/PNG 보존 integrated35A060…57AC5, 최종15pass·기존회귀7·양 strict/fmt0. 최초 strict101과 원 r2/r3 보존, 보호609/source11 차이0·owned[]·heavy/snapshot PM 반환. host17는 별도 미실행 source로 유지하고 코어/호스트 혼합 검증 완료를 주장하지 않는다. rootlive 통합은 PM 소유다.

최종 독립 검토의 취소 flag 반례 1건을 source11 r2로 수리했다. manifest AAB8F0753960A38E20C9A8A97C4D051F1C33F623113A2B90B1AAB943B88C86A7, contracts/service/tests만 변경, domain4+application11 작성/미실행이다. r1은 보존한다. PM은 다음 host session registry와 typed SDK/CLI 연결을 구현 범위로 승인했으며, API 중앙 파일은 PNG 마감 후 exact diff로 병합해야 한다. 장수명 owner·opaque host generation·endpoint별 세션·restart 실효/durablefalse·native Unknown 차단 계약을 유지하고 아직 실행 배정은 없다.

PM assigned exact9 + registrations2; source11 candidate completed, no Cargo execution. [Core source contract](../../sots/player/application/manual-segment-skip.md), manifest2107CA8397CCB505F463040E715F8A46932AE781E72074C40B594F13A5F33167. Registrations are separate additive patch F0EE98E59A2200D045DE60B66F94371323C0E3B9C3BF1078166BF040FDF615B8, not whole mod replacement. Core uses exclusive ephemeral SkipSession rather than introducing a speculative persistent store; durable storage remains explicitly unsupported. Unknown-duration manual execution is conservatively blocked, marker editing allowed. Domain4/application10 fixture tests are written/unexecuted. Next: independent source review, allocated core Cargo gate, then separately assigned SDK/CLI + observed seekability/persistence/tick integration. Existing modules/Cargo/API shared files and live snapshot unchanged.

2026-10-20 PM asked for a bounded source survey and implementation assignment request. No source merge/Cargo/model/download/inference performed. Observation is current product source, not a claim that placeholders are supported.

## Observed reuse and gaps

- `src/application/api/capabilities.rs` lists segments.detect/edit/accept_skip/set_skip_policy, but no executable segment branch or domain annotation/policy DTO was found. They remain unsupported.
- Approved `.spec/sots/player/presentation/studio/segments.md` specifies SegmentAnnotation identity/media/fingerprint/kind/in_us/out_us/origin/evidence/verified_revision, OP/ED Off/Manual/Auto, manual start/end editing, explicit verification, notice3s, undo5s, no repeat after re-entry, next-episode default off. Unknown duration, unverified/overlapping/invalid ranges disable automatic skip. These are design contracts, not implementation.
- `src/application/workspace_read/preview_inspector.rs` returns static Manual/notice3s/undo5s values. This is preview projection, not saved preference or playback policy.
- Reuse `DecimalU64` wire integers and `PlaybackControl::Seek`, its MAX_SEEK_US bound and validate(). Reuse `EpisodePlaybackSessions`/`EpisodePlaybackEngine::control(ControlEpisodePlayback)` for engine-instance-checked seek; no new engine command string/OS adapter.
- `EpisodePlaybackObservation` binds engine_instance_id and resolution_artifact_id to PlaybackStatus. Its position_us is engine playback time, explicitly not source PTS. Duration/position/paused may be unknown; published_age is publication age, not guaranteed native event freshness. There is no seekable observation. Never infer seekability merely from a nonzero duration or successful queue admission.
- No reusable generic segment interval/fingerprint DTO was identified in the narrow domain search. SubtitleRange is format-specific clipping/export policy, so it will not become OP/ED policy or be copied wholesale.

## Small independent scope requested

Request source-only ownership of these new Rust directories, preserving existing playback/SDK modules:

1. `src/domain/skip_segments/mod.rs`: public segment identity/kind/timebase/origin/verification types, explicit origin Manual only in first slice.
2. `src/domain/skip_segments/annotation.rs`: half-open integer-us range, checked positive duration/seek precision, same-media binding and overlap validation. EnginePlayback timebase explicit; no unproved source-PTS conversion.
3. `src/domain/skip_segments/policy.rs`: independent OP/ED Off/Manual/Auto(default Manual), pure evaluation/eligibility and notice/override/one-pass guard state.
4. `src/domain/skip_segments/tests.rs`: boundaries, mixed OP/ED modes, overlap/unknown/stale binding/manual override/re-entry/cancel cases with distinct fixtures.
5. `src/application/skip_segments/mod.rs`: exports and small service entry points.
6. `src/application/skip_segments/contracts.rs`: strict typed edit/verify/preference/evaluate/skip/override requests and typed decision/receipt; local session revision and expected revision are explicit.
7. `src/application/skip_segments/ports.rs`: narrow session observation (existing observation plus independently observed seekability Unknown/Yes/No) and session-state persistence boundary. Unknown by default. No guessed support on existing engine.
8. `src/application/skip_segments/service.rs`: state transitions/evaluate/explicit skip, reuse existing identity-checked seek port and ControlReceipt; caller-controlled cancellation/override checked before admission.
9. `src/application/skip_segments/tests.rs`: counted fake state/observation/seek ports; no-op cases submit0, accepted vs unobserved landing, stale instance/revision rejection, rejected seek leaves guard retryable without implicit replay.

Request separate central minimal module registrations `src/domain/mod.rs` and `src/application/mod.rs` when allocated for compilation. Existing SDK client/capabilities/dispatch/schema, persistence adapters and CLI wiring remain centrally assigned follow-up, not silently owned. No new crate/dependency/Cargo changes, giant utils, GUI or new engine implementation. Initial service is explicit session-bound state with an injected store boundary; do not advertise durable episode/series settings until an actual store adapter is assigned and verified. Full cross-episode copy/series overrides/AI detections/next-episode navigation remain outside the first slice.

## Proposed policy decisions

- A manually authored segment stays Manual origin; confirmation never changes it to Detected/AI. Auto means automatic execution of an explicitly verified interval, not automatic detection. Candidate/unverified annotation is never auto eligible. Editing endpoints invalidates prior verification and pending notice.
- Ranges use [in,out). Auto requires matching current media/resolution+engine session, verified current annotation revision, nonoverlap, explicit Auto preference and explicit known seekable Yes, known position/duration, positive bounded interval within duration, playing state and an authorized observation. Unknown duration disables auto. No generic wall-clock freshness threshold is invented; stale/unknown trusted observation blocks auto.
- Manual action remains explicit. It may propose a valid endpoint with unknown duration, but seek submission still requires observed seekable Yes and same active session; known duration must contain the endpoint. The proposal does not claim a successful seek landed merely because it was enqueued. Seekable No/Unknown gives Unsupported/NeedsInput respectively, submit0. Whether unknown-duration manual submission should be further restricted is an explicit PM decision before source freeze; conservative default can withhold execution without disabling marker editing.
- Notice3s and undo5s come from the approved design, use checked playback-time arithmetic and injected observation/clock; paused/unknown position does not progress an automatic countdown. Auto submission must not occur merely because a persisted flag says Auto. No autonomous polling thread or scheduler is introduced in this slice; a typed evaluate/tick call drives state. Actual unattended host ticking is a later integration obligation.
- Watch-this-once/explicit cancellation or manual seek invalidates pending automatic action for that segment/session pass before submission. Do not close playback. A session-level already-attempted guard prevents automatic repeat after undo/re-entry; only explicit new pass/session policy resets it. Do not silently assume a completed seek or manufacture a landing position.
- Cancel before submit means submit0. After existing queue acceptance there is no supported revoke operation, so report too-late/admitted/unobserved rather than claiming cancellation. Existing ControlReceipt Accepted/DeferredUntilHandoff and Completion::Unobserved remain authoritative. Deferred is not a completed skip; auto should require a ready observation and must not automatically resubmit ambiguous admission.
- Preserve manual play/pause/seek precedence; reject same-session stale revisions/changed endpoints/media replacement with no seek. Storage/engine errors propagate and never become confirmed/detected evidence.

## Proposed SDK/CLI contract after core assignment

Reuse existing names segments.edit (manual annotation), segments.accept_skip (confirm current revision, not seek completion), segments.set_skip_policy (OP/ED preference). Add explicit segments.inspect, segments.evaluate, segments.skip and segments.override only under central SDK ownership. detect remains Unsupported; no automatic detection claim. Each method requires current media/session identity and expected state revision where mutating. CLI and typed SDK share the same service; no GUI automation. Persisted editing/preference support and engine submit support must be exposed separately; schema may not claim auto execution if seekability/tick/store is unavailable.

Core acceptance Given matching verified segment + Auto + known playing/seekable/duration → exactly one end-seek admission; When paused/unknown/unverified/overlap/outside/stale/override → submit0 and explicit reason. Given explicit manual skip → same target/control receipt as existing seek and no automatic retry. Given cancellation before/after admission → respectively no write or truthful too-late. Distinct fixtures retain source/manual origin and never invent detection confidence. Later fresh CLI gate must cover edit/confirm/policy/read/evaluate/seek rejection/save/reopen/cancel with actual supported adapters; fake port tests alone cannot mark real skip support.

Skill source: modoo.today `modootoday/product-planning/requirement-to-testable-stories`, full body previously read and reused after a focused catalog search. Applied checkable normal/failure/boundary stories, explicit dependencies and unresolved policy decisions; no fabricated thresholds. Initial search returned finetune/frontend-first/LLM eval matches which were rejected as unrelated. Rust only, SRP and resource holds preserved.

Next action requested from PM: assign the exact new domain/application9 source scope (plus central registration timing) and settle unknown-duration manual execution conservatively. Then implement immediately source-only rather than conducting another broad research pass. Core gate and actual SDK/CLI integration are staged resource allocations, not repeated user approval requests.
