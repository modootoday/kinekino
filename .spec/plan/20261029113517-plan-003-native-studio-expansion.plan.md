---
id: plan:kinekino/20261029113517
title: 네이티브 우선·미디어 제작·노트북 AI 확장 계획
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 네이티브 우선·미디어 제작·노트북 AI 확장 계획

## 현재 상태

후속 확정: 화자 분리·겹침/미지정 검토·라벨 편집·출력 보존을 자막 범위에 포함한다. 모든 단계는 [공통 SDK/GUI·AI CLI 선행 계획](plan-006-sdk-gui-cli.md) 이후 동일 API로 연결한다. 제품명은 가제이며 변경 가능한 metadata다. Pages 추가 갱신은 중단했다.

사용자 확정: 네이티브 기본 UI, Windows/Rust/단일 창/PotPlayer 참고, 광고 제거, HF 모델, 노트북에서도 실행 가능한 경량 AI. 원본 프레임+썸네일, 수동·추천·일괄 숏폼, SRT/VTT/ASS+OCR, 다운로드, 커뮤니티/Instagram 카드뉴스, ASR 자막, 피사체 인식과 2D/3D 투영 에셋을 설계 범위에 넣었다.

정본은 [UX](../../sots/player/presentation/viewing/sot-011-native-first-studio-ux.md), [제작](../../sots/player/application/production/sot-012-media-production.md), [HF/AI/투영](../../sots/player/application/analysis/sot-013-local-ai-projection-assets.md). [ADR005](../../adrs/player/adr-005-native-first-production-workspace.md), [ADR006](../../adrs/player/adr-006-huggingface-local-model-pipeline.md)에 선택 근거를 남겼다. 새 기능은 아직 구현하지 않았다. DPI·VM 설치는 계속 사용자 보류이며 이 계획의 선행조건이 아니다.

## 순서와 완료 증거

| 단계 | 작업 | 완료 게이트 |
|---|---|---|
| 0. 기존 재생 정리 | 호스트 반복 전환/웹 30분/mpv 60분, 주소/목록 동기화 수정, 최종 빌드 갱신 | [실제 검증](../../sots/verification/playback/sot-010-host-endurance-validation.md). 수정 빌드의 별도 범위를 표시 |
| 1. 네이티브 기본 | 빈 영상/열기, 탐색 패널, 네이티브 우선 회차 열기, 명시적 웹 복구, 포커스/전체화면 | 세 카테고리/로컬 파일·실패·취소·이전/다음 실제 조작. 이 단계부터 UX 방향을 실행 앱에 반영 |
| 2. 공통 기반 | MediaAsset/Track/PTS, project schema, job state/ports, 저장 경로/원자적 출력, 취소 | 계층 의존·작업 상태 테스트, 회차 변경 중 입력 고정·앱 재시작 복구 |
| 3. 기초 저장 | 프레임 원본/썸네일, 자막 원본/변환, 영상 다운로드/리먹스 | 실파일 열기·프레임/색/자막 포함 검사, cue 변환 손실, HTTP/HLS·만료·충돌/부족/취소 |
| 4. 수동 제작 | 정확 구간 숏폼·crop/fit·자막/오디오, 카드뉴스 편집/플랫폼별 출력 | 경계 프레임·A/V sync·문구 넘침·순서·실제 출력 해상도/용량 확인 |
| 5. 노트북 AI 기반 | HF catalog/revision/hash·선택 설치·offline, tiny/base ASR·mobile OCR·소형 sLM | 사용자 지정 동급 노트북 기준인 현 호스트에서 CPU/RTX 실측, 메모리/RTF/품질·재생 경쟁 예산 통과 |
| 6. 자동 추천·일괄 | 원문 근거 연결, 후보/문구 교정, 회차 묶음 큐·부분 실패 복구 | 수동 경로와 동일 프로젝트/출력, 중복/입력 혼선 없음, 원문에 없는 내용 검토 |
| 7. 피사체·투영 | 선택/마스크 수정·추적, 2D PNG/manifest, depth·2.5D·GLB package | 대상 도구 round-trip·좌표/PTS/색/알파·레이어 정렬, 실제 노트북 예산 |
| 연구 트랙 | 완전 3D 복원/리깅·장시간 실시간 AI·고급 자동편집 | 별도 모델·품질/성능 게이트. 2.5D 패키지 완료로 대체하지 않음 |

‘모두 설계’는 모든 단계를 한 번에 구현 완료한다는 뜻이 아니다. 단계마다 동작하는 파일/영상/프로젝트와 관측 근거를 남기고 후속으로 확장한다. 큰 공통 AI/Studio 매니저 대신 책임 있는 모듈을 추가한다.

## 모델 선정 spike

