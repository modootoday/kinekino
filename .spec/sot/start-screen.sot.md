---
id: sot:kinekino/start-screen
title: 시청 시작 화면 공통 컴포넌트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 시청 시작 화면 공통 컴포넌트

## 책임과 자료

`components::start_screen`이 직접 작성 StartMark SVG·제품명·파일/탐색 동작·키 안내의 시각 조합을 소유하도록 이관한다. 아직 구현하지 않았다. 파일 선택/브라우저/네이티브 재시도 정책은 기존 caller에 남고 컴포넌트는 클릭 의도만 반환한다. 제품명은 단일 제품 설정에서 전달한다. 실제 상태 메시지와 재시도/웹 복귀는 선택적 슬롯으로 전달하며 예시 성공/미디어를 생성하지 않는다.

기준은 `.spec/artifacts/design/v2/viewing/native-player-start-v2.png`, 표식 확대는 [r163](../../../verification/presentation/viewing-start-detail-audit-2026-10-19.md), 실제 main1180 패널 전환은 [r166](../../../verification/presentation/viewing-main-panel-review-2026-10-19.md)이다. 시안 전체와 표식4배를 다시 직접 확인했다. reference1586×992의 caption약44와 actual1180×760의caption32를 기준으로 논리 단위를 대조하되 렌더 blur/왜곡 글꼴을 복제하지 않는다.

## 배치 계약

- caller가 제공한 실제 영상 작업 영역 안에서 Taffy Flexbox 중앙 정렬한다. 회차 패널260이 열리면 남은 작업 영역을 사용한다. overlay/popup이 열렸다는 이유로 viewport 전체 중심을 사용하지 않는다.
- 시작 표식/제품명/동작 행/키 안내를 서로 다른 row로 구성한다. content-sized column을 중간의 grow 영역에 놓으며 `available_height×0.46−80` 같은 수동 spacer를 제거한다. 작은 높이는 최소 padding과 내부 스크롤로 caption/transport 접근을 유지한다.
- 버튼 행은 동일 폭 두 동작을 사용하고 좁아지면 wrap한다. 아이콘·label·focus 영역을 control_icon/action 또는 공용 Button 책임으로 유지한다. 상태 메시지 긴 줄은 wrap하며 버튼 위치나 caption을 밀지 않는다.
- 공용 palette/semantic색·border·radius·space·text 토큰을 사용한다. 표식 크기는 Tailwind 범주의96(size24)부터 실제 대조하고, 기존100 지역값을 이름만 붙여 유지하지 않는다. 새 space24·행/버튼 크기도 실제 대조 후 공용 토큰에 정의한다. 최종 치수 확정은 실행 검토 후 기록한다.
- SVG circle/path는 표식 전용 optical proportion이다. 현재 viewBox100/circle47/stroke1.8은 시안보다 가늘다. 원형과 삼각형 stroke/cap/join을 함께 검토하고 일반20px아이콘 전체 stroke를 바꾸지 않는다. 최초 후보 두께는 기존 대비 증가하되 최종값은 같은 논리 크기의 실제 표시 후 확정한다.
- 제목은 승인된 Pretendard heading23·semibold 계약을 유지한다. raster 시안의 폰트 모양을 기준으로 별도 폰트를 혼합하지 않는다. 키 안내는 small/secondary를 사용한다.

## 입력과 실제 인수

Ctrl+O는 기존 공통 입력 처리, 파일/탐색 클릭은 caller로 반환한다. 패널 P/Escape, 볼륨 popup·Home/End, caption 접근과 focus ring이 겹치지 않아야 한다. 데모 표식은 실제 재생 버튼으로 위장하지 않는다.

새 고정 EXE에서760/1180, 회차패널 닫힘/열림, 긴 제품명·상태, 시청↔제작 복귀를 실제 대조한다. mark 원/삼각 비례·stroke·center, 제목baseline·행간, 버튼양끝·gap·radius·focus, 키안내를 각각 판정한다. fmt/test/clippy와 실제 관측을 구별하며 source lease 중 미구현 계획을 완료로 표기하지 않는다.
