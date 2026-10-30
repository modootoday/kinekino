---
id: evidence:kinekino/20261029114004
title: TSX Web UI의 첫 토큰 대응 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# TSX Web UI의 첫 토큰 대응 검토

적용 스킬: modoo.today skills 카탈로그의 `modootoday/design-delivery/design-to-code-fidelity-diff`. skill_find에서 광고 CRM 등 무관 후보를 제외하고 설명 적합성을 확인한 뒤 skill_call로 본문을 읽었다. 노드별 expected/built/token/verdict 표와 비교 불가능 범위 분리 원칙을 아래 실제 source 검토에 적용했다. 별도 스크립트·다른 언어 도구·추론은 실행하지 않았다.

기준 이미지는 v2/viewing/native-player-start-v2.png를 직접 열어 전체 구성을 확인했다. 아래 exact expected는 raster 추정치가 아니라 기존 승인 screen-composition/window-chrome/typography 및 Rust design_tokens의 논리 계약이다. TSX/CSS source 작성과 브라우저 렌더 검증을 혼동하지 않는다.

| 컴포넌트·속성 | 기대 논리값 | Web source값 | 토큰 | source 판정 |
|---|---|---|---|---|
| WindowChrome 높이 |32|32px|--aw-titlebar|일치 |
| Caption hit 폭 |42|42px|--aw-caption-width|일치 |
| Caption glyph |약10~12|12px|--aw-caption-icon|범위 내, 광학 중심 미검증 |
| Control radius |5|5px|--aw-radius-control|일치 |
| 주요 텍스트 |RGB236/238/242|#eceef2|--aw-text|native projection 일치, 시안 raster 색 미측정 |
| Amber accent |RGB232/188/90|#e8bc5a|--aw-accent|native projection 일치, 시안 raster 색 미측정 |
| 기본 글꼴 |Pretendard400·JP fallback|동일 @font-face 선언|--aw-font-family|선언 일치, asset 전달·로드 미검증 |
| Start mark 크기 |기존96 후보·미인수|96px|--aw-start-mark|시안 치수와 exact 비교 불가, 완료 아님 |

공용 WindowChrome/TransportBar/ViewingStart를 별도 source-owned TSX로 나누고 ViewingPlayer가 import해 조합한다. shell을 화면마다 복사하지 않았다. Button은 등록된 source-owned component import를 유지했다. span은 SVG alpha mask 표시 영역이며 glyph를 폰트문자로 대체하지 않는다. 현재 asset packaging·Button 중첩 요소·custom component composition 지원은 Meari 인계 항목이다.

전체13화면/25하위탭·hover/pressed/focus/disabled·responsive·확대1:1·실제 Web 실행은 아직 검증하지 않았다. 현재 source 숫자 일치 수를 화면 일치 점수로 환산하지 않는다. CSS 토큰은 기존 Rust 토큰의 명시적 Web projection이며 자동 동기 exporter가 아직 없다는 한계를 유지한다. 같은 source의 변환/런타임 보완 뒤 실제 렌더에서 다시 판정한다.