- 공개 HF 후보의 모델 카드/라이선스/파일 구조·토크나이저·변환본 계보를 확인하고 실행 조합을 고정한다. 현재 조사 링크는 후보이며 release manifest가 아니다.
- ASR tiny/base와 품질 옵션 small/Qwen ASR, sLM 0.6B급 양자화, 한국어/일본어 mobile OCR을 먼저 비교한다. 단어 시간은 정렬 모델/엔진으로 검사한다.
- EdgeTAM/SAM tiny와 depth small은 정적/짧은 구간부터 측정한다. PyTorch/WSL 의존 경로를 기본 설치로 묶기 전에 Windows CPU/ONNX 또는 고정 worker 배포를 검증한다.
- 추가 메모리 예산은 절약 1.5GiB/균형 3GiB의 선택 목표다. 사용자가 목표 노트북이 현 호스트와 동급이라고 확인하여 현 호스트를 주 실측 기준으로 삼는다. 배터리/발열과 8GB/16GB 저사양은 선택 확장 시험으로 남긴다.
- 모델 수치가 아니라 CER/WER·시간 정렬·환각·마스크 경계/추적·작업 전체 peak RAM/VRAM·RTF를 기록한다. 실시간/품질 보장은 측정 후 정한다.

## 2026-10-18 확정 진행 상태

추가 웹 보충 시험으로100/150% 호환 모드 전체화면 진입·자동숨김·P 패널 복원·Escape 해제도 확인했다.200% 웹 시험과 함께 세 배율의 웹/mpv 전체화면 표본을 채웠다. 실제 앱 DPI·스크린샷·원본 로그/해시 및 교정한 일반 회차 집계 정책은 아래 앱 DPI 정본에 보존했다.

최신 후속: [앱 HWND DPI 실기 정본](../../sots/verification/display/sot-016-app-dpi-matrix.md)의96/144/192 및 네이티브 전체화면·컨트롤·회차 패널 표본 완료. 아래 모니터 probe 이후 앱 미검증 상태는 이 후속 결과로 갱신한다. 좁은 창·혼합 모니터 이동·반복 배율 크기 복원·고배율 장시간 등 확장 시험은 남는다.

자체 개발 언어는 [ADR007](../../adrs/player/adr-007-rust-only-development.md)에 따라 Rust로 통일한다. 기존 비 Rust 검증·주입 스크립트는 이관 부채이며 제품 전체 이관 완료는 아니다. 새 tools/validation은 정책/유스케이스/Windows·엔진 어댑터를 분리했다.

[호스트 내구 시험](../../sots/verification/playback/sot-010-host-endurance-validation.md)의 웹30분59초/mpv60분28초 표본과 반복 전환10회는 완료됐다. SourceChanged 주소·목록 누락 수정과 짧은 수정 빌드 회귀도 완료됐다. 수정 빌드60분 재시험·실제30초 만료·다양한 콘텐츠/오류 조건은 후속이다.

[최신 추론 조사와 CPU Q4 실측](../../sots/player/infrastructure/inference/sot-014-inference-engine-research.md)을 완료했다. mistral.rs와 llama.cpp를 고정 데이터로 측정했으며 최종 엔진/품질은 미선정이다. ASR/OCR/분리·추적·깊이와 GPU/동시 재생 측정을 다음 모델 spike로 둔다. 연구·설계 문서는 제품의 Studio 기능 구현 완료를 뜻하지 않는다.

[VM 대체 가상 디스플레이](../../sots/verification/display/sot-015-host-virtual-display-review.md)는 설치/96·144·192 probe 측정/시험 모니터 제거까지 완료했다. 앱 HWND DPI 확인 및 전체 UI 매트릭스는 남는다. VM/ISO 정리는 자동 승인 검토 차단으로 수행되지 않았다.

## 남은 결정/외부 검증

- [상세 UI와 AI 제안](../../sots/player/presentation/review/sot-019-detailed-ui-ai-proposal.md)의 D1~D11을 Pages에서 검토한다. OP/ED 음악·반복 기반 구간 후보, 수동/자동 정책, 쿠키 영상 보존, 다음화 자동 재생 분리, 회차별 구간 편집, 재생 우선 작업 큐를 후속 구현 범위에 추가한다. 현재 모델·정확도·지연·메모리는 미확정이며 기능 구현/시험 완료가 아니다.

- 사용자는 현 데스크톱과 동급 노트북을 목표로 확인했다. 현 호스트 CPU/RTX를 주 기준으로 실측하고, 최소 Windows 버전과 실제 노트북의 배터리/발열 예산은 별도 확정한다.
- 모델 고정 revision/파일 해시·배포 크기·허용 라이선스 목록, FFmpeg 구성/라이선스·encoder 가용성.
- 사이트 자막의 실제 분리 트랙 여부, 정상 세션에서 저장 가능한 소스 매트릭스, 만료/재개.
- 최신 Instagram 비율/캐러셀 장수·용량, YouTube 게시물 장수/용량은 공식 본문/실기 재확인. 임의 고정값을 플랫폼 규정처럼 넣지 않는다.
- 투영 대상 도구/좌표·GLB 커스텀 데이터 처리, 폰트 패키지·한글 줄바꿈/색 관리.
- 자동 추천은 원문 근거가 있는 초안, 외부 게시/업로드는 별도 범위. 계정 연결과 게시 자동화는 현재 구현에 넣지 않는다.

변경 구현 뒤 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`와 기능별 실기 검증을 수행한다. 문서만으로 기능 완료 표시하지 않는다.
