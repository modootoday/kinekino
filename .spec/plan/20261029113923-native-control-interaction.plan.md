---
id: plan:kinekino/20261029113923
title: 현재 native-default 컨트롤 상호작용 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 native-default 컨트롤 상호작용 연결

2026-10-21 PM 후속 source-only 배정. root9ffd855의 실제 main은 presentation::run_with_native_playback → eframe PlayerApp을 기본으로 사용하며 Meari Viewer main 전환은 아직 별도 단계다. 이 작업을 Meari 시각/실행 완료로 표시하지 않는다. 기존13화면 목표와 Native 전환 계획을 유지한다.

## 수정 전 관측과 exact4

기존 chrome.rs는 공통 application::ControlVisibility(전체화면3초 유휴)를 소비하고 pause/error/episode/address/scrub/popup/키보드/drag는 pinned로 묶는다. 그러나 **표시된 transport/toolbar 위에서 포인터를 멈춘 상태**를 포함하지 않아, 조작 중인 영역이3초 뒤 사라질 수 있다. ctx.egui_is_using_pointer는 단순 stationary hover의 대체가 아니다. studio 표시와 focus 복귀도 명시 연결한다.

정확 source4는 `src/presentation/chrome.rs`, `src/presentation/player_app.rs`, 신규 `src/presentation/chrome/interaction.rs`, 신규 `src/presentation/chrome/interaction/tests.rs`다. 시작 전에 독립 overlay/before4/기존 root 및 index를 고정한다. source8의 assembly 경로와 교집합0이며 원source8 r1~r3를 변경·합성하지 않는다. application 정책·SDK·Meari·플랫폼 API를 새로 구현하지 않는다.

## 공통 계약 소비

interaction 모듈은 직전 실제 그려진 toolbar/transport rect와 현재 egui pointer/focus를 대응하는 presentation 자료만 소유한다. focused 상태의 조작부 hover는 기존 pinned 입력으로 보낸다. focus가 돌아오면 기존 active 입력으로 보내어 원3초 시간을 갱신한다. 영역은 매 observation 후 비워 다음 render가 다시 등록하며 숨겨진/다른 화면의 옛 rect를 계속 고정하지 않는다. popup·episode·pause·scrub·a11y/키보드 입력의 기존 pinned 조건을 보존하고 studio 표시도 포함한다. 전체화면 영상 유휴는 기존 application 시간 정책을 그대로 사용한다.

egui0.36.2 로컬 소스에서 Panel::show의 InnerResponse/response.rect 및 InputState.focused/PointerState.hover_pos 서명을 확인했다. presentation에서 OS API를 호출하지 않으며 WindowActivity::pointer_moved 기존 adapter를 계속 소비한다.

추가 소스 검토: 고정 egui0.36.2 `Context::egui_is_using_pointer`는 단순 hover에서 false이며, `Context::egui_wants_keyboard_input`의 실제 본문은 `memory.focused().is_some()`이다. 따라서 source4는 stationary hover에 필요한 연결을 추가하되 기존 button/text focus의 pinned 처리도 그대로 소비한다. 버튼 focus를 놓친다는 추정으로 별도 키보드 정책을 추가하지 않는다. 이 서명/본문 확인은 실제 Tab/접근성 동작 검증이 아니다. 최종 Meari consumer의 focus 대응은 원 공통 입력 계약을 따라 별도 실제 host 자료로 검증한다.

## 커서 범위·검증

PM 독립 감사는 source4 actual bytes/SHA, 고정 egui 서명·렌더/관측 순서·공통 정책 보존을 확인했고 추가 확정 P1/P2가 없었다. 실행 인수는 아니다. 별도 [union12 실제 입력](../../../../runtime/coordination/ui-viewer-chrome-union12-r1/HANDOFF.md)에 Viewer8+controls4/교집합0/기존before7·신규5를 고정했다. source12 SHA83B9084E...C543EFD이며 직접fmt12 check0, 예상CPU74/ignored7 및 별도OS2는 아직 실행0이다.

작성 인계: source4 `8C1F48851802EF674418E1C9EEE33F36CC5609D4FDDE93469F8AEAA99DB0E07C`와 before4, 작성 순수4 및 직접fmt4 check0은 [source 인계](../../../../runtime/coordination/ui-chrome-interaction-r1/source-r1/HANDOFF.md)에 고정했다. root 기존2 SHA/index 차이0/source8 교집합0이다. 컴파일/회귀/OS 실행은 아직0이다.

현재 mpv pointer_policy.rs는 `--input-cursor=yes`, `--cursor-autohide=no`다. 이전 문서의 no 인자를 현재 사실로 사용하지 않는다. 이것은 SDK 수정의 소스 관측이며 실제 hover 커서 복귀 증거는 아니다. 기존 WindowActivity API는 pointer_moved()->bool만 제공하며 native child 커서 표시/숨김 명령은 없다. PlaybackControl에도 cursor 동작이 없다. egui 부모 커서 변경만으로 mpv child 전체의 복귀/유휴 숨김을 완료했다고 주장하지 않는다. 이 추가 infrastructure/host 경계는 실제 서명 근거와 함께 PM·SDK와 조율한다.

SDK 읽기 조율 결과: 현재 SDK 편집 배정은 ready/CLI 안내/offline replay이며 포인터 실행 수리는 포함하지 않는다. cursor를 PlaybackControl/NativeControlPort에 추가하면 재생 정책과 window 입력 수명이 섞이므로 이 방향을 채택하지 않는다. 후속 제안은 기존 공통 visibility 결정에서 application의 좁은 window-pointer 포트로 전달하고, 원 surface/owner에 묶인 infrastructure window/child adapter가 표시/숨김·종료 복원을 맡는 구조다. 실제 parent↔child 이동/focus/close/실패 복원 관측이 필요하며 전역 cursor 상태 변경은 도입하지 않는다. 이는 **제안이며 아직 새 포트·adapter 구현/편집 배정/실행 인수가 아니다**. source4와 별도로 PM에 범위를 인계한다.

순수 회귀는 공통 ControlVisibility를 실제로 소비해 stationary controls 유지→영역 이탈 후 원 유휴 숨김, 오래된 rect 제거, focus 복귀 재표시, inactive window의 stale hover 거절을 확인한다. 직접fmt 뒤 before/after manifest와 코드 읽기를 인계한다. Cargo/GUI/커서 실동작·청취/전체화면/VM·DPI 실행0이며 슬롯 이후 실제 영상 enter/move·panel/menu·유휴/복귀 표본으로 검증해야 한다. 시안 확대1:1/시각 완료는 별도 미완료다.
