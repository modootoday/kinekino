---
id: plan:kinekino/20261029113913
title: 원 Inspector의 stable mixed-slot 생산자 인수 요구
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 Inspector의 stable mixed-slot 생산자 인수 요구

2026-10-20 UI source는 동결 상태다. Meari 담당이 검증0589 기반 별도 worktree에서 공통 수리를 시작했음을 확인했다. 제품 caller나 원Frame/Dock에 임의 key/래퍼를 넣어 맞추지 않는다. 실행 가능한 기존 consumer 재현은 `studio_frame_sdk`의 `inspector_card_same_sdk_controlled_dirty_identity_and_join` exact1이며 실제 SDK를 사용하는 경로다.

## 관측된 최소 경계

원Frame의 aw-studio-layout은 positional workspace div와 owned InspectorDock slot을 함께 갖는다. 원 Dock의 DialogContent는 positional header와 owned editor children slot을 함께 갖는다. 초기 mount는 성공하고 첫 명시 Boolean open 승인 때 같은 root·owner·key/reference로 다시 발급한 slot props가 Core 검증을 통과한다. Native retained_shape가 두 parent의 [false,true] keyed flags를 보고 갱신을 거부한다. 실제 parent path/span과 원 typed 로그는 consumer-r5.stdout/stderr.log에 있다.

필요 입력은 같은 ModuleProgram graph/factory·same document.node_slot, 고정 Dock/editor key, 명시 controlled Boolean과 현재 export다. invalid lease/다른 registry/JSON 권한이나 임의 HTML이 필요하지 않다. 생산자 단위 fixture에서는 SDK command를 복제할 필요 없이 같은 named child slot에 Input을 넣어 원Frame/Dock의 siblings 조합을 보존할 수 있다. 다만 제품 통합 인수는 원 실제 SDK consumer로 다시 수행한다.

## 보존과 거부 기준

- 동일 parent·positional index·tag/shape의 고정 형제와 동일 owned key/reference의 배치에서 controlled open/hidden/modal·표시 props 갱신을 허용한다.
- root/child instance identity, input NodeId/useId·dirty text·focus, callback scope/export 교체·실패 rollback을 보존한다. 새 props 실패 때 old export가 살아 있고 성공 뒤 stale handle이 거부되는 기존 경계를 유지한다.
- 고정 sibling 이동/삽입/삭제, 중복 key, 같은 placement의 다른 key/reference, parent 이동, foreign/retired lease와 callback authority 변경은 안정된 mixed profile의 지원 범위를 넘으면 정확히 거부한다. 임의 list migration 전체를 허용하지 않는다.
- 기존 all-keyed 목록의 reorder/명시 key 교체와 P1a all-keyed workspace 교체 회귀는 별도로 보존한다. mixed profile의 제한을 기존 지원 범위 전체의 금지로 확장하지 않는다.
- Native preflight만 완화하고 하위 patch가 positional/keyed 경로를 다르게 매칭하는 상태를 남기지 않는다. Core emission/Native patch/Web 대응에서 slot identity의 의미가 같아야 하며 Web 실행 검증은 실제 수행 범위를 구분한다.

## 제품 재개 순서

생산자 고정 commit/변경 inventory/새 회귀/strict·입력 핀을 PM이 인수한 뒤, 현재 미인수 UI9와 old0589 source evidence를 보존하면서 별도 명시 overlay revision으로 전환한다. 생산자 수리와 제품 Caller 수리를 한 과거 revision으로 덮어쓰지 않는다. UI 단위3·새 Inspector4+기존frame5·on/off strict/fmt의 남은 gate는 중앙 슬롯을 받은 뒤 같은 consumer에서 마감한다. 전체13/영상/native child·OS input·사람 품질 승인은 별도다.
