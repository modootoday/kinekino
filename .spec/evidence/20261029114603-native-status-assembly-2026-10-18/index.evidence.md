---
id: evidence:kinekino/20261029114603
title: 네이티브 상태 SDK 조립
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 네이티브 상태 SDK 조립

후속 [2026-10-19 회귀 재검증](shared-sdk-integration-2026-10-19.md)은 현재 전체94tests와 고정 CLI 전수 대조, 동일 GUI provider 구성의 capability 차이1개를 확인했다. 새 GUI 실기나 CLI IPC 완료로 확대하지 않는다.

후속 실제 관측: UI 담당의 [r53 검토](../presentation/native-playback-status-ui-review-2026-10-18.md)가 추가됐다. root가 보존된 player.exe SHA256 `097a328581fe7fbe800703502333ad3a71ec33485385907bede6da441727263d` 및 stdout.jsonl의 native-playback time51.875/duration888.032, paused/idle 관측 flag와1280×544/6ch를 직접 재확인했다. GUI의 관측 없음→실제 paused 상태→소유 엔진 종료 후 clear 실기는 해당 담당 검토 근거를 따른다. root가 화면을 다시 조작한 시험으로 표현하지 않는다. 아래 작성 시점의 미검증 범위 중 이 읽기 경로는 보충됐으며 playing DTO·정상 정지·소스 교체 경합·CLI IPC는 남아 있다.

2026-10-18. `src/main.rs`에서 application의 `native_status_channel()`을 생성하고 reader를 `PlayerSdk::with_playback_status`, publisher를 presentation의 `run_with_native_status`에 전달한다. main은 채널과 의존성 조립만 담당한다. 상태 변환·정책은 application, 실제 mpv 관측은 기존 어댑터와 presentation lifecycle이 담당한다.

GUI는 provider가 있으므로 `playback.status` 지원을 표시한다. 관측 전에는 `not_found`이며 임의 idle 상태를 만들지 않는다. CLI에는 GUI IPC가 없어 해당 명령은 계속 unsupported다. 이는 같은 계약의 어댑터 제공 차이이며 CLI의 GUI 자동화나 IPC 구현을 의미하지 않는다. 나머지 capability는 동일해야 한다.

`tests/sdk_capability_parity.rs`는 동일 provider 구성의 SDK와 실제 CLI 전수 대조를 유지하고, GUI 구성의 capability 차이가 `playback.status` 한 개뿐인지 별도 검사한다. 과거 동결 증거의 desktop-equivalent 표현은 당시 provider 없는 조립 기준이며 새 GUI 구성의 실제 실행 증거로 사용하지 않는다.

연결 코드 검증과 실제 재생 검증은 구분한다. 실제 mpv snapshot의 SDK 조회, stop/source 변경 이후 clear, GUI 상태 표시 실기는 UI 담당의 후속 관측이 필요하다. 이 문서 작성 시 해당 실기 통과를 주장하지 않는다. [기존 application 계약과 시험](playback-status-contract-2026-10-18.md)을 따른다.

## 조립 회귀 검사

`cargo fmt --check` 및 `cargo clippy --locked --offline --target-dir runtime/parallel-build/sdk -j 2 --all-targets -- -D warnings` 통과. 같은 target-dir의 전체 `cargo test --locked --offline -j 2`는 86 passed/8 ignored로 종료했다. library63, CLI2, CLI contract4, request identity1, frame2, playback status3, capability parity2, subtitle9다. 외부 미디어 추가 시험8개를 이번 실행으로 통과시킨 결과는 아니다. 실제 CLI52 schema 및 미지원 명령의 전수 응답 대조와 provider 한 개 차분을 포함한다.

최초 공용 `target` 검사에서는 기존 test EXE 잠금으로 LNK1104가 발생했다. 다른 앱/프로세스를 종료하거나 삭제하지 않고 분리된 SDK target에서 재실행했다. 파일 잠금 실패를 테스트 통과로 합산하지 않았다. 이 검사들은 실제 GUI 조작이나 실제 mpv 상태 공개 관측을 대신하지 않는다.
