---
id: plan:kinekino/20261029113808
title: "다음 구현 업무 1개: 기본 chooser의 저장·명시 채택·동일 항목 재열기 공통 fixture"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 다음 구현 업무 1개: 기본 chooser의 저장·명시 채택·동일 항목 재열기 공통 fixture

후속 PM source-only 배정으로 정확2 private 후보를 작성했다. `runtime/development/default-chooser-commit-journey-j4-r1/HANDOFF.md`/READY와 [source 검토 상태](../../../sots/verification/presentation/default-chooser-commit-journey-source-2026-10-24.md)에 연결한다. leaf fmt/check만 통과했으며 type·등록·actual은 아직0이다. 아래 업무는 private 구현 후보 단계이며 제품 채택·기능/시각 인수 완료가 아니다.

PM 최종 source 검토 뒤 후보를 동결했다. 다음 CPU 부모 대조는 같은 private 디렉터리 `DEPENDENCIES.md`와 dependency-closure23/stable-import-pins JSON에 분리했다. SDK r3의 원 preimage62/backup만 사용했고 활성 S는 읽지 않았다. 직접 post-card guard 부모1과 최신 a6/core7 all-target 조합용 외부21을 구별하며, 추가 공급/CPU/actual은 여전히 별도 배정 전0이다.

후속 SRP 검토·PM 승인으로 r1을 보존한 `default-chooser-commit-journey-j4-r2` private 후계(existing2+child2)를 작성했다. 새 상태/응답 관측과 판정만 분리하고 원 wrapper/expected4·행동 계약을 유지한다. [r2 source 관측](../../../sots/verification/presentation/default-chooser-commit-journey-source-r2-2026-10-24.md)과 private HANDOFF/READY로 연결한다. leaf fmt·이동 텍스트 대조 외 type/등록/actual은 아직0이다.

후속 [r2 CPU 검증](../../../sots/verification/presentation/default-chooser-commit-journey-r2-cpu-2026-10-24.md)에서 승격된 source62 기준 own4만 공급해 fmt/native strict·lib no-run·원 J4/I4/D/B 등록과 전체 nonignored lib1152/0/107을 확인했다. promoted S 기준 복원을 완료했다. 위 source-only 당시 type/등록0 기록은 역사로 유지하며 **실제13요청 journey4/시각 인수는 아직0**이다. 새 actual은 별도 배정한다.

후속 actual 배정의 [첫 Card는 실패](../../../sots/verification/presentation/default-chooser-commit-journey-r2-card-first-failure-2026-10-24.md)해 나머지3을 중지했다. 원 accepted 저장 r3는 있으나 전체13 응답·화면 정착/재열기는 미관측이다. 원 raw·forced124/Empty와 정상 cleanup 미확인을 보존하고 writer를 반환했다. 이 기록이 위 actual0 준비 시점보다 최신이며, 추가 수리·actual은 별도 배정 전0이다.

2026-10-24 읽기 감사. root `a6ca29c4fb7ff1de3f621d835fdd2f69a663b3ea`/Meari f85 기준이며 소스 변경·빌드·실제 GUI·D/B actual·index/shared source0이다. SDK sole-heavy를 유지한다. PM 인계상 자막 accepted export는 a6ca29c에 CLI 검증/커밋 완료, Card PNG type-r2는 CLI 빌드 성공 뒤 actual 대기다. 이 export 상태를 아래 proposal 편집·채택 여정의 완성 또는 출력 품질 승인으로 대체하지 않는다.

## 현재 연결 — 구현 존재와 실제 인수를 분리

공통 prefix는 `src/presentation/web_ui_adapter/proposals/`다.

