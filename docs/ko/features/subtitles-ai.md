---
title: "자막과 AI"
description: "지금은 자막을 직접 편집하고, 음성 인식·화자·번역은 계획 단계입니다."
ref: subtitles-ai
nav_order: 12
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: features, cols: 2 }
---

자막은 영상에 구운 글자가 아니라 데이터입니다. 변환하고, 구간만 내보내고, 큐와 화자를 한 줄씩 고칠 수 있습니다.

## 지금

- **변환**: `subtitles.convert`로 SRT와 WebVTT. `status:available`
- **구간 내보내기**: 시간 구간 안의 큐만. `status:available`
- **큐·화자 편집**: 직접, 리비전과 함께. `status:partial`
- **음성 구간 감지**: 말소리 구간 찾기. `status:partial`

## AI로 할 계획

- **음성 인식**: 클라우드 whisper-large-v3 또는 로컬 whisper.cpp large-v3-turbo. `status:planned`
- **화자 분리**: 기본은 꺼짐, 켜면 로컬 pyannote community-1. `status:planned`
- **정렬**: 큐를 말소리에 맞춤. `status:planned`
- **번역과 요약**: 큐 단위, 저장 전에 리뷰. `status:planned`
- **OCR과 검색**: 화면 글자 읽기, 전체 자막 검색. `status:planned`
- **ASS·이미지 자막**: 스타일 렌더링. `status:planned`

> [!NOTE]
> AI 결과는 제안으로 옵니다. 리뷰에서 수락하기 전에는 자막에 아무것도 쓰지 않습니다.
