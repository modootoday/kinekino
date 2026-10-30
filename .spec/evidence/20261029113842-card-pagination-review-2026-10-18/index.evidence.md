---
id: evidence:kinekino/20261029113842
title: CARD-04/06 번호·페이지·키보드 선택 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD-04/06 번호·페이지·키보드 선택 검토

2026-10-18. 기준 cards.png 전체와236,350,184,190을4배 확대했다. 파생물 `runtime/design-inspection/card-number-r54-4x.png`. 시안은 thumbnail image 좌상단의 어두운 번호 badge·현재 amber outline과 큰 canvas 안쪽 하단 current/total을 사용한다. 기존 소스는 번호와 current/total을 이미지 밖의 label로 표시했다.

## 공용 변경

page_marker::badge/counter에서 페이지 의미를 공유한다. badge24·inset4·CONTROL radius·CHROME/TEXT·body13, counter bottom16은 토큰 기반이다. 부모/image clip을 함께 적용한다. 기존 Artboard::fitted_rect를 사용해 focus 선도 image 내부에 그린다. 순서/선택 데이터는 기존 DesignSample item을 사용하며 가짜 실제 카드 문서를 만들지 않는다. keyboard_selection을 조합해 세로 rail은 위/아래,760 가로 strip은 좌/우로 이동한다. Taffy 재계산 시 응답은 item별 슬롯에 저장한다.

## 실제 실행

`runtime/review-runs/ui-card-pagination-20261018-r54/player.exe`, PID52632, --ui-demo의 명시적 DesignSample. 기존 가상 모니터3500,100에서1440×900/760×480을 검토했다. 크기는 데모의 기존 선택 기능을 사용했다. VM·DPI·모니터 구성 변경 없음.

- 1440 기본: image 좌상단1/2/3 번호 badge, 현재 amber 선택, canvas 내부1/3 표시를 확인했다.
- 두 번째 thumbnail 클릭: 선택 outline/focus·큰 canvas 제목/본문·검사기 제목/본문·2/3이 같은 카드로 바뀌었다.
- Down: 세 번째 카드로 이동해 위 표시가 모두3번째/3/3으로 바뀌었다.
- 760: 세로 rail이 가로 strip으로 바뀌고3번째 선택이 유지됐다. thumbnail 클릭 후 Left로2번째 선택이 이동했다. 스크롤 후 canvas 안쪽2/3을 확인했다. 고정 푸터로 번호 표시가 번지지 않았다.
- Empty760: thumbnail badge/선택/current-total이 제거되고 소스 이미지 연결 필요 placeholder만 남았다.
- LongLabels760: 긴 한글·日本語·English 문구와 번호 badge를 함께 관측했다. 첫 줄과 badge 사이 여유가 매우 작아 예약 영역/문서 layout 보완이 필요하다. 이 상태를 시각 일치 통과로 표시하지 않는다.

화면 근거는 반환 이미지의 직접 관측이며 별도 저장 screenshot을 주장하지 않는다. 캡션 닫기 후 player/mpv 조회가 비어 있음을 확인했다.

## 검사와 남은 차이

최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드 통과. 실제 UI 근거와 구분한다.

시안의 thumbnail selected 외곽 inset/더 두꺼운 주황 테두리·중성 기본 테두리, 긴 문구 badge 안전영역, 캔버스 text/document layout, 실제 preset/템플릿 전환,4:5/3:4/9:16,200카드 rail·드래그·저장·alt 편집은 미완료/미검증이다. 현재 표지/본문/마지막은 검사기 선택 역할이며 임의로 상위 탭을 추가하지 않았다. 일부 번호·키보드 검증을 CARD 전체 시각 일치 완료로 확대하지 않는다.
