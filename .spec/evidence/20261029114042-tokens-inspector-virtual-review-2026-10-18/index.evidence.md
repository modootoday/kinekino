---
id: evidence:kinekino/20261029114042
title: 토큰·공용 컴포넌트·가상 모니터 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 토큰·공용 컴포넌트·가상 모니터 관측 — 2026-10-18

## 변경 범위와 책임

사용자 후속 지시를 반영했다: UI 인터페이스는 현재 UI 세션에서 컴포넌트와 함께 구현/검증하고 SDK와 공유 경계를 조율한다. Tailwind 범주의 토큰을 적용하고 shadcn-ui식 소스 소유·variant·조합 원칙으로 공용 컴포넌트를 만든다. React/Tailwind 런타임을 제품에 추가한 것은 아니다.

- `design_tokens.rs`: palette/semantic color, spacing/header 관계, sizing/type family, border/radius, breakpoint. 직접 색상 생성과 명시적 border/radius 숫자를 공용 토큰에 모았다. 남은 legacy 크기/간격의 전체 이관은 후속이다.
- shell: 본문/검사기 padding12, 제목/설명4·탭 전16·탭 후8, 탭높이32. caption과 반응형 치수도 토큰에 연결했다.
- `CanvasInspectorView`: 기존 SDK snapshot/선택layer를 borrow해 source PTS·절대pixel pivot/bounds를 표시한다. live 입력 없음에 demo 설정/PTS를 주입하지 않는 테스트를 추가했다. alpha 설정은 명시된 DesignSample 예시이며 실제 edit/export command를 실행하지 않는다.
- 공용 Button Primary/Secondary, Field/Inspector primitives, LayerRow props/thumbnail slot을 도입했다. A2D 파일은 데이터·조합을 담당한다. 새 eye-off/unlock SVG는 직접 작성하고 IconId에 등록했다.

## 가상 디스플레이

사용자가 물리1대/PiP 충돌 문제로 검증 가상 모니터 사용을 지시했다. 기존 설치 드라이버를 Rust validation adapter로 사용했다. 재설치·인증서/보안 변경·VM 작업은 하지 않았다.

1. inventory: 물리 DISPLAY1 bounds[0,0,3440,1440], primary=true, probe96 DPI만 관측.
2. add: 소유 connector ID9 요청.
3. inventory: 물리는 동일하고 가상 DISPLAY5 bounds[3440,0,7280,2160], primary=false, probe96 DPI 확인.

가상 화면 추가는 DPI 변경이 아니다. probe HWND의 DPI는 player HWND 실측과 구별한다. 사용자가 확인할 player/가상 모니터는 현재 유지하며 자동 제거·종료하지 않는다.

## 실제 UI 관측 — ui-next 빌드

`runtime/parallel-build/ui-next/debug/player.exe --ui-demo --record-validation --validation-position=3500,100`, PID33516. 반환된 단일 해당경로 창을 선택했다. WGC screenshot metadata origin[3500,100], size1440×900으로 가상 화면 안임을 확인했다. 물리/PiP 창은 조작하지 않았다.

- 토큰 적용된 헤더·설명·탭·본문 정렬과 A2D 전용 검사기를 관측했다.
- A2D foreground 선택: pivot730/480, bounds450/60/560/840, sourcePTS00:52.000, canvas1672×941 표시.
- background row 클릭: pivot836/470, bounds0/0/1672/941, 배경label로 전환, canvas 선택경계가 전체canvas로 전환됨을 확인했다.
- 최초 자산 decoding 중 큰 loading arc/일부checker가 보였고 다음 관측에서는 모든 이미지가 로드됐다. 이를 영구missing asset으로 기록하지 않는다.
- 알파 slider가 작은 사각 손잡이처럼 보이는 문제를 실제 발견했다. 이후 공용 Inspector.scalar가 기본 egui Slider 대신 MediaRail을 재사용하도록 수정했다. 이 수정은 별도 최종 빌드의 실제 재관측이 필요하다.

ui-next 단계 fmt/test67pass(5 external ignored)/clippy -Dwarnings/build는 통과했다. 이후 공용 LayerRow/새SVG/focus/MediaRail 수정은 최종 검사·실제 관측 결과를 추가로 구분한다. 새 inspector의 모든 상태·760/980 drawer·전체화면·live 데이터·실제 mask/export는 완료로 표시하지 않는다. 사용자 확인용 창은 종료하지 않았다.

## 최종 후속 관측 — review-runs r1/r2

최종 소스의 cargo fmt --check/cargo test(67pass/5external ignored)/cargo clippy --all-targets -- -D warnings/cargo build --bin player를 모두 통과했다. 실행 중 빌드 경로를 덮어쓰지 않도록 새 검토 빌드는 `runtime/review-runs/ui-components-20261018-r1/player.exe`와 `r2/player.exe`에 복사했다. 두 검토 창도 사용자 지시에 따라 임의 종료하지 않았다.

- r1(PID56484): WGC origin3500/100·1440×900 확인. A2D 알파 scalar가 가는 track/원형 thumb로 수정됐고, layer foreground의 unlock/background lock 표식을 구분했다. EyeOff는 fixture가 모두 visible이므로 실제 숨김 상태 미검증이다.
- r1 760×480: 설정 drawer가 화면 안에서 열리고 내부 scroll/하단 검증·저장 동작에 접근 가능했다. Escape는 설정만 닫고 A2D workspace를 유지했다.
- r1에서 불필요한 단일 '레이어·마스크' 탭 줄과 넓은 tool rail을 발견했다. 공용 Tabs로 분리하면서 단일 탭 줄을 제거하고980 미만tool rail44 토큰을 적용했다.
- r2(PID54492): 가상 모니터 동일좌표에서1440×900 A2D 단일 탭 줄 제거 후 header→mode→canvas 간격을 확인했다. 760×480에서도 heading/description/modes·아이콘 tool rail44·canvas·설정·caption·queue가 화면 안에 유지됐다.
- r2 760에서 레이어 목록/원본 참조가 여전히2열이다. 현재 detail breakpoint가 내용폭600 기준이므로 창980 미만 적층 계약과 차이가 남는다. 별도 후속 수정/실제 검증 대상으로 남겼다. 하단 전체 내용은 내부 scroll이 필요하며 모든 행 접근·keyboard focus 검증 완료가 아니다.

실제 모델·저장/레이어mutation·전체화면/PiP 경쟁·가상창의 player HWND DPI는 이번 관측으로 입증하지 않았다. 디스플레이probe96과 WGC 가상위치 확인을 해당 검증으로 확대하지 않는다. 전체 UI goal은 계속 미완료다.
