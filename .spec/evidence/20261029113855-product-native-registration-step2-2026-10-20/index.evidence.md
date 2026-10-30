---
id: evidence:kinekino/20261029113855
title: 제품 Native 등록2단계 — canonical UI façade
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 Native 등록2단계 — canonical UI façade

2026-10-20 등록2단계 최종 기록이다. [등록 계획](../../../plans/player/presentation/product-native-registration-step2.md)과 [SDK 등록1단계](../application/product-native-registration-step1-2026-10-20.md)의 후속이며, 이전 [UI OS 소비 gate](native-pending-window-sdk-2026-10-20.md)와 입력/시험 수를 구별한다. main/launch 연결·시각13화면 인수가 아니다.

## 입력·변경 경계

SDK root5 inventory `36914414b13f3c05d0b1ee32f3983203dee533bb85071cf9d7db3201b7545a8b`를 실제 재hash해 인수했다. UI19 기준 `26335921b555b8732b3609f9d22a7ad7dda93b9f5e3d4998111381818774f35c` 중 승인된 proposals/mod 선언과 두 integration import 외 구현 본문은 변경하지 않는다. root Cargo/lock/lib/infrastructure/assembly/main/launch와 Native/Core/CSS/SEG/CLIP는 이 단계에서 동결한다.

presentation 부모·web_ui_adapter 부모·proposals의 네 Native 모듈을 Windows/native-proposal-window에 등록했다. consumer feature는 root feature를 전달한다. on의 proposal_view 및 관련 integration module은 root 정의를 re-export하며 off의 기존 path source와 동시에 컴파일하지 않는다. root→tool 의존은 추가하지 않는다.

before10파일, 최초 로그·receipt·consumer resolver diff는 `runtime/quality-review/product-native-registration-step2-r1/`에 보존한다. root는 기존 C: 공통 target, consumer는 기존 sdk-product-parent-consumer target을 사용하며 단일 runner/affinity3/BelowNormal Job/offline/locked/-j1/RUST_TEST_THREADS1을 유지한다.

## 실제 관측

| 회차 | 결과 | 범위 |
|---|---|---|
| on-compile-first | 657ms/exit101 | root의 optional Native 간선이 consumer lock에 없어 locked 갱신 거부. 소스 오류나 type 성공이 아님 |
| resolve | offline workspace resolution1회/exit0 | package 추가0·기존 package identity 이동0. diff는 root kinekino-player dependency에 meari-native 한 줄만 추가 |
| on-compile-r2 | 433825ms/exit0 | canonical root UI를 사용하는 전체 consumer 시험 실행 파일 생성. 시험 실행 통과와 구별 |
| on-consumer-first | 12534ms/exit101 | document_factory19pass/1fail. 새8entry 시험이 같은 field 연속 입력의 기존 coalescing을 독립9항목으로 잘못 가정. 원source/log 보존, 제품 정책 변경0 |
| on-capacity-r2 | 23391ms/exit0,2pass | 실제 두 field를 번갈아 enqueue해8entry 경계와 registry8을 확인 |
| on-consumer-r2 | 46280ms/exit0,55pass/0fail/9ignored | 수정된8entry 시험에 SDK revision 불변도 확인. 원래83 의미와 신규2의 공개 행동 회귀를 canonical 경로에서 소비. OS9은 미실행 |
| root-on-lib-first | 277786ms/exit0 | 제품 lib347pass/5ignored, workspace lib 합계395pass/0fail/5ignored. root canonical UI 단위31 이름을 별도 보존 |
| root-off-test-first | 331397ms/exit0,426pass/0fail/23ignored | feature-off 제품·CLI·workspace 기능 호환. 아래 단일 heavy 위반으로 경과 시간을 자원 준수/성능 근거로 사용하지 않음 |
| off-consumer-first | 중단·검증 미채택 | 원receipt terminal=false를 보존. 별도 중단·가계 회수 관측으로 연결 |
| off-consumer-r2 | 403442ms/exit0,93pass/0fail/9ignored | root 원exec exit/terminal/가계[] 확인 뒤 단일 실행으로 재개. 기존91+공개 행동 회귀2 |
| root-on-strict-first | 41336ms/exit0 | all-targets/native-proposal-window/-D warnings. 기존 경로와 새 root UI를 함께 검사 |
| on/off-consumer-strict-first | 40074/33864ms,양쪽exit0 | on canonical import와 off 독립 path 조건부 컴파일을 각각 검사 |
| root/consumer-fmt-first | 7304/3297ms,양쪽exit0 | cargo fmt --all -- --check |
| canonical OS9 | 각각exact1pass/exit0 | 아래 별도 실제 Native/SDK 검사. 기본 ignored9과 합산해 신규 기능 수로 주장하지 않음 |

## private 단언의 공개 행동 대체

기존 bootstrap의 registry_pending=8/entries=8/text_bytes=8192 private field 단언은 feature-off에 유지한다. on에서 getter를 새로 공개하지 않는다. 공개 행동의 정확 매핑은 다음과 같으며 실행 전 통과로 표시하지 않는다.

