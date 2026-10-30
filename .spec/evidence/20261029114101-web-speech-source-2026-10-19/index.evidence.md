---
id: evidence:kinekino/20261029114101
title: 음성 인식 Web 선언의 요소 대응
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 음성 인식 Web 선언의 요소 대응

2026-10-19 source 작성 기록. 실제 Meari 파싱/등록/렌더·시각 일치 인수가 아니다. [요소 정본](../../player/presentation/studio/subtitles-visual.md), [기존 확대 관측](subtitle-elements-2026-10-18.md), [토큰 투영](web-ui-token-projection-2026-10-19.md)을 적용한다.

기준 subtitles.png 전체와 기존 Rust crop `runtime/verification/design-crops/sub-asr-tools-2x.png`를 이번 턴에 직접 열었다. crop 좌표는 원본280,580,900,330/2배이며 내부 모든 bounds의 정밀 측정값은 아니다. 이미지에는 파형의 선택면·양끝 grip·playhead·시간 눈금, 편집4버튼 뒤 구분선과 undo/redo2버튼, 원문/교정안 표가 있다. 생성된 시안의 병합 glyph·가짜 cue·동일 원문/교정 텍스트를 기능이나 실제 데이터로 복제하지 않았다.

| 요소 | 기준·정본 | 작성 source 대응 | 판정 |
| --- | --- | --- | --- |
| SUB-09a 편집 | 분할/병합/추가/삭제, icon+label 한 버튼 | cue-edit-toolbar.tsx의 독립4 callback, 명시 can*/reason | source만 작성, actual event 미검증 |
| SUB-09a 기록 | 구분선 뒤 undo/redo | 별도 group+divider, callback2개 | source만 작성, history SDK 미연결 |
| 크기·간격 | hit32/icon20/gap8/group16/border thin/control radius | aw-button-compact와 기존 semantic token만 사용 | CSS 대응, 렌더 미검증 |
| glyph | scissors/merge/plus/delete/undo/redo | 기존clip/merge/plus/delete/undo/redo.svg mask와 의미role6개 | Meari 공식 catalog/resolver 전환·실제 전달 후속 |
| disabled/focus | 지원 사유, focus 외곽선·hover/pressed 공통 | can*와title reason, 공용button 상태 CSS | 키보드·a11y 실제 검증 후속 |
| SUB-03 파형 | 실제 duration/time/선택 cue 구간 | speech-workspace.tsx의 별도 waveform slot | amplitude/time을 만들지 않음, 실제 데이터/렌더 후속 |
| SUB-08 표 | stable cue·원문/교정안 별도 | 별도 cues slot, 기존 cue-table 소비 예정 | 화자/다중선택/편집 SDK 포함 후속 유지 |

speech workspace는 media→waveform→입력 상태→toolbar→cue 순서이며 예시 파형·진행률·인식 언어/모델 준비를 자동 생성하지 않는다. 슬롯/함수 props/일반 boolean식은 현재 Meari 보완 범위이므로 이 source가 통과했다고 주장하지 않는다. r2의 고정4파일 graph 증거를 이 새 source로 소급 확대하지 않는다.

사용한 modoo.today `modootoday/design-delivery/design-to-code-fidelity-diff`의 요소별 expected/built/token/verdict 방법을 이어 적용했다. 실제 시각·키보드·좁은 창은 중앙 슬롯에서 후속 검증한다. 새 GUI/browser/VM/DPI 조작은 하지 않았다.
