---
id: evidence:kinekino/20261029113930
title: 비교표 라벨 열 배경 후속 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 비교표 라벨 열 배경 후속 검토

2026-10-19. [r138](correction-text-comparison-review-2026-10-19.md)의 글자 폭/높이에만 붙은 label 면 차이를 수정했다. 시안 기준은 동일 correction.png 확대 영역이다. 기존 Taffy sidebar72가 배치를 소유하고, 예약 background shape를 실제 scope response rect에 맞춰 갱신한다. 라벨 면은72폭과 행 전체 높이를 사용하며 THIN 세로 구분선, 첫/끝 행의 좌측 CONTROL 모서리를 적용한다. 텍스트·highlight·row_id 투영은 바꾸지 않았다.

## 실제 관측

r139 `runtime/review-runs/ui-comparison-labels-20261019-r139/player.exe`, PID28548. 기존 가상 모니터3500,100에서 확인했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900 미선택 두 행: 원문/교정안의 배경면 오른쪽 끝이 같은 위치이며 세로 선이 정렬됐다.
- 1440 LongLabels 첫 행 선택: 양쪽 본문이 여러 줄로 늘어나도 label 면이 각 행 아래까지 이어졌다. 글자 주변의 짧고 서로 다른 폭 배경은 사라졌다. 원문·후보와 amber 강조는 유지됐다.
- 760×480 drawer: 원문/교정안 label 열이 같은 폭을 유지했다. 내부 scroll로 후보 끝까지 이동하며 아래 행 전체 면·수평/세로 구분선·외곽 모서리를 확인했다. footer는 고정됐으며 Escape는 drawer를 닫고 같은 교정 화면으로 돌아왔다.
- LongLabels 메뉴 선택 때 도구가 사용자 입력 감지를 반환해 입력을 반복하기 전 fresh state를 다시 관측했다. 이후 메뉴 상태에서 선택을 진행했다.
- caption 종료 후 CIM에 player.exe/mpv.exe가 없었다.

fmt--check·전체 cargo test(lib86pass/1ignore+기본 통합/doc)·clippy all-targets-Dwarnings·player 빌드가 통과했다. UI target runtime/parallel-build/ui,-j1을 사용했다. 이번 수정 후 Empty/일본어 두 번째 행/980/1180·스크린리더는 실제 재검토하지 않았다. 라벨 면 보완의 범위이며 전체 SUB 시각 완성·제안 명령 인수가 아니다.
