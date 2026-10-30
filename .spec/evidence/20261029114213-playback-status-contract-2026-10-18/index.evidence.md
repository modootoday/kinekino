---
id: evidence:kinekino/20261029114213
title: playback.status 읽기 계약과 네이티브 관측 연결 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# playback.status 읽기 계약과 네이티브 관측 연결 경계

후속 현재 상태는 [native 상태 SDK 조립](native-status-assembly-2026-10-18.md)과 [GUI 실제 paused·엔진 종료 clear 검토](../presentation/native-playback-status-ui-review-2026-10-18.md)를 따른다. GUI에는 reader/publisher가 연결됐고 CLI는 host IPC 부재로 provider 없는 계약을 유지한다. 아래 GUI 미연결·5개 지원 및85tests는 이 계약을 처음 준비한 시점의 범위다.

2026-10-18 준비 단계. 목표는 legacy 네이티브 상태를 공통 application API로 읽을 수 있는 포트·DTO·선택 provider 계약을 만들고, 실제 관측과 미상 값을 구분하는 것이다. **당시 GUI에는 provider가 연결되지 않았다.** 이 계약 구현 범위에서 IPC host, 재생 제어, 모델 실행은 추가하지 않았다. 기존 동결 증거는 변경하지 않았다.

## 조사한 현재 경로

- `infrastructure/mpv/ipc.rs`가 named pipe로 mpv의 time-pos/duration/pause/idle-active/volume/video-params/audio-params/fullscreen을 observe한다. `events.rs`가 application의 NativePlayback으로 변환하고, `Mpv::snapshot`이 수신 큐를 소진하고 child 종료 여부를 확인한다.
- `presentation/player_app.rs::update_native`가 snapshot을 받아 native_started/pause intent 및 browser 표시 정책을 처리한다. `stop_native`는 mpv를 없앤다. 이 소유권은 그대로 유지했다.
- `application/session.rs::Session`에는 media URL/referer를 가진 MediaSource, 자유 문자열 status, 브라우저 WebPlayback이 들어 있다. 이 객체 전체를 직렬화하면 안 된다. 브라우저 관측·GUI의 의도 상태·native_started를 실제 native 관측으로 대체하지 않았다.
- 기존 NativePlayback의 paused/idle bool은 기본값과 실제 property 관측이 구분되지 않았다. `pause_observed`, `idle_observed`를 추가하고 mpv property parser에서 유효 bool을 받은 경우만 기록한다. null/잘못된 property는 미상으로 되돌린다. legacy bool의 기존 fallback 동작은 유지했다.

## 구현한 application 경계

`application/playback_status`는 typed `PlaybackStatus`, `PlaybackStatusPort`, 기본 unavailable provider, `native_status_channel()`의 단일 publisher와 clone 가능한 reader를 제공한다. publisher는 NativePlayback을 안전한 값으로 투영하고 마지막 snapshot 하나만 저장한다. reader는 프로세스·GUI를 조작하지 않는다. writer clear/drop 이후에는 이전 snapshot을 반환하지 않는다. lock 오류는 원문 없는 internal 오류다.

| 응답 필드 | 제공 근거와 미상 처리 |
|---|---|
| origin | `native_observation` 계약. DesignSample enum/대체 경로 없음. 실제 입력의 신뢰성은 조립자가 보장해야 하며 테스트 fixture가 실제 관측이라는 증명은 아님 |
| connected | NativePlayback의 native 연결 확인 bool. media 준비·재생 성공·건강 상태를 의미하지 않음 |
| paused / idle | 해당 property를 실제 관측한 경우만 bool. 미관측/잘못된 data는 null |
| position_us / duration_us | 유효한 비음수 finite native seconds를 1000000배 후 내림한 decimal u64 문자열. 미상·음수·NaN/Infinity·범위 초과는 null. native position은 원본 source PTS가 아니며 f64 이상의 시간 정밀도를 약속하지 않음 |
| volume_percent | 실제 finite 비음수 native volume. GUI 설정·추정값을 사용하거나 100으로 임의 clamp하지 않음. 미상/비정상 값은 null |
| video_size | 둘 다 양수 u32인 실제 width/height만 제공. 한쪽 부재·0·overflow는 null |
| audio_channels | 양수 u32 실제 관측만 제공. 부재·0·overflow는 null |
| has_error | 원문 오류의 존재 여부만 제공. URL·토큰·프로세스/pipe/path가 들어갈 수 있는 raw error는 보관/반환하지 않음 |
| published_age_ms | 마지막 publish 이후 monotonic 경과 시간, decimal u64 문자열. mpv event 생성 시각·최종 engine 응답 age는 현재 미상이며 이 값으로 보장하지 않음 |

position/duration 및 나머지 값은 마지막으로 게시한 snapshot이다. 연결이 끊긴 snapshot도 마지막 관측을 담을 수 있으므로 connected와 함께 읽는다. owner의 close/소스 교체 시에는 clear해야 한다. reader는 임의 freshness cutoff나 가짜 idle/playing 상태를 만들지 않는다. rate·선택 track·media/episode ID·원본 URL·세션 비밀·browser host·raw status·fullscreen 등은 이 DTO 계약에 포함하지 않았다. 응답 DTO는 알 수 없는 필드를 거부한다.

