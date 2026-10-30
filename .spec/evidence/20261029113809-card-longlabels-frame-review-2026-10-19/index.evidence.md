---
id: evidence:kinekino/20261029113809
title: CARD 긴 문구·중간 창 크기 후속 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD 긴 문구·중간 창 크기 후속 검토

2026-10-19. r132는 r131 검토 바이너리를 별도 경로 `runtime/review-runs/ui-card-longlabels-20261019-r132/player.exe`에 복사한 실행이다. 소유 PID100552. 기존 가상 모니터의 시작 위치3500,100에서 실행했으며 VM·DPI·모니터 구성은 변경하지 않았다. 새 Rust 변경은 없다. 프레임의 기준 차이와 구현은 [r131](card-thumbnail-frame-review-2026-10-19.md)을 따른다.

## 실제 관측

- 1440×900 LongLabels: 한국어·일본어·영어가 포함된 긴 카드 제목이 canvas와 세 thumbnail에서 줄바꿈된다. 번호 badge와 하단1/3은 문구와 분리돼 있다. inspector 제목은 한 줄 표시 폭을 넘기며 전체 원문을 한 번에 표시하지는 않는다.
- 1180×760: 세로 thumbnail rail과 canvas를 유지하고 설정은 drawer 진입 버튼으로 전환된다. 긴 제목은 canvas에서 세 줄로 표시됐고 subtitle과1/3이 분리됐다.
- 980×760: 탐색은 icon rail로 축소되며 카드 thumbnail은 세로로 유지된다. 두 번째 thumbnail 이미지 바깥 오른쪽 여백 (188,380)을 클릭하면2/3과 두 번째 제목·본문으로 변경된다. Down으로 세 번째 선택/3/3과 별도 내부 focus를 확인했다.
- 980 drawer: 선택 카드3의 제목과 ‘다음에, 또 다른 곳에서’ 본문을 확인했다. Escape 후3/3과 선택선이 유지됐다.
- 760×480 전환: 세 thumbnail이 가로 strip으로 바뀌며 카드3 선택은 유지됐다. 초기 viewport에서 canvas 일부만 보이지만 본문 영역을 스크롤하면 카드 전체·줄바꿈 제목·subtitle·3/3·샘플 안내에 도달한다. 이를 전체 canvas가 최초 viewport에 맞는다는 근거로 사용하지 않는다.
- 760 drawer: 오른쪽 고정 drawer의 본문만 스크롤해 카드3 제목·본문 필드를 확인했다. footer의 두 저장 버튼과 로컬 저장 안내는 고정됐다. Escape 후 drawer가 닫히고 canvas 스크롤 위치와 카드3 선택이 유지됐다.
- caption 닫기로 검토 창을 종료했고 후속 CIM에서 player.exe/mpv.exe가 없었다.

## 판정과 남은 범위

이번 표본에서 프레임 inset의 새 배치 파손이나 패널 문맥 소실은 발견하지 않았다. 원문 전체는 canvas hover 안내에서도 보였으나 inspector 필드의 전체 원문 접근성·키보드 읽기와 스크린리더는 검증하지 않았다. 임의 비율/실제 CardDocument/200카드/모든 hover·pressed 상태 및 전체 toolbar·inspector의 시안 일치는 여전히 남는다. 좁은 창에서 canvas를 보려면 스크롤이 필요한 현재 배치를 시각 완성으로 판정하지 않는다. 이번 작업은 실행 관측과 문서만 추가했으며 별도 빌드·테스트를 반복하지 않았다.
