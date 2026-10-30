---
id: evidence:kinekino/20261029113829
title: 제품 공통 프레임·시청 소스 누락 점검
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 공통 프레임·시청 소스 누락 점검

2026-10-25 PM 배정의 source-only 점검이다. 새 GUI/클릭/화면 캡처·Cargo·브라우저 실행0, canonical/index 변경0이다. 승인된 PotPlayer 방향과 기존 선택 시안을 기준으로 공통 프레임 및 일상 시청 선언을 대조했다. 작업별13개 workspace의 세부 canvas/inspector를 새로 시각 검토했다는 뜻이 아니다. 아래 source finding은 실제 화면의 시각 불합격/합격 관측과 구별한다.

## 고정 기준과 방법

- 기준 `.spec/artifacts/design/v2/viewing/native-player-playback-v2.png` 1586×992, SHA `9DD0440D884B8D27DA92900D06B8D5F825F3CE06A6D507874F73C98A81000540`을 열어 제목줄·하단 rail/transport·오른쪽 회차 패널의 관계를 재확인했다.
- 기존 원본 파생 crop `runtime/verification/design-crops/caption-detail-v2-4x.png` SHA `8ABC810BC1AEB926A35CB1D3AD3C753DF70186350317C96F677A4ACDD0B1F540`, seek-detail-v2-4x.png SHA `10788841B6FA4F66CD13A38B849EC8292ABEC0D54CD6C870A81DEA690B42D6A6`를 실제 열어 무프레임 선형 glyph와 track/transport 경계의 형태를 확인했다. 좌표는 [기존 확대 기록](viewing-elements-2026-10-18.md)을 따른다. 생성 이미지 오류/샘플 숫자를 새 정책으로 복제하지 않는다.
- source predecessor는 root77fd593e, UX16 `ECE5632F894D03F3361BA2CB2D8C12B05EC546AB549302A328DEFB51B30BBCBE`, range r2 effective17 `0B31EF741AA52E6D8182C6BEF00C72746630B66A4429F015C05B0A65FEDFF16E`다. range/Meari nested 실행 인수는 여전히0이다. 현재 중앙 retirement283 검사에 아래 후계를 합성하지 않는다.

## 새로 구체화한 소스 누락

| ID·요소 | 정확한 소스 관측 | 조치/남은 구현 |
| --- | --- | --- |
| SA-01 / VIEW-02 제목 | 공통 window-chrome.tsx context span에는 전체 title 속성이 없고 components.css에는 제품명/작품명 사이 구분선이 없다. 빈 context도 span이 남는다 | 독립 style4에서 비어 있지 않은 context만 토큰 border/padding, 빈 context display:none, 원 문자열 title 전달. Viewing/Studio가 같은 컴포넌트를 사용한다. 실제 긴 제목·캡션 정렬은 미검증 |
| SA-02 / 공통 입력 상태 | components.css의 일반 aw-button에는 active가 있으나 aw-icon-button/aw-caption에는 hover만 있다. Rust design_tokens에는 이미 CAPTION_PRESSED/ DANGER_PRESSED가 있다 | style4에서 기존 Rust 의미 색을 CSS 토큰으로 투영하고 caption/close/icon button active에 사용. 새 색 체계/임의 화면별 색을 만들지 않음. disabled selector 유지. 실제 pressed/focus/hover 광학 비교 미검증 |
| SA-03 / VIEW-08 폭 | viewing.css grid의 오른쪽 열은 공통 Studio inspector300 토큰을 사용한다. 기존 Rust sizing::EPISODE_PANEL 및 VIEW-08은260이다 | style4에서 전용 episode-panel260 토큰으로 연결. 이 수정만으로 패널 닫힘·반응형·회차 구성 완료 아님 |
| SA-04 / VIEW-08 닫힘 | WindowUiProjection은 episodePanelClass만 토글하며 ReadRoot에서는 CatalogBrowser를 감싼 내부 div만 그 class를 받는다. 바깥 observation-pane와 grid의 오른쪽 열은 항상 남는다 | 단순 display:none으로 바깥 pane를 지우면 별도 오류/취소/Jobs/후보 진입도 사라진다. 회차 목록과 공통 상태·오류·문제 해결을 분리하고 typed root layout으로 영상 영역을 회수하는 후계 필요. 지금은 전체 패널 접힘 구현으로 계상하지 않음 |
| SA-05 / VIEW-09 전체화면 | window_ui.rs는 실제 fullscreen 관측을 내부에 보유하지만 게시하는 것은 label/disabled/transportClass/episodePanelClass/error다. player-frame.tsx는 WindowChrome을 항상 렌더하며 grid의 titlebar 행도 고정이다 | 실제 fullscreen 관측 Bool/레이아웃의 명시 producer→closed ReadRoot→frame 전달과 제목줄/캡션 제거 후계 필요. native host의 전체화면 창 변경 성공과 제품 제목줄 숨김을 분리하여 인수. 영상 위 overlay 합성도 현재 선언에서 입증되지 않음 |
| SA-06 / VIEW-10 유지 사유 | can_idle 인자는 viewing/stopped/fullscreen/window focused/inside/panel_open뿐이다. pause/error/현재 drag/menu/키보드 요소 focus는 predicate에 없다 | 정지·오류·드래그·메뉴·focus 유지 사유를 원 UI 상태로 연결하는 후계 필요. 예: 패널을 닫고 stationary drag 상태에서 idle deadline이 지나도 rail과 cursor가 남는지 실제 입력 확인. 현재 timer 존재를 이 전체 계약 지원으로 확대하지 않음 |
| SA-07 / 오류의 기본 접근 | NativeTransportBar는 nativePlaybackDiagnostic 전체를 diagnosticsExpanded로 숨긴다. windowError는 별도이지만 transport 자체가 숨겨질 수 있고, catalog error는 닫힌 내부 패널 안에 있다 | 기술 상세와 기본 오류 요약의 별도 producer가 필요하다. 문자열을 분석해 오류/성공을 추정하지 말고 기존 SDK 이벤트·typed 관측을 표시 계층에서 구별한다. 접힌 상태의 사용자에게 원 실패/불확정·복구 진입이 보이는지 최종 계획에 추가. SDK 재시도 정책 변경0 |
| SA-08 / VIEW-08 현재 회차 | CatalogRow schema는 key/title/disabled뿐이고 RadioGroup의 selectedEpisode는 탐색 선택이다. 실제 playing/connecting 표시 필드는 없으며 현재 행 marker/번호 slot/닫기 header가 선언되지 않았다 | 탐색 선택을 실제 현재 회차로 칠하지 않는다. 원 engine/artifact와 연결된 현재 행과 대기 상태의 명시 UI view model, 제공된 제목 보존, 현재 glyph/amber 선/헤더 닫기/본문 스크롤 후계 필요. 회차 번호를 제목에서 추측하지 않음 |
| SA-09 / VIEW-05/07 rail 형상·배치 | range17의 공용 Range는 label/input/preview가 한 행이며 seek와 volume을 transport 버튼 뒤의 별도 두 행에 배치한다. aw-range-input은 flex/height32/accent만 지정한다. track3/thumb10 및 volume72/좁은 popover의 부품 토큰 소비가 없다 | 기능 입력 소비 후계와 시각 rail 후계를 분리한다. 원시 range 기본 그림을 PotPlayer 시안 일치로 판정하지 않음. Meari가 실제 제공/검증한 Native/Web range 부품 스타일 계약으로 seek를 영상 바로 아래, volume을 오른쪽 compact group에 배치해야 함. pointer hit32와 시각 track3을 구별하고 focus/drag/unknown 상태 재검토 |

