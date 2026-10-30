---
id: evidence:kinekino/20261029114800
title: CLI 예약 endpoint 오류의 요청 ID 보존
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI 예약 endpoint 오류의 요청 ID 보존

2026-10-18. root가 현재 source의 SDK/CLI 지원 범위와 GUI 호출 지점을 다시 대조했다. 실제 CLI의 예약 `--endpoint` 미지원 오류에서 유효한 요청 ID가 소실되는 문제를 발견해 수정했다. 새 host/IPC나 AI 실행 기능을 추가한 것은 아니다.

## 오류와 수정

runner는 완성된 Request의 validate가 통과한 뒤 endpoint 사용을 unsupported로 반환했지만, 새 ApiError의 request_id를 채우지 않아 Response::failure가 `cli`로 대체했다. 실제 CLI 회귀에서 기대 `endpoint-요청` 대신 `cli`가 나오는 것을 수정 전에 재현했다.

수정은 validate 이후 이 오류에만 검증된 Request.context.request_id를 옮긴다. raw JSON에서 ID를 부분 복구하지 않으며 잘못된 ID/unknown field/JSON 거부 정책은 유지한다. endpoint 값은 응답에 반사하지 않는다. 해당 명령은 계속 exit3/unsupported이며 로컬 host를 실행하지 않는다. GUI/application/domain/infrastructure 코드와 Cargo manifest/lock은 이번 변경에서 수정하지 않았다.

## 실제 검증

새 `reserved_endpoint_errors_preserve_valid_request_correlation_without_echoing_endpoint` 테스트는 positional+request-id와 full Request JSON stdin의 두 모드를 실제 CLI 프로세스로 실행한다. 둘 다 response/error ID 일치·exit3·unsupported·가짜 endpoint 비반사를 확인했다. 수정 전1test 실패, 수정 후 통과했다. 기존 identity9case 회귀도 전체 검사에서 통과했다.

```text
cargo fmt --check
cargo test --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 -- --include-ignored --test-threads=1
cargo clippy --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 --all-targets -- -D warnings
```

전체77tests가 통과했다:54library+2CLIunit+4CLIcontract+1requestidentity+6frame+10subtitle. include-ignored를 명시해 외부 로컬 미디어 검사도 포함했다. 실제 자막 왕복과 local PNG 캡처 회귀를 포함하며 새 장시간 재생·AI 모델 추론·VM/DPI·GUI 시각 인수는 포함하지 않는다. 이전 담당76/과거root73과 이번77은 source 시점·새 테스트 차이가 있는 별도 관측이다.

## 보존과 현재 통합 범위

[별도 evidence](../../../evidence/player/cli-endpoint-correlation-2026-10-18-r1/)에 실제 새 CLI EXE·runner/test source 사본·당시 관련 source/Cargo hash 목록·JSON 입력·두 모드 stdout/stderr/exit·capabilities stdout/stderr를 보존했다. 추가 보존 실행은 full JSON **파일** 모드이며 stdin 모드는 Rust 회귀에서 확인했다. 전체 소스·toolchain의 hermetic build 증거로 확대하지 않는다.

| 보존 파일 | SHA256 |
|---|---|
| player-cli.exe | 805b5b1ff481f9351e4ed8bb95f52a76ec9971151dce57f8b813d368487662ac |
| positional.stdout | 9b9619c9305bfa6a02e8ccefac1408f50aedda3294b8d870471e6592e6505371 |
| json.stdout | 89b251fa94c0460e6bb180dd63e697f39b2fcff488037d1244677a3099c9396b |
| capabilities.stdout | 7c5b4500d7f427a68d20afbb73c7ec10374b6b95173c7513d326984d91b11bb8 |

두 보존 endpoint 실행은 exit3이며 stderr가 비어 있다. 실제 capability는 version/capabilities/schema/subtitles.convert/production.capture_frame의5개 supported를 반환했다. 나머지47개는 unsupported이며 모델 스파이크 실행을 제품 API 구현으로 계산하지 않는다. frame은 이 호스트의 FFmpeg/FFprobe adapter 가용 조건이며 다른 호스트까지 보장하지 않는다.

source의 main과 CLI는 같은 application SDK와 파일 adapter를 조립하고, GUI subtitle worker는 typed convert_subtitles를 호출한다. GUI frame 호출·legacy native/WebView playback의 공통 facade 연결은 여전히 미구현이다. WorkspaceReadPort의 live provider는 Unsupported이고 preview는 DesignSample이므로 모델/작업 UI 표시를 실추론 결과로 세지 않는다. 이번 검사는 실제 GUI를 다시 조작하거나 승인 시안 일치를 확인한 검사가 아니다.
