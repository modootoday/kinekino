---
id: evidence:kinekino/20261029114048
title: 시청 요소 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 시청 요소 확대 관측 — 2026-10-18

## 근거와 방법

직전 전체 구성 검토 이후 시청 요소 계약을 구체화했다. 원본은 `.spec/artifacts/design/v2/viewing/native-player-start-v2.png`(1586×992), playback/fullscreen/skip/web v2다. Rust `tools/visual-review`의 nearest crop으로 아래 파일을 생성하고 직접 열었다. 제품의 실제 화면 캡처가 아니며 시각 인수 결과가 아니다.

| 출력(runtime/verification/design-crops 기준) | 원본 영역 x,y,w,h | 배율 | 관측 |
|---|---|---|---|
| caption-detail-v2-4x.png | 1360,0,226,50 | 4 | 기본 캡션 배경 무프레임, 가는 최소화/정사각/대각선 glyph, 창 우측 정렬 |
| start-actions-v2-2x.png | 580,320,440,360 | 2 | 원형 outline play, 제품명과 동작의 세로 계층, 폴더/외곽선 amber, 탐색 중성 외곽선, 둥근 모서리 |
| seek-detail-v2-4x.png | 0,880,420,70 | 4 | 검정 영상면 하단의 가는 track와 원형 thumb, transport 면과 분리 |

crop 좌표는 추출 영역이며 요소의 정확한 bounds 측정값이 아니다. 원본 좌표와 최종 논리 단위는 [요소 계약](../../player/presentation/viewing/visual-elements.md)에서 구별한다. 색상의 정확한 pixel 샘플링·모든 glyph의 광학 중심 수치 측정은 아직 하지 않았다.

## 현재 코드와의 대조

| 요소 | 소스에서 확인한 사실 | 설계와의 관계/다음 조치 |
|---|---|---|
| 제목 | title_bar.rs는 미디어 제목이 있으면 제품명 대신 그것만 표시 | playback 시안의 제품명+구분선+작품명 구조와 차이. 분리된 title 구성 필요 |
| 캡션 | caption_controls.rs 42×32/3개, SVG canvas16/stroke1, max↔restore 구분, hover/focus | 구조는 정본과 대응. 실제 모든 hover/pressed/focus의 확대 비교는 미검증 |
| 시간 | transport.rs clock은 항상 HH:MM:SS, readiness가 있으면 현재값 표시 | 계약은 1시간 미만 MM:SS. missing time을 0으로 대체하는 경로도 있어 unknown 독립 표현 조율 필요 |
| 회차 닫기 | episode_panel.rs의 텍스트 '닫기' 일반 버튼 | 작은 X glyph/hit32 공통 패널 닫기로 정리 필요 |
| 회차 행 | 일반 selected egui 버튼에 제목만 표시, 높이40 | 번호/현재 표시선/play glyph/connecting 상태 별도 구성 필요 |
| seek/volume | MediaRail track3, thumb5/6, hit32, keyboard 지원 | source 기반 형상 대응. 실제 재생 seek/음소거 등 동작 검증과 시각 상태 확대 비교 별개 |

이 표는 소스 관측으로 확인한 차이다. 이번 턴에서 새 player.exe를 실행하거나 클릭한 것으로 기록하지 않는다. 기존 실제 실행 기록은 [media controls](media-controls-2026-10-18.md)에 한정한다. 사용자 확인용 player는 사용자가 직접 종료한다. VM·DPI는 변경하지 않았다.
