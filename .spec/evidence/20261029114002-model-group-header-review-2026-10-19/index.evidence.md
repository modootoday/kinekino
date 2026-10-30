---
id: evidence:kinekino/20261029114002
title: MODEL 그룹 헤더 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 그룹 헤더 시각 검토

2026-10-19. MODEL-01b. models-refined.png 전체와 (290,240,790,450)2배 확대를 비교했다. 파생물 `runtime/review-runs/model-groups-r118-2x.png`. 시안의 chevron→기능 glyph→제목 정렬을 기준으로 공용 Accordion을 구현했다. 기존 기능별 텍스트만 있던 그룹에 Mic/OCR/music/document 직접 작성 SVG와 기존 Subject/Cube를 연결했다.

후보 CollectionState 문구와 실측 행 수는 유지하고 미연결 그룹에 임의0개를 추가하지 않았다. 별도 카탈로그 정책을 만들지 않는다. 전체 폭 클릭·native 펼침 상태와 명시 ID를 유지한다. glyph20/chevron18/최소높이32/간격8 토큰을 공유한다.

r118 PID96576,1440 MODEL에서 그룹 glyph와 정렬을 확인했다. 기본 삼각형이 시안의 가는 chevron과 다른 것을 발견해 공용 ChevronDown SVG와 열림 정도 회전으로 교체했다. r118은 caption으로 종료했다.

r119 PID101716에서1440 화면의 가는 SVG chevron과 기능 glyph를 확인했다. 음성 인식 헤더 오른쪽 빈 영역 클릭으로 본문이 열렸다. 그러나 Space 뒤 재관측에서도 열린 상태를 유지했고 Tab은 제목줄 링크로 이동했다. 클릭이 헤더의 키보드 포커스를 요청하지 않는 문제로 분류하여 공용 Accordion의 header response 클릭 시 request_focus와 토큰 기반 외곽선을 추가했다. r119는 caption 닫기로 종료했으며 다음 실행에서 수정 결과를 검증한다.

r120 PID106056,1440×900에서 수정 결과를 재검토했다. 음성 인식 오른쪽 빈 영역 클릭으로 본문이 열리고 전체 헤더에 amber focus 외곽선이 표시됐다. Space로 접혔으며 Tab은 다음 OCR 헤더로 이동했다. Enter로 OCR 본문이 열리고 chevron이 아래로 향했다. 후보2 카드 본문 클릭과 End 후보3 선택 후 우측 모델 정보 badge 동기화를 확인했다. caption 닫기로 종료했다. 기존 가상 모니터만 사용했으며 VM·DPI·모니터 구성은 변경하지 않았다.

수정 후 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 라이브러리71통과/1ignored이며 외부 미디어를 요구하는 ignored 검사는 이번에 실행하지 않았다.

## 미완료 범위

이번 최종 실행은1440 Populated 표본이다. 다른 폭·Empty/LongLabels의 새 Accordion 입력·pressed·스크린리더는 미검증이다.

실제 기능별 카탈로그 분류/개수, 체크박스의 기본 모델 설정과 설치 capability, 단일 외곽선과 연속된 행 배치·선택 모델 본문 등은 여전히 별도다. 그룹 아이콘 추가를 전체 MODEL 시각 일치로 판정하지 않는다.
