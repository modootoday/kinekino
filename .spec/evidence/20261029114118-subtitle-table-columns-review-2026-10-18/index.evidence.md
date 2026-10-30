---
id: evidence:kinekino/20261029114118
title: SUB cue 표 열 폭·번호 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SUB cue 표 열 폭·번호 검토

2026-10-18. 대상 SUB-08a. 기준 subtitles.png 표 영역(280,731,896,183)을2배 확대한 `runtime/design-inspection/sub-table-r63-2x.png`를 확인했다. 시작/끝은 내용보다 좁은 정렬 열이며 원문/교정안이 유동 폭이다. 시안 checkbox는 실제 다중선택 계약 미연결이므로 이번 변경으로 추가하지 않았다.

## 구현

ASR/교정 표를 모든 열 균등 분배에서 번호48·시작/끝132·화자80과 유동 원문/교정안으로 바꿨다. 고정열 합은 공통 TABLE_FIXED_MAX65% 제한을 따른다. 공용 with_row_numbers는 stable row_id를 보존한 UI projection이며 cue ID를 순번으로 교체하지 않는다. OCR도 같은 함수로 번호를 부여한다. Time kind의 내용은 monospace, 일반 본문은 기존 Pretendard·truncate/전체 hover를 유지한다.

## 실제 실행

r63(PID22688, `runtime/review-runs/ui-sub-table-20261018-r63/player.exe`)을 기존 가상 모니터에서 실행했다.

- 1440×900 ASR: 번호1/2/3, 시작/끝 전체 timestamp, 좁은 화자와 유동 원문/교정안 열을 확인했다. 두 번째 번호 클릭 후 Down으로 세 번째 번호에 focus/선택이 이동하고 파형도50–65초 구간을 표시했다.
- 760×480 ASR: 시작/끝의 HH:MM:SS.mmm가 모두 표시되고 번호·화자·본문 열이 창 안에 들어왔다. 긴 본문은 truncate/전체 hover 방식을 유지하며 다중행 표 구현은 아니다.
- 760 OCR: 번호 열이 하나만 표시되고1/2/3 순서와 전체 timestamp를 유지했다. 두 번째 번호 클릭→Home 후 첫 번호 선택/포커스가 표시됐다. 설정 내부를 스크롤해 원본 텍스트가 첫 cue의 '저 산 너머로 길이 이어져 있어요.'이며 후보가 '교정 제안 예시 · 미적용'임을 확인했다. 표시 순번을 추가해도 stable 선택/검사기 연결이 유지됐다.
- Escape는 설정만 닫고 OCR 표의 첫 행 선택과 본문 스크롤 위치를 보존했다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. 테스트 통과를 전체 화면의 수동 시각 인수로 확대하지 않는다.

소유 창 닫기로 r63 종료 후 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 미검증과 남은 범위

교정/화자 탭·Empty/LongLabels·980/1180의 실제 표 회귀는 이번 표본 밖이다. 다중선택 checkbox, 원문 다중행/교정 diff·실제 편집, 다른 시간열 사용 화면의 전체 monospace 회귀는 후속이다. 시작 시 SEG 시간열은 새 monospace가 표시됨을 관측했으나 모든 열·상태의 인수 근거는 아니다.
