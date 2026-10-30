---
id: plan:kinekino/20261029113956
title: 인터페이스 계약 안정화
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 인터페이스 계약 안정화

상태: 진행 중. 필드 오류 반복에 대한 2026-10-25 사용자 질문을 계기로 확인한 관리·검증 누락을 수정한다. 문서 존재, Rust 타입 검사, 일부 상태 시험, 실제 GUI 인수는 서로 다른 근거다.

## 관측과 원인

- Rust props 생성부·TSX 선언·후단 feedback가 따로 변경됐다. strict NodeSlot은 잘못된 입력을 거부했지만, 해당 상태를 실행하기 전까지 불일치가 드러나지 않았다.
- 초기 bootstrap과 상태 flag만 확인하던 시험은 working draft·저장·오류·판정·이력의 실제 props 조립을 입증하지 못했다. ignored 시험의 호출 옵션 누락과 신규 IntentPins fixture 필드 오류도 별도 결함으로 보존한다.
- verification 기록 갱신이 설계 정본 동기화를 대신했다. CLI automation-contract의 과거 지원 설명이 현재 실행 근거와 맞지 않는 것을 PM이 직접 확인했다.

근거: [상태별 props 감사](../../sots/verification/presentation/manual-repair-state-props-source-audit-2026-10-25.md), [현재 통합 판정](current-review-build.md).

## 역할과 진행

| 책임 | 담당 | 필요한 산출물 |
|---|---|---|
| SDK/CLI 정본 동기화 | SDK | 실제 Rust 명령·schema·capabilities와 기존 실행 근거를 대조한 지원/조건부/미지원/미검증 표. 검증 실행 반환 후 수행 |
| 제품 UI 경계 | UI | 명시 props producer·선언·callback signature·원 토큰/소유권 및 초기/편집/저장/오류/판정/이력의 실제 strict 조립 회귀 |
| 단일 선언/타입 강제 재사용 조사 | Meari | 기존 compiler/typed node/schema 기능의 실제 지원 범위와 최소 재사용 방안. 새 코드 생성기 구현은 아직 배정하지 않음 |
| 통합 인수와 변경 통제 | PM | 소스 선택·시험 실행·정본 링크의 일치, 지원 범위보다 넓은 완료 주장 방지 |

## 인수 기준

1. 속성/콜백 변경은 같은 후계에 선언·Rust producer·binding과 영향 상태 시험을 포함한다. SDK 정책은 Rust application/domain에 유지한다.
2. 동적 선언 기반의 임의 키 삭제, unknown props 묵살, strict guard 완화로 시험을 통과시키지 않는다. 불필요한 값은 생산 책임에서 제거하고 필요한 값은 실제 화면에 연결한다.
3. 시험은 실제 선택된 컴포넌트의 required/extra key·value type·callback signature와 원 NodeSlot을 사용한다. 누락/변조 음성 시험도 유지한다.
4. 필수 ignored 시험은 실행 조건과 실제 passed/failed/ignored 수를 확인한다. ignored 또는 미실행을 pass로 계상하지 않는다.
5. 실제 GUI 실패 조건과 시험 fixture의 작업 종류·초안 상태·소유 경로 차이를 명시한다. 원 오류 carrier·취소·정리를 보존한다.
6. 설계 정본은 현재 구현의 권위 있는 계약을 가리키고, 구현/실행 검증/사용자 품질 승인을 분리한다. 검증 일지 링크만 추가한 상태를 정본 동기화 완료로 표시하지 않는다.
7. 새 타입 강제 방식은 조사 결과와 구현/검증이 있기 전에는 제공한다고 표시하지 않는다. 현재 중앙 후보267의 정적 성공도 실제 GUI 인수를 대신하지 않는다.

리소스·소유권·Rust·비공개 Git 경계는 기존 최상위 지침을 유지한다. 이 작업 때문에 현재 동결 후보나 실행 중인 gate를 변경하지 않는다.

## 현재 정본 연결

