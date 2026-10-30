---
id: evidence:kinekino/20261029114642
title: Native 오디오 실제 WebView 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 오디오 실제 WebView 인수

[전 단계 CLI 근거](native-audio-cli-2026-10-19.md)와 [이번 인수 계획](../../../plans/player/native-audio-actual-webview.md)을 잇는다. 증거 루트는 `runtime/quality-review/native-audio-webview-r1`이다. UI bridge r2의 명시 heavy/root 동결 반환과 중앙 배정 뒤 source-before의 전체 src/tests·Cargo/lock 및 입력 pins를 보존하고 등록했다.

## 이번 소스

실제 fixture3개는 browser/audio_backend의 ignored test와 하위 media/process 모듈이다. 기존 STA Apartment/HiddenWindow는 cfg(test) reexport와 crate 가시성으로 공유한다. 실제 Browser audio owner·LegacyAudioHandoffPorts·공통 coordinator·NativeControlBinder/같은 SDK·LocalHost·실제 CLI를 소비한다. 생성된 AVI/WAV를 실제 엔진에 전달하며 fake AudioHandoffPorts/종료 proof를 만들지 않는다.

두 제품 경계를 함께 수리했다. intent owner가 마지막 bound engine identity를 scope clear 뒤에도 유지해 더 최신 lifetime 종료 뒤 오래된 게시·정리를 거부한다. coordinator는 AwaitingMedia 이후 native disconnected를 실패로 관측해 실제 stop proof 전 WebView lease를 유지한다. 다른 lifetime의 종료 상태 보존과 Ready disconnect/stop 실패/명시 cleanup 재시도 회귀를 추가했다.

Cargo/lock·공개 SDK parent API96a9…e51a·lib/main·presentation parent는 변경하지 않았다. UI/Meari source도 변경하지 않았다. 기존 full371pass·CLI actual97.51s는 이전 source의 근거이며 이번 수리의 최종 검증으로 소급하지 않는다.

## 실행 현황

- 첫 all-target check: 20824ms/exit0, 경고 없음.
- focused `cargo test --lib audio_handoff`: 43272ms/exit0, 36pass·0fail·1ignored·245filtered, 본문0.70s. 두 새 경계 회귀와 SDK query 검사를 포함한다. 실제 WebView case는 별도 exact ignored 실행이다.
- 첫 strict: 13617ms/exit0, 수리 없음. 전체 cargo test: 263427ms/exit0, 373pass·0fail·19ignored. fmt check: 2584ms/exit0, bins: 1352ms/exit0. 이 입력은 source-registered와 registered-pins.txt에 보존했다.
- 첫 exact actual: 35744ms/exit101. 실제 native AVI에서 width/height32×32·duration10·position0.2·connected=true·muted=true를 확인했고, Browser native mute lease 획득과 CLI pending-volume의 deferred_until_handoff/unobserved도 통과했다. 이후 새 WAV 문서의 IsDocumentPlayingAudio=true를 기다리다 15초 deadline에 도달했다. 아직 정상 전환·복원 성공으로 인수하지 않았다.
- actual-first-source.rs와 첫 harness SHA c44fac6a76bb91d4ad0f52fb2e4b27965ecacf1fe237124375494df423c6b9fd를 보존했다. runner5000/harness120436/mpv108580/browser25732는 종료 뒤 부재였고, owned profile/실행 경로 inventory는 actual-first-owned-after.json의 []다.
- r2는 test-only로 controller visible 상태를 실제 제품의 전환 전 상태에 맞추고, native IsDocumentPlayingAudio 성공 조건을 유지한 채 고정 read-only DOM 진단 callback을 추가했다. parent HWND는 계속 hidden이다. DOM 진단은 silence/readiness 근거로 쓰지 않는다. 공통 정책·제품 API·manifest는 변경하지 않았다. 재빌드/영향 focused는 26745ms/exit0이다. 첫 실패의 원인이 visibility 하나였다고 두 변경을 분리하지 않은 실행으로 단정하지 않는다.
- actual-r2: 9135ms/exit0, exact case `infrastructure::browser::audio_backend::actual::actual_webview_native_audio_handoff_and_guard_loss_cleanup` 1pass·0fail·0ignored·281filtered, 본문6.94s. 진단 추가 후 최종 exact source의 strict4932ms/exit0·fmt1294ms/exit0·전체 회귀103183ms/exit0(373pass·0fail·19ignored)가 통과했다. 최종 전체의 ignored19에 위 실제 case가 포함되며 exact actual의 별도 성공을 일반 검사에 합산하지 않는다.

## 실제 세 leg의 근거

