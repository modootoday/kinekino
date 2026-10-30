---
id: plan:kinekino/20261029113825
title: UI 공개 소스 경계 단순화 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI 공개 소스 경계 단순화 계획

2026-10-20 PM 요청. 현재 P1a freeze 동안 경로 이동·공유 ignore 수정은 하지 않는다. exact 파일 목록은 관측 inventory로만 보존하고 신규 파일마다 예외를 추가하는 정책으로 고착하지 않는다.

| 기존 → 목표 | 책임과 영향 |
|---|---|
| src/presentation/web_ui → 동일 | 공개 TSX 선언·components/studio/routes/styles와 합성 verification source. 디렉터리 전체 허용. 실제 사용자 정보/영상/스크린샷을 넣지 않는다. Rust include_str 경로 변경0 |
| src/presentation/web_ui_adapter → 동일 | Rust SDK presentation 및 composition. 디렉터리 전체 허용, 계층 유지 |
| assets/design/icons → 동일 | 제품 전용 작성 SVG·README 출처. 디렉터리 전체 허용, icon_asset include 경로 변경0 |
| assets/fonts → 동일 | 고정 Pretendard TTF·OFL·출처 README. 공개 전용 디렉터리 허용, resolver 경로 변경0 |
| assets/design/demo → 동일 | 생성 합성 PNG·SVG mask·프롬프트/해시 출처. 공개 전용 디렉터리 허용, 실제 영상·모델·키는 금지. design_assets include 경로 변경0 |
| .spec/artifacts/design 및 runtime → 동일 비공개 | 승인 시안·관측 캡처·실패 원본·실행 로그를 공개 코드와 분리, 전체 거부 |
| private .spec compile 계약·fixture → SDK 지정 공개 계약/합성 fixture 디렉터리 | SDK가 7개 private compile 의존을 분리. UI는 최종 인계 후 확정된 public 경로를 소비하며 사전 중복 이동0 |

최소 ignore 정책은 공개 전용 상위 디렉터리 허용과 `.spec`/`runtime` 등 비공개 루트 거부의 조합이다. 구체 패턴은 PM이 현재 상위 ignore/다른 소유 파일과 대조해 적용한다. 폰트·SVG·합성 demo 디렉터리에 provenance/license를 함께 두며 unused 외부 파일을 일괄 공개하지 않는다.

단계는 (1) UI 최종 pin/소유권 반환, (2) SDK private compile 의존 분리와 include/fixture 링크 수리, (3) PM 디렉터리 정책 단순화, (4) 공개 기준선 clean checkout의 offline fmt/test/clippy·자산 경로 검증이다. 별도 역할별 커밋으로 만들고 기존→신규 경로가 실제 바뀌는 경우만 SDK/소비자 링크를 함께 갱신한다. `.spec` 전체 재배치는 승인 전 실행하지 않는다. 13화면 coverage/시각 인수 상태는 공개 파일 포함 여부로 승격하지 않는다.
