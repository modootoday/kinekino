---
id: plan:kinekino/20261029113756
title: Capture 선택 목록 NodeSlot lifecycle 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture 선택 목록 NodeSlot lifecycle 후속

[현재 기능 근거](../../../sots/verification/presentation/capture-product-caller-surface-2026-10-21.md)의 고유116 통과와 구별한다. 후속 Rust ignored 시험4를 별도 source-only 후보로 작성했으며 컴파일·빌드·실행0이다. 공통 라이브러리나 생산 UI 정책을 변경할 필요가 있는지 실제 실패 근거로 판단한다.

## 목표와 책임

UI 세션은 실제 제품 inventory/ModuleProgram/InteractiveDocument/ProductFrameBlueprint를 재사용해 선택 목록의 mount·교체·제거를 시험한다. SDK 세션의 정책이나 worker를 fixture로 대체하지 않는다. 정상 원 host를 유지하고 기존 callback export와 update_module_props_with_callbacks 트랜잭션을 사용한다.

## 다음 시험 단위

1. 기본 Empty→선택된 Owned→Empty에서 radio group/items와 host-owned callback target의 존재·제거를 확인한다. false에서 factory 호출0도 helper 근거와 별도로 확인한다.
2. 같은 선택으로 반복 true를 투영할 때 고정 key의 component instance/state identity를 확인한다. 데이터 변경과 component remount를 구분한다.
3. 선택 항목 focus 후 제거 시 이전 radio focus/a11y 노드가 남지 않는지 실제 document에서 확인한다. OS 창·전체화면을 열지 않는다.
4. Rail callback을 실제 항목에서 발생시켜 drain한 뒤 Empty로 commit하고 같은 callback을 전달하면 거절되는지 확인한다. 재표시 뒤에도 이전 callback은 거절하고 새 callback만 수락해야 한다.
5. visible true에서 누락/잘못된 props, foreign slot, callback preflight fault를 주입해 실패한 commit이 원 program/export/SDK attachment와 마지막 성공 frame을 보존하는지 확인한다.
6. 제품 편집 후 원 Viewing으로 복귀할 수 있는 경로가 capture route에 제공되는지 먼저 확인한다. 미구현 경로를 기존 generic Cards 복귀 근거로 대체하지 않는다. 필요한 제품 기능은 별도 최소 후보로 책임·UX 동작을 정의한다.

새 시험은 Rust로만 작성하며 일반 helper 검증을 그대로 반복하지 않는다. 기존 제품 fixture에 결합할 경우 fixture 책임을 별도 Rust 하위 모듈로 분리하고 원 fixture/입력 전후 SHA를 보존한다. 실제 실행은 PM의 다음 슬롯 배정 뒤 fixed BIN과 등록 목록으로 집계한다.

## 완료 경계

exact 입력·host identity·callback 퇴역·last-good·focus/a11y의 actual 근거와 원 close/EOF를 수집해야 기능 완료로 표시한다. GUI 시각 일치나 사용자 품질 승인은 별도다.

## 작성 후보와 인계

10/23 추가 읽기에서 r1의 미수출 binding 시험 이름에 하이픈이 있어 이름 검증에서 먼저 거절되는 P2를 발견했다. 기존 CallbackReference::validate와 CallbackBindings::resolve의 검증 순서에 따른 소스상 반례이며 실제 실행 오류로 보고하지 않는다. 원 r1을 보존한 final-source2-r2 `124DAC0759FE62050022627D47F84DFD45ED5AC5409AA9AD7D8B0BB16A7A4FC7`는 해당 문자열만 유효한 미수출 이름 unexportedCaptureChoice로 교체한다. 기존 not-exported 오류 단언·last-good·원 SDK 소유권 검증은 유지한다. PM 재감사 전 실행 입력으로 자동 승격하지 않으며 컴파일·실행0이다.

2026-10-23 재개 시 원 source2는 보존했다. 별도 final-source2-r1 `17956614E9ACB01C0DD2996BBF6FDF0856CA95F58C04DA8483995572862F342F`는 성공 save/consume 뒤 draft revision을 초기값.checked_add(1)로 확인하고 실제 저장 front text를 단언한다. PM 독립 읽기에서 원 P2 반례가 해소됐으나 컴파일·실행0이다. 제품 복귀는 [별도 후보](capture-viewing-return.md)이며 이 helper의 강제 투영으로 완료를 대신하지 않는다.

기준 root `e9aee3ec4359e33b2c46f3a3bf219f5281e4dc91`, library `0b9ea03`, lock E469다. stage는 `runtime/development/ui-capture-choice-lifecycle-r1`이다.

- source2 SHA256: `33B826B799EDB8CB89B0434A54A4FF525B91F0C3AB08FAC8F54E7399D5C7F002`
- before2 SHA256: `783CFBC68E3C819D9230F67D6C7128861094502D06F29F7A444512BFF97E28D5`
- 기존 product_surface.rs에는 하위 시험 모듈 선언1줄만 추가한다. 원 bytes를 before 사본에 보존한다.
- 새 product_surface/slot_lifecycle.rs는 기존 with_rig/bind_product/실제 프로젝트·SDK/IPC·원 ViewerHost·RadioGroup·ExecutionOwner·원 close 보존 fixture를 재사용한다.
- authored-tests4.json은 작성 이름 목록이며 actual 등록/통과 목록이 아니다. Rust leaf format만 수행했다. root 적용·Cargo·S/T 변경·OS probe·GUI 실행은 하지 않았다.

| 작성 시험 | 검증하려는 내용 |
|---|---|
| empty_owned_empty_removes_actual_radio_focus_accessibility_and_callback_targets | 실제 Empty→Owned→Empty, 동일 host, element origin/콜백 target 제거, 원 focus와 AccessKit 노드 제거, SDK 무변경 |
| repeated_visible_projection_preserves_actual_rail_instance_and_radio_dom_identity | 반복 true에서 actual Rail element origin의 InstanceHandle과 radio DOM ID 유지 |
| drained_rail_callback_stays_retired_after_removal_and_remount_while_new_item_can_edit | 항목에서 발생시켜 drain한 콜백을 제거/재표시 뒤 원 host에서 거절하고 새 항목 콜백은 실제 SDK 저장·명시 소비 성공 |
| failed_slot_and_callback_preflight_keep_last_good_frame_export_and_sdk_owner | 누락/잘못된 값/미수출 binding 실패가 원 stamp·component·instance·DOM·SDK attachment와 이전 read callback을 보존 |

reproject helper는 시험 전용 입력으로 기존 workspace_props→update_module_props_with_callbacks 원 트랜잭션을 호출한다. 제품에 선택 목록 표시 토글 API를 추가하지 않는다. InstanceHandle은 slot 호출 origin이 아니라 실제 Rail element origin으로 조회한다.

Capture route의 Viewing 복귀는 현재 request_viewing_return이 cards.active를 요구하여 이 surface의 완료 경로가 아니다. 이번 후보는 해당 동작을 새로 만들거나 우회하지 않는다. 실제 실행에서 발견할 수 있는 lifecycle 실패는 원 입력/BIN/첫 오류를 보존한 별도 수리로 처리한다. Media sole-heavy 배정 중이므로 새 실행 승인을 자동으로 추정하지 않는다.
