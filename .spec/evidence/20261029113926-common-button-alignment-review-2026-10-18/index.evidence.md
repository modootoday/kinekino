---
id: evidence:kinekino/20261029113926
title: 공용 전폭 버튼 문자 정렬 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 전폭 버튼 문자 정렬 검토

2026-10-18. 기준 export.png의 취소/뒤로/저장 시작 영역과 기존4× 파생물을 다시 확인했다. 기준 문자는40높이 버튼의 가로·세로 중앙이다. r79 OUT 관측에서 하단 문자가 좌측으로 치우친 차이를 남겼고 현재 Button 코드를 확인했다.

## 원인·공용 수정

egui 0.36.2 Button은 AtomLayout 정렬을 부모 Ui에서 상속한다. 공용 Button은 min_size만 늘려 왼쪽 정렬이 유지됐다. 전폭 variant에는 Ui::add_sized의 centered_and_justified 영역을 적용했다. compact는 기존 내용 기반 폭을 유지했다. 토큰40·variant·활성 조건과 명령 연결은 바꾸지 않았다. 화면별 정렬 helper를 추가하지 않았다.

## 실제 실행

r80(PID75516), `runtime/review-runs/ui-button-alignment-20261018-r80/player.exe`에서 확인했다.

- 1440×900: SEG 구간 수정/현재 회차 저장, OUT 취소/뒤로/저장 시작, MODEL 모델 다운로드, CLIP 미리보기/내보내기 문자가 각각 버튼 중앙에 표시됐다. 버튼 프레임/폭·비활성 상태는 유지됐다.
- 760×480: CLIP 설정 drawer의 두 하단 문자가 중앙에 표시되고 겹치지 않았다. Escape는 drawer만 닫고 수동 편집 화면을 유지했다.
- 760 탐색 CLIP 클릭→End는 OUT로 이동했고 탐색도 마지막 항목까지 스크롤됐다. OUT 본문 끝의 세 버튼 문자는 각각 중앙에 표시됐다. 제목·확인사항·로컬 저장 안내와 버튼 사이의 접근을 유지했다.
- fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. 외부 미디어 조건의 ignored 검사와 실제 GUI 관측은 별개다.

소유 r80을 캡션 닫기로 종료했고 player/mpv 프로세스 조회는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

이번 실제 표본은 미연결 비활성 버튼이다. 활성·hover/pressed/keyboard focus·아주 긴 버튼 문구·980/1180·compact 및 모든 사용 화면의 세부 검토는 남는다. 명령을 가짜로 활성화해 상태 표본을 만들지 않았다. 기본 egui Button과 legacy inspector/action 중 공용 primitive로 이관되지 않은 사용처는 별도 추적 대상이다. OUT info glyph·요약 미디어/속성·정상 시청 전체 검토도 완료가 아니다.
