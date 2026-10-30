---
id: evidence:kinekino/20261029113757
title: A2D 브러시 설정 배치 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A2D 브러시 설정 배치 검토

2026-10-18. A2D-04b. asset2d.png의(298,401,160,182)을3배 확대한 `runtime/design-inspection/a2d-brush-r70-3x.png`에서 도구 구분선 아래의 크기/부드러움 label·slider·값을 확인한다. 기존 r65 실제 화면은 이 위치에 마스크 편집/도구 배치 예시만 표시했다.

## 구현

공용 brush_settings::unavailable을 wide rail에 조합하고980 미만은 같은 컴포넌트를 설정 검사기로 이동한다. 공통 input::unavailable_scalar를 기존 inspector_field Scalar와 공유해 중성 트랙·미상값 —를 표시한다. 크기는 원본px이며 부드러움의 단위/허용범위는 아직 계약 전이다. 시안24/50을 기본값으로 발명하지 않고 thumb/fill/가짜0을 표시하지 않는다. 실제 brush edit/mask mutation은 연결 전이다.

## 실제 실행

r70(PID73072, `runtime/review-runs/ui-a2d-brush-20261018-r70/player.exe`)을 기존 가상 모니터에서 실행했다.

- 1440×900: 도구 구분선 아래 크기(px)·부드러움·중성 트랙·미상값 —·연결 필요가 rail112 안에 들어왔다. thumb/fill/시안 숫자는 없으며 canvas/bounds·레이어 구조를 밀어내지 않았다.
- 760×480: rail44에는 도구만 유지됐고 설정 서랍의 frame 정보 아래에 브러시 설정/크기/부드러움이 표시됐다. 같은 공용 component가 넓어진 가용 폭을 사용했다.
- 설정 내부 스크롤로 변환 스케일/출력 옵션까지 도달하면서 합성 검증/패키지 저장은 고정 하단에 남았다. Escape 후 canvas/인물 선택·본문 위치가 유지됐다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. headless 전체 fixture 통과를 실제 모든 화면/상태의 인수로 확대하지 않는다.

소유 r70 창 닫기로 종료했고 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

실제 값·활성 slider·편집 명령·키보드 수치 조절·도구별 값 적용·LongLabels/Empty·980 경계와 추출된 공통 scalar의 모든 타 화면 실제 회귀는 후속이다. 시작 SEG의 중성 미상 track은 표시됨을 관측했으나 전 화면 검증 근거는 아니다. 전체 A2D 일치 완료를 의미하지 않는다.