- registry8 → canonical_registry_eight_pending_slots_resume_without_duplicate_save. 기존 registry1 회귀를 같은 helper로 보존한다.
- entries8 → canonical_eight_entry_budget_preserves_ninth_callback_until_resume.
- text8192 → 기존 native_host_bridge_aggregate_text_bytes_preserve_suffix_until_queue_drain 및 single oversized 거절.

feature-on에서 없어지는 중복 initial/controller 단위 시험은 root on lib test에서 실행한다. consumer 통과 수 감소를 기능 삭제나 새 성공 수로 해석하지 않고 실제 SDK3task·OS 종료/복구는 canonical 경로에서 별도로 검사한다.

off 예상93은 이전91+새 공개 행동2다. canonical consumer55와의 차이38은 독립 path proposal_view 단위24와 두 integration target에 중복 포함됐던 initial7*2다. root에서는24+고유initial7을 한 번 실행해야 한다. 이 수를 전체 신규 기능 수로 합산하지 않고 root 실제 시험 이름/결과와 대조한다.

`root-on-canonical-test-names.txt`에는 실제 root 이름의31pass를 보존했다. `presentation::web_ui_adapter::proposals::initial_panel::tests::{ack_transport_failure_retains_preparation_and_retries_only_ack_before_mount,canonical_failure_ack_retry_retains_same_raw_and_typed_acceptance,canonical_success_with_wrong_selection_cannot_be_acknowledged_as_failure,late_canonical_failure_is_explicitly_accepted_before_worker_close,queued_cancel_never_executes_initial_inspect_and_can_be_acknowledged,repeated_module_and_parent_failure_reuses_same_preparation_without_reinspect,unacknowledged_success_keeps_same_startup_and_never_mounts}`는 각각 한 번 실행·통과했다. 나머지24의 correction/pending/controller/projection/queue 의미도 같은 목록으로 연결한다.

## 단일 heavy 위반과 회수

root-off 원session65037이 계속 active인데 다음 consumer를 시작한 담당 오류가 있었다. 새 off-consumer-first runner49892/rustup75968/cargo85640과 확인된 rustc 가계만 회수했고 사용자 앱·다른 세션에 종료 명령을 보내지 않았다. 중앙 Budget Job은 no-kill-on-close이므로 runner 종료만으로 자식 회수를 가정하지 않았다. 원 PID·명령·후속[]·중단 stdout/stderr/receipt는 `single-heavy-incident.md`, `overlap-final-owned.json`, `root-off-terminal-owned.json`에 보존했다. 원root는 중단/소스 변경 없이426pass로 끝났으며 원exec exit0·원receipt terminaltrue·가계[]를 모두 확인했다.

중첩 consumer는 별도 target의 컴파일 단계였고 source나 root target을 쓰지 않았다. 기능 결과 오염을 관측하지 않았지만 단일 자원 규칙은 위반했으며 시간·성능 근거로 사용하지 않는다. 강제 중단된 rawreceipt를 exit0/terminaltrue로 덮어쓰지 않는다. 이후 모든 단계는 앞 단계의 원exec/receipt/가계 종료를 확인한 뒤 진행한다.

## 남은 검증과 후속 범위

이번 입력에서 canonical OS9를 실제 재실행했다. correction23463/cards5894/highlights6773/doc0-prepared2609/doc0-error2052/closed-notifier4683/guard3696/partial4908/commit3472ms로 각각exact1pass/exit0이다. 각각 원receipt/본문·종료 가계[].owned.json을 보존했다. UI 입력은 normalized callbacks이며 physical keyboard/IME·시각13화면·OS compositor 표시 ACK나 실제 player.exe 런처 인수가 아니다.

실제 원본 TSX/actual scope·같은 SDK/client/ExecutionOwner·initial prepare/ACK·Native Ready/checked frame·SDK5명령과 의미 보존·typed close/타이머 join·원 raw/diagnostic·guard wake1·부분 same preparation/worker 복구·commit 전 거절/명시 retry를 canonical 정의로 확인했다. 내부 join/ACK 호출 계수의 직접 관측 한계와 old export rollback의 기존 Core/Native 근거 매핑은 앞선 UI OS 검증 문서와 같으며 소스 중복을 만들어 우회하지 않았다.

최종 등록11파일 inventory `runtime/quality-review/product-native-registration-step2-r1/final-registration-pins.json` SHA256은 **215b94167313293829674568b504154ec4349a27a67de9e92f446d9df4d73332**다. before10/source-final11/registration-source.diff·consumer-lock.diff·최초 실패/수리/중단 원로그를 보존했다. 최종11 실제hash 차이0, SDK 등록5·SDK wrapper 소스3·Native316·generic5·허용 mod/두 integration import를 제외한 UI16 구현 본문 차이0, 최종 소유 프로세스 관측 `final-owned.json=[]`다. wrapper 이전5 inventory의 consumer Cargo/lock2는 이번 승인 변경이므로 전체22E 핀이 불변이라고 주장하지 않는다.

단일 heavy 슬롯을 PM에 반환하고 현재11파일을 동결한다. production Correction entry는 verification 경로와 분리해 고정 product source inventory로 주입해야 하며 다음 launch helper/main 배정으로 인계한다. application plain DTO→UI selection 한 번 변환·동일 client/owner·실패 반환 owner 보존을 유지한다. root main/launch·실제 player.exe 실행은 아직 변경/인수하지 않았다.
