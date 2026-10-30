---
id: evidence:kinekino/20261029113902
title: Flexbox 전환 기록 · 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Flexbox 전환 기록 · 2026-10-18

## 기준과 참조의 구분

시각 기준은 `.spec/artifacts/design/` 승인 이미지다. `%USERPROFILE%/hell-is-us-mod`는 패키지와 UI 코딩·구조 방식의 참조이며 테마 이식 대상이 아니다.

관측한 참조 의존성은 `egui_taffy = "0.14"`, `eframe = "0.36"`다. 실제 해석 버전은 egui_taffy 0.14.0 / taffy 0.9.2 / egui 0.36.2다. 기존 egui 0.33과는 Ui 타입이 달라 SDK 담당 세션에서 eframe·egui_extras를 0.36.2로 함께 정렬했다.

## 구현

- `src/presentation/layout/styles.rs`: 전체 폭, 콘텐츠 높이 열 Grid, 동일 폭 Grid, Flexbox 행.
- `src/presentation/layout/leaves.rs`: min-width 0의 축소 가능한 본문과 콘텐츠 크기를 보존하는 제어 위젯 래퍼.
- 검사기 필드: Grid 안의 Flexbox 레이블·컨트롤 행.
- 자막 비교: 동일 폭 Grid와 공통 본문 래퍼.
- 작업 화면 제목: 유동 제목과 콘텐츠 크기를 보존하는 설정 버튼.
- 재생 컨트롤: 유동 왼쪽 그룹과 콘텐츠 크기 오른쪽 그룹. 수동 spacer로 우측을 미는 방식을 제거했다.
- egui 0.36의 App::ui, Panel, 컨텍스트 입력 API에 맞췄다. headless 검증은 렌더러가 없으므로 texture delta를 명시적으로 비운다.

## 검증 상태

최초 버전 정렬 후 clippy는 통과했다. 테스트에서 새 egui의 미소비 texture delta 검사로 5개 headless 테스트가 실패해 공통 테스트 어댑터를 수정했다. 수정 후 전체 테스트를 다시 실행 중이다. 이 기록은 시각 일치 완료를 의미하지 않는다. 실제 창에서 Flexbox 전환 후 긴 경로·좁은 폭·캡션 상태와 이미지 세부 요소를 다시 비교해야 한다. 나머지 수동 본문 레이아웃도 공통 컴포넌트로 계속 전환한다.

## 후속 실제 관측

최신 일반검사: fmt --check, cargo test 59개 통과/외부미디어5개ignored, clippy --all-targets -D warnings 통과. 별도Rust visual-review 도구clippy도통과했다. 실제1440×900 --ui-demo에서페이지별샘플표/검사기/3개타임라인구간/queue35%를확인했다. Populated→LongLabels선택전환과길어진표셀말줄임이동작했다. 일본어글리프누락을관측후PretendardJP1.3.9fallback을추가했고재실기SUB행의일본어표시를확인했다. PNG scene해안프레임과2D투명캐릭터checkerboard합성도직접확인했다. demo최대화는확인했으나사용자입력과창capturebounds변동으로복원·최소화는미검증이다. 새실기에서는전면activate후정상capture했다. 모든12화면의시안세부상태일치를완료한것은아니다.

생성자산3개등록과프롬프트/해시/알파관측은 assets/design/demo/anime/README.md. 실제미디어추론/분할성공을의미하지않는다.
