---
id: plan:kinekino/20261029113752
title: Native 오디오 공통 계약 등록·인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 오디오 공통 계약 등록·인수

[오디오 소스 감사](../../sots/verification/application/native-audio-handoff-audit-2026-10-19.md)와 [순차 수리 계획](native-pointer-and-audio.md)을 따른다. 준비본 등록 뒤 common coordinator·intent owner·native WebView adapter·SDK 관측 query를 연결했다. [CLI 실행 정본](../../sots/verification/application/native-audio-cli-2026-10-19.md)은 기존371pass18ignored와 실제 CLI native mute/volume 제어를, [WebView 후속 정본](../../sots/verification/application/native-audio-webview-2026-10-20.md)은 두 lifetime 수리·373pass19ignored·실제 controlled local fixture의 전환/guard-loss/최신 사용자 의도 복원과 종료를 구분해 기록한다. 실제 지원 provider Web→native 회차 동일성·사용자 청취·전체 GUI 시각 인수는 남아 있다. 아래 원래 등록 map은 준비 시점의 입력 구분이며 완료 범위는 실행 정본을 따른다. 큰 mpv 패키지 분리·libmpv·제품 host/factory 등록을 이 패치에 섞지 않는다.

## 책임별 등록 경로

| 준비본 | 대상 책임 |
| --- | --- |
| audio_intent | src/domain/native_audio.rs |
| domain_playback_control | src/domain/playback_control.rs 문맥 변경, SetMuted |
| application_playback_control | src/application/playback_control/mod.rs 문맥 변경, strict parser/schema와 native_lifetime 재수출 |
| native_shutdown | src/application/playback_control/native_lifetime.rs |
| audio_handoff | src/application/audio_handoff/mod.rs |
| audio_handoff_tests | src/application/audio_handoff/tests.rs |
| web_silence_owner | src/application/audio_handoff/web_silence.rs |
| web_silence_owner_tests | src/application/audio_handoff/web_silence/tests.rs |
| native_playback | src/application/native_playback.rs 문맥 변경, nullable muted |
| playback_status | src/application/playback_status/mod.rs 문맥 변경 |
| playback_status_observed | src/application/playback_status/observed.rs 문맥 변경 |
| session | src/application/session.rs 문맥 변경, 원 메시지 generation과 관측 순번 보존 |
| mpv_control | src/infrastructure/mpv/control.rs 문맥 변경, fixed mute property |
| mpv_events | src/infrastructure/mpv/events.rs 문맥 변경, 실제 mute property 관측 |
| mpv_ipc | src/infrastructure/mpv/ipc.rs 문맥 변경, mute observer |
| mpv_shutdown | src/infrastructure/mpv/shutdown_observation.rs |
| web_audio_backend | src/infrastructure/browser/audio_backend.rs |
| browser_pause | src/infrastructure/browser/audio_handoff.rs; 기존 고정 pause 요청만, silence 증거 아님 |
| api_capabilities | src/application/api/capabilities.rs 문맥 변경 |
| api_schema | src/application/api/schema.rs 문맥 변경 |

준비 부모의 pending 파일 경로 속성은 등록 때 정상 mod 선언으로 바꾼다. 제품에서 runtime 경로를 include하지 않는다. 기존 파일은 전체 덮어쓰기하지 않고 baseline과 현재 UI 반환 입력을 비교한 문맥 패치로 연결한다. domain/application/mpv/browser 부모와 GUI/SDK actor 연결은 별도 등록 지점이며 아래 미완료 연결을 생략하지 않는다.

## 확정한 관측·소유 경계

SetVolume과 SetMuted는 별개의 typed 제어다. volume0을 muted=true로 추정하지 않는다. control receipt는 접수/미관측, NativePlayback.muted와 SDK status.muted는 실제 IPC mute property 또는 null이다. GUI 관측값을 사용자 저장 의도로 승격하지 않는다.

기존 주기 DOM paused/muted의 수신 순번만으로 실제 요청 이후의 source freshness를 입증하지 않는다. Session은 generation·checked sequence와 UI 상태를 보존하지만 그 표본의 view_muted/confirmed_for_engine은 None이다. 이 표본만으로 native 소리를 해제하지 않는다. `ICoreWebView2_8`의 SetIsMuted/IsMuted를 Rust adapter로 호출해 전체 WebView 출력 mute를 직접 읽는다. 정책·증거 검증은 application, COM은 infrastructure에 둔다. 기존 pause 메시지는 CPU/재생 정지 요청이며 소리 차단의 증거가 아니다. 새 JavaScript 정책은 추가하지 않는다.

