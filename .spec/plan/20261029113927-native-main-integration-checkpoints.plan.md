---
id: plan:kinekino/20261029113927
title: 기본 Native Viewer 후속 체크포인트
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 Native Viewer 후속 체크포인트

2026-10-21 PM 조율 계획. 읽기 연구 제안이며 구현·실행 인수가 아니다. 현재 권한은 [실행 배정](../execution-leases.md), 전체 완료 기준은 [통합 게이트](../full-product-integration-gates.md)를 따른다. CLI 기능 검증 우선순위를 유지한다.

## 관측과 선행 조건

초기 연구(root0ecfc62/UI host7 r6)에서는 ViewerController/ViewerOwners/cleanup의 NoChild 구체형과 실제 Viewing video 선언이 후속 대상이었다. 이는 아래 인수 이전의 이력이다. root9ffd855의 현재 source에서는 typed generic carrier와 NoChild alias, public NoMedia opt-in 준비가 구현되어 있고, 실제 단일 semantic video graph/V1 검사를 인수했다. [V1 근거](../../../sots/verification/presentation/native-viewing-video-v1-2026-10-21.md)와 [V2b CPU 근거](../../../sots/verification/presentation/native-no-media-v2b-2026-10-21.md)를 따른다.

현재 main은 presentation::run_with_native_playback의 eframe 경로를 기본으로 사용한다. NoMedia Viewer main 연결은 미구현이며, [실제 Viewer source8](native-no-media-v2-end-to-end.md)은 별도 overlay source-only 인수/직접fmt 단계다. 실제 공개 prepare/run·proof·Completion·child-first 종료의 OS 시험은 아직 미실행이다. 기존 CPU/별도 ownwindow 근거를 Viewer 실제 실행으로 승계하지 않는다.

host7 공동검증 성공만으로 생산 opt-in 설치·실영상·기본 Native 전환을 완료 처리하지 않는다. 사용자 승인 디자인의 공통 토큰/용어/컴포넌트 일관성을 유지한다.

## 단계와 인수

현재 main/공급 SDK·LocalHost/NativeProposal와 일반 Viewer의 차이, caller 후보6·caption/route/실제 media 미구현은 [실제 caller 경계 조사](native-main-caller-boundaries.md)에 분리했다. 이 조사는 source12 뒤의 생산 편집 배정이나 기본값 전환 승인이 아니다.

| 단계 | 담당 작업 | 필요한 증거 |
|---|---|---|
| V1 문서 | Viewing graph에 단일 비활성 semantic video 자리와 typed ID를 선언하고 graph/style 계약을 함께 정의 | CPU 문서 mount·정확 ID/단일성·원 source/definition/document stamp 확인 |
| V2 opt-in 조립 | 기존 NoChild 기본 경로를 보존하고 NoMedia 전용 launch/controller/receiver·typed cleanup carrier를 연결 | 같은 원 preparation/registrar/children·실패 owner 보존·callback/ACK/commit·child-first 정리 회귀 |
| V3 실제 OS | CLI 게이트 이후 별도 중앙 슬롯에서 명시 opt-in 한 번을 검증 | 실제 frame callback의 checked proof 소비·owned 결과·중복/Busy/expiry·close/remaining0. CPU 성공을 HWND/렌더 증거로 승계하지 않음 |
| V4 main 연결 | 검증된 launch 선택 분기를 얇게 연결한 뒤 별도 핀에서 기본 경로 전환 | 원 SDK/LocalHost/ExecutionOwner 재사용·실제 종료·기본 실행 동작. 전체 디자인/미디어/DPI 완료와 구분 |

## 권한·수명 불변

request.origin은 mount 완료한 같은 NativeDocumentHandle.stamp에서 얻고 semantic video는 같은 graph의 typed 카탈로그에서 선택한다. 임의 selector/없는 node/브라우저 메시지로 만들지 않는다. request ID/deadline/frame budget은 한 시도에서 고정하며 NoMedia가 영상 재생·visible parent 권한을 발급하지 않는다.

receiver와 consumer는 같은 request에서 한 번 생성·설치하고 실제 startup preparation의 document/registrar/children에 bind한다. 실패는 원 document/host/preparation/consumer/native owner를 보존한다. 새 loop·SDK worker·LocalHost·등록 factory·cleanup driver를 복제하지 않는다. NoChild alias만 바꿔 기존 owner/RuntimeMismatch/Rc 동일성 검사를 우회하지 않는다.

위 단계는 편집 배정이 아니다. source 경로와 담당을 단계별로 고정한 뒤 구현하며 현재 공동검증12 및 CLI 조립6 소스에는 섞지 않는다. VM/DPI·수동 시각 검토·새 장시간 시험은 기존 보류를 유지한다.
