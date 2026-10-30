---
id: evidence:kinekino/20261029113758
title: 간결한 공용 도구 모음 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 간결한 공용 도구 모음 실제 검토

2026-10-19. SEG/영상 preview/PROJ 공용 도구 모음. skip-settings.png 전체와 작업 영역(1020,770,144,80)4배 확대를 비교했다. 파생물 `runtime/review-runs/segment-actions-r111-4x.png`. 기존 구현은 아이콘 버튼과 이름 label을 별도 위젯으로 배치했다. SEG 삭제에는 지우개, PROJ 원본 프레임에는 카메라 기호를 사용했다.

## 변경과 발견

toolbar::unavailable를 이미지와 이름을 묶은32높이·20SVG의 프레임 없는 버튼으로 변경했다. Taffy FlexWrap은 동작 전체를 줄바꿈하며 동작별 연결 필요 사유를 같은 response에 둔다. SEG 삭제는 Delete, PROJ 원본은 FileImage SVG로 교체한다. 실제 명령은 비활성 상태다.

r111 PID58608,1440 demo SEG에서 실제 재생 도구와 하단 편집 도구를 확인하는 중 겹침과 반복 Taffy recalculation 경고(166→725 frames)를 발견했다. 동일 부모의 두 도구 모음이 같은 ID를 사용한 회귀다. 실행본은 caption으로 종료했다. 호출자별 stable ID(segment-edit/preview-transport/projection-view)를 계약에 추가하고 서로 다른 두 도구 모음이240/760/1440 폭에서 discard를 끝내고 idle1pass로 안정되는 회귀 검사를 추가했다. 반복 경고를 정상으로 취급하지 않는다.

## 최종 재검토

r112 PID119376,최종 빌드 실제 player.exe.1440 SEG에서 재생/볼륨/전체화면과 하단 추가/수정/삭제의 아이콘·이름이 겹치지 않았고 연속 캡처에서 반복 recalculation 경고가 보이지 않았다. Delete SVG가 휴지통으로 표시됐다.1440 PROJ에서 원본 프레임의 그림 SVG와 나머지 두 동작이 분리되지 않고 표시됐다.760 SEG에서 내부 스크롤로 표와 편집 도구에 도달했고 아이콘·이름의 결합과 간격이 유지됐다. 설정 drawer 열기→Escape 후 스크롤 위치와 도구가 유지됐다. 이 관측을 모든 화면의 장시간 성능 측정으로 확대하지 않는다.

최종 cargo fmt --check, cargo test(lib71 passed/1 ignored 및 CLI/통합 검사), cargo clippy --all-targets -- -D warnings, cargo build --bin player 통과. 새 회귀 검사는 두 도구 모음의 재계산 안정과 idle1pass를 검증한다. 외부 미디어 의존 ignored 항목은 실행하지 않았다.

r111/r112는 기존 가상 모니터 위치3500,100에서 한 개씩 실행하고 caption으로 종료했다. 후속 player.exe/mpv.exe 조회가 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 미검증

편집 명령·실제 source transport는 연결되지 않았다. 삭제 SVG 수정은 시안 표 내부의 행별 수정/삭제 동작 배치를 완성한 것이 아니다. 모든 소비 화면·문구·hover/pressed·키보드 접근성과 실제 미디어는 별도다.
