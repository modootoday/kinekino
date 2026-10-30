---
id: evidence:kinekino/20261029113927
title: SUB 편집 도구 모음 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SUB 편집 도구 모음 검토

2026-10-18. 대상 SUB-09a. 기준 `.spec/artifacts/design/v2/studio/subtitles.png`의(280,674,844,54)을3배 확대한 `runtime/design-inspection/sub-toolbar-r62-3x.png`를 확인했다.

## 요소별 차이와 수정

| 요소 | 기준 관측 | 기존 구현 | 수정 |
|---|---|---|---|
| 버튼 외형 | icon과 label이 같은 얇은 테두리·둥근 버튼 안에 배치 | 작은 icon 버튼과 별도 label | 공용 unavailable_grouped, theme Button·hit32·icon20·gap8 |
| 병합 | 두 대상을 연결하는 의미 glyph | Layers가 이미지 glyph에 연결됨 | 전용 Merge SVG 추가. 기존 Layers 전역 매핑은 다른 책임이므로 변경하지 않음 |
| 삭제 | 휴지통 | Eraser | 전용 Delete SVG 추가 |
| undo/redo | 곡선 화살표, 편집 도구 뒤 구분선 | 뒤로 이동 glyph와 undo만 있음 | 전용 Undo/Redo SVG, 명명된 두 동작, 그룹 구분선 |
| 가용성 | 이미지의 일부 버튼 활성 | 실제 명령 미연결 | 모두 비활성/이유 유지. 이미지의 활성 상태를 기능 지원으로 복제하지 않음 |

새 그룹 모음은 Taffy Flex wrap으로 배치하며 기존 다른 화면의 toolbar::unavailable 동작은 유지한다. SVG는24 viewBox·1.75 stroke·round cap/join을 공유한다. Icon::image는 기존 SVG 자산을 egui image+text Button에 전달하며 paint도 같은 자산 생성 경로를 사용한다.

## 실제 실행

r62(PID118688, `runtime/review-runs/ui-sub-toolbar-20261018-r62/player.exe`)을 기존 가상 모니터에서 실행했다.

- 1440×900 ASR: 분할/병합/추가/삭제 네 버튼과 구분선 뒤 되돌리기/다시 실행이 한 줄에 표시됐다. 각 SVG와 label이 같은 테두리 안에 정렬되며 새 삭제/곡선 undo·redo glyph를 확인했다.
- 760×480 ASR: 본문 끝까지 스크롤해 여섯 버튼이 한 줄 안에 모두 들어오고 다시 실행 label/테두리가 잘리지 않음을 확인했다. 이 표본은 실제 wrap 발생 검증은 아니다.
- 760 교정 탭: 같은 여섯 버튼·구분선·파형 아래 배치를 확인했다. 설정 열기→Escape 후 교정 탭과 도구/표의 스크롤 위치를 유지했다.
- 최종 fmt 검사·cargo test·Clippy all-targets warnings deny·player 빌드가 통과했다. headless 모든 fixture/탭 렌더 통과는 실제 전 화면의 시각 인수를 대신하지 않는다.

소유 창 닫기 버튼으로 r62를 종료했고 이후 player.exe/mpv.exe 조회 결과가 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 남은 범위

실제 split/merge/delete/history 명령·활성/선택/hover/pressed/focus와 긴 번역 label의 줄바꿈은 아직 미검증이다. 비활성 버튼의 클릭 동작 성공을 주장하지 않는다. Empty/화자 탭과 새 SVG image API를 사용하지 않는 기존 paint 경로의 타 화면 실제 회귀도 별도다. 전체 자막 표·검사기·transport의 일치 완료를 의미하지 않는다.
