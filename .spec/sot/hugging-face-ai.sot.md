---
id: sot:kinekino/hugging-face-ai
title: Hugging Face 기반 로컬 AI와 투영 에셋
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# Hugging Face 기반 로컬 AI와 투영 에셋

2026-10-26 [제작·분석 task 통합](../production/task-integration-contract.md)에 모델/원미디어 입력 고정·작업별 출력·오류/취소/저장/재시작 인수를 연결했다. 기존 모델 선정·실표본 provenance·미검증을 유지한다. 새 추론·다운로드를 재개하는 지시가 아니다.


본문은 2026-10-18 승인된 책임별 재배치에서 아래 파일로 손실 없이 분리했다. 이 문서는 통합 범위의 진입점이며 기능 정책을 중복 정의하지 않는다. 기존 미구현·후보·관측 한계를 각 문서에 유지한다.

- [확정 요구와 해석](scope.md)
- [후보와 미확정 항목](model-candidates.md)
- [HF 모델 저장소 계약](model-manifest.md)
- [현재 호스트 관측과 노트북 예산](resource-budget.md)
- [ASR → 검토 가능한 자막](asr.md)
- [OCR → 자막](ocr.md)
- [피사체 인식과 제작 재사용](subjects.md)
- [2D·2.5D·3D 투영 에셋 계약](projection-assets.md)
- [작업 수명과 계층](job-boundaries.md)
- [평가 게이트](acceptance.md)

후속 추가: [화자 분리·자막 정렬](speaker-pipeline.md)은 ASR와 별도 작업/모델 계약이다. 화자 값/출력 손실은 [도메인 정본](../../domain/subtitles/speaker-contract.md)을 따른다. 기존 후보/실측 문서가 화자 모델 선정·실행 결과를 뜻하지 않는다.
