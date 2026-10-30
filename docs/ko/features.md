---
title: "기능"
description: "kinekino가 하도록 설계된 모든 것과 각 부분의 솔직한 상태."
ref: features
nav_order: 1
hero: { c: prose }
sections:
  - { c: features, cols: 2 }
  - { c: prose }
---

배지는 로드맵 계획과 코드로 확인했습니다. `status:available`는 현재 빌드에서 동작, `status:partial`는 일부만 또는 한 OS에서만 동작, `status:planned`는 설계만 되어 있고 아직 만들지 않았다는 뜻입니다.

## 기능 영역

- **[재생](features/playback.md)**: mpv 재생, 회차, 이어 보기, 건너뛰기. `status:partial`
- **[스튜디오](features/studio.md)**: 타임라인 구간, 클립, 스냅샷, 리뷰, 작업. `status:partial`
- **[자막과 AI](features/subtitles-ai.md)**: 자막 편집, 음성 인식, 화자, 번역. `status:partial`
- **[로컬·클라우드 추론](features/inference.md)**: 로컬과 클라우드 모델을 하나의 포트로, 동의와 비용 보호 포함. `status:planned`
- **[AI 에이전트](agents.md)**: CLI, 요청 JSON, MCP 서버. `status:partial`
- **[설정과 개인정보](settings.md)**: 추론, 재생, 에이전트를 한 설정 화면에서. `status:planned`

## 전체 기능

| 기능 | 상태 | 비고 |
|---|---|---|
| 조회: `version`, `capabilities`, `schema` | `status:available` | CLI |
| 자막 변환·구간 내보내기(SRT/WebVTT) | `status:available` | CLI |
| PNG 프레임 캡처 | `status:partial` | 로컬 mp4/mkv/webm, FFmpeg 필요 |
| 스냅샷 내보내기 | `status:partial` | 지금은 Windows, Linux는 P4 |
| mpv 재생 | `status:partial` | Windows는 직접 확인, Linux는 P2 |
| 회차 이동, 인트로 건너뛰기 | `status:partial` | 수동 건너뛰기, 자동 감지는 나중 |
| 이어 보기와 환경설정 기억 | `status:planned` | P2 |
| 미디어·AI 제안 리뷰 | `status:partial` | |
| 프로젝트, 지속 작업 | `status:partial` | 리비전과 멱등 키 |
| 카드, 쇼츠, 에셋 | `status:partial` | 실행기는 P4에서 완성 |
| 큐·화자 편집 | `status:partial` | 직접 편집 |
| ASS·이미지 자막 | `status:planned` | P5 |
| 음성 인식, 화자 분리, 정렬 | `status:planned` | P1b, P5 |
| 번역과 요약 | `status:planned` | P5 |
| OCR과 자막 검색 | `status:planned` | P5 |
| 로컬 모델 팩 | `status:planned` | P5 |
| 추론 동의와 비용 보호 | `status:planned` | P1b |
| 설정 화면 | `status:planned` | P6, 스키마 확정 |
| MCP 서버 | `status:planned` | P9 |
| 메아리 UI(Windows) | `status:partial` | 설정 창은 모든 OS(P6), 플레이어 Linux GUI는 mpv(P2) 대기 |
| 편집 출처 기록 | `status:partial` | |
