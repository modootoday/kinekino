---
id: plan:kinekino/20261029113821
title: 제품 UI 연결4단계 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 UI 연결4단계 계획

2026-10-20 PM 실제 구현 배정. SDK 입력8 inventory B2A69A908163CC3282E36848A67FA25716FDD3221A2C1EFD14B68F64BFB11CAE를 실제 읽은 파일과 대조했다. main의 분기 및 desktop_mode staged guard는 SDK 후속 단계이며 변경하지 않는다. SDK8/Native316/Core/Cargo 및 CSS·기존 화면 시각 폴리시는 동결한다. 원칙은 Rust/SRP/계층 유지와 한 SDK/context/worker/actual scope/registry다.

## 구현 경계

1. `proposals/launch_selection.rs`: plain NativeProposalSelection의 checked_item_index를 사용한 BootstrapSelection 변환. full DocumentTextRef/정밀 revision/ID/SHA를 그대로 옮기며 repair Unselected=None, Selected=Some을 구별한다. 실제 membership 검증은 기존 project_initial에 둔다.
2. `correction_panel.rs`, `initial_panel.rs`, `document_factory.rs`: 초기 repair 선택과 live ReviewSession 선택을 동일한 공통 projection으로 맞춘다. Some은 전체 reference가 snapshot에 일치하는 위치를 명시적으로 찾고 기존 choose_document_text 정책으로 설치한다. None은 미선택이다. index0/첫 source/None fallback 금지. 인수 prepared raw/typed Arc와 ACK 순서는 유지한다.
3. `product_window.rs`: 제공받은 기존 client/context/ExecutionOwner와 plain 요청을 소비해 고정 product TSX inventory/entry, labels/budget/pins와 WindowStartupInput을 만든다. Cards/Highlights/Correction은 실제 product 경로를 사용하며 verification entry를 승격하지 않는다. loop/SDK/store를 만들지 않는다. visual asset/CSS는 신뢰한 조립 입력 경계로 두며 직렬화 요청이 선택할 수 없다.
4. `window_cleanup.rs`와 기존 startup/factory/bridge 최소 port: 초기 입력/prepare/ACK/mount/bridge/attach 실패의 원 typed 원인·남은 owner를 그대로 보존한다. terminal 수락/ACK는 명시 API로만 한다. SDK quiescence/actual join과 Native Closed/ForcedIncomplete/미확인을 서로 다른 결과로 반환한다. bounded 관측은 최대 횟수+deadline/양수 delay를 사용하고 예산 소진 후 owner를 버리지 않는다. 새 loop/worker fallback 및 반환 owner만으로 Closed 처리 금지.

## 확인한 공백

초기 correction projector는 index=None일 때 selected_document=None만 전달하고 factory의 새 ReviewSession도 미선택으로 시작한다. 따라서 initial Some 거부를 제거하는 것만으로는 prepared props/live props 일치가 보장되지 않는다. 새 선택 전달은 준비→실제 factory review에 같은 reference를 적용해야 한다.

기존 product correction TSX에는 sourceChoice/startManual props가 선언됐지만 실제 repair 입력/버튼이 없다. 최소 기능 연결이 필요하면 기존 공통 컴포넌트/props를 재사용해 repair controls를 노출하며 디자인/13화면 완료로 주장하지 않는다. 기존 document-repair TSX는 verification component에 의존하므로 production entry로 그대로 승격하지 않는다.

Native runner는 consumer 실패에서 원 오류를 보존해 loop를 종료하고 driver Drop을 거친다. 반환 runtime이 ForcedIncomplete이면 SDK cleanup을 마쳐도 정상 Native teardown receipt가 아니다. runtime.request_close가 forced 상태에서 host를 다시 호출하지 않을 수 있으므로 별도 UI-owned SDK cleanup 관측 port가 필요하다. Native/Core 수명 구현은 바꾸지 않는다.

## 검증·인계

