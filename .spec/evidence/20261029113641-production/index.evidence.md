---
id: evidence:kinekino/20261029113641
title: 실제 제작 경로 통합 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 제작 경로 통합 검증

최신 root 회귀는 [SDK/실제CLI 지원 범위 전수 대조](../application/sdk-capability-parity-2026-10-18.md)를 따른다. 현재 호스트는5명령 지원/47미지원이며52개 schema와 미지원 응답을 공통 client와 대조했다. 기본81tests 및 외부 미디어 추가7개를 구분한다. 아래 시점별 제작 이력은 유지하며 AI 작업·GUI 현재 프레임 통합 완료로 확대하지 않는다.

2026-10-18. 모델 추론과 모델 불필요 제작 경로를 분리한다. 테스트만을 위한 가상 자막 외에 배포된 실제 Sintel 자막을 공통 CLI로 검증한다.

## 자막 변환 corpus

Rust 스파이크 `corpus-conversion`는 sdk target의 실제 `player-cli.exe`에 Request JSON을 stdin으로 전달하고 SRT→WebVTT→SRT를 수행한다. 출력 파일은 매 실행의 새 PID 폴더에 만들고 FFprobe의 PTS/duration/data 패킷으로 원본·중간·왕복 의미를 대조한다. 원본 해시 보존과 손실 미수락 정책도 확인한다. 제품 파서 구현을 검증 도구에 복사해 자체 검증하지 않는다.

첫 실제 입력 `sintel_ko.srt`의 시작은 단일 LF 후 숫자 cue다. 당시 파서가 블록 앞의 빈 줄을 cue 번호로 읽어 `invalid_argument: SRT requires a numeric cue index`, exit 2를 반환했다. [실패 응답](../../../evidence/ai/model-spikes-2026-10-18/results/cli-corpus-80136/forward.json)을 보존한다. 이 실행은 성공 출력이 없고 왕복/FFprobe 단계가 실행되지 않았다.

SDK 담당이 바깥·여러 구분 빈 줄을 허용하되 본문 들여쓰기/끝 공백은 보존하도록 수정하고 실제 원본 회귀 시험을 추가했다. 원래 세션에서 수정 빌드 CLI로 새 폴더의 [왕복 결과](../../../evidence/ai/model-spikes-2026-10-18/results/cli-corpus-83444/manifest.json)를 직접 확인했다. 26개 cue의 원본→VTT와 원본→왕복 SRT 모두 독립 FFprobe PTS/duration/UTF8 payload가 완전 일치했다. 원본 SHA256 `241f05979875655eaadf7ed98f5ae98cbb712845d33c80bc2fc540b349a61391`도 유지됐다. 실행 CLI SHA256 `686fb1ad8b1fd7abcb1159f33a5646186befcefd3f76e4b36f62f705d43510b0`를 기록했다.

이 corpus는 plain text이므로 ASS·스타일·화자 구조 보존을 증명하지 않는다. 실패는 파일 IO 또는 음성 모델이 아니라 입력 호환 문제였으며 수정 전후를 함께 보존한다. GUI 실제 변환은 별도 주입/화면 작업 중이다. 원래 세션의 후속 통합 검사에서 fmt·47개 일반 테스트·Clippy가 통과했고 외부 corpus 의존 테스트 1개는 기본 실행에서 ignored로 표시됐다. 별도 실제 CLI+FFprobe 시험은 위와 같이 직접 수행했다. 병렬 세션의 후속 수정은 다시 검증한다.

## 에셋 제작

후속 root 통합 감사에서 `player-cli capabilities`는 로컬 프레임 저장을 supported로 응답하지만 `--help`가 미디어 명령 전체를 unsupported로 안내하는 불일치를 발견했다. 안내문을 MP4/MKV/WebM→PNG와 FFmpeg/FFprobe 가용 조건·capabilities 조회로 수정했다. 실제 CLI 프로세스 help 회귀를 추가했고, include-ignored 전체70개(lib49/CLI2/CLI계약3/프레임6/자막10)·fmt·Clippy가 통과했다. 수정된 실제 help 출력도 확인했다. playback·AI 명령과 local host transport는 여전히 unsupported다. 이 수정은 독립 AI 도구의 제품 통합이나 GUI 시각 인수를 의미하지 않는다.

후속 공통 SDK/CLI는 production.capture_frame도 지원한다. 담당 세션의 [실제 캡처 계약/검증](../../../evidence/player/capture-frame-validation-2026-10-18.md)과 Sintel52초 응답/PNG를 원래 세션에서 읽고 직접 확인했다. canonical CLI capability도 capture=supported, 미설치 어댑터는unsupported로 구별됨을 확인했다. GUI 현재 프레임 동기 캡처·HDR·회전·색 일치·편집 완료를 뜻하지 않는다.

egui/eframe/egui_extras0.36.2 및 egui_taffy0.14 정렬 후 원래 세션의 `cargo test -- --include-ignored --test-threads=1`에서59개(외부 FFmpeg/실제 corpus 포함) 전부 통과했고 fmt·Clippy도 통과했다. 이후 병렬 UI의 폰트/시안 세부 변경은 별도 최종 인수 대상이다.

마스크 기반 RGBA와 평면 glTF, 실제 깊이 기반 GLB의 세부 검증은 [영상 에셋](vision.md)을 따른다. 실제 수동 카드4개·12초 세로 인코딩과 수정·디코딩 결과는 [제작 스파이크](manual-export.md)를 따른다. 편집 가능한 공통 프로젝트 저장 및 제품 SDK/GUI 렌더 연결은 남아 있다. 텍스트 모델의 카드 문구 출력만으로 카드 제작 완료로 표시하지 않는다.