## 공통 SDK/API 계약

- 기존 `PlayerSdk::default()`, `PlayerSdk::new(storage)`, 명시적 2-generic 타입과 `with_frame_capture`를 유지한다. 세 번째 generic의 default는 UnavailablePlaybackStatus다. 선택 `with_playback_status(provider)`를 추가했고 frame/provider builder의 양쪽 순서가 유지된다.
- 객체 안전 ApplicationClient에 default typed `playback_status(RequestContext)`를 추가했다. 기존 구현에 새 필수 메서드를 요구하지 않는다. 공통 Request/Response 경로와 동일한 query를 사용한다.
- provider 없음/available=false: playback.status는 unsupported/exit3/data=null이다. 이 준비 단계의 실제 CLI·main 조립은 모두 provider가 없었고5 supported/47 unsupported·52 schemas 계약을 확인했다. 후속 GUI의 provider 연결은 위 조립 정본을 따른다.
- provider 있음: capability와 schema에서 playback.status만 조건부 supported다. 아직 publish가 없거나 clear/drop 이후면 not_found이며 ready/idle 성공을 합성하지 않는다. 실제 snapshot이 있을 때만 안전한 DTO가 data에 나온다. 파일/FFmpeg/provider가 모두 연결되면 지원 명령은 조건부6개가 될 수 있지만 현재 제품 조립의 지원 개수를 승격하지 않는다.
- payload는 생략 또는 빈 object만 허용한다. null/array/unknown fields는 invalid_argument다. expected_revision/idempotency_key/timeout_ms는 unsupported로 명시 거부하고 schema도 null만 허용한다. 이 동기 snapshot 읽기에 미구현 deadline/저장 revision/재실행 보장을 붙이지 않는다.
- playback.open/play/pause/seek 등 나머지 제어, local_host_transport는 계속 unsupported다. 기존 공개 queries.execute/capabilities/supported 함수도 provider 없는 wrapper로 유지했다.

## 실제 연결을 위해 남긴 최소 인수 지점

[coordination](../../../../runtime/coordination/sdk.md) 맨 위에 정확한 소유권을 게시했다. root와 UI 소유자가 main 조립에서 reader를 SDK에, writer를 native lifecycle owner에 전달하고, 실제 `update_native` snapshot 직후 publish하며 `stop_native`·교체·실패·Web 전환에서 clear해야 한다. 기존 생성 경로 호환을 유지하는 새 전달 경로는 그 소유자가 조율한다. 이 세션은 presentation/**·main·CLI 조립을 수정하지 않았다.

다음 인수에는 실제 mpv 연결 중 관측/일시정지/idle/null property/종료·소스 교체/오류를 읽어 실제 SDK와 대조해야 한다. GUI 연결 없이 메모리 fixture가 통과한 이번 결과를 제품 재생·IPC 연결의 완료로 표시하면 안 된다. 별도 CLI 프로세스가 GUI 관측을 읽는 host/transport도 아직 없다.

## 검증

새 `tests/playback_status.rs`의 3개 시험:

1. 실제 CLI full Request stdin의 provider 부재 unsupported/exit3/빈 stderr/data=null 응답을 SDK와 전체 비교. 기존 기본·명시적2-generic 생성자도 컴파일 확인.
2. NativePlayback 메모리 fixture→publisher→다른 thread의 Arc<ApplicationClient> typed read: 정확한 위치/길이·volume·size·channels, 미관측 bool null, 실제 false 관측, raw URL/token 오류 비노출, NaN/Infinity/음수/overflow의 null, clear/drop의 not_found 검증. 이 시험은 실제 엔진 연결 시험이 아니다. 최종 DTO의 unknown source_url 필드 거부도 검증했다.
3. 선택 provider의 capability/schema/strict payload/context 계약과 나머지 playback 제어의 unsupported 검증.

`infrastructure/mpv/events.rs`의 새 parser 회귀는 실제 property-change 형식의 JSON을 입력하여 미관측→관측 false→null에 따른 nullable status를 확인한다. 이것도 실제 mpv 프로세스 시험과 구분한다.

```text
cargo fmt --check
cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 2 -- --test-threads=2
cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 2 --test playback_status
cargo clippy --locked --offline --target-dir runtime/parallel-build/sdk -j 2 --all-targets -- -D warnings
```

전체 기본 검사에서 85 passed/8 ignored: library63 + CLI unit2 + CLI contract4 + request identity1 + frame2 + 새 playback3 + SDK/실제CLI 전수대조1 + subtitle9. 전수 대조는 기존52 schema/47 unsupported와 기본 capability를 다시 확인했다. 외부 추가7개는 이번 읽기 계약 범위에서 다시 실행하지 않았고 기존 동결 근거로 유지한다. 최종 DTO strict deserialize/unknown 필드 시험 후 targeted3·fmt/clippy도 다시 통과했다. GUI·새 재생·다운로드·AI 추론·Pages·VM·DPI·다른 앱 종료는 실행하지 않았다.
