---
id: evidence:kinekino/20261029114107
title: 자막 하위 화면 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막 하위 화면 확대 관측 — 2026-10-18

[전체 시안 검토](design-reference-review-2026-10-18.md)에 이어 선택 원본 4장의 세부를 Rust visual-review crop으로 확대해 직접 열었다. [시각 계약](../../player/presentation/studio/subtitles-visual.md)에 구성·공통화·미결정을 기록했다. 이번에는 제품 코드 변경이나 새 player 실행을 하지 않았다.

| 원본(.spec/artifacts/design/v2/studio 기준) | crop x,y,w,h | 배율 | 출력(runtime/verification/design-crops 기준) |
|---|---|---|---|
| subtitles.png | 280,580,900,330 | 2 | sub-asr-tools-2x.png |
| ocr-refined.png | 500,480,390,120 | 2 | sub-ocr-roi-2x.png |
| correction.png | 1135,50,450,430 | 2 | sub-correction-inspector-2x.png |
| convert.png | 260,500,905,415 | 2 | sub-convert-comparison-2x.png |

ASR에는 파형 선택면·handle·playhead와 별도 도구 그룹, 원문/교정안 표·행 checkbox·선택 외곽선이 있다. OCR ROI는 amber선/흰 원형 corner handle이며 영상 자막과 연결된다. 교정 검사기는 원문/제안 두 행·추가 문장부호 강조·source PTS 근거·수락/기각/undo가 있다. 변환 비교는 원본/출력 패널 모두 선택된 cue의 읽는 텍스트와 문법을 표시하며 하단에 손실 banner가 있다. crop 좌표는 추출 영역이며 내부 요소 bounds 정밀 측정값이 아니다.

## 현재 소스와의 차이

1. studio/mod.rs는 DesignSample의 SUB 하위 탭을 모두 editor::show로 보낸다. 하위 탭별 실제 구성/데이터가 분리돼 있지 않다.
2. editor.rs는 SUB에 일반 timeline과 공통 표를 사용한다. 이는 파형·ROI·교정 근거·변환 두 패널과 같지 않다. 도구는 텍스트 일반 버튼이며 SVG와 세부 그룹을 아직 맞추지 않았다.
3. 실제 변환은 subtitles.rs의 두 패널을 쓰지만 원본에는 경로/설명, 출력에는 receipt 경로/cue 수/bytes만 표시한다. 원본·출력 cue 문법의 1:1 비교는 미구현이다.
4. preview_inspector.rs는 SUB 전체에 오디오/언어/번역/범위/모델/형식 필드를 제공한다. OCR·교정·변환 전용 검사기를 대체하지 못한다.
5. 실제 변환에는 손실 수락, 입력 변경 시 수락 초기화, 비동기 응답, 경로 picker가 있다. 이를 삭제하지 않고 시안 구조에 연결할 read preview 계약이 필요하다.
6. 화자 분리 탭은 기존 시안 제작 뒤 추가됐다. 전용 이미지가 없으므로 이미지 매칭 완료를 판정할 수 없다. 계약과 이미지 가용성을 구분한다.

이번 관측은 원본 확대와 소스 대조다. 실제 GUI의 다국어 IME·원본/출력 비교 스크롤·ROI drag·파형 seek·필드 focus·실제 저장 결과는 새로 검증하지 않았다. Rust compile/test 성공으로 이 항목을 완료 처리하지 않는다. VM·DPI는 변경하지 않았으며 사용자 확인용 player의 종료는 사용자에게 맡긴다.