기존 계획의 창 caption 호스트 미연결, VM/DPI 보류, 실제 소리/전 화면 순회·사용자 품질 승인 미완료는 그대로이며 중복 구현 완료 주장이나 새 gate로 추가하지 않았다. 현재 play/pause가 동시에 별도 버튼으로 표시되는 선언도 시안의 단일 상태 affordance와 대조 대상이다. 이는 기능 정책 변경이 아닌 관측 상태에 따른 표시 설계 후계이며 지금 자동 교체하지 않았다.

## 공급 경로에서 확인한 사실

product_styles.rs는 고정 tokens/components/viewing/studio/subtitles CSS와 hash 고정 Pretendard Regular/SemiBold·JP fallback 및 메모리 자산을 준비한다. player_shell_document.rs는 원 product frame이 있으면 prepare_product_frame_styles를 사용한다. 따라서 소스에 font/assets 공급 경로가 없는 것으로 단정하지 않는다. 실제 glyph/fallback/font weight·색·mask 결과와 GUI267r3에서 보인 형상은 새 실행에서 재검토해야 한다. 현재 CSS 공급 소스의 존재는 실제 스타일 적용 성공 증거가 아니다.

공통 WindowChrome과 icon-button 수정은 이를 소비하는 Viewing/Studio 전반에 영향이 있다. 작업별 전체탭의 선택/오류/표/canvas 상세는 [화면 구성](../../player/presentation/shared/screen-composition.md) 순회 범위를 유지한다. 특정 공통 책임을 고친 것을 전체탭 시각 인수로 계산하지 않는다.

## 독립 최소 수리와 다음 배정

private `runtime/development/product-frame-style-audit-r1/source4.json` SHA `20DF2B717837C17E5C56497AA2EDC4047AEFC570E0FABEC29B147F5251888FF1`, predecessor4 SHA `9CD3F500755347E08FCE894D180D1BC5F6A3AA488D7B4AF054B0F69C365410CB`는 SA-01~03의 최소 수정이다. CSS3+공통 TSX1이며 Rust/SDK 정책·새 props·콜백·파일 자산을 추가하지 않는다. 기존 Rust CAPTION_PRESSED=(48,52,60), DANGER_PRESSED=(158,30,22), EPISODE_PANEL=260을 그대로 투영한다. parser/type/test/GUI 미실행이며 레이아웃 후보일 뿐이다. CSS selector·토큰·title renderer 처리와 기존 style/asset 검사를 후속 중앙 배정에서 검증하고 실제 창의 동일 crop으로 인수한다.

SA-04~09는 소스로 확인한 구현 미완성이다. final GUI에서 볼 체크 항목만으로 바꾸지 않고 먼저 typed producer/선언/표시 책임의 별도 후계를 작성해야 한다. 정상 수명·actual4 뒤 배정되는 GUI는 해당 선택된 소스에서의 사실을 관측할 수 있으나, 이 미완성 항목을 전체 시각 합격으로 표시하지 않는다.

modootoday/software-qa/test-fixture-and-mock-build의 실제 공급원/가정·실행 범위 구분을 적용했다. 기준 이미지·기존 확대 파생물·현재 선언을 대조했고 새 수동 실행/사용자 상태 변경은 하지 않았다.

기록한 SA 지적은 modootoday/software-qa/code-change-review-verification 본문과 필수 references/verdict-table.md(SHA `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`)로 재검토했다. private style-audit-r1/REVIEW.md는 실제 선언 인용·소스 판정과 이를 가를 후속 입력을 분리한다. 소스의 누락은 확인했지만 실제 화면 결과는 미검증이며, window 오류 복구 경로가 있는 것을 무시해 모든 오류가 숨겨진다고 확대하지 않는다.
