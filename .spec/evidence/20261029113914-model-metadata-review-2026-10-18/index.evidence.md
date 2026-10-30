---
id: evidence:kinekino/20261029113914
title: MODEL 검색·중성 메타데이터 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 검색·중성 메타데이터 검토

2026-10-18 MODEL-01. models-refined.png 전체와 검색(289,163,783,57)2×, tag/설치(363,309,673,70)4×를 확인했다. 검색 glyph·얇은 입력 외곽선·별도 filter와 작은 pill의 값 구분이 대상이다. 생성 시안의 미설치·허가/호환 상태를 실제 결과로 복제하지 않는다.

## 변경 전 실행

r86(PID71472), `runtime/review-runs/ui-model-metadata-20261018-r86/player.exe`,1440×900에서 검색은 아이콘 없는 입력뿐이었고 task/precision/install은 같은 행의 일반 텍스트였다. 후보 선택 카드와 inspector는 표시됐다. 캡션 닫기로 소유 실행을 종료했다.

## 공용 책임

unavailable_search는 search SVG20·입력32·border1/radius5를 공유한다. MODEL 상단의 필터160은 비활성·연결 필요이며 실제 검색/필터 능력을 만들지 않는다. badge는 중성 FIELD/BORDER/SECONDARY·최소24·pill12·padding6/4·small11을 사용한다. task/precision/install의 원문을 유지하고 성공/허가 의미의 색이나 새 상태를 부여하지 않는다. 긴 값은 부모 폭에서 truncate+원문 hover이며 badge 사이 wrap을 사용한다. 카드의 stable ID와 선택/포커스/키보드 처리는 유지한다.

초기 컴파일에서 egui0.36 TextEdit::frame이 bool 대신 Frame을 요구해 빈 Frame 객체로 수정했다. 최종 검사는 아래 실행 기록과 구분한다.

## 수정본 실행

r87(PID95848), `runtime/review-runs/ui-model-metadata-20261018-r87/player.exe`,1440에서 search glyph·별도 미연결 filter와 세 metadata badge를 확인했다. 두 번째 카드 본문 클릭→Down은 세 번째 카드와 inspector로 이동했다. 검색 힌트는 icon보다 위로 치우쳐 TextEdit vertical_align(Center)를 추가했다. r87은 캡션 닫기로 종료했다.

r88(PID91336), `runtime/review-runs/ui-model-metadata-20261018-r88/player.exe`에서1440/760 검색 힌트와 icon의 세로 정렬 보완을 확인했다.760 Populated는 세 badge/필터가 들어갔지만 LongLabels에서 앞 badge가 폭을 차지해 뒤 설치 정보가 작은 ellipsis로 밀리는 문제를 발견했다. 통과로 판정하지 않고 badge 최대 폭240 토큰과 Taffy FlexWrap 리스트로 수정했다. r88은 캡션 닫기로 종료했다. 최종 실행 결과는 후속 기록한다.

## 최종 배치 실행 확인 r89

`runtime/review-runs/ui-model-metadata-20261018-r89/player.exe`(PID37296)를 기존 가상 모니터 위치3500,100에서 한 개만 실행했다.1440×900의 검색 glyph/힌트 세로 정렬, 필터 분리, 세 중성 badge를 확인했다.760×480 Populated에서도 카드 안에 표시됐다. LongLabels에서는 앞 두 badge가 첫 줄, 설치 badge가 다음 줄에 놓여 r88의 작은 ellipsis 밀림이 재현되지 않았다. 각 긴 값은 제한 폭에서 말줄임 처리됐다. 원문 hover 표시는 이번 실행에서 확인하지 않았다.

760 LongLabels 첫 카드 본문 클릭→End로 세 번째 카드 선택과 자동 스크롤을 확인했다. 설정 패널에는 세 번째 모델 정보가 표시됐고 Escape로 닫은 뒤 선택과 목록 스크롤이 유지됐다. Empty 전환 후 카드·badge가 사라지고 후보 모델(0)과 비활성 검색/미연결 필터가 표시됐다. 이0은 데모의 빈 목록이며 실제 검색 결과 수가 아니다.

r89는 캡션 닫기로 종료했고 후속 CIM 조회에서 player.exe/mpv.exe가 없었다. VM·DPI·모니터 구성은 변경하지 않았다. 앞선 최종 wrap 수정 후 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings 및 cargo build --bin player가 통과한 바이너리를 사용했다. 이번 재개 구간은 해당 바이너리 실제 검증과 문서 보완만 수행했다.

980/1180, 실제 catalog 연결, hover 원문, 모든 입력/오류 상태의 시각 검증은 남아 있다. 이 기록은 MODEL 검색 영역·배지 배치와 명시한 키보드/패널 범위에 한정한다.

## 남은 범위

기능별 실제 catalog 그룹·개수·모델 상세/HF revision·호환/라이선스 상태·필터/검색0건·실제 설치/자원 계약은 후속이다. badge는 데이터의 검증 성공을 의미하지 않는다. 전체 모델 화면 시각 인수는 아직 완료가 아니다.
