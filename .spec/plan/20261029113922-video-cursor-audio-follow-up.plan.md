---
id: plan:kinekino/20261029113922
title: 영상 hover 커서 결함과 오디오 상태 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 영상 hover 커서 결함과 오디오 상태 후속

상태: 사용자 결함 확정·소스 읽기 대조, 새 GUI/커서 실동작·청취 검증0. 영상 hover/이동 시 커서가 사라진다는 보고를 정상 유휴숨김으로 종결하지 않는다. SDK의 실제 mpv launch/adapter 진단과 제품 GUI 정책을 분리해 수정한다. 현재 단일 heavy는 SDK 소유이며 UI는 등록 source 변경·추가 cargo/GUI를 실행하지 않는다.

## 읽은 현재 제품 source

application/control_visibility.rs는 전체화면·입력 없음·unpinned일 때3초 뒤 컨트롤을 숨기고, 입력/pinned/창 모드에서 표시한다. presentation/chrome.rs는 Windows pointer 이동, egui 입력, web_input을 조합하고 pause/popup/키보드·포인터 사용 등을 pinned로 전달한다. 이 코드는 컨트롤 표시 정책이며 OS/mpv 커서 복구 자체의 증거가 아니다.

infrastructure/window_activity.rs는 전역 좌표 변화와 부모 창 foreground·bounds를 대조한다. 현재 last 좌표는 창 바깥에서도 갱신되고 enter를 별도 이벤트로 노출하지 않는다. 자식 영상 표면 위 move/enter가 부모 입력과 다를 수 있는 경계는 미검증이다. 실제 OS 호출/자식 표면 커서 복원은 infrastructure 책임이며 presentation이 직접 플랫폼 API를 호출하지 않는다.

읽은 mpv/mod.rs와 network_plan.rs에는 --input-cursor=no가 있다. 이 인자만으로 커서 결함의 확정 원인을 단정하지 않는다. 현재 SDK 담당이 실제 launch/adapter 진단·수리를 소유한다. 기존 eframe GUI와 SDK 검증용 실제 mpv 창을 같은 실행으로 취급하지 않는다. Meari의 범용 cursor/style 제공과 제품의 재생 유휴숨김 정책도 구분한다.

## 제품 수정·최소 인수 범위

| 조건 | 요구되는 동작·남은 확인 |
|---|---|
| 영상 pointerenter | 커서를 즉시 복귀시키고 유휴 타이머를 갱신. 창 모드 단순 hover로 숨기지 않음 |
| 영상 pointermove | 이미 숨겨진 상태에서도 복귀. 자식 surface 입력/foreground/좌표에 실제 배선됐는지 확인 |
| 컨트롤 영역 진입·popup·seek drag | 커서와 조작부 표시 유지. 표시만 복구되고 cursor는 사라진 상태를 허용하지 않음 |
| 전체화면 재생 유휴 | 승인된 자동숨김 조건에만 적용. pause/오류/키보드 focus/a11y 조작·창 전환/복원과 별도 상태 관리 |
| 키보드/접근성 | 키보드 조작부 focus/탐색·접근성 action을 cursor 좌표 이동으로 위조하지 않음. 조작에 필요한 표시/focus는 유지 |
| 전체화면 해제·alt-tab/enter·source 이동 | 숨김 상태가 다음 창/세션에 남지 않게 인계. unrelated 창의 커서·프로세스 소유를 조작하지 않음 |

SDK 수정 후 root GUI 배선이 필요한 차이를 정확 인계받아 owner 범위에서 수리하고 중앙 슬롯에서 최소 enter/move/controls 복귀와 유휴/해제 표본을 실동작 확인한다. 가상 모니터 배정과 실행 프로세스1개·사용자 종료 규칙을 유지한다. 현재 mouse move 복귀 성공을 주장하지 않는다.

## 오디오 현재 관측과 미완료

추가 source 대조: legacy infrastructure/mpv/mod.rs의 launch 인자에 --mute=yes가 고정되어 있다. GUI volume80과 별개로 초기 mute가 강제되는 결함이며 SDK 담당이 수리한다. network_plan에는 이 고정 mute가 없고 CLI actual volume0은 명시 검증 설정이다. 기존 초기 unmute 전송 경로가 있다는 관측만으로 강제 mute 결함을 정상 처리된 것으로 종결하지 않는다.

현재 volume_control::show는 volume 값 하나와 changed bool만 다루고, 아이콘도 volume<=0인지로 고른다. 버튼은 popup을 열며 명시 mute toggle/state가 없다. transport는 값 변경 시 PlaybackControl::SetVolume만 보내고 mpv/control::wire는 set_property volume만 만든다. 이 경로에 명시 mute=false가 없어 강제 mute 해제를 보장하지 않는다. PlaybackControl은 Play/Pause/Seek/SetVolume뿐이며 SetMute 공통 계약은 현재 없다. presentation에서 별도 GUI 전용 mute 정책/임의 raw IPC 우회를 추가하지 않고 SDK 담당과 정확 확장 범위를 협의한다.

NativePlayback·PlaybackStatus와 IPC observe 목록/events의 읽은 현재 source에는 muted 관측 필드/구독이 없다. WebPlayback.muted는 웹 상태이며 native mute 관측으로 대체할 수 없다. PlayerApp::update_native의 mute=false는 native 시작 조건 충족 시 전송하는 legacy 경로이고 해당 mute 속성의 실제 관측 ACK는 없다. 명령 접수와 실제 mute 해제·청취를 구별한다. scoped UI source에서 persistent save/storage hook은 확인되지 않았다. 이는 프로젝트 전체에 저장이 없다는 증명이 아니라 실제 volume/mute 저장·복원 계약을 추가 대조해야 하는 gap이다.

읽은 PlayerApp 초기 volume은80이며 snapshot의 유효한 실제 volume으로 갱신한다. native 시작 시 mute=false 전송과 실패 표시 경로가 있다. 이것은 실제 청취 성공이 아니다. SDK actual 검증의 volume0은 해당 검증 설정이며 제품 기본 무음 정책이 아니다. 현재 scoped source 읽기에서 지속 volume/mute 설정 저장 계약은 확인되지 않았고, 별도 mute 상태와 저장·초기 복원은 기존 settings 정본/SDK 계약에 대조할 후속이다. 현재 UI의 숫자80을 새 다국어/settings 정책으로 승격하지 않는다.

이후 volume과 mute를 구분해 명시 변경·실제 관측·저장 여부·재시작 초기값을 하나의 설정 책임에 연결한다. 음량0·mute·오디오 트랙/디바이스 오류·테스트 무음을 혼동하지 않는다. 새 정책/SDK mutation을 중복 구현하기 전에 기존 공통 API에서 빠진 정확 계약을 SDK와 조율한다. source/text 관측과 실제 청취는 끝까지 구별한다.
