---
id: evidence:kinekino/20261029114245
title: 공통 native playback 제어 포트 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# 공통 native playback 제어 포트 검증

기준 통합 관측은 [기존 SDK 통합 검증](shared-sdk-integration-2026-10-19.md)이다. 기존 동결 CLI 산출물과 근거는 수정하지 않는다. 이번 변경은 presentation/main을 수정하지 않으며 기존 GUI playback.status provider 연결을 유지한다.

## 계약과 구현

- domain `PlaybackControl`은 Play/Pause/Seek/SetVolume만 허용한다. application `PlaybackControlPort`, typed `ApplicationClient.playback_control`, `PlayerSdk.with_playback_control`을 GUI/CLI 공통 진입점으로 제공한다. 기존 생성자와 frame/status builder는 유지한다.
- play/pause payload는 빈 object, seek는 `position_us` decimal u64 문자열(0..9007199254740991), volume은 유한한 `volume_percent` 0..100이다. unknown field/null/잘못된 숫자는 invalid_argument이고 원본 payload를 오류에 반사하지 않는다. seek는 엔진 재생 위치 기준 절대 us이며 미디어 duration 제한·프레임 정확도 보장은 없다.
- adapter가 없으면 네 명령은 unsupported다. 실제 native adapter를 명시적으로 주입하면 capability/schema가 supported다. 이는 provider 존재를 뜻하고 현재 엔진·연결 준비를 보장하지 않는다. 기본 CLI 및 현 GUI 조립에는 새 provider를 주입하지 않았다.
- 성공 receipt는 `acceptance=accepted`, `completion=unobserved`, opaque decimal `engine_instance_id`이다. 실제 엔진별 큐 enqueue 성공만 뜻한다. property 관측/JSON IPC ACK/명령 완료·적용·성공 여부와 동일시하지 않는다. 기존 playback.status는 별도 실제 snapshot 경로를 계속 사용한다.
- revision/idempotency/timeout context는 unsupported다. durable job/완료 조회/명령별 ACK correlation/외부 CLI host IPC는 구현하지 않았다. request_id는 공통 Request/Response correlation을 따른다.

## Native adapter 경계

`native_control_channel()`은 binder와 Send+Sync port를 제공한다. `Mpv.control_handle()`은 해당 프로세스의 고유 큐에 고정된다. `bind/clear` 쓰기 잠금과 submit 읽기 잠금으로 대상 선택~enqueue를 직렬화한다. 이전 핸들은 새 엔진으로 재지정되지 않는다. clear는 바인딩만 해제하고 이미 수락한 요청을 취소하지 않는다. 기존 엔진에서 접수한 요청은 이후 실패할 수 있으며 새 엔진으로 재생하지 않는다. 클라이언트의 이전 의도를 expected engine ID로 비교하는 기능은 없다.

공유 mpv 명령 큐는 64개 bounded channel이며 legacy GUI send도 이 큐를 사용한다. GUI send signature는 유지하지만 포화 시 즉시 실패한다. 새 typed 경로는 unbound/ended=not_found, 연결 전=needs_input, full=resource_exhausted, closed=engine_failed로 반환한다. 연결 상태는 실제 IPC connect/exit와 Mpv drop/child exit에서 관리한다. 프로세스 생성 전에 핸들을 할당하여 식별자 할당 실패 시 자식이 남지 않도록 한다.

wire는 고정된 set_property pause false/true, seek seconds absolute+exact, set_property volume만 만든다. 외부 JSON command/URL/실행 파일 인자를 받지 않는다. 실제 mpv 프로세스 실행 검증은 이번 범위에서 수행하지 않았다.

## 검증 근거와 한계

새 Rust 테스트 `tests/playback_control.rs` 및 `mpv/control.rs`는 실제 native adapter와 bounded channel을 사용한다. 연결 준비 flag는 fixture에서 설정하며 실제 mpv·실제 named pipe 관측을 대체하지 않는다. 네 명령 wire/receipt, strict payload·수치 경계, provider 부재, 조건부 capability/schema, 미지원 context 무전송, 준비 전/포화/닫힘/종료, clear/rebind/drop와 기존 핸들 대상 보존을 확인한다.

검증 명령은 `cargo fmt --check`, `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --quiet`, `cargo clippy --locked --offline --all-targets --target-dir runtime/parallel-build/sdk -j 1 -- -D warnings`이다. 최종 모두 exit 0이다. 전체 **103 passed / 0 failed / 8 ignored**(lib 76, CLI unit 2, cli_contract 4, cli_request_identity 1, frame_capture 2, playback_control 4, playback_status 3, sdk_capability_parity 2, subtitle_conversion 9 passed). 새 제어 테스트는 native adapter unit 5개와 integration 4개다. 교체 경합 receipt/수신 큐 ID 일치 및 accepted가 status의 paused/position을 바꾸지 않는 시험을 포함한다. Clippy의 테스트 초기화 방식 지적 1건은 struct initializer로 수정 후 통과했다. 외부 미디어 ignored 시험·GUI 제어 연결·실제 재생 적용·endurance는 이번 검증 결과에 포함하지 않는다.

## GUI 인계

정확한 조립/소유권/정리 순서는 `runtime/coordination/sdk.md`의 2026-10-19 native control 절을 따른다. [계획 006](../../../plans/player/plan-006-sdk-gui-cli.md)의 남은 GUI/host/완료 관측 범위는 유지한다.
