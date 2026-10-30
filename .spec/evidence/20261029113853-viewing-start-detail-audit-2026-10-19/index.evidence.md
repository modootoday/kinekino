---
id: evidence:kinekino/20261029113853
title: VIEW 시작 세부 후속 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VIEW 시작 세부 후속 감사

[StartScreen 이관 계약](../../player/presentation/viewing/start-screen.md)에 caller/공통컴포넌트 책임·Taffy 중앙정렬·Tailwind 크기 후보·표식 전용 stroke 재검토와 실제 인수 항목을 구체화했다. 구현/새 EXE 검증은 별도이며 현재값을 정본으로 역추인하지 않는다.

2026-10-19 r163 문서 감사다. 기준 native-player-start-v2.png 전체와 표식(716,326,158,157)4배 파생물 `runtime/review-runs/view-start-mark-r163-4x.png`를 열어 확인했다. 실제 비교 근거는 r162 고정 실행본의1180×760·무소스 관측이며 S2 변경본 실행이나 수정 완료가 아니다.

| 요소 | 현재 근거 | 차이·후속 책임 |
|---|---|---|
| VIEW-01a 원형 표식 | 직접 작성 start-mark.svg viewBox100/circle47/path37,30→69,50→37,70/stroke1.8 | 실제에서는 기준 확대의 원/삼각형보다 선이 가늘게 보인다. 원본 이미지 배율·blur를 그대로 수치로 복제하지 말고 같은 논리 크기에서 표식 전용 stroke/광학 비례를 재검토한다. 일반20px 아이콘 stroke를 전체 확대하지 않는다 |
| 시작 세로 묶음 | workspace::idle_workspace가 vertical_centered와 available_height×0.46−80 수동 여백을 사용 | 원형→제품명→두 동작→키 안내의 중심/간격을 공용 StartScreen 조합으로 소유해야 한다. 회차/볼륨 popup이 영상 영역을 바꿀 때 실제 중심을 다시 확인한다 |
| 크기·여백 | 100,12,24,312,44,150 등의 지역 상수 | 기존 size/space/button 토큰과 Tailwind 범주의 크기 척도로 정리해야 한다. 단순 이름 붙이기로 임의 치수를 추인하지 않고 시안/실제 대응으로 결정한다 |
| 제품명/동작/안내 | ui.heading·공용control_icon::action Folder/Search·Ctrl+O | Pretendard와 공용 의미 색/아이콘을 유지한다. 시안 raster의 글꼴 왜곡을 복제하지 않으며 heading23 semibold/body13/small11 계약을 기준으로 선명도/광학 비례를 검토한다 |

계획: S2 고정·실자료 인수 뒤 별도 checkpoint에서 공용 StartScreen의 Taffy 중앙 정렬/토큰·표식 전용 비례를 설계하고 구현한다. P 패널·볼륨 popup/Escape,760/1180·원문 긴 제품명,시청↔제작 복귀와 실제 캡션 접근을 다시 확인한다. 현재는 코드 변경/새 Rust gate/시각 일치 완료가 아니다. 최초 폴더/탐색 동작과 실제 미디어 상태를 가짜 값으로 채우지 않는다.
