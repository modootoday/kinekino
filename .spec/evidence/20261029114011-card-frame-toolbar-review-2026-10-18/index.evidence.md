---
id: evidence:kinekino/20261029114011
title: CARD·FRAME 공용 도구 모음 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD·FRAME 공용 도구 모음 검토

2026-10-18. CARD-01/05 및 FRAME-02/03/06. cards.png의(247,123,664,59),raw-frame.png의(418,808,588,42)를3배 확대했다. 파생물은 `runtime/design-inspection/card-tools-r66-3x.png`와 `frame-tools-r66-3x.png`다.

## 요소와 구현

카드는 두 장 겹침 복제·휴지통·곡선 undo/redo가 시안에 있다. 기존 복제가 image glyph인 Layers, 삭제가 Eraser, undo가 Back이며 redo는 누락됐다. Copy SVG를 직접 작성하고 Delete/Undo/Redo 공통 자산을 조합한다. 편집3동작과 history2동작을 구분선으로 나눈다. 프레임 두 하위 화면은 이전/고정/다음/원본 비교의 기존 의미를 유지하며 icon+label을 공통 Button 안에 넣는다. 프레임 시안은 버튼별 외곽선, 카드는 toolbar 외곽선/아이콘 중심이다. 구현은 승인된 공통32 높이·20 icon·간격8·얇은 border/radius 조합으로 일관화하며 개별/전체 frame 차이는 미세 조정 대상으로 남긴다.

실제 문서 명령·history·neighbor PTS가 없으므로 모두 비활성/이유를 유지한다. 카드 선택/문구/페이지와 frame 선택 계약은 변경하지 않는다. 이전 도구 모음 관측은 r52/r55 근거이며 수정본 실행 증거가 아니다.

## 실제 실행

r66(PID64280, `runtime/review-runs/ui-card-frame-tools-20261018-r66/player.exe`)을 기존 가상 모니터에서 실행했다.

- 1440×900 CARD: 추가/복제/삭제와 구분선 뒤 undo/redo 다섯 버튼이 표시됐고 Copy·Delete·곡선 화살표 SVG를 확인했다.
- 1440 FRAME 원본/편집: 원본은 인접 thumbnail 아래, 편집은 canvas 아래·thumbnail 위에 같은 네 버튼이 표시됐다. 미연결 PTS 안내는 유지됐다.
- 760×480 FRAME 두 탭: 편집 네 버튼이 한 줄에 들어왔고 원본도 본문 스크롤 후 네 버튼 전체에 접근했다. label/우측 원본 비교 버튼이 잘리지 않았다.
- 760 CARD: 다섯 버튼과 그룹 구분선이 한 줄에 들어왔다. 두 번째 thumbnail 클릭→Right 후 세 번째 선택·focus, canvas '장면 예시3'·counter3/3이 함께 변경됐다. 설정 열기→Escape 후 세 번째 카드와 본문 scroll 위치를 유지했다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드가 통과했다. 이것을 전체 화면의 시각 일치나 비활성 명령 성공으로 확대하지 않는다.

소유 r66 창 닫기 버튼으로 종료했고 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

실제 명령/활성 상태·긴 번역 wrap·Empty/NeedsInput·FRAME 자체 설정/Escape 재검토·CARD 외곽 toolbar 면/버튼별 면의 세부 차이는 별도다. 실제 wrap을 유발하지 않은760 표본을 긴 번역 wrap 검증으로 표시하지 않는다. 전체 두 화면의 시각 인수를 의미하지 않는다.