제품 caption 시험 등록 메타데이터 보정: 실제 JOB의 모듈 경로는 route.rs의 `#[path="tests/product_route.rs"] mod product_route_tests`다. 원 expected4는 보존하고 expected4-registration-r2 `16CDE9B0B4A44CEE23E88A1C6D9609C33602833A5F3476479DC87DB0A5933EC3`를 사용한다. PM이 REGISTRATION.md를 읽었다. 코드 source10/effective371는 불변이며 실제 등록 후보는4개, helper와92개 단언은 별도다. 컴파일 후 원 artifact의 실제 --list 대조 전 등록/실행 성공으로 계상하지 않는다.

Meari 입장 경로 식별: submodule 상대 `packages/meari/Cargo.toml`은 전체 저장소에서 `packages/meari/packages/meari/Cargo.toml`인 core crate다. 전체 `packages/meari/Cargo.toml` workspace와 혼동하지 않는다. SDK가 확인한 workspace9D7C 동일성과 core의 beforeSha 차이는 별개다. PM은 core canonical의 fonts include/bundled-fonts 선언과 C core의 부재를 직접 읽었으며, 실제 전체 diff·원 preimage를 보존한 Cargo1 최소 후계로 입장을 이어가도록 배정했다. stage/Cargo 이전의 불일치를 묵살하거나 전역 의존성·lock 변경으로 해결하지 않는다.

실제9 표시 후계의 재대조: Media는 제품 effective55 `3C7729407D44240C83AF68882966CE1C493093BB732BBC62D0B34A798E1CC1F9`에서 선택4leaf를 원 표본 필드와 읽기 전용 비교했다. 알려진 문제 렌더, 미선택/선택null/원범위+PTS미검증의 구분, 일반 이력 decision_id와 원 attempt 표시가 소스상 추가됐다. 실제 factory의 구체 렌더·missing/wrong type 거부·쓰기0 시험 선언도 확인했으며 이 범위 새 결함은 보고되지 않았다. 세 누락의 소스상 수리는 확인됐으나 시험 등록·실행·GUI·사용자 품질 승인은 여전히 미검증이다.

창 제어 DTO 중복 제거: PM이 작성 중 WindowUiProjection의 caption_view와 props가 같은 disabled/label/icon 값을 각각 계산하는 것을 발견해 단일 생산을 요청했다. UI는 중복 파생을 제거하고 caption_view().props()를 Viewing·초기 document·Studio가 공유하도록 수정 중이라고 보고했다. 각 route equality fixture를 함께 작성하며 원 callback owner key/lease는 합성하지 않는다. 아직 미고정 WIP·컴파일/실행 전이므로 계약 안정화 완료로 계상하지 않는다.

닫기 알림 검증 후계: Meari r4 effective9 `5DFC497EDCE0D146B75413E3F30CD6286A0730A2B80DEB11B4E7213E009FEC09`는 r2 production을 보존하고 `actual_window_user_veto_preserves_coalesced_completion_action_and_timer_owner` Windows ignored 시험을 추가한다. PM이 HANDOFF와 소스를 읽었다. 원 prepared driver/host에서 User veto 후 Completion·action 적용·원 timer 취소/재예약·one-shot 완료와 이후 User close checkpoint를 확인하도록 작성됐다. 아직 컴파일·등록·실행0이며 empty child fixture여서 실제 미디어 자식 정리나 제품 unsaved 작업 인수를 입증하지 않는다. 다음 통합에서 명시 등록/실행 여부를 따로 확인한다.

로컬 transport 진단 후보의 PM 소스 검토: 기존 bounded TransportDiagnostics를 선택적으로 재사용하고 jobs.get/submit/cancel/other의 고정 분류로 admitted/running/execute-return/reply-sent 또는 receiver-gone을 기록한다. execute-return을 업무 성공으로 판정하지 않는다. 원 Request/Response/oneshot와 admission/close 의미를 유지하고 payload/endpoint 값을 진단에 넣지 않는다. 작성 중 소스 검토이며 시험·실제 지연 위치 확인은 미완료다.

