---
id: evidence:kinekino/20261029114056
title: A2D 레이어 선택·키보드 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A2D 레이어 선택·키보드 검토

2026-10-18. A2D-05a/b. 시안 asset2d.png의 레이어 영역(299,657,509,231)을2배 확대한 `runtime/design-inspection/a2d-layers-r64-2x.png`에서 eye/thumbnail/label/lock/grip의 배치와 중성 행을 확인했다. 탐색의 그림 모양 아이콘은 시안과 일치하므로 이름만 보고 전역 자산을 변경하지 않았다.

## 수정 전 실제 관측

r64(PID120444,1440×900, `runtime/review-runs/ui-layer-navigation-20261018-r64/player.exe`)에서 인물 행 클릭 후 Down을 눌러도 인물 선택·인물 bounds·검사기 정보가 유지됐다. 키보드 선택 경로가 없다는 소스 관측과 일치한다. 소유 창 닫기로 종료했고 player/mpv 조회는 비어 있었다.

## 수정 책임

LayerRow의 상호작용 ID를 stable layer_id에 연결한다. 선택은 중성면과 amber 좌측선, 키보드 focus는 내부 외곽선으로 분리한다. A2D 목록은 앞 레이어가 위에 오도록 SDK 역순을 유지하며 최신 Taffy response를 index별로 보존하고 공용 keyboard_selection의 Up/Down/Home/End를 적용한다. 선택 ID를 기존 canvas/검사기 경로에서 사용한다. eye/lock/grip 상태와 layer 순서·asset·SDK 정책은 변경하지 않는다.

## 수정본 실제 실행

r65(PID65700, `runtime/review-runs/ui-layer-navigation-20261018-r65/player.exe`)에서 확인했다.

- 1440×900: 인물 클릭 후 Down으로 배경 선택/포커스가 이동하고, canvas 경계가 전체 원본으로 바뀌며 검사기 bounds가0,0,1672,941로 표시됐다. Home으로 인물에 복귀하면 bounds450,60,560,840과 pivot730,480이 복원됐다. 실제 파일/레이어 mutation이 아닌 보기 선택이다.
- 760×480: 인물 행 클릭→End가 마지막 배경을 선택하고 목록 행을 보이도록 스크롤했다. 설정 열기→Escape 후 배경 선택 표시선과 본문 위치가 유지됐고 focus 외곽선은 설정 버튼으로 옮겨진 상태와 구분됐다. 배경 행 재클릭→Up은 인물 행과 인물 bounds로 복귀했다.
- 760 Empty: 이전 이미지/선택선/레이어 목록/bounds가 사라지고 프레임 또는2D 에셋 선택·표시할 레이어 없음·원본 프레임 연결 필요 안내와 비활성 도구가 남았다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드가 통과했다. headless 전체 fixture 통과를 모든 수동 시각 상태의 완료로 확대하지 않는다.

소유 r65 창 닫기 버튼으로 종료했고 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 남은 범위

LongLabels/NeedsInput·980/1180·세 개 이상 레이어와 변경된 revision의 선택 복구는 이번 실제 표본 밖이다. 실제 레이어 숨김/잠금/정렬·마스크 편집/출력은 미연결이며 이번 선택 기능을 편집 지원으로 표시하지 않는다. 시안3레이어와 현재2레이어 fixture 차이·checker·brush 설정·mode/zoom 외형도 별도 남은 범위다.
