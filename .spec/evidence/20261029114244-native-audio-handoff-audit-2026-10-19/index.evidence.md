---
id: evidence:kinekino/20261029114244
title: Native audio handoff 소스 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native audio handoff 소스 감사

## 관측 사실

사용자의 무음 관측을 실제 청취 성공으로 해석하지 않는다. SDK 실제 CLI gate는 명시 volume0으로 실행하며 owned network_plan에는 고정 mute가 없다. GUI state의 초기 volume은80이지만 legacy `infrastructure/mpv/mod.rs::spawn`은 고정 `--mute=yes`로 시작한다. 이 차이는 단순한 사용자 기본 설정의 차이라고 결론 내릴 수 없다.

`presentation/transport.rs::start_native`는 웹 재생을 native 준비 때까지 유지한다. `player_app.rs::update_native`는 connected·non-idle·error 없음·time>0·video width·예상 전체 duration 오차2초 미만의 `application/native_playback::matches_video` 조건을 만족한 뒤 browser.pause_media, play/pause 제출, raw mute=false를 요청하고 웹을 숨긴다. 이 때문에 초기 mute는 중복 audio 방지 의도가 있는 임시 handoff다. 고정 mute만 제거하면 웹/native 동시 audio 회귀가 생길 수 있다.

그러나 현재 구현은 native_started=true를 요청 전에 설정한다. browser.pause_media 결과를 버리고, mute=false는 큐 전송 오류만 확인한다. NativePlayback·PlaybackStatus·IPC property 목록/events에는 muted 관측이 없으므로 실제 음소거 해제를 확인할 수 없다. 전체 duration 조건이 충족되지 않으면 초기 mute가 유지되며 해당 상태를 명시 audio handoff 상태로 노출하지 않는다. 이 소스 감사는 사용자 무음의 단일 원인을 입증하지 않지만 영구 무음에 머물 수 있는 미관측 경계를 드러낸다.

현재 PlaybackControl 공통 계약은 Play/Pause/Seek/SetVolume이며 SetMute가 없다. SetVolume wire는 volume만 변경한다. volume UI는 숫자와 popup으로 구성되고 별도 mute 상태/toggle이 없으며 WebPlayback.muted는 웹 관측이다. GUI volume 지속 저장 및 stored mute 설정 경로는 현재 감사에서 확인하지 못했다. 사용자의 저장 상태가 있다는 가정으로 새 설정을 덮어쓰지 않는다.

## 수리 계약과 미완료 범위

UI 전용 raw 분기를 추가하거나 고정 mute를 삭제하는 것만으로 완료하지 않는다. 공통 application handoff가 웹 정지 요청/관측·native 재생 준비·명시 mute 요청/관측을 구별하고 중복 audio 방지를 소유해야 한다. mute 제어가 필요하면 typed SDK/CLI 명령과 nullable muted 관측을 함께 추가하며 요청값을 관측값으로 복제하지 않는다. ready는 요청 접수만으로 설정하지 않고, 실패/미관측·기한 초과를 명시한다. 전체 media identity 검증을 약화해 소리를 내는 것으로 대체하지 않는다.

제품의 volume/mute 선택·지속 저장은 공통 use case와 연결할 별도 계약으로 남는다. OS audio endpoint·오디오 채널 존재·muted=false·volume>0 관측도 실제 청취 품질을 입증하지 않는다. CLI gate의 volume0 및 기존 실패 원문은 보존한다.

이 수리는 커서 어댑터 변경보다 넓은 application/API/관측 계약 작업이다. 현재 [커서·오디오 순차 계획](../../../plans/player/native-pointer-and-audio.md)에 등록하고 중앙 다음 입력 동결과 슬롯을 조율한다. 본 문서는 수리 완료가 아닌 source audit 및 명시적 잔여 계약이다.
