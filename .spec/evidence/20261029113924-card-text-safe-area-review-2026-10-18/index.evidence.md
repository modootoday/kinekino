---
id: evidence:kinekino/20261029113924
title: CARD 문구와 페이지 표시 예약 영역 재검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD 문구와 페이지 표시 예약 영역 재검토

2026-10-18. [r54 번호 검토](card-pagination-review-2026-10-18.md)에서 LongLabels 첫 줄과 번호 badge 사이 여유가 작은 것을 확인한 후 수정했다. 기준 cards.png와 기존4배 확대 파생물 card-number-r54-4x.png를 유지한다.

## 공용 수정

page_marker::text_bounds는 badge24+여백4×2를 thumbnail 기준132에 대한 비율로 환산해 image 상하 영역을 예약한다. 문구 폭84%와 그룹 중앙을 유지한다. canvas와 thumbnail이 같은 비율을 사용한다. Artboard::show_with_text_bounds의 선택 callback으로 카드만 이 bounds를 사용하고, 기본 show/paint 호출과 FRAME 등의 기존 계약은 유지한다.

artboard_text의 font scale은 원래 image 폭을 기준으로 계산하고 줄바꿈 폭과 허용 높이는 명시 bounds에서 얻는다. 원문/ellipsis/최소50%/넘침 안내 계약을 유지한다. painter는 bounds와 부모 clip을 교차한다. 긴 문구를 임의로 삭제하거나 그림 영역 밖으로 흘려보내지 않는다.

새 회귀 `card_copy_stays_between_number_badge_and_page_counter`는 실제 번들 폰트를 설치한 egui에서132/223/645 크기의 한글·日本語·English 제목을 layout한다. 원문 유지/비말줄임·그룹 높이·badge 아래 top·counter 위 bottom을 확인한다. 기존 다국어 중앙 정렬 검사도 통과했다. 이 시험은 실제 raster 관측을 대신하지 않는다.

## 실제 재검토

실행 `runtime/review-runs/ui-card-text-safe-20261018-r55/player.exe`, PID102044, --ui-demo DesignSample. 기존 가상 모니터3500,100 위치에서1440×900/760×480을 사용했다. 크기는 기존 데모 선택 기능으로 조절했고 VM·DPI·모니터 구성은 변경하지 않았다.

- 1440 기본: 짧은 제목/본문의 중앙 그룹과 thumbnail 번호·canvas1/3이 유지됐다.
- 1440 LongLabels: 제목이 여러 줄로 들어가고 thumbnail 첫 줄이 번호 badge 아래에 여유를 두었다. canvas 문구와1/3도 분리됐다.
- 760 LongLabels:132단위 가로 thumbnail 세 개에서 첫 줄과 badge 사이 여유를 확인했다. 기존 r54의 촘촘한 배치가 완화됐다.
- 본문 스크롤 후223단위 canvas의 긴 제목·본문 전체와 하단1/3을 함께 관측했다. 제목/본문은 카드 중앙, counter는 하단에 분리됐고 고정 푸터에 번지지 않았다. 이 표본에서 넘침 안내나 ellipsis는 관측하지 않았다.

카드 전환 직후 이전 frame이 반환돼 후속 안정 상태를 관측했다. 입력 실패로 단정하지 않는다. 화면 근거는 반환 이미지의 직접 관측이며 저장 screenshot을 주장하지 않는다. 캡션 닫기 후 player/mpv 조회가 비어 있음을 확인했다.

## 검사·남은 범위

최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드 통과. sample 문구/크기에 대한 예약 영역 검증이며 임의 길이/글꼴/언어 전체를 보장하지 않는다. 최소 글자 크기에 도달한 실제 넘침·경고와 marker의 공존, 비정사각 카드와 사용자 문서 좌표/출력 revision, FRAME/A2D 등 기본 Artwork의 이번 변경 후 별도 raster 회귀는 남는다. 시안의 선택 외곽 inset·두꺼운 주황 테두리·기본 중성 border·preset/템플릿/alt/저장 등 r54의 다른 미완료는 유지한다. CARD 전체 일치 완료가 아니다.