WebAudioSilenceOwner는 원 native WebView mute 값을 setter 전에 캡처하고 lease를 먼저 소유한다. setter 실패도 효과가 있었을 수 있으므로 lease를 보존한다. 실제 native property read는 요청한 engine/generation과 after_sequence에 결합한다. 세대 변경은 confirmation을 무효화하지만 전체 WebView의 원상복원 책임을 버리지 않는다. 복원에는 같은 native instance의 process/IPC/stdout/stderr 종료를 관측한 비역직렬화 StoppedNativeEngine이 필요하다. native 종료 실패 때 web 소리를 먼저 복원하지 않는다.

host mute 변경은 같은 owner로 전달한다. lease 중 사용자 변경은 원래 값을 무조건 복원하는 대신 최신 의도로 기록하며, 숨긴 WebView를 즉시 unmute하지 않는다. 원상복원 때 그 의도를 적용한다. owner 밖의 관측 가능한 unmute 충돌은 덮어쓰지 않고 lease/오류를 유지한다. 같은 값을 owner 밖에서 다시 쓰는 의도는 속성값만으로 판별할 수 없으므로 모든 host 수준 변경의 단일 owner 경로가 등록 계약이다. DOM mute·OS audio mixer 설정을 이 adapter가 변경하지 않는다. 지속 저장은 아직 별도 미구현이다.

Handoff는 media 일치→web silence→native 두 제어 접수→native 오디오 실제 관측으로 진행한다. native_started는 마지막 조건 뒤에만 준비 완료다. 초기 Web→native 중복 audio 방지를 유지한다. 사용자 intent 변경은 새 plan ID를 쓰며 이전 receipt가 새 계획을 승인하지 못한다. 실패/30초 미관측은 종료 상태이며 원 오류를 보존하고 자동 재시도하지 않는다. 타임아웃 뒤 실제 audio가 적용되지 않았다고 단정하지 않는다.

## 등록 전에 끝낼 연결

- 공통 handoff coordinator/API handle와 STA effect 실행 경계. SDK는 큐 접수·상태 조회만 하고 UI 이벤트/SDK mutex를 잡은 채 COM을 기다리지 않는다. GUI와 CLI host는 같은 application 정책을 소비한다.
- Browser의 exclusive output-mute owner 설치·generation 변경·lease 정리, 기존 pause 요청 실패 전파, BrowserMessage 원 generation의 Session 전달. 단순 수신 순번이나 page payload를 native mute 증거로 바꾸지 않는다.
- legacy 초기 mute의 typed release 및 실제 관측, native stop 실패 시 process/worker/lease 보존. raw mute=false 분기와 요청 직전 native_started=true를 대체한다. 가짜 resolution artifact로 legacy 종료 proof를 만들지 않는다.
- 기존 volume 입력·handoff 중 intent 변경·사용자 mute 의미의 공통 클라이언트 연결. 저장됐다고 가정한 값이나 DOM 관측으로 사용자 intent를 덮어쓰지 않는다.

## 최소 인수

1. baseline·등록 snapshot·첫 오류를 보존하고 type→준비 테스트→fmt/test/strict를 실행한다. 후보의 formatter parse는 type/test를 대체하지 않는다.
2. default CLI unsupported와 configured CLI/SDK capability·strict bool payload/schema·instance-bound SetMuted·nullable mute 관측을 인수한다. 실제 mpv에서 mute true→false, volume 변경과 mute 독립, stale instance/close 후 거부를 확인한다. 기존 actual volume0과 PNG4초·전체 해상도 조건은 유지한다.
3. 실제 WebView2 native IsMuted 초기값→lease mute→독립 readback, native 실제 mute/volume 관측 뒤 handoff 준비 완료를 확인한다. DOM 캐시 표본·다른 generation/engine/plan·ack만으로는 완료되지 않아야 한다.
4. setter 일부 효과 후 실패·관측 실패·web pause 실패·native close 실패·세대 변경·복원 실패/재시도·원 mute 값·최신 사용자 의도·원 오류 보존을 검사한다. 모든 owned process/worker/브라우저 종료와 소유 PID 부재를 확인한다.
5. 고정 source/bin/CLI/harness/runner/mpv SHA와 정확 actual case를 정본에 연결한다. 속성·오디오 스트림 관측과 실제 청취 품질은 별개다. GUI embedded/transport/fullscreen 시각 완료나 전체 제품 완료로 확대하지 않는다.

공식 근거: [WebView2 IsMuted](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_8). 현재 local webview2-com-sys0.39.1 바인딩에서도 IsMuted/SetIsMuted 서명을 읽었다. 실행·실제 COM 관측은 아직 하지 않았다.