실제9 입력 대조의 표시 계약 누락: Media가 재열기 RawReviewPanel의 알려진 품질 문제 목록, range=null/영상 PTS 미검증 안내, 일반 판정 이력의 decision_id 누락을 UI에 인계했다. PM은 실제 raw-review-panel.tsx에서 rawReviewIssues 선언은 있으나 렌더가 없는 것을 직접 확인했다. recovery 이력의 ID 표시와 일반 이력 표시를 구별한다. UI에 원 typed 데이터/공통 상태 표시와 실제 재열기 factory 시험을 같은 후계에 작성하도록 배정했다. 이는 소스 대조이며 GUI 재현·사용자 품질 승인 근거가 아니다. 실제9 판정·수정·저장 쓰기는 수행하지 않는다.

2026-10-25 PM 소스 검토 추가: Meari user-close 후보의 event-loop close 분기에서 무조건 continue하면 User veto/invalid 이후 같은 take()에 합쳐진 COMPLETE/ACTIONS/TIMER_UPDATE/LIFECYCLE/RETRY 알림이 처리되지 않을 가능성이 있다. 실행 실패로 확정하지 않고 담당에 원 wake·close 동작 대조, 동시 알림 재현 시험, 최소 후계 수리를 배정했다. 정상 owner가 남는 경우 각 알림의 원 처리·소유권을 유지해야 하며 새 timer나 busy retry로 우회하지 않는다. UI caption 소비자는 원 핀과 수리 후계를 교차 확인한다.

- [ManualRepair 투영 계약](../../sots/player/presentation/studio/manual-repair-projection-contract.md): 선언 키·타입·callback·소유권·상태별 시험의 실제 범위와 source pin별 상태.
- [공통 요소 검토 계약](../../sots/player/presentation/shared/element-review-contract.md): 선언/생성/바인딩/상태 시험의 동시 인계 기준.
- Meari `packages/meari/.spec/evidence/research/declaration-projection-contract-reuse-2026-10-25.md`: 기존 AST→Rust Props 생성·SourceSchema·strict binder·owned factory 재사용 범위. 제품의 동적 map과 SDK binding을 자동 연결하는 컴파일 계약은 아직 제공되지 않는다.

SourceSchema fingerprint는 본문을 무시하는 선언 계약이므로 전체 소스나 live 권한 정체성으로 쓰지 않는다. 동일 callback signature라도 다른 SDK action을 연결하는 오류는 shape 검사만으로 검출할 수 없어 원 queue dispatch 검증을 별도로 유지한다. 공통 helper/CI의 신규 구현은 아직 미완료이며 현재 후보 시험 결과와 구분한다.

## Native route 후계의 공통 계약 검토

PM은 SDK private `application/route_media/mod.rs`와 UI `product-live-native-retire-gate-r1`→`product-live-native-retire-rearm-guard-r1`의 실제 본문을 읽었다. begin/poll retirement, cancel, begin/poll viewing의 공통 타입·호출 이름은 일치한다. UI는 원 opaque ticket을 유지하고 Pending 중 새 inspection/mount를 허용하지 않으며 native Complete와 retained route 관측을 함께 기다리도록 작성됐다. 이는 소스 검토이며 컴파일·실제 OS 성공은 아직 아니다. SDK의 실제 coordinator 회귀와 final manifest를 기다린 뒤 중앙 단일 실행을 배정한다.

적용 스킬: modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문(2026-10-25 재열람, skill_call). 모델은 실제 리소스 동작을 입증하지 못하므로 원 child coordinator 시험과 최종 정상 GUI 전환을 별도 인수로 유지한다. fixture는 실패시에도 해제하고 새 lease/등록 복구를 확인해야 한다. 기존 assertion을 보존하고 미실행 시간은 추정 성공으로 기록하지 않는다. 새 타 언어 검증 코드는 도입하지 않는다.
### 후계 시험 범위 소스 확인 (실행 전)

PM은 private route-media-retirement-r1의 실제 시험 본문과 UI HANDOFF를 확인했다. 정책6은 중복 begin·미완료 차단·foreign scope/port·이전 ticket·새 generation·실패/취소와 첫 오류를 다룬다. coordinator3은 실제 WindowLifecycle/PlaybackSurfaceChild 조립에 Probe 엔진/surface를 연결해 engine→surface 순서, 실패 소유자 보존·복구, 새 child lease와 stale 거부를 검사한다. 이는 실제 coordinator 시험이지만 실제 HWND/mpv 엔진 정리의 증거는 아니다. UI 모델2와 원 SDK scratch ignored2는 Draining pending 중 inspection0·동일 ticket 완료 후 inspection·close 중 ticket 보존을 다룬다. 기존27과 합쳐 최소40개 영향 회귀가 예정되며, 정확한 명령/선택/개수는 SDK final manifest 인계 때 고정한다. 현재 실행0이며 시험 함수 존재를 통과로 계상하지 않는다.
### TransitionSlot 후계의 동시성 검토

