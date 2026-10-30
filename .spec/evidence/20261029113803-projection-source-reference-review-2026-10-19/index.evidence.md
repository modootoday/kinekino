---
id: evidence:kinekino/20261029113803
title: PROJ 원본 참조 카드 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PROJ 원본 참조 카드 실제 검토

2026-10-19. 소유 가상 모니터의 기존 위치(3500,100)에서 실행했다. VM·DPI·모니터 구성은 변경하지 않았다.

## 기준과 발견

PROJ-03, 기준 `.spec/artifacts/design/v2/studio/projection3d-refined.png`의 (250,160,285,190)을 4배 확대했다. 파생물은 `runtime/review-runs/projection-source-r103-4x.png`다. 시안은 좌상단에 제목·테두리·원본 비율 이미지로 구성된 참조 카드를 둔다.

r103(PID104496,1440×900) 실제 2.5D 깊이 화면에는 이 카드가 없고 기울어진 평면만 있었다. 공용 projection_scene에 카드 책임을 추가했다. 토큰의 panel 색·border·radius·inset을 사용하고 원본 비율을 보존한다. DesignSample 자산만 사용하며 선택 파일 미리보기로 표시하지 않는다. 우상단 도구와 함께 놓을 폭이 부족하면 카드가 도구 아래로 이동한다.

## 수정 후 관측

r104(PID13036), 최종 빌드의 실제 player.exe:

- 1440×900: 2.5D 깊이/투영 패키지 양쪽에 원본 카드 표시. 카드와 우상단 도구가 겹치지 않았다. 제목·이미지·얇은 테두리·둥근 모서리를 확인했다.
- 패키지 mask.png 클릭 후 Down으로 depth.png 선택과 선택 안내가 바뀌었다. 원본 카드 이미지는 선택 파일 결과로 바뀌지 않았다.
- 760×480 패키지: 원본 카드와 도구가 겹치지 않고 canvas 안에 표시됐다. 설정 drawer 열기 → Escape 닫기 후 투영 패키지 탭과 카드가 유지됐다.
- 같은 크기의 빈 상태: 원본 카드와 평면 이미지가 사라지고 입력/깊이 연결 안내가 표시됐다.
- r103/r104는 각각 caption 닫기로 종료했다. 종료 후 player.exe/mpv.exe 프로세스 조회 결과가 비어 있었다.

## 검사와 제한

초기 수정은 Taffy sizing pass의 일시적 좁은 폭에서 음수 폭 오류로 테스트가 실패했다. 카드 내측 여백보다 작은 폭에서는 표시를 건너뛰도록 수정한 후 최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. lib 70 passed/1 ignored이며 기존 외부 미디어 의존 ignored 항목은 실행하지 않았다.

원본 카드 하나의 누락을 수정한 결과다. 시안과 실제 자산 내용은 다르며 scene 전체 높이·레이어별 분리·깊이 모드·카메라 slider·깊이 legend·파일 glyph/상세·실제 GPU 깊이 처리와 파일 미리보기는 완료가 아니다. 980/1180 실제 표시, 긴 문구, 카드가 도구 아래로 이동하는 임계 폭, 전체 hover/focus/pressed와 실제 미디어는 이번에 확인하지 않았다. 전체 PROJ 시각 일치로 확대하지 않는다.