engine1은 native decode/진행과 whole-view silence 뒤 새 WAV를 로드했다. `playing-under-guard.json`은 실제 IsDocumentPlayingAudio=true와 IsMuted=true를 기록한다. 같은 SDK의 CLI `ready.stdout`은 engine1/phase=ready/lifetime_status=ready/ready=true/lease1/published_age_ms55를 반환했다. `ready-properties.json`의 실제 native status는 muted=false/volume0/paused=true/connected=true/position600000us/duration10000000us/video32×32/has_error=false였다. audio_channels=null은 audio 없는 AVI의 실제 값이며 native audio track decode를 가장하지 않는다. WAV audio decode·재생은 별도 WebView native 속성으로 관측했다.

engine1 close는 native_stop_confirmed=true 뒤 lease=null/cleanup_complete=true/error=null을 기록하고 원래 WebView mute=false로 복원했다. 정상 close는 lifetime_status=closed이며 내부 core cancel의 phase=failed만으로 오류가 있다고 해석하지 않는다.

engine2는 유한 WAV 종료 뒤 native SetIsMuted(false)를 의도적으로 호출한 guard-loss fault다. production ports/coordinator가 `Owned WebView audio silence was lost`의 engine_failed를 보존하고 실제 native 종료 proof 뒤 lease=null/cleanup_complete=true로 정리했다. 이것은 실제 COM property fault이며 실제 IPC 연결 손실·COM setter 실패·OS kill 실패를 유발했다는 뜻이 아니다. IPC disconnect의 별도 수리는 집중 회귀 근거를 유지한다.

engine3은 WebView가 원래 mute=true인 상태에서 시작했다. 같은 SDK를 CLI로 호출한 user-mute-true/user-mute-false는 engine3의 accepted/unobserved receipt다. 테스트는 각 요청 뒤 실제 native snapshot.muted의 true→false를 기다리고 그동안 leased WebView IsMuted=true를 계속 확인했다. close는 실제 native 종료 proof 뒤 최신 사용자 의도 false로 복원했으며 원래 true를 무조건 복사하지 않았다. user-intent-restored.json은 error=null/cleanup_complete=true/lease=null이다. 디스크 저장을 검증한 것은 아니다.

BrowserProcessId52712는 살아 있는 COM 객체에서 읽고 read-only HANDLE로 캡처했다. 모든 view/context와 LocalHost를 해제한 뒤 그 HANDLE의 종료를 실제 대기해 browser_exited=true를 기록했다. runner86712/harness94504도 종료됐으며 actual-r2-owned-after.json은 고정 실행 경로와 owned profile의 WebView process 조회에서 []다. 임의 프로세스 종료나 수동 GUI 조작은 하지 않았다.

## 고정 입력

| 입력 | SHA256 |
|---|---|
| CLI | b67c11ae3620ffb417297a68b5d0165a8caa3ed3a29a1fb992fda6ed0d4dab57 |
| GUI | 2b2ed52d3fbc584a8bfb3a4bd47ce2cf58a541c00b1d8fc2a81431a5af08d769 |
| actual-r2 lib harness | d8174a2456b4f699a67e9678d8de7d8dc8067c1927d2709e79c4e66263e682f4 |
| runner | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| mpv | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |

최종 src/tests/assets·5 media package·Cargo/lock snapshot은 source-final에 보존했다. source-final-pins.json의 SHA256은 `66157cf7509cfa7202504ae0700d02bbf0b7995308624302210e2e01896b1f41`다. registered-final.diff는 source-before 대비 정확10파일 변경이며 모두 application 오디오2파일/회귀2파일·browser actual3파일/module1파일·STA cfg(test) reexport/visibility2파일이다. Git CRLF/LF 정규화 경고는 변경 목록과 구분했다. UI/Meari source 변경은 없다.

root API96a9…e51a·Cargo03ee…9a34·lock5032…da5f7·lib534b…c7b95·maind1ab…8611e·presentation parentaa78…430a는 shared-final-pins.txt에서 기존값을 유지했다. 첫 harness와 r2 harness는 각각 보존하며 새 해시를 첫 실행에 소급하지 않는다. 최종 full runner92640/rustup115536와 browser52712는 종료 뒤 부재이며 owned-processes-final.json도 []다. 남은 heavy 실행은 없고 소비 입력을 동결해 중앙에 반환한다.

실제 controlled local fixture의 세 leg 성공과 종료·복원 근거를 인수했다. [Microsoft API 참조](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_8?view=webview2-1.0.4191.47)는 IsMuted=true인 상태에서도 IsDocumentPlayingAudio가 true일 수 있음을 명시하므로, native playing 조건을 DOM 표본으로 바꾸지 않았다. 지원 provider Web→native 회차 동일성, 사용자 endpoint audibility/청취 품질, GUI 시각 일치와 제품 전체 완료는 별도로 남는다.