PM은 작성 중인 route-transition-retirement-r1의 slot/retirement.rs를 읽었다. pause_route의 paused=true가 mutex 이전에만 설정되고 resume_route가 mutex 안에서 false로 풀면, 새 pause가 lock 대기 중 설정한 true가 앞선 resume에 덮이는 순서가 가능하다. 이는 source-level interleaving finding이며 실제 race 재현/실행 실패를 관측한 것은 아니다. SDK에 mutex 획득 후 pause 상태와 retirement ticket의 정합성을 다시 보장하는 최소 수리 및 결정적 경합 회귀를 요청했다. parent closed bit의 영구 거부와 원 ticket 검증은 유지한다. 현재 frozen275-r2 실행 결과를 이 새 후계에 승계하지 않는다.
### exact48의 오류 반환과 재시도 계약 구분

PM 추가 본문 확인: Meari native_host/runtime.rs의 after_dispatch는 consumer Err에서 retry_after_dispatch 결과를 turn_retry에 저장하고 typed last_host_error와 HostAccessError::Invalid를 함께 반환한다. ViewerHost::retry_after_dispatch는 ReadConsumerRetry를 After(delay)로 매핑한다. 따라서 Invalid 반환만으로 production의 복구 불능을 확정하지 않는다. exact48이 positive fixture에서 이를 예상 실패로 판정한 사실은 유지하며, window/Application의 원 turn_retry 소비와 fixture 처리 차이를 확인해야 한다. 기존 생산 재시도 계약을 정확히 검증하는 fixture 수리도 허용하되, 모든 Retry 묵살·원 오류 삭제·단언 완화는 금지한다. 새로운 HostEffects 필드/공통 API 확장이 필수라는 가정을 하지 않는다. 이 구분을 UI 담당에게 직접 인계했다.
UI 담당은 원 runtime 계약을 대조한 뒤 valid NativePending을 Err로 표현한 생산 경계를 최소 수정하기로 했다. 후보 방향은 원 HostEffects의 정상 무커밋 반환+기존 WindowUiHost NativeHostTimer의 5ms one-shot 예약이며, 실제 registration redraw가 있으면 그 원 요청을 우선 전달한다. 이는 아직 소스 작성 중이며 실행 성공이 아니다. 실 native 오류의 첫 Arc/동기 commit receipt/inspection 차단을 유지한다. PM은 같은 timer의 idle control deadline이 재시도로 유실·영구 연장되지 않는 소유권/기한 회귀와 close/foreign scope 정리를 요구했다. fixture는 물리 timer만 명시 모델로 대체하고 원 runtime의 오류 없음·inspection0을 확인하며 Retry 일괄 묵살을 도입하지 않는다.
수정 방향 확정 전 추가 대조 결과(앞선 생산 timer 후보 취소): UI가 WindowPort→Application.observe_dispatch의 오류 경로에서 실제 schedule.defer(clock, After5ms, limits)를 확인했다. 따라서 기존 runtime의 Invalid+typed retry 반환은 원 스케줄러가 소비하는 계약이며, 새 생산 timer/API 확장안은 구현 전에 취소했다. 최소 후계는 원 시험 fixture의 scheduler 모델링이다. variant6/Draining/callbacksReleased/원 retire ticket/inspection 없음/정확한 ReadConsumerRetry5ms/원 retry_after_dispatch After5ms를 모두 단언하고, 원 오류 기록을 보존한 뒤 같은 runtime의 후속 turn으로 진행한다. 그 외 phase/delay/실오류는 기존 positive failure를 유지한다. 이는 아직 실행 전 소스 선택이며 정상 GUI의 실제 timer 성공을 의미하지 않는다. 원 source6/guard2/redraw4 생산 소스는 변경하지 않는다.
