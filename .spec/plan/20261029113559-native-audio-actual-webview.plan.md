---
id: plan:kinekino/20261029113559
title: 오디오 실제 WebView 포트 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 오디오 실제 WebView 포트 인수

[공통 등록 계획](native-audio-checkpoint.md)과 [CLI 실행 근거](../../sots/verification/application/native-audio-cli-2026-10-19.md)를 잇는다. UI bridge r2의 반환 후 중앙 배정으로 root 등록·검사를 진행했으며 [실제 WebView 실행 정본](../../sots/verification/application/native-audio-webview-2026-10-20.md)에 actual-r2의 세 leg 성공과 첫 실패를 보존했다. 아래 준비 설명은 동결 기간의 역사이며 현재 검증 상태는 실행 정본을 따른다.

## 준비 입력

`runtime/coordination/native-audio/actual_webview.pending.rs`, `actual_media.pending.rs`, `actual_process.pending.rs`는 Rust 초안이다. 독립 rustfmt edition2024/skip_children로 문법·포맷만 확인했으며 아직 type/test/실제 COM 실행을 하지 않았다. 기존 full371pass를 이 초안의 검증으로 확대하지 않는다.

UI 반환 뒤 `src/infrastructure/browser/audio_backend/actual.rs`와 그 하위 `media.rs`·`process.rs`에 등록한다. 기존 audio_backend에는 cfg(test) 모듈만 추가한다. browser_provider의 기존 HiddenWindow/Apartment를 cfg(test)로 crate 내부에 재수출하며 타입 가시성만 crate 범위로 조정한다. 제품 API, Cargo/lock, Meari, root 제품 factory는 변경하지 않는다. 최소 접근 연결로 컴파일되지 않으면 첫 오류를 보존하고 범위를 다시 확인한다.

## 실제 효과와 관측

독립 프로필의 숨긴 Wry WebView로 Browser의 실제 audio owner를 구성하고 기존 `LegacyAudioHandoffPorts`를 그대로 사용한다. Rust가 생성한 유한 raw-video AVI는 실제 legacy Mpv::launch_local로 연다. 네트워크와 지원 provider의 미디어 동일성 문제를 이 fixture와 섞지 않는다. 테스트 Browser 구성에는 원격 페이지와 메시지 수집을 넣지 않으며 about:blank의 owned controlled 문서만 쓴다.

NativeControlBinder와 공통 intent owner를 같은 SDK에 연결하고 LocalHost를 연다. 실제 고정 player-cli가 pending set_volume을 호출해 deferred receipt를 받고, coordinator가 native IPC 상태와 전체 WebView IsMuted를 관측하며 준비 완료로 전환한다. CLI playback.audio_handoff와 playback.status로 같은 instance의 실제 readiness/mute/volume/pause를 읽는다.

WebView를 먼저 native 전체 mute로 막은 뒤 유한 WAV audio 요소를 새로 생성한다. 실제 IsDocumentPlayingAudio=true와 IsMuted=true의 동시 관측을 기록한다. DOM paused/muted 값을 침묵 근거로 사용하지 않는다. 유한 audio의 종료를 native 속성으로 관측한 뒤 coordinator.close가 native process/세 worker의 실제 종료 proof를 얻고 원래 WebView mute로 복원하는지 확인한다.

실패 leg에서는 audio가 끝난 동일 owned fixture에서 native SetIsMuted(false)를 외부 fault로 넣는다. 같은 production ports/coordinator가 guard loss를 검출하고 원 오류를 보존하며 native 종료 후 lease 정리를 하는지 확인한다. 가짜 AudioHandoffPorts나 가짜 StoppedNativeEngine을 만들지 않는다. 다른 COM API 실패와 OS 종료 실패를 실제로 유발했다는 주장으로 확대하지 않는다.

실제 COM BrowserProcessId를 view가 살아 있을 때 읽고 read-only process HANDLE을 캡처한다. 모든 view/context를 해제한 후 그 HANDLE의 종료를 message pump와 함께 기다린다. 임의 프로세스를 찾거나 종료하지 않는다. native 종료 proof·WebView lease·LocalHost drop·소유 실행 경로 inventory를 함께 인수한다.

AVI는 native width/height32×32와 두 시점 이상에서 0.2초 이상 위치 진행을 실제 IPC로 확인한다. WAV는 전체 view가 muted=true인 동안 새 audio의 native IsDocumentPlayingAudio=true를 관측한다. 이 속성은 endpoint audibility나 실제 청취 품질을 뜻하지 않는다. 추가 정상 leg는 WebView 초기 mute=true에서 시작하고, 같은 CLI의 native mute true→false 실제 관측 동안 lease의 view mute=true를 유지한 뒤 최신 사용자 의도 false로 복원하는지 검사한다.

## 함께 수리할 lifetime 경계

`runtime/coordination/native-audio/lifetime-follow-up`의 후보와 baseline을 보존하고 root에 등록했다. intent owner는 마지막 bound engine identity를 scope clear 뒤에도 유지하여 새 lifetime이 종료된 뒤 오래된 coordinator의 게시·정리를 거부한다. coordinator는 AwaitingMedia 이후 IPC connected=false를 실패로 처리하고 실제 stop proof 전에는 WebView lease를 유지한다. 새 lifetime 종료 뒤 stale publisher 거부와 Ready 뒤 disconnect/stop 실패/명시적 cleanup 재시도 집중 회귀가 통과했다. 정상 owner close/EOF와 예기치 않은 연결 손실을 같은 성공 상태로 취급하지 않는다. 실제 guard-loss fault는 COM 속성 해제이며 실제 IPC 단절 fault와 구별한다.

## 게이트

1. root 등록 전 snapshot과 현재 source pins를 보존한다.
2. check/focused compile과 첫 오류를 보존한다. 초안을 type 성공으로 추정하지 않는다.
3. strict/fmt/필요 회귀 후 lib harness와 CLI/mpv/runner SHA를 고정한다. legacy launch가 쓰는 mpv는 harness 옆 고정 복사본이며 Rust 테스트도 SHA를 검증한다.
4. 새 evidence root에서 exact ignored actual case를 실행한다. 관측 이력은 append jsonl, 요청·응답·오류·receipt는 보존한다. 실패 디렉터리를 재사용하지 않는다.
5. actual 성공/실패/복원·프로세스 종료를 정본에 기록한다. controlled local fixture는 실제 지원 provider Web→native 회차 동일성, 원격 reachability, GUI 시각 일치와 실제 청취 품질을 증명하지 않는다. 그 남은 범위를 별도로 유지한다.