| 작업 | 기본 Native route·입력 | 기존 typed 공통 연결·범위 |
| --- | --- | --- |
| 카드 | `studio.cards`; `panel_model.rs::prepare_panel`의 front/back, onFrontChange/onBackChange·source_refs | 같은 ReviewSession과 onSave/onAccept Action. I4는 두 입력의 변경·복원과 원 Save1 뒤 VIEW 복귀를 실제 확인. B의 Card는 저장된 draft 재열기 근거이며 이번 같은 여정의 명시 Accept 실적 아님 |
| 하이라이트 | `studio.shorts.recommendations`; 같은 panel의 title/onTitleChange·source_refs | 동일 Save/Accept/feedback. I4 title 입력·Save 복귀와 B의 미리 accepted된 Highlight 재열기는 서로 다른 fixture 근거 |
| 자막 교정 | `studio.subtitles.correction`; `correction_panel.rs::prepare_correction`의 onDraftChange, Cue 또는 DocumentTextRef 선택 | 동일 Save/Accept. I4 Cue/Document 입력 각각과 B DocumentCorrection exact-ref 재열기 근거가 있음. DocumentRepair 자동 복구와 실자막 export는 별도이며 이 진입 성공으로 승격하지 않음 |

route 실제 선택은 `src/assembly/meari/viewer_host/live_entry.rs::route`, `TrustedLiveChoices::from_trusted_local`, `ViewerHost::admit_live_entry/advance_live_entry`에 있다. 기본 choicesNone→명시 문서·항목→Open은 `viewing_chooser.rs::projection`과 기존 chooser owner를 사용한다. 별도 route·SDK client·worker 생성이 다음 업무가 아니다.

`panel_model.rs`와 `correction_panel.rs`는 Save/Accept를 서로 다른 Action으로 등록한다. `callback_adapter.rs::QueueBinding`은 Text/Action signature를 구분하고 `intent_queue.rs::ReviewIntents`는 같은 controller의 save/accept로 전달한다. `pending_controller.rs::PendingReviewController::{save,accept,apply}`가 서로 다른 PendingRequest와 token을 사용한다. `pending_host.rs`의 원 request 인코딩은 `proposals.edit`/`proposals.accept`와 expected revision을 typed SDK plugin으로 전달한다. 성공 mutation 뒤 원 inspect→`ReviewSession::complete_refresh`를 수행하며 UI가 CAS/저장/receipt를 자체 구현하지 않는다.

## 오류·dirty·owner의 현재 경계

- `session.rs::ReviewSession::{ready,set_input,begin_save,begin_accept}`는 scope/Arc input owner/draft generation·pending을 검사한다. dirty 상태의 Accept는 SaveRequired이며 **Save 후 명시 Accept**가 기존 순서다. 자동 저장·자동 채택으로 사용자의 한 Action을 확장하지 않는다.
- `PendingReviewController::apply`는 Save/Accept 응답과 후속 inspect를 구별한다. `ReviewSession::{confirm_mutation,require_refresh,complete_refresh}`는 mutation은 성공했지만 inspect가 실패한 상태를 단순 저장 실패로 바꾸지 않는다. `complete(Err)`는 원 ApiError와 현재 draft를 보존하며 pending을 끝낸다. 이는 UI가 모든 실패를 영구 first-error로 보존한다는 뜻이 아니다. `begin`에서 새 명시 요청은 표시 오류를 지울 수 있다.
- `review_feedback.rs::ReviewFeedback::{from_review,apply}`는 draft/output revision·dirty/pending/confirmed·SDK/local error를 공통 표시한다. A의 표시/회귀 근거는 재사용하되 최신 기본창에서 실제 SDK 오류를 발생시킨 연속 실적으로 승격하지 않는다.
- `window_bridge/navigation.rs::ProposalWindowBridge::navigation_guards`는 dirty/pending/needs_drain/ticket/close_waiting 및 confirmed/projection/staged/retired cleanup을 구별해 이동을 차단한다. `return_viewing.rs::ViewerHost::request_viewing_return`는 원 turn stamp·active owner·guard를 검사한 뒤 요청 ticket만 보유하고 즉시 editor를 해제하지 않는다. ViewingReturn의 원 first_error/retained proof와 정상 cleanup이 commit까지 이어진다. callback 게시/wake 또는 forced Empty만으로 정상 완료를 표시하지 않는다.

