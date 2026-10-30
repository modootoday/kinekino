---
id: evidence:kinekino/20261029114020
title: 실제 SDK → Native proposal startup 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 SDK → Native proposal startup 소비 검증

2026-10-20, UI 세션의 소비 gate 최종 기록이다. 전체 제품·기본 런처·시각13화면 인수가 아니다. 계획은 [typed async SDK](../../../plans/player/presentation/typed-async-sdk-s1.md), 이전83 의미 기준은 [pending document SDK](pending-document-sdk-2026-10-20.md), SDK 조립 계약은 [startup 조립](../../../plans/player/sdk-proposal-window-startup.md)을 따른다.

## 입력과 소유

Native 최종316파일 inventory `2827a5a6b4b902ce00b4f1acbb1631b157c5c93dd41dea5ca50521753b63132a` 및 UI16 before의 실제 파일 hash 차이는0이다. before 사본·관측은 `runtime/quality-review/ui-native-pending-r1/`에 있다. generic5는 수정하지 않는다. SDK core E195 입력도 SDK 담당이 동결했다.

SDK 최초 조립5파일 `9CA03C7A223B7CF5332FE4CAEBEB035BD36F8E00CDB8A8A32BC6CC7668B6579C`를 먼저 소비했다. compile-first terminal 뒤 SDK 담당이 승인된 trusted notifier-wrapper seam을 추가했고, 후속5파일 `22E1F7C92A17A6B23C36184F4158A51CCA88F15AA70239F109A76BEE8E2453ED`를 실제 재hash해 인수했다. consumer Cargo/lock EF8465…71BC/E5C8BB…04D0는 그대로다. root Cargo/Native/Core/SDK core/generic/SEG/CLIP/CSS/테마는 이번 UI 구현 범위가 아니다.

## 실제 관측

| 회차 | 입력·결과 | 의미와 제한 |
|---|---|---|
| compile-first | 중앙 runner, offline/locked/-j1, affinity3/BelowNormal, 기존 consumer target, RUST_TEST_THREADS1. 583412ms/exit0 | 전체 시험 실행 파일 생성. OS 시험의 미사용 Arc 경고1을 보존 후 수리. 실제 시험 실행0 |
| focused-first | 동일 제약,69741ms/exit0. native_pending7pass/6ignored, pending_document10pass | initial_panel7이 두 실행 파일에 중복 포함된다. 원래 실제 SDK correction/cards/highlights3과 초기 소유/ACK/부분 조립 회귀가 통과. 신규 OS6은 미실행 |
| full-first | 252870ms/exit0,91pass/0fail/6ignored | 기존83 의미+registry-guard/provider Busy 구분 회귀1+새 실행 파일의 초기 시험 중복7. 실제OS6 미실행. controlled guard 실패→provider ACK Busy 구분→ACK1/edit1·재실행0을 확인 |
| strict-first → r2 | 68404ms/exit101 →13062ms/exit0 | OS 시험의 collapsible_if 원문/소스 보존 후 allow 없이 수리 |
| strict-r3/fmt-first | 3639ms/exit0,3840ms/exit0 | doc0 원 관측 저장 및 typed exit/알림2/raw Arc 보강 뒤의 검사 |
| strict-commit-first → r2 | 10338ms/exit101 →3507ms/exit0 | 새 시험의 Option 타입 추론 오류 원문/소스를 보존하고 명시 타입으로 수리. commit port를 포함한 최종 Rust 소스의 strict 통과 |
| OS 최초6 | prepared43576/error1808/closed-notifier1664/correction5725/cards3113/highlights3149ms, 각각exit0·exact1pass | 각각 별도 프로세스. Ready/checked frame 및 실제 SDK 의미 보존·문서0 cleanup 검증. 아래 원OS6 입력 변경 뒤 영향 재검증과 구분 |
| OS guard-first → r2 | 57561ms/exit124 →39374ms/exit0·exact1pass | 최초 시험의 전체 stamp equality가 잘못됨. owner/definition/docrev와 raw ticket은 유지되며 실패 관측도 turn_sequence를1 증가시킨다. 원소스/로그·worker barrier timeout을 보존하고 정확 단언으로 수리 |
| OS partial-first | 2986ms/exit0·exact1pass | 실제 등록 뒤 handle fault→quiescent unbind→같은 prepared Arc·worker로 재부착. SDK initial inspect 추가0 |
| OS commit-first | 7366ms/exit0·exact1pass | trusted apply 경계의 commit 전 거절1회→임시 staged export Stale·doc revision/owner 유지·pending projection 유지→명시 retry 실제 receipt1/flag clear. SDK5명령 전체 유지·edit 추가0 |
| full-final | 82040ms/exit0,91pass/0fail/9ignored | commit port를 포함한 최종 입력. 원83 의미+guard 구분 회귀1+새 실행 파일의 initial 중복7. OS9은 별도 실행 |
| OS 최종6 | prepared2604/error1800/closed-notifier1735/correction3304/cards3004/highlights3096ms, 각각exit0·exact1pass | 원OS6의 helper 추출·trusted port 추가 뒤 같은 최종 source에서 영향 재검증. guard-r2/partial-first/commit-first3도 같은 최종 source의 exact1pass |
| fmt-final | 5438ms/exit0 | 최종 source의 cargo fmt --all -- --check. strict-commit-r2 입력과 최종 Rust source는 같으므로 strict를 중복하지 않음 |

