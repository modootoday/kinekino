---
title: "활용 사례"
description: "사람, 스크립트, 에이전트가 kinekino로 하는 일과 지금 되는 부분."
ref: use-cases
nav_order: 2
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: features, cols: 2 }
---

사례마다 필요한 부분을 적었습니다. 사례의 준비 정도는 가장 덜 된 부분을 따릅니다.

## 보는 사람과 편집자

- **시리즈 보기**: 회차와 건너뛰기는 되고 이어 보기는 계획. `status:partial`
- **장면으로 쇼츠 만들기**: 구간 초안, 쇼츠 렌더, 리뷰. `status:partial`
- **자막 고치기**: 변환, 구간 내보내기, 큐·화자 편집. `status:partial`
- **화자 포함 강연 자막**: 음성 인식과 화자 분리. `status:planned`
- **영상 번역**: 큐마다 리뷰하는 번역 제안. `status:planned`

## 스크립트와 에이전트

- **자막 일괄 변환**: 파일마다 `subtitles.convert`. `status:available`
- **썸네일용 프레임**: FFmpeg로 `production.capture_frame`. `status:partial`
- **에이전트 보조 편집**: 에이전트가 제안, 사람이 리뷰. `status:partial`
- **완전 오프라인**: 로컬 팩과 로컬만 모드. `status:planned`
- **MCP 클라이언트 연결**: `kinekino mcp serve`. `status:planned`