## 남은 공백과 단일 구현 제안

**새 제품 기능보다 먼저, 기존 기본 chooser에서 동일 proposal/item/ref의 실제 수정 → Save → 별도 Accept → VIEW → 동일 항목 재열기를 잇는 공통 opt-in 기술 fixture 1개를 구현한다.** task 인자는 Card/Highlight/Cue/DocumentCorrection으로 공유하고 UI/SDK 정책은 추가하지 않는다. 한 작업을 네 개의 독립 제품 구현으로 나누지 않는다.

이유: 현재 `viewer_native_probe/live_retained.rs::Probe::current_input_notice`는 수정값을 원문으로 복원한 뒤 `save_current_source`를 원 복귀 전제1회로 호출하고, 기록에도 “no Accept”를 명시한다. `current_input_projection`의 onAccept는 binding/enabled 확인이며 dispatch 실적이 아니다. 반면 `persisted_notice`·`persisted_case!`는 **미리 만들어진** accepted 저장소를 재열 뿐 지금 편집·저장한 결과를 이어 열지 않는다. 기존 구현이 없다고 주장하지 않고 이 두 근거 사이의 연속 연결을 구현·관측 대상으로 좁힌다.

후속 source 배정 후보는 기존 두 leaf뿐이다.

1. `src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/live_retained.rs`: 원 `run_case_source`/`seed_default_chooser`/`chooser_transcript_passes`/기록 client·같은 supervised child를 재사용하는 공통 journey fixture와 task별 인자. 원 I4/D/B 명단·기대와 역사 fixture를 수정하지 않는다.
2. `src/assembly/meari/viewer_native_probe/live_retained.rs`: 원 `Case::route`, `current_input_projection`, `set_text`, `save_current_source`, `activate`와 source VIEW/chooser 재진입을 재사용하는 별도 opt-in 관측 상태. 기존 I4 모드의 복원/Save1/Accept0 기대를 보존한다. 두 leaf 안에서 표현할 수 없다면 실제 필요한 제3 경로·이유부터 PM에 인계하며 host/domain을 임의 수리하지 않는다.

판정은 같은 owner의 선택된 ref·변경 내용이 Save 응답/inspect 후 표시되고, 별도 onAccept Action의 원 receipt/output revision이 표시된 뒤 원 guard가 풀려 VIEW를 허용하며, fresh chooser key로 **같은 proposal/item/ref와 실제 저장된 변경/receipt**를 다시 표시하는가다. SDK 요청의 원 typed 응답/독립 inspect를 사용하고 revision·receipt를 fixture가 만들어 성공을 대신하지 않는다. 실제 mutation 오류/불확실 inspect가 나오면 현재 input·원 error/confirmed·owner를 보존하고 중지한다. 오류 주입·자동 재시도·새 export 버튼은 이 단일 업무에 추가하지 않는다.

구현·CPU·actual은 아직 배정0이다. 공통 SDK/CLI의 해당 기능 실제 지원과 원 fixture·예산을 PM이 먼저 인수한 뒤 source 검토→배정 CPU→별도 actual 순서로 진행한다. 현재 D/B actual을 미리 실행하지 않는다. 원20/2/18·기존 quota/소유·첫 실패 보존을 유지하고 시간 내 불가하면 deadline을 늘리지 않고 최소 여정 분리 필요만 보고한다. 수동 시각·물리 입력·미디어/모델·새 기능 확장0이다.

## 재사용 근거와 스킬 적용

