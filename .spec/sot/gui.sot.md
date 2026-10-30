---
id: sot:kinekino/gui
title: GUI 네이티브 재생 관측 읽기
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# GUI 네이티브 재생 관측 읽기

2026-10-18. [SDK 계약 근거](../../../verification/application/playback-status-contract-2026-10-18.md)를 UI 생명주기와 연결한다. 원본 URL·referer·파일 경로·pipe·Session.status·raw error를 새로운 공개 상태에 넣지 않는다.

## 조립과 소유권

presentation::run_with_native_status(smoke, start_url, position, client, paths, publisher: Option<NativeStatusPublisher>)를 제공한다. 기존 run/run_with_client 및 PlayerApp::new 호출은 유지하며 선택 builder로 publisher를 전달한다. main의 channel/SDK reader 조립은 root 세션 소유다.

PlayerApp::update_native에서 Mpv::snapshot이 반환한 NativePlayback을 그대로 publisher에 투영한다. pause intent·native_started·설정 volume·DesignSample로 관측값을 합성하지 않는다. stop_native/소스 launch 전 clear하며 channel owner drop의 clear는 application 구현을 사용한다. publish/clear lock 실패는 새 SDK 읽기에서 오류로 보고되며 성공 값으로 합성하지 않는다.

## 표시 계약

실제 workspace 검사기의 접힌 SDK 연결 정보에서 같은 ApplicationClientHandle의 typed playback_status를 수동 조회한다. 미조회/unsupported provider/not_found 관측 없음/안전 DTO/조회 실패를 구분한다. DesignSample은 이 라이브 진단을 추가하지 않으며 가짜 native snapshot을 게시하지 않는다. 조회 결과는 그 시점의 게시 값이며 이후 엔진 변화가 자동 반영된다고 표시하지 않는다. native playback 위치와 source PTS를 동일시하지 않는다. publication age를 engine event freshness로 해석하지 않는다.

이 작업은 읽기 관측 통합이며 새 GUI 제어/CLI IPC를 제공하지 않는다. GUI의 provider 주입과 별도 CLI 프로세스의 provider 없음은 다른 실행 환경이다. 실제 연결 검증과 미검증은 책임별 실행 기록에 둔다.
