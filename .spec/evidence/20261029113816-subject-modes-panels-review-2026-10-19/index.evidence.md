---
id: evidence:kinekino/20261029113816
title: OBJ 보기·패널 후속 실행 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ 보기·패널 후속 실행 검토

2026-10-19. subject.png 전체와(930,157,241,48)4배 확대를 검토했다. 파생물은 `runtime/review-runs/subject-modes-r134-4x.png`. r134는 r133 바이너리 복사본이며 새 Rust 변경이 없다. 실행 경로 `runtime/review-runs/ui-subject-20261019-r134/player.exe`, 소유 PID101604. 기존 가상 모니터3500,100을 사용했고 VM·DPI·모니터 구성을 변경하지 않았다.

## 관측

- 1440×900: 합성 기본 화면에 청록 contour·반투명 채움·bounds/white handles와 좌측5도구·우상단3보기 overlay가 표시된다. 원본 클릭은 mask/bounds를 숨긴다. Right로 마스크를 선택하면 검정 바탕/흰 silhouette, End로 합성을 선택하면 원본+청록 mask로 돌아왔다.
- 선택 보기에는 amber 외곽선이 있지만 키보드 이동 직후 별도의 pale focus 외곽선은 시각적으로 구분되지 않았다. 공용 segmented의 명시적 선택 stroke와 별도 focus 책임을 후속 확인해야 한다.
- 760×480: 도구 rail과3보기는 영상 안에서 서로 겹치지 않았다. 영상 하단 일부는 본문 스크롤 아래에 있다. 마스크 클릭→설정 drawer→Escape 후 마스크 보기와 선택선이 유지됐다.
- inspector는 텍스트 카드3개를 표시하며 subject.png의 thumbnail·사용자 label·메뉴·선택 amber 행과 다르다. 소스상 `snapshot.table.rows`에서 목록을 만들지만 mask snapshot에는 하나의 track만 있다. 현재 목록과 mask track의 대응/선택 동기화를 구현했다고 볼 수 없다.
- caption으로 검토 창을 종료했고 후속 CIM에 player.exe/mpv.exe가 없었다.

## 다음 구현 경계와 미검증

목록의 시각 행과 mask track을 연결하는 UI read projection이 필요하다. stable track_id·optional label/thumbnail·review 상태를 구분하고, 대응이 확인되지 않은 table row에 mask를 임의로 붙이지 않는다. 공용 행은 thumbnail/label/status/menu slot/선택/focus를 분리한다. 라벨이 없는 track은 이름을 만들어 사용자 별명으로 표시하지 않는다. 새 SDK 데이터가 필요한 범위는 SDK 세션과 조율하며 UI 컴포넌트와 투영은 이 세션이 소유한다.

전체 OBJ 시각 일치, 실제 추론·마스크 편집, 목록과 canvas 선택 동기, 모든 focus/hover/pressed 상태, Empty/NeedsInput 후 보기 수명,980/1180,실제 track 다수는 미검증이다. 이번 작업은 실행 관측과 후속 계약만 추가했으며 검사 통과를 새로 주장하지 않는다.