기존 `proposal-default-route-gap-audit.md`는 과거 c9 기준 감사로 유지하고 최신 상태는 이 계획과 `default-chooser-db-after-core7.md`로 보완한다. I4 actual Card/remaining3, A 공통 feedback, D source 여정, B4 persisted 재열기, core7 CPU의 각 SOT를 원 source/binary/시점별로 재사용한다. core7 fulltest1137pass/native strict·등록은 최신 CPU 근거이며 실제 명시 Accept→재열기 성공이나 수동 시각 인수가 아니다.

이 세션에서 이미 본문·참조를 읽은 modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey`와 `modootoday/software-qa/test-fixture-and-mock-build`를 재사용했다. 공통 컴포넌트/typed port를 재작성하지 않고 현재 reachable 경로·관측 공백을 나눴으며, 고정 fixture/원 client·owner/실패·teardown provenance를 다음 업무의 범위로 유지한 판단이다. 스킬 이름만으로 새 framework·다른 언어 코드·GUI 우회 검증을 도입하지 않는다.

2026-10-24 J4 r3: native fmt/strict/no-run/list passed; Card exact1 outer101/18059ms inner124 forcedEmpty, no retry/other3. Marker shows calls/replies10 and settled guardrev3 before wake; next Accepting check unobserved. Shared writer returned slot75329AEE. SOT: ../../../sots/verification/presentation/default-chooser-commit-journey-r3-diagnostics-2026-10-24.md. Private r4-candidate only rearms existing probe ReturnReady source_redraw latch for separate Accept; unvalidated, PM review required before supply/build, no production event change. Source-only HANDOFF runtime/development/default-chooser-commit-journey-j4-r4-candidate/HANDOFF.md.

2026-10-24 J4 r4: CPU fmt/strict/no-run/list passed; original Card exact1 PASS outer0/8641ms inner0/forcedfalse/WorkersQuiescent(Closed). Actual13requests/responses, Save1/explicitAccept1, accepted/reopened draft3output3; source refs/edited fields preserved by unchanged original Rust assertions. Other3/D/B actual0/manualvisual0. Shared writer returned slot0D11C373; carry24987F5C339. SOT ../../../sots/verification/presentation/default-chooser-commit-journey-r4-card-2026-10-24.md. Root/public integration pending PM, original r2/r3 failures preserved. No further execution allocation.

2026-10-24 J4r4 remaining3: samefixedEXE Highlight0/7902ms Cue0/7936ms Document0/8284ms exact1each; each13actualrequests/responses Save1Accept1 accepted/reopened3 refsfields preserved, WorkersQuiescent(Closed)/forcedfalse. Combined earlierCardexact1 now4/4. No newCargo/sourcewrites/Cardrepeat/D/B/manualvisual. ReturnedslotC52D9F14 cache95A17174/carry24987F5C339/protectiondelta0/EOF21/PID9absent/writer0. SOT ../../../sots/verification/presentation/default-chooser-commit-journey-r4-remaining3-2026-10-24.md. Rootintegration/commit/visualacceptance pending PM; oldfailures frozen.

2026-10-24 J4r4 root exact4 integrated by narrowhunks+new2; afterfrozen4 match, unrelatedWIP/indexpreserved. Minimum fmt/nativealltargetstrict/libno-run/list4+old10 PASS, mirrorrestored/protectiondelta0/EOF10/PID10absent/writer0. Sourceidenticalactual4/full1152/default1163 explicitreuse/newactual0. SlotF2E65AAB/carry251CBA79399. PM exactstage4packet runtime/verification/default-chooser-commit-journey-j4-r4-root-integration-r1/EXACT-STAGE4.md; UIstagecommit0. SOT ../../../sots/verification/presentation/default-chooser-commit-journey-r4-root-integration-2026-10-24.md.

PM exact4 commit cb6ba50f4510b3a3b985b3ed5d8fb238e39538fe completed (test(ui), push0); rootintegration stage/commit pending item resolved. Technical J4actual4 coverage only; visual/uninstrumented/D/Bnewactual scope unchanged. UI extraexecution0; committedUI4 may be protected/supplied in SDK mirror under PM lease.
