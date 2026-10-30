---
id: evidence:kinekino/20261029113738
title: MODEL 연속 그룹 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 연속 그룹 시각 검토

2026-10-19. MODEL-01b 후속. 기준 `.spec/artifacts/design/v2/studio/models-refined.png` 전체와 (290,240,790,450)2배 파생물 `runtime/review-runs/model-groups-r118-2x.png`를 다시 비교했다. 시안은 하나의 얇은 외곽선과 수평 구분선 안에 기능별 헤더가 이어지지만 r120은 개별32높이 둥근 상자와 간격을 사용했다.

공용 Accordion에 group 프레임을 추가했다. 외곽선1·radius5·PANEL 배경, 기능 헤더48높이 토큰, 행 간격0을 적용한다. 내부 헤더의 개별 border/radius/hover 확장을 제거하고 구분선을 사용한다. native 상태와 명시 ID, 클릭 focus·Space/Enter·Tab을 유지한다. 실제 카탈로그가 기능별로 분류되지 않았으므로 후보 영역은 별도로 유지하며 시안의 임의0개·설치 checkbox를 추가하지 않는다.

## 실행 관측

r121 PID108596의1440×900 MODEL에서 개별 헤더 사이의 빈 간격과 둥근 테두리가 제거되고 하나의 외곽선/수평 구분선으로 이어진 여섯 행을 확인했다. SVG chevron·기능 glyph가48높이 행 중심에 정렬됐다. 음성 인식 오른쪽 빈 영역 클릭으로 본문이 열리고 내부 focus 외곽선이 표시됐으며 Space로 접혔다. 후보 영역은 별도로 유지됐다.

760×480에서 후보 영역을 오른쪽 빈 공간 클릭으로 접은 뒤 음악 그룹 클릭/Space로 펼침·접기를 확인했다. 스크롤 아래에서 여섯 행과 깊이 마지막 행 하단에 도달했다. 설정 drawer를 열고 Escape로 닫은 뒤 그룹 접힘과 목록 스크롤 위치가 유지됐다. caption 닫기로 종료했다. 기존 가상 모니터에서 수행했으며 VM·DPI·모니터 구성은 변경하지 않았다.

cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, cargo build --bin player 통과. 라이브러리71통과/1ignored이며 외부 미디어 요구 ignored 검사는 실행하지 않았다.

## 남은 범위

기능별 실제 모델 분류·설치/기본 모델 capability, 후보를 기능 그룹 안에 연결하는 데이터 계약, 시안의 후보 행 전체 구성은 남아 있다. 이번 변경으로 전체 MODEL 일치를 주장하지 않는다.

Empty/LongLabels,980/1180,pressed와 스크린리더는 이번 실행에서 확인하지 않았다. 외곽 모서리와 첫/마지막 행 hover 배경의 곡률은 확대 재검토가 필요하다.

후속 r122에서1440 Empty의 첫 그룹 클릭/Space와 선택 안내,1440/980 LongLabels 후보 선택/End·설정 정보를 확인했다. 작은 설정 창이 필드를 숨기는 문제를 발견해 [서랍 크기 검토](settings-drawer-sizing-review-2026-10-19.md)로 이어졌다. 기능별 헤더는 LongLabels fixture에서 길어지지 않으므로 긴 헤더 제목 검증으로 확대하지 않는다. 모서리 hover 확대는 여전히 미검증이다.
