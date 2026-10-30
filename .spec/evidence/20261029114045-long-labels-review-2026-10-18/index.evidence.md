---
id: evidence:kinekino/20261029114045
title: 긴 다국어 문구 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 긴 다국어 문구 실제 검토

2026-10-18. 소유 가상 모니터3500,100·1440×900에서 r26(PID77076, runtime/review-runs/ui-all-pages-20261018-r26/player.exe)을 하나만 실행했다. LongLabels fixture를 실제 메뉴에서 선택했다. VM·DPI·모니터 구성은 변경하지 않았다.

## 수정 전 관측

- MODEL: 긴 한국어·日本語·English 모델명과 부가 정보를 카드 안에서 줄바꿈했다. 우측 상세도 폭을 유지했고 다운로드 footer가 남았다.
- JOB: 작업·입력·출력 표는 말줄임, 상세 작업명은 줄바꿈, 입출력 readout은 말줄임으로 폭을 유지했다. 세 동작 footer가 유지됐다.
- CARD: 중앙 title과 세 thumbnail title이 한 줄로 렌더돼 좌우가 잘렸다. 기존 단순 painter.text는 줄바꿈을 하지 않았다. 자동 맞춤 안내만으로 이 차이를 해결했다고 판단하지 않는다.

r26은 수정 전 관측이며 캡션 닫기로 종료했다.

## 수정과 자동 검증

공용 artboard_text를 분리해 중앙 정렬 줄바꿈·title/subtitle 그룹·최소50%까지 크기 조절을 구현했다. 원문을 유지하고 최소 크기에서도 넘치면 검토 안내를 표시한다. 공용 readonly 입력과 artwork response는 전체 내용 tooltip을 제공한다. 미디어 및 카드 문서 정책은 변경하지 않는다.

multilingual_title_fits_canvas_and_thumbnail_without_ellipsis regression은 동일 title의 원문 보존·비생략·가로폭·세로영역을132/645 크기에서 확인한다. 실제 색/서체/레이아웃 인수를 대체하지 않는다. 수정 후 실제 CARD LongLabels 재검증은 후속 기록으로 구분한다.

## r27 첫 재검증에서 추가 발견

초기 수정의 fmt/test/clippy/build는 통과했지만 실제 r27(PID114464, runtime/review-runs/ui-all-pages-20261018-r27/player.exe)의 CARD LongLabels에서 줄바꿈된 title이 왼쪽으로 치우쳐 다시 잘렸다. 크기만 확인한 최초 regression은 실제 drawing origin을 다루지 않아 이 차이를 놓쳤다.

중앙 정렬 LayoutJob의 galley.rect는 원점에 대해 좌우로 펼쳐질 수 있는데, 그림 위치에 다시 size/2를 빼서 이중 정렬한 것이 원인이었다. 실제 galley.rect 중심으로 origin을 계산하도록 보완했다. regression에도 canvas/thumbnail의 변환 후 좌우 경계와 중심 일치를 추가했다. r27은 이 문제를 확인한 뒤 캡션 닫기로 종료했다. 후속 actual 재검증 전 최종 해결로 기록하지 않는다.

## r28 최종 재검증

보완 후 fmt/test/clippy/build 통과(77 passed,8 ignored; 외부 미디어/운영체제 의존 검사는 별도). r28(PID64800, runtime/review-runs/ui-all-pages-20261018-r28/player.exe)을 같은 가상 모니터·1440×900에서 실행했다.

LongLabels 메뉴 선택→CARD 진입 후 안정 프레임에서 title이 '장면 예시1'부터 '검토합니다'까지3줄로 중앙 표시되고 subtitle이 별도 아래 줄에 놓였다. 세 thumbnail도 각 title의 시작/끝이 줄바꿈되어 영역 안에 표시됐다. 이전 좌우 잘림과 이중 중앙 정렬은 해당 fixture/폭에서 해결됐다. inspector의 원문 입력·색상표·고정 저장 버튼은 유지됐다. 검토 후 캡션 닫기로 종료했다.

좁은 CARD 창·더 긴 본문/emoji·overflow 경고 상태·실제 저장 이미지의 레이아웃 검증과 font/preset 편집은 아직 남아 있다. 모든 페이지의 모든 LongLabels 상태를 완료한 것은 아니다.

## r29 좁은 창과 상태 전환

새 코드 변경 없이 최종 빌드 r29(PID86540, runtime/review-runs/ui-all-pages-20261018-r29/player.exe)를 소유 가상 모니터3500,100에서 실행했다. 이전 player0개 확인 후 하나만 실행했고 VM·DPI·모니터 구성은 변경하지 않았다.

CARD LongLabels→창 크기 메뉴760×480. 세 thumbnail이 가로 strip으로 바뀌고 title의 시작/끝이 유지됐다. 본문을 내리면 중앙 카드가 전체 표시되고 긴 title과 subtitle이 카드 경계 안에 줄바꿈됐다. 이 창 크기에서도 잘림을 재현하지 않았다.

같은 창에서 Empty로 바꾸면 기존 이미지/title/thumbnail/페이지 숫자가 사라지고 소스 이미지 연결 필요가 표시됐다. 설정을 열면 게시 대상·비율·템플릿이 미선택이며 저장 버튼은 비활성 고정 footer였다. 본문 scroll로7색 한 줄·선택값 연결 필요에 접근했다. Escape는 설정만 닫았다.

NeedsInput으로 바꾸면 디자인 샘플3개가 다시 표시되고 상단 상태는 입력 필요였다. 이는 fixture 구성의 관측이며 모델/경로/실제 준비 조건을 검증한 결과가 아니다. 개별 입력 부족 사유의 typed projection과 실행 명령 연결은 남아 있다. 단순 샘플 재표시를 작업 성공으로 기록하지 않는다.

r29는 캡션 닫기로 종료했다. 이번은 실제 추가 관측으로 코드 변경·새 cargo 검사를 수행하지 않았다. 다른 화면의 좁은 LongLabels·emoji·overflow 경고·실제 출력은 미검증이다.
