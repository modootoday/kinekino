---
id: evidence:kinekino/20261029114415
title: SDK와 실제 CLI 지원 범위 전수 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK와 실제 CLI 지원 범위 전수 대조

후속 [2026-10-19 공통 SDK 통합 재검증](shared-sdk-integration-2026-10-19.md)에서 현재 전체94tests·새로 고정한 CLI의52schema/47unsupported 대조와 GUI 상태 provider 차분을 통과했다. 아래는 이전 provider 구성·증거의 이력이다. GUI playback.status의 현재 지원과 실제 paused 관측은 후속 정본을 따른다.

2026-10-18. Windows 현재 호스트에서 실제 CLI의 capability·52개 schema·47개 unsupported 명령 응답을 데스크톱과 같은 adapter 구성의 객체 안전 ApplicationClient와 대조했다. **전체 응답이 일치했다.** GUI를 조작한 검증이나 모든 제작 기능의 완료를 의미하지 않는다. 별도 AI 스파이크20개 역할/모델 묶음을 제품 SDK 명령 지원으로 승격하지 않는다.

## 실제 지원 범위

| 구성 | 지원 | 범위 |
|---|---:|---|
| 기본 SDK | 3개 | version·capabilities·schema |
| 실제 파일 adapter와 FFmpeg/FFprobe가 있는 SDK/CLI | 5개 | 위3개 + subtitles.convert·production.capture_frame |
| 알려진 나머지 명령 | 47개 | schema 조회 가능, 실행은 unsupported/exit3/data=null |

capture는 이 호스트의 외부 도구 가용 조건이다. 배포 기기에서도 항상5개 지원한다고 가정하지 않는다. playback·catalog·episodes·AI·models·jobs·projects는 현재 public SDK/CLI에서 unsupported다. 기존 GUI의 네이티브/WebView 재생이 public playback API로 이관됐다는 뜻도 아니다.

`src/main.rs`의 SubtitleFiles + FfmpegFrameCapture 구성과 같은 Arc<ApplicationClient>를 테스트에 사용했다. presentation의 SdkStatus는 같은 client의 capability를 조회하고 자막 변환은 typed client를 사용한다. GUI 현재 프레임→capture 연결, legacy playback facade, IPC·durable jobs·취소·AI 작업 연결은 남아 있다. DesignSample 화면 데이터는 live 모델/프로젝트 상태가 아니다.

## 실행 증거

새 [Rust 통합 시험](../../../../tests/sdk_capability_parity.rs)은 실제 CLI subprocess에 UTF8 JSON Request를 전달한다. 동일 요청에 대한 SDK·CLI response 전체를 비교하며 한글 request ID, 단일 JSON stdout와 빈 stderr를 확인한다. capability 중복, schema status/명령, unsupported exit3/error/data를 검사한다. 임의 shell이나 GUI 자동화로 CLI 기능을 대신하지 않는다.

증거 r2에서는 첫 요청 전에 CLI를 별도 파일로 복사하고 원본과 바이트가 같은지 확인한 뒤 모든 요청을 이 고정 EXE로 실행했다. 파일이 바뀔 수 있는 공용 target 경로에만 의존하지 않는다. 초기 r1 결과는 별도로 보존했으며 r2로 덮어쓰지 않았다.

- [r2 결과](../../../evidence/player/sdk-capability-parity-2026-10-18-r2/summary.json): capability1 + schema52 + unsupported47 = 실제 CLI 요청100개. 테스트케이스1개가100개 서로 다른 요청을 확인한 것이며 모델 실행100회가 아니다.
- 고정 CLI SHA256: `83fc68bdc7abcb282e63ede7d9cd3b10016a130716e169f465702985f53e1ef2`.
- 담당 시험 소스 SHA256: `a9b2fa606ac1c4749c0cc4a11dd4f2c51090a73ee02df0fc7f3c0d1ed326ce85`.
- raw SDK/CLI/request/exit 응답을 명령별 JSON으로 보존했다. reviewed desktop assembly는 시험 후 확인한 소스 사본이며 제품 전체의 컴파일 입력 attestation은 아니다. 병렬 UI 수정 전체나 향후 binary에 결과를 소급하지 않는다.
- [해시 봉인 manifest](../../../evidence/player/sdk-capability-parity-2026-10-18-r2/seal-manifest.json)은 raw 응답·고정 EXE·시험 소스·별도 캡처 자산을 기록한다. manifest SHA256은 `596d15936be87220bf0935adf94e35b972eddd33639b617b58ca230b3e913161`이다. Rust 봉인 도구는 컴파일 입력106개와 EXE1개를 선보존한 `9a7f794d3e4b7d8cd67b281a0d121fcf9d38f28b871ad1d354c109bee36cb686`이며 새 추론은0회다. 도구 test41/fmt/clippy/locked offline build도 통과했다.

## 제품 회귀와 실제 미디어

| 실행 | 결과 | 해석 |
|---|---|---|
| cargo test 기본, test-threads2 | 81통과·8ignored | library62 + CLI2 + CLI계약4 + request ID1 + frame2 + 전수대조1 + 자막9 |
| frame_capture 명시 include-ignored | 8통과 | 위 기본2개와 외부 FFmpeg 추가6개. 중복을 새8개로 합산하지 않음 |
| 원본 Sintel 자막 corpus 명시 ignored | 1통과 | 원본의 선행 빈 줄을 수정하지 않고 왕복 검증 |
| cargo fmt check / clippy all-targets -D warnings | 통과 | 해당 체크포인트 source 범위 |

서로 다른 top-level 시험은 총88개가 통과했다. 기본 ignored8 중7개 외부 미디어 시험을 명시 실행했고, 나머지1개는 timeout 시험이 내부 child로 사용하는 의도적인 subprocess fixture다. 이를 일반 실행 미검증 기능과 혼동하지 않는다.

별도 실제 capture 시험은 Sintel source PTS52000000us를 요청해 같은 PTS·1280×544 PNG635663바이트를 출력했다. [원본 응답](../../../evidence/player/sdk-capability-parity-2026-10-18-r2/sintel-capture-response.json)과 [PNG 사본](../../../artifacts/spikes/2026-10-18/sdk-cli-parity-r2/sintel-52s.png)을 보존했다. native 시험은 공용 Cargo binary를 사용한 별도 실행이며 전수 대조의 사전 고정 EXE로 소급하지 않는다. HDR·색 관리·회전·GUI 표시 프레임 일치까지 검증한 것은 아니다. source PTS seek와 독립 frame 열거/픽셀 대조, 음수 PTS 거절, timeout·동시 게시·덮어쓰기 방지 시험도 통과했다.

## 남은 통합

공통 API 지원과 실제 CLI 응답의 불일치는 이 표본에서 발견되지 않았다. 이후 우선순위는 GUI capture/재생 application 포트와 CLI host 연결, AI 작업·모델 registry·프로젝트 저장·취소/재시작 계약이다. AI 품질 게이트는 [모델 상태 정본](../ai/model-status.md), UI 정밀 인수는 [시각 일치 후속 계획](../../../plans/player/presentation/visual-fidelity-follow-up.md)을 따른다. 이번 검증에서는 모델 추론·다운로드·GUI 실행·Pages·VM·DPI·새 장시간 시험을 수행하지 않았다.
