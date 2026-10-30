---
id: evidence:kinekino/20261029114033
title: r162 고정 시청 실행의 키보드·패널 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r162 고정 시청 실행의 키보드·패널 검토

2026-10-19. 문서 sourceV2 빌드 대기 중 기존 고정 `runtime/review-runs/ui-proposals-20261019-r162/player.exe` SHA256 `05284974561addb3fbd973e9da8b284c2fe45607e2d09af08a288d62a31b670e`를 새 소유 PID73592로 실행했다. --record-validation --validation-position=3500,100, proposal root 없음,1180×760·무소스·일반 시청이다. 변경 중 S2 source 검증으로 소급하지 않는다.

P를 눌러 우측 회차 목록과 빈 안내가 열리는 것을 확인했다. 영상 작업영역이 패널 앞에서 끝나며 중앙 시작 동작이 그 영역에 맞춰 이동했다. 하단 transport는 전체 폭을 유지했다. 볼륨 아이콘 클릭으로 팝업을 열면 별도 안쪽 focus와80%가 표시됐다. Home은0%·zero SVG·왼쪽 thumb, End는100%·오른쪽 thumb를 표시했다. 첫 Escape는 팝업만 닫고 패널을 유지했으며, 두 번째 Escape는 회차 패널을 닫았다. 원형 thumb·선·focus와 glyph는 실제 관측 범위다.

캡션 닫기 후 fresh list_windows에서 해당 경로 창0개를 확인했다. VM/DPI/모니터 설정 및 사용자 앱을 변경/종료하지 않았다. 실제 음량 적용·재생/전체화면·채워진 회차·760폭·시안 확대1:1 전체는 이 실행의 검증이 아니다. 앞선 시청 시각 계약과 남은 차이를 유지한다. 코드 변경이 없는 고정 바이너리의 실제 회귀 관측이며 새 Rust gate를 실행하지 않았다.
