---
id: sot:kinekino/manual-op-ed-skip-core-source-only
title: "Manual OP/ED skip core, source-only"
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Manual OP/ED skip core, source-only

## 코어 gate 최종 상태

[실제 검증 정본](../../verification/application/manual-skip-core-2026-10-20.md)에 따라 최종 r3 코어15/기존 control·status7/양 strict·fmt가 통과했다. r2의 첫 strict101은 보존하고 policy.rs의 동등한 if-let 조건 합치기만 r3로 수리했다. frozen source11 9C1BE6D389B41E9E23D089E49058F88CC6A383A068AC53C8A301641069EF15FE, PNG 등록 보존 integrated11-r3 35A0605279B8FB27EA35224B3A4D7C0406F7F6CECC7DED1404EF2B6842D57AC5다. 최초·수리 전후 시험을 중복 합산하지 않는다. source11/보호609 차이0·owned[]/명시 반환 완료이며 host17와 Deserialize 확장은 이 검증 입력에서 제외했다. SDK/CLI skip, durable 설정, host tick과 실제 Native seek 지원 완료 주장은 여전히 없다.

## 최종 읽기 검토 수리 r2

독립 검토의 확정 반례는 취소 flag 1건이었다. 선행 accepted/deferred/실행 미확정 오류 후 cancelled=true 요청이 이전 큐를 취소하지 못하는데도 BeforeAdmission으로 표시하던 경로를 고쳤다. skip(cancelled=true)는 기존 cancel()을 그대로 호출하고 `Cancelled {session_revision,status}`를 반환한다. 기존 cancel()의 BeforeAdmission/TooLateOrUnobserved 정책을 복제하지 않는다.

원 r1 2107…3167과 source/는 보존했다. 새 manifest `runtime/coordination/manual-segment-skip-r2/source11.json` SHA256 `AAB8F0753960A38E20C9A8A97C4D051F1C33F623113A2B90B1AAB943B88C86A7`, repair.diff SHA256 `128ED68D0060B64D6C56F1E33006591933C281FCFA787EB607267F89B529C501`. 변경은 application contracts/service/tests 3파일뿐이고 domain4/나머지 application2/등록2는 불변이다. 등록 patch는 기존 F0EE98…15B8 그대로이며 전체 mod 덮어쓰기를 금지한다.

추가 회귀는 accepted/deferred/error 각각 선행 skip 후 최신 revision의 cancelled flag를 보내 TooLateOrUnobserved 및 추가 observe/submit0을 검사한다. 작성된 시험은 domain4+application11=15개이며 실제 실행·컴파일은 아직 0이다. 직접 rustfmt check/diff check만 통과했다. SDK/CLI·durable 설정·host tick·live native skip 미지원 경계와 자원 보류를 유지한다.

독립 r2 재검토는 원 취소 반례 차단과 추가 확정 결함0으로 마감했다. 다만 신규 flag 취소 회귀는 revision이 정확히 +1인지, stale flag 취소가 상태를 바꾸지 않는지를 직접 assert하지 않는다. 두 번의 revision 검사 사이 외부 변경이 없는 구조와 상태 변경 전 오류 반환은 읽기 근거이며, 이 직접 시험 공백을 실행 검증으로 대신 주장하지 않는다. 제품 큐 제거·취소 완료나 실제 15pass 근거는 없다.

2026-10-20 PM assigned new domain/application9 plus two additive module registrations. This is an unexecuted Rust implementation candidate, not SDK/CLI or native playback support.

## Exact source and integration boundary

Development checkout `%USERPROFILE%/.codex/worktrees/subtitle-range-export/com.kinekino.player`, base b0c19f17a59f54a6469361b05b7fbae3ef349a85. Frozen source11 manifest `runtime/coordination/manual-segment-skip-r1/source11.json`, SHA256 `2107CA8397CCB505F463040E715F8A46932AE781E72074C40B594F13A5F33167`, source/ contains exact files. New domain4/application5 include tests. Root domain/mod.rs and application/mod.rs each add one registration; apply only `module-registration.patch` SHA256 `F0EE98E59A2200D045DE60B66F94371323C0E3B9C3BF1078166BF040FDF615B8` additively. Never replace current whole mod files, including concurrent PNG registration.

