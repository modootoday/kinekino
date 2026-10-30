---
id: evidence:kinekino/20261029113917
title: 좁은 제작 창의 캡션 cycle 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 좁은 제작 창의 캡션 cycle 실제 검토

크기 preset 혼동은 [r116 후속](review-size-control-review-2026-10-19.md)에서 현재 논리 크기/최대화 표시와 동일 preset 복원으로 수정하고 실행 확인했다. 아래 r113 관측은 당시 상태로 보존한다.

2026-10-19. r113 PID109988, `runtime/review-runs/ui-studio-caption-20261019-r113/player.exe`. 이전 r112 최종 빌드를 복사해 실행했으며 이번에는 Rust 코드를 변경하지 않았다. 기존 캡션 기준과 확대 근거는 [창 규칙](../../../../.agents/rules/window-chrome.md), [일반 시청 r97](caption-cycle-review-2026-10-18.md)을 따른다.

## 관측

- 데모 SEG를1440에서760×480으로 변경했다. 세 캡션이 우측 끝에 붙고 탐색/설정과 겹치지 않았다.
- 최대화 버튼 클릭 → 안정된 캡처3840×2112,원점3440,0. 복원 SVG 클릭 →760×480,원점3500,100. OS 애니메이션 중간 캡처는 두 크기가 겹쳐 보였으므로 최종 배치 증거로 사용하지 않았다. 전환 직후4frame recalculation 경고가 잠시 보였고 안정된 후 캡처에서는 보이지 않았다. 지속 성능 측정은 아니다.
-760 설정 drawer를 연 뒤 최소화 버튼 클릭. 캡처 API가 'window is minimized'를 반환했다. 같은 returned window를 activate/get_window로 복귀했을 때 drawer와 SEG가 유지됐다. Escape로 drawer를 닫았다.
- Tab 순서는 데모 제목 링크 → 회차 예시 → 시나리오 → 검토 크기 → 최소화 → 최대화였다. 최소화/최대화 각각 내부 focus outline을 확인했다.
- 최대화에 focus를 둔 채 Space →3840×2112,복원 glyph와 focus 유지. 같은 focus에서 다시 Space →760×480,원점3500,100,최대화 glyph와 focus 유지.
- 다음 Tab으로 닫기 focus outline 확인 →Enter로 종료. 후속 player.exe/mpv.exe 조회 결과가 비어 있었다.

## 범위와 남은 차이

VM·DPI·모니터 구성을 변경하지 않았다. 최대화는 기존 가상 모니터 작업 영역을 사용했다. 데모의 검토 크기 선택기760×480 표시는 최대화 중에도 복원 목표 크기 preset으로 남았다. 이를 실제 현재 창 크기 표시로 오해할 수 있으므로 검토 도구의 preset/실측 치수 구분은 후속이다.

최대화된 SEG에서는 preview 높이가320으로 유지돼 넓은 작업 영역 대비 영상이 작고 여백이 크다. 캡션 동작 성공을 전체 최대화 화면의 시각 일치로 판정하지 않는다. 실제 미디어·일반 제작의 긴 제목·다른 탭·980/1180·최소화 Space·pressed 지속 상태·screen reader는 이번 범위에서 미검증이다. 읽기 전용 실제 조작 검토와 문서 기록만 수행했으므로 Rust 검사 재실행은 하지 않았다.