### 기능 fixture와 제품 시각 인수 경계

2026-10-20 PM 전달 사용자 관측은 흰 바탕과 흩어진 요소다. 관측 창의 실행 ID/fixture/capture가 연결되지 않았으므로 특정 테스트 창으로 확정하지 않는다. 소스에서 확인한 `tools/player-ui-sdk-bridge/tests/native_pending_window_sdk.rs:813,819` 및 새 `product_window_sdk.rs`의 config는 `css: String::new()`, `assets: None`이다. 이 경로는 실제 SDK/DOM/수명주기 기능 검증이며 제품 스타일·Pretendard·레이아웃 시각 인수가 아니다.

제품 helper는 `ProductWindowVisuals`의 신뢰된 CSS/assets를 WindowStartupInput에 전달하며 `product_css()`는 기존 tokens/components/studio/subtitles CSS를 합친다. 현재 main 조립은 후속 SDK 단계이므로 실제 제품 실행에 고정 product_css와 검증된 폰트/이미지 bundle을 연결했다는 근거는 아직 없다. CSS가 빈 fixture, 제품 CSS/asset 연결 여부, 연결 후 렌더러의 속성/폰트 지원 여부를 각각 별도 검증한다. 렌더러 호환성 문제가 흰 화면의 원인이라는 판단은 미검증이다.

후속 필수 게이트는 실제 product 실행 입력 기록 → CSS/VerifiedAssets 해시 및 전달 확인 → 렌더러 지원 공백 측정 → 승인 전체 13화면 및 세부 요소 1:1 대조다. 현재 기능 테스트 성공과 headless/OS 종료 receipt를 디자인 완료로 승격하지 않는다. 추가 GUI 실행 없이 이번 단계는 소스 근거만 기록한다.

before/후보·원 실패·수리·SDK8/Native316 재hash를 `runtime/quality-review/product-native-window-helper-r1/`에 보존한다. 먼저 pure DTO/selected/unselected/위조 fullref/정밀 revision과 initial/live props·명시 ACK·실제 SDK headless 동일5명령 및 실패 owner/budget 소진 회귀를 검사한다. 다음 root on lib·canonical consumer 영향 회귀와 fmt/strict, 마지막 새 helper/outcome 연결 경계의 필요한 OS만 검증한다. 기존 OS9/전체CLI/시각13 결과를 새 입력으로 상속하지 않는다.

중앙 Rust runner/2CPU/BelowNormal/Job/offline/locked/-j1/testthreads1, 기존 C root·D consumer cache만 사용한다. 각 원exec exit·receipt terminal·소유 가계 종료를 확인하기 전 다음 Cargo를 시작하지 않는다. runner는 no-kill-on-close이며 필요한 회수는 정확 자기 가계만 대상으로 기록한다.

최종 API/미검증/보호 입력 차이·source pin·owned[]와 단일 heavy 반환을 PM/SDK에 인계한다. 이후 SDK가 bounded 파일 adapter→동일 client/owner 조립→main Native 단일 분기·staged guard 해제·실제 player.exe gate를 수행한다.

## UI4 인계 상태

구현·영향 검증 checkpoint를 마치고 변경21 source inventory `535EC6B64BF0737C3C19384620E4EBD856FE982DCD963A77610330FEFBC855A9`로 동결했다. 원 실패/수리/미확정 timeout과 최종 실제 통과 회차는 [검증 정본](../../../sots/verification/presentation/product-native-window-helper-2026-10-20.md), public API/선택/정리 계약은 [UI 정본](../../../sots/player/presentation/product-native-window-helper.md), 후속 조립 세부사항은 runtime/quality-review/product-native-window-helper-r1/handoff.md에 기록했다. PM에 heavy를 반환하며 이후 배정 전 새 개발/빌드/GUI를 시작하지 않는다. main/실제 product CSS·폰트·assets/시각13은 후속 필수 게이트다.
