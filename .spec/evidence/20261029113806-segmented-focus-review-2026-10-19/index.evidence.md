---
id: evidence:kinekino/20261029113806
title: 공용 보기 선택·포커스 분리 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 보기 선택·포커스 분리 검토

2026-10-19. [r134](subject-modes-panels-review-2026-10-19.md)에서 OBJ 보기 선택 stroke와 키보드 focus가 별도로 구분되지 않았다. segmented::show에 enabled/has_focus일 때 내부 FOCUS색·THIN선·CONTROL radius 링을 추가했다. inset은 FOCUS두께×2이며 선택선·버튼 크기·grid 배치는 그대로다. SDK 값이나 저장 동작은 바꾸지 않았다.

## 실제 실행

r135 `runtime/review-runs/ui-segmented-focus-20261019-r135/player.exe`, PID97304. 기존 가상 모니터3500,100·1440×900에서 검토했다. VM·DPI·모니터 구성 변경 없음.

- OBJ 원본 클릭: amber 선택선 안의 별도 얇은 pale 링을 확인했다. Right→마스크에서 검정/흰 마스크와 선택·focus 링이 함께 이동했다.
- Tab→합성: 마스크는 amber 선택 상태를 유지하고 합성은 중성 바탕/외곽선 안에 pale focus만 표시됐다. Enter 후 합성이 선택되고 pale 링도 유지됐다. 선택과 focus가 다른 항목에 놓인 실제 표본이다.
- A2D 원본 클릭→Right 알파: 동일 공용 링과 선택선이 보였고 canvas가 원본→흰 alpha로 전환됐다. 기존 레이어 선택과 정보는 유지됐다.
- OBJ 첫 진입 즉시 한 캡처에 이미지 로딩 spinner가 보였고 후속 캡처에서 원본/filmstrip이 정상 표시됐다. 이를 첫 프레임 완성으로 판정하지 않는다.
- caption 닫기 후 CIM 조회에 player.exe/mpv.exe가 없었다.

## 검사·한계

cargo fmt --check, 전체 cargo test(lib84pass/1ignore 및 기본 통합·doc test), cargo clippy --all-targets -- -D warnings, player 빌드가 모두 통과했다. UI 전용 target `runtime/parallel-build/ui`와-j1을 사용했다.

이번 수정 후760/980/1180·disabled CARD/분석 mode·Empty·모든 hover/pressed·스크린리더는 실제 재검토하지 않았다. 시안은 정적 선택 상태 근거이며 focus 추가는 프로젝트 접근성/상태 구분 규칙에 따른다. OBJ 목록 thumbnail/track 대응과 전체 화면 시각 차이는 남는다.