원 로그/receipt는 각 회차 이름을 유지하며 덮어쓰지 않는다. compile-first 후보11파일 사본과 pin도 별도 보존했다. elapsed는 성능 개선이나 병목 원인의 근거가 아니다.

최종 UI source19 inventory는 `runtime/quality-review/ui-native-pending-r1/final-source-inventory.json`, SHA256 **26335921b555b8732b3609f9d22a7ad7dda93b9f5e3d4998111381818774f35c**다. source-final 사본·before16·ui-source.diff·최초 실패 소스를 보존했다. 최종19 실제 hash 차이0, Native316/SDK core/generic5 재hash 차이0, `final-owned.json=[]`이다. consumer target 아래 소유 시험 프로세스가 남지 않았음을 확인했고 단일 heavy 슬롯을 PM에 반환한다.

## 관측 의미와 제한

실제OS3task는 같은 SDK/client/ExecutionOwner로 원본 TSX의 normalized 입력을 처리한다. initial prepare→ACK 전 mount0, actual scope의 edit/inspect/accept/inspect와 worker thread가 UI thread 밖임을 확인한다. Ready 및 FrameSubmitted의 render-time HostStamp를 검사하며 checked CPU 제출을 OS compositor 표시 ACK로 확대하지 않는다. 수동 Input turn과 Native completion turn을 구별한다. commit 주입 시험의 edit terminal 수용/명시 retry는 수동 Completion turn이며 나머지 terminal은 실제 Native wake로 처리한다.

doc0 prepared/error는 invalid budget 실패 뒤 같은 startup stamp/client/notifier로 retry하고 wrapper 생성1회를 확인한다. 실제 inspect terminal1+worker-exit1의 총 알림2, 최종 notifier를 Native에 먼저 위임한 뒤 barrier, 실제 WorkerExitPending{loop_finished:true}와 최소3 After 관측, release 뒤 추가 알림/입력0으로 실제 Closed receipt를 확인한다. 반복 Closed가 같은 정상 상태를 유지하고 canonical NotFound/정상 결과의 원 raw Arc를 ACK·join 뒤에도 보존한다. SDK 내부 join/ACK 호출 카운터는 이 제품 API에서 직접 노출되지 않으므로 '정확 호출1회'의 직접 계수로 주장하지 않는다. 동일 owner의 실제 joined receipt와 기존 SDK 내부 회귀·Core accepted ACK 회귀를 구별한다.

닫힌 Native notifier 시험은 같은 실제 SDK worker의 원 terminal/응답 성공을 유지하고 notification_error의 canonical SourceExpired/kind=closed를 확인한 뒤 명시 ACK와 join을 검사한다. guard 시험은 실제 edit terminal의 raw Arc/ticket/렌더 revision을 유지한 registry borrow 실패를 typed guard 원인으로 확인하고, 해제 후 wake1·SDK mutation 추가0을 검사한다. provider/ACK Busy는 자동 guard retry로 취급하지 않는다.

commit port는 PM이 승인한 UI 소유 construction-time trusted apply 경계다. 기본은 실제 Native update_module_props_with_callbacks에 위임한다. 시험은 성공 뒤 오류로 위장하지 않고 commit 전에 export를 인수해 오류1회를 돌려준다. 임시 staged owner/lease가 폐기돼 captured export resolve가 Stale이 되고 실제 commit 수가 증가하지 않음을 확인한다. 실패 뒤 old export 권한·실제 renderer preflight rollback/명시 retry는 기존 native_document_sdk.rs와 document_factory.rs의 회귀 결과를 매핑한다. 최종 원 async terminal의 ACK 상태가 재시도 전후 Acknowledged임을 확인하며 callback/source 정책을 복제하지 않는다.

전체 OS 시험은 각각 별도 프로세스이며 normalized callback 검증이다. 물리 keyboard/IME·시각13화면·전체 탭·native video/browser child·기본 player.exe 런처 인수로 확대하지 않는다.

OS fixture의 deadline은 소유 시험 프로세스만 exit124로 실패 종료한다. 이는 worker join·정상 Native teardown receipt가 아니며 성공으로 집계하지 않는다. 사용자 player.exe와 다른 앱에는 종료 명령을 보내지 않았다. partial mount의 OS 회복은 위 별도 시험으로 확인하며 driver attach 이후 failure/forced teardown의 모든 조합을 포괄한 것으로 주장하지 않는다. root 제품 등록·실제 player.exe 연결은 소비 gate 뒤 필수 다음 배정이다.
