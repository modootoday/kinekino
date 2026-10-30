---
id: evidence:kinekino/20261029114810
title: 현재 SDK·CLI·GUI 코드의 root 통합 회귀 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 SDK·CLI·GUI 코드의 root 통합 회귀 검사

2026-10-18. 이 문서는 실제 자동 검사와 read-only CLI 관측을 담당한다. 제품 기능 계약, 전체 시각 승인 또는 AI 품질 승인을 대신하지 않는다. 사용자 앱·Pages·VM·DPI·새 endurance 작업은 변경하지 않았다.

## 실제 실행한 전체 검사

workspace root에서 다음 명령이 순차 통과했다.

```text
cargo fmt --check
cargo test -j2 --target-dir runtime/parallel-build/sdk -- --include-ignored --test-threads=1
cargo clippy -j2 --target-dir runtime/parallel-build/sdk --all-targets -- -D warnings
```

총73개: library52, CLI unit2, 실제 CLI 계약3, frame capture6, subtitle conversion10이다. main/doc test는0개다. include-ignored로 실제 로컬 Sintel 프레임·자막 왕복 검사도 포함했고 제외된 테스트는 없었다. 자막의 Unicode/시간/손실 검토/덮어쓰기 거부·동시 export와 프레임의 실제 decode·PTS·프로토콜 차단·PNG 손상/크기 거부·timeout·동시 저장을 검사했다. 모델을 로드하거나 GUI 창을 닫지 않았다.

GUI unit 검사는 compact/full shell·모든 demo tab/fixture state의 렌더링, 누락된 live canvas를 DesignSample로 대체하지 않는 경계 등을 포함한다. 실제13화면의 모든 요소·상태가 선택 시안과 시각적으로 일치한다는 검사가 아니다. 이후 UI source 수정에는 해당 빌드 이후 별도 검증이 필요하다.

## 실제 CLI 관측

이 빌드의 `runtime/parallel-build/sdk/debug/player-cli.exe` SHA256은 `54f483ef938aca6f5127615cdb0102e939aa979b3de7187055070027782773b9`다. 실제 version·capabilities·schema subtitles.convert 요청과 request_id를 원시 응답에 보존했다.

- [version](../../../evidence/player/sdk-integration-regression-2026-10-18-r1/version.stdout): API/schema1, SDK0.1.0, 표시 이름과 안정 ID·GUI/CLI 실행 파일명을 분리해 반환했다.
- [capabilities](../../../evidence/player/sdk-integration-regression-2026-10-18-r1/capabilities.stdout): discovery3개와 subtitles.convert·production.capture_frame이 supported다. frame 지원은 현재 FFmpeg/FFprobe 어댑터가 발견된 이 환경의 결과이며 모든 호스트에서 무조건 지원한다는 뜻이 아니다.
- [schema](../../../evidence/player/sdk-integration-regression-2026-10-18-r1/subtitle-schema.stdout): 자막 변환 요청 계약을 반환했다. schema를 조회할 수 있다는 것과 모든 다른 명령의 실행 가능성은 별개다.

실제 CLI help는 UTF-8 SRT/VTT 변환·조건부 local frame PNG 저장과 의존성 조회를 설명한다. network·HDR·ASS·OCR·local host transport·공통 playback 및 AI/jobs/models/project 명령은 현재 unsupported로 구분했다. GUI의 legacy 재생 제어와 공통 SDK playback 미지원의 비대칭은 기존 작업 범위이며 이 회귀 검사로 해소됐다고 하지 않는다. 실제 unsupported/invalid argument exit code·비밀 입력 비반사 검증은 CLI 계약3개에 포함한다.

## AI 도구 및 증거 검사와의 구분

root 모델 도구의 최신18tests/fmt/Clippy-Dwarnings/build와 spec-maintenance5tests/fmt/Clippy도 통과했다. 원본 Candle strict 관측은 별도 [참조 검증](../ai/embedding-reference.md)에 둔다. release spec-maintenance full check는 immutable105개·정본203문서 링크오류0·보존 upstream오류1과 참조2manifest59파일을 검사했다. 새 문서나 source 추가 후에는 그 변경을 포함하는 다음 검사가 필요하다.

공통 기능의 source/설계 문서 대조는 SDK 담당의 별도 감사 범위다. 이 문서의 테스트 통과를 그 감사의 완료나 전체 기능 구현 완료로 확대하지 않는다.
