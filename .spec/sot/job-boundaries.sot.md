---
id: sot:kinekino/job-boundaries
title: 작업 수명과 계층
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 작업 수명과 계층

domain/analysis는 TranscriptRevision, RecognitionSpan, SubjectTrack, MaskAsset, DepthAsset, ProjectionSpec, EvidenceLink 등 엔진 비의존 계약만 보유한다. application/analysis는 transcribe_audio, recognize_subtitles, suggest_highlights, segment_subject, estimate_depth, package_projection 유스케이스를 분리한다. application/model_catalog는 모델 설치·버전 선택·가용성 계약을, application/jobs는 기존 제작 큐·의존성·취소를 담당한다.

infrastructure/models(HF 메타데이터/아티팩트), inference(asr/text/ocr/segmentation/depth별 어댑터), assets(이미지/GLB/package 쓰기)를 나눈다. presentation/analysis는 교정·마스크·깊이 프리뷰와 입력만 담당한다. 모델 프로세스·GPU·파일 API를 직접 호출하지 않는다. 모델마다 거대한 공통 AI 매니저에 정책/추론/UI를 넣지 않는다. 경로는 목표이며 현재 미생성이다.

기본 목표는 재생 우선·무거운 추론/인코딩 동시 1개, 작업별 RAM/VRAM 상한과 취소 지점, 유휴 모델 해제다. 실제 상한값은 측정 후 확정한다. 모델 교체·실패가 mpv 재생을 종료하지 않아야 한다. 스케줄러는 다운로드→ASR/OCR→검토→클립/카드/에셋 의존성을 처리하고 완료 결과만 재사용한다. 음성/프레임/자막을 외부 추론 API로 자동 보내지 않는다.