No shared API/client/schema/capability/dispatch/Cargo or live root/index modifications. No GUI, model, inference, worker, download or Cargo execution. Direct rustfmt --check on new modules/registration files and git diff --check passed. Domain4 + application10 tests are written, not run or compiled by Cargo.

## Implemented candidate contract

ManualSegment has current annotation revision, verified_revision, Opening/Ending, integer [in,out), Manual-only origin and EnginePlayback-only timebase. Confirmation never becomes model detection; edit invalidates verification. Range validation reuses existing PlaybackControl::Seek validation/precision limit; no duplicated seek bound or source-PTS conversion.

SkipPolicy has independent Off/Manual/Auto, default Manual. Evaluation requires verified nonoverlapping interval, matching session observation, authorized connected/non-error/non-idle playback, explicit known seekability, known position and duration containing the endpoint. Both manual and automatic execution conservatively block unknown duration; marker editing remains available. Automatic additionally requires Auto mode and playing; notice uses approved 3 seconds in engine playback time. No invented freshness timeout, wall-clock countdown, autonomous scheduler or host tick is provided.

SkipSession exclusively owns ephemeral segments/preferences/pass, bound to engine_instance_id + resolution_artifact_id. It uses checked session/annotation/pass revisions and typed edit/verify/policy/evaluate/skip/cancel/manual override/new-pass functions. Inspect explicitly returns durable=false and detection_supported=false. Session-bound engine/resolution identity is not a durable media fingerprint; cross-session annotation reuse and series settings require a future persisted binding/store contract. No duplicate persistent store is introduced before that scope is assigned.

SkipPlaybackPort supplies trusted observation and accepts existing ControlEpisodePlayback. Its adapter for EpisodePlaybackSessions delegates to existing engine-instance-checked control. Native observations currently lack seekability and guaranteed event freshness, so this adapter always returns Seekability::Unknown and observation_authorized=false, blocking real skip until an actually observed capability is integrated. New request DTOs cannot inject observations; the host owns the port. Fake ports proving a positive path do not prove native support.

One-pass guard records attempted segment before external admission. Automatic repeats are blocked after accepted, deferred, mismatched or error returns, preserving completion=Unobserved. Original submit errors propagate; session revision has advanced to retain attempted state, so callers must inspect after an admission error. Explicit manual retry remains possible with the new revision; no automatic replay is authorized. Incorrect engine receipt returns EngineFailed and still retains the attempt guard.

Backward authorized position observation or explicit manual-seek override blocks automatic actions for the remainder of that pass. Returning forward or changing preference does not reactivate auto. Only explicit user begin_new_pass increments pass identity and resets guards; no host observation calls it implicitly. Cancel/watch-this-once suppresses that segment in the pass. Before current admission a cancelled request performs no observe/submit. After an attempt cancel reports TooLateOrUnobserved and never claims to revoke a queued seek. Explicit manual action may override suppression. There is no actual undo control or approved 5-second undo presentation in this core slice.

## Written behavior tests and limits

Domain4: half-open/3-second notice/native precision boundaries; independent Off/Manual/Auto and manual origin; overlap vs adjacent; duration/seekability/pause/authorization unknown.
Application10: exactly one existing typed seek with Unobserved receipt; pre-/post-admission cancellation and explicit manual override; unknown/pause submit0; stale revision/session submit0; backward→forward stays blocked until explicit new pass; admission error preserved/auto replay0/manual retry explicit; edit invalidates verification/notice; deferred and mismatched receipts not replayed; Off/overlap0 then adjacent1; manual override survives policy changes.

Actual native seek/landing, durable save/reopen, unattended host tick, SDK/CLI commands, real cancellation races outside this exclusive owner, cross-session fingerprint binding, deletion/undo/series overrides/next episode and all model detection remain unimplemented or unverified. Existing segments.* capability entries remain unsupported. Registration makes the source reachable for a future allocated Cargo gate, not proof of compilation.

## Applied skill

modoo.today `modootoday/product-planning/requirement-to-testable-stories` full body previously read; focused catalog discovery selected it after unrelated finetune/frontend-first/eval matches were rejected. Applied Given/When/Then failures/boundaries to call-count, stale binding and one-pass fixtures rather than mere DTO mirrors. Rust/SRP, typed existing engine controls and source-vs-execution evidence separation preserved.
