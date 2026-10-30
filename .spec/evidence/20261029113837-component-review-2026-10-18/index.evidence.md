---
id: evidence:kinekino/20261029113837
title: 컴포넌트 수정과 실제 A2D 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 컴포넌트 수정과 실제 A2D 관측 — 2026-10-18

## 구현한 변경

- 공통 WindowTitle 컴포넌트로 제품명·구분선·작품명을 분리했다. 제품 식별 설정을 주입하고 작품명은 남은 폭에서 말줄임한다. title drag와 caption 영역은 shell이 유지한다.
- 시청 clock은 1시간 미만 MM:SS, 그 이상 HH:MM:SS로 변경했다. unknown position의 독립 표현은 아직 후속이다.
- A2D Fit은 view zoom뿐 아니라 pan offset도 초기화한다. source/레이어 데이터나 SDK 정책은 변경하지 않았다.

제품 Rust 검사: cargo fmt --check, cargo test(65pass/5 external ignored), cargo clippy --all-targets -- -D warnings, cargo build --bin player 통과. UI target-dir는 runtime/parallel-build/ui다. 테스트 결과를 실제 시각 일치 증거로 확대하지 않는다.

## 실제 관측

새 빌드 `runtime/parallel-build/ui/debug/player.exe --ui-demo --record-validation`을 실행했다(PID24340, 1440×900). Computer Use로 해당 빌드 경로와 제목을 반환한 단일 창을 선택해 확인했다.

1. SEG 데모의 탐색·영상·일반 시간축/표·검사기를 관측했다. 일반 골격이며 새 상세 계약의 전용 구성을 완료한 화면은 아니다.
2. A2D 탐색을 클릭하고 다음 상태를 재관측해 화면 전환을 확인했다. composite forest/foreground, blue bounds8 handles·amber pivot, 레이어2행/thumbnail/eye/lock/grip, 원본 참조가 표시됐다.
3. Hand 모드를 선택한 뒤 canvas (760,420)→(850,470) 드래그했다. 이미지와 인물·bounds/pivot이 함께 우하단으로 이동하고 viewport에서 잘렸다.
4. Fit 버튼을 클릭했다. 이미지와 bounds/pivot이 드래그 전 맞춤 위치로 복귀했다. offset 초기화 수정의 실제 동작을 확인했다.

검토용 player는 사용자 지시에 따라 종료하지 않고 열어두었다. VM·DPI는 변경하지 않았다. title/clock 수정은 이 studio demo 경로에서 검증되지 않았다. source 있는 시청 제목/시간·긴제목·모든hover/focus·narrow canvas·실제mask편집/저장은 미검증이다.

## 남은 시각 차이와 인터페이스 책임

검사기는 아직 일반 문자열 필드이며 canvas의 실제 선택layer·절대pivot/bounds와 연결되지 않는다. alpha mode는mask grayscale이 아니고 전용brush/softness·mask layer도 없다. source그림/선택경계/row는 보여도 전체 시안 일치로 판정할 수 없다. [A2D 상세계약](../../player/presentation/studio/assets-2d-visual.md)을 기준으로 전용 inspector와 UI용 projection/read interface를 함께 구현해야 한다.

사용자 최신 지시에 따라 UI에 필요한 인터페이스 설계·구현은 **현재 UI 세션**이 시각 근거와 컴포넌트 검증을 바탕으로 담당한다. SDK 세션과 공용 application/domain 경계·정책·파일 변경 범위를 조율한다. 모든 인터페이스를 SDK에 위임하고 기다리지 않는다. SDK 세션은 workspace_read/presentation을 현재 수정하지 않는다고 회신했다.
