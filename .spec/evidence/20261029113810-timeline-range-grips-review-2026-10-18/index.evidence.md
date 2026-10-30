---
id: evidence:kinekino/20261029113810
title: 공용 filmstrip 범위 손잡이 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 filmstrip 범위 손잡이 검토

2026-10-18. CLIP-12 및 SEG/OBJ 공용 타임라인의 시각 보완. 편집 기능 완료 기록은 아니다.

## 기준·변경

기준 `.spec/artifacts/design/v2/studio/short.png` 전체와 범위596,704,321,88을4배 확대했다. 파생물 `runtime/design-inspection/clip-grip-r49-4x.png`. 시안은 amber 테두리 양끝 중앙에 세로 grip을 두고 있다. 기존 filmstrip은 테두리 아래 원형 점을 사용해 형태·수직 정렬이 달랐다.

공용 filmstrip에서 하단 원형 점을 제거하고12×32 논리 단위의 세로 grip과 얇은 내부 mark를 그린다. 크기는 TIMELINE_RANGE_GRIP_WIDTH/HEIGHT, 색은 ACCENT/TEXT, 모서리는 SMALL, 선은 THIN 토큰을 사용한다. 범위가 좁으면 grip 폭을 제한하고 label이 겹치지 않을 때만 그린다. 범위·grip·label은 track와 부모 clip을 모두 적용한다. 데이터와 구간·PTS 변환은 유지하며 실제 드래그 편집이나 resize cursor를 제공하지 않는다.

## 실제 확인

실행 `runtime/review-runs/ui-timeline-grips-20261018-r49/player.exe`, PID87788. --ui-demo의 DesignSample임을 상단과 본문에 표시한다. 기존 가상 모니터3500,100 위치에서1440×900/760×480을 검토했다. 창 크기는 데모의 기존 크기 선택 기능으로 변경했으며 native 모서리 drag 성공 근거가 아니다. VM·DPI·모니터 구성은 변경하지 않았다.

- SEG1440: 세 구간 양끝 중앙에 세로 grip이 표시되고 아래 표와 간격을 유지했다. 시작0 구간의 좌측 grip은 track 경계에서 절반이 잘려 외부로 번지지 않는다.
- CLIP1440: 영상/보라 자막/녹색 오디오 트랙과52초 재생선이 유지됐다. 영상 구간 양끝 grip과 자막 구간 경계가 같은 x축에 정렬됐다.
- CLIP760: 본문 스크롤 후 세 트랙과15초 눈금·52초 재생선·여섯 grip이 보였다. 본문 잘림 경계와 고정 푸터를 침범하지 않았다.
- OBJ760: 기존 수동 mask/선택 bounds와 tool rail을 유지했다. 스크롤 후 filmstrip의 세 구간과 grip을 확인했다. 스크롤 중 푸터 위에서 잘린 부분은 푸터에 번지지 않았다.
- OBJ Empty760: 이미지·mask·구간·grip·재생선이 제거되고 소스 이미지 연결 필요 placeholder가 표시됐다.

탭 클릭 직후 전환 전 frame이 한 번 반환돼 후속 관찰로 안정 상태를 확인했다. 이를 탭 입력 실패로 단정하지 않는다. 화면은 반환 이미지로 직접 관측했고 별도 저장 캡처를 주장하지 않는다. 캡션 닫기로 종료 후 player/mpv 프로세스 조회가 비어 있음을 확인했다.

## 검사·남은 차이

최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드 통과(80pass/8ignored). 실제 UI 검토와 별개 근거다.

시안은 CLIP 단일 선택 범위를 표시하지만 현재 공통 DesignSample은 검토 구간3개를 표시한다. 임의로 하나를 실제 선택이라고 만들지 않았다. active range/read model 조율, 손잡이 hover/pressed/focus 및 drag 편집, 원본 frame sampling, 트랙 glyph, crop 안전영역, 시안의 더 밝은 orange 및 투명 선택면은 남아 있다. 이번 확인을 CLIP/SEG/OBJ 전체 시각 일치 완료로 확대하지 않는다.
