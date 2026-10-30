---
id: sot:kinekino/job-boundaries-2
title: 작업 큐와 책임 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 작업 큐와 책임 경계

상태: `Draft → Preflight → Queued → Running → Verifying → Completed`, 분기 `NeedsInput/Failed/Cancelled`. 진행률은 다운로드 bytes·ASR 구간·렌더 time 등 실제 근거를 사용하며 ETA는 추정으로 표시한다. 중단 가능한 다운로드와 재시작이 필요한 인코딩을 같은 ‘일시정지’로 속이지 않는다. 취소 시 소유 프로세스/임시파일만 정리하고 완료 파일을 지우지 않는다.

| 계층 | 목표 모듈과 단일 책임 |
|---|---|
| domain/media_asset | 미디어/트랙 식별·시간 기준 |
| domain/subtitles | cue·형식·변환 손실 정책 |
| domain/production | ClipRange, CropTransform, CardDocument, ExportPreset; 외부 엔진 비의존 |
| application/production | save_subtitles, download_media, capture_frame, render_short, export_cards 유스케이스와 포트 |
| application/jobs | 작업 상태·의존성·취소·재시도·자원 예산 |
| infrastructure/ffmpeg | probe/command construction/worker/progress 분리. argv로 신뢰된 고정 실행 파일 호출 |
| infrastructure/download | HTTP/HLS 전송·재개·세션 수명 |
| infrastructure/storage | 임시파일/원자적 확정·프로젝트 스키마 마이그레이션 |
| presentation/studio | 작업별 패널·프리뷰·편집 입력, OS API/프로세스 직접 호출 금지 |

위 경로는 책임 배치의 목표 예시이며 모든 경로가 현재 미생성이라는 뜻이 아니다. 2026-10-26 소스 대조에서 application/download와 infrastructure/download, 자막·프로젝트·snapshot/card export 등 실제 책임별 구현을 확인했다. 기존 구현·owned jobs 수명과 호출 경로를 먼저 재사용하고 표의 경로명에 맞추려고 중복 모듈을 만들지 않는다. 정확한 기능별 노출·미연결·미검증은 [추적표](../../../../plans/player/design-completion/requirements.md)에 구분한다. `utils`나 하나의 거대한 studio/AI 모듈로 모으지 않는다. UI 스레드에서 다운로드/추론/인코딩하지 않는다. 프로젝트 입력은 데이터만 허용하고 실행 파일·명령·임의 filtergraph/script를 받지 않는다.
