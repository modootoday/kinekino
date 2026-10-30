---
id: plan:kinekino/20261029113849
title: 네이티브 UI 시각 정밀 검증 잔여 작업
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 네이티브 UI 시각 정밀 검증 잔여 작업

## 최신 실행 목표 — Meari React/TSX 제품 UI

2026-10-20 Native/Web 비교 범위 정정: 현재 Inspector 소비 gate는 기능·노드 identity·dirty/focus·callback/rollback 검사다. 실제 Web renderer에서 제품을 실행해 Native와 시각 비교한 근거는 아직 없다. Native CPU DOM raster나 대표 pixel, 원 시안 crop, headless 입력 성공을 Native/Web 동등성 또는 제품 시각 일치로 표시하지 않는다. [현재 Inspector gate](native-inspector-consumer-gate.md)의 mixed/소비 회귀를 먼저 마감한다.

후속 선행조건은 동일 제품 source revision·typed 상태/선택·viewport·scale·동일 Pretendard/JP와 SVG/미디어 asset 핀을 두 target에 전달하는 실제 Meari consumer 경로다. Web/WASM widget·slot·callback·asset 지원을 먼저 확인하고 미지원은 정확히 기록한다. 별도 HTML/JS 복제품으로 비교 조건을 대체하지 않는다. 실행 슬롯을 받은 뒤 Native와 실제 Web 렌더를 각각 확보하고, source hash/상태/viewport/scale/font·asset hash/backend를 기록해 대응시킨다. 기준 시안의 전체 및2×/4× crop과1:1 대조하여 레이아웃·여백·텍스트 기준선·폰트·아이콘·색/border/radius·클리핑·캡션을 검토한다. source 참조 이미지의 미디어/장식과 실제 application 상태는 구분한다.

담당은 UI가 제품 시안·화면별 요소·공통 제품 토큰/컴포넌트의 일치를, Meari가 동일 source의 공통 Native/Web 렌더 동등성과 위젯/asset 전달을 소유한다. 차이는 제품 source와 공통 renderer 원인을 나눠 담당에게 전달하고 수정 후 같은 조건으로 재검토한다. 현재 이 후속 실제 렌더·확대 비교는 미실행이며 이번 문서 갱신은 GUI/browser/WASM 실행 배정이 아니다. 전체13화면·상태의 기존 미검증 범위는 유지한다.

2026-10-20 PM 후속 준비는 [consumer 계획의 단일 프레임→전체13화면 작업표](meari-consumer-gaps.md)를 실행 순서의 최신 기준으로 삼는다. SDK 새 exe6입력과 제품 CSS/TTF3·대표 chrome pixel 관측을 기존 white 기능fixture와 구분하며, 공통 shell→router/편집 session→native media 영역→전체13군 전용 구성 순으로 연결한다. source freeze와 SDK 단일 heavy 소유는 유지한다. 이번 단계는 기존 자료/계획 정리뿐이며 코드·빌드·GUI·새 mockup/Pages/AI/VM/DPI 실행은0이다. 아래 기존 확대 검토와 미완료 항목을 중복 재작성하거나 삭제하지 않는다.

사용자 직접 지시로 UI 선언·디자인 제작을 재개했다. 현재 실행 정본은 [Meari consumer 목표·잔여·다음 gate](meari-consumer-gaps.md)다. 아래 native 수동 검토 보류 이력과 원래 전체13화면군/키보드/패널 미검증 범위는 보존하며 TSX source 제작을 막는 지시로 해석하지 않는다. 제품 SDK/CLI 기능 인수·중앙 자원 배정·소유권/freeze는 계속 유지한다. 실제 변환/실행/확대 시각 검증 전 완료로 표시하지 않는다.

## 최신 사용자 우선순위 — 수동 UI 검토 보류

리소스가 적게 드는 계약/타입/소스 검토·얇은 어댑터부터 진행한다. 다음은 캐시 활용 최소 자동 검사, 마지막은 필요한 통합 빌드 순서다. 같은 checkpoint의 중복 검사·빈번한 폴링·개별 보고를 줄이고 이미 정상 실행 중인 작업은 중단하지 않는다. 필수 fmt/test/strict Clippy는 다음 중앙 조율 묶음에서 수행하며 검증 전 완료를 선언하지 않는다. 현재 다음 연결은 [저장 프로젝트 typed 조회](../../../sots/player/presentation/shared/project-inspection.md)로 한정했고 StartScreen 폴리시 소스 변경은 0이다.

사전 수동 UI 점검·버튼 클릭·세부 시각 비교와 StartScreen 폴리시를 보류한다. 현재 GUI/fixture/관측 자료를 보존하고 새 GUI 자동조작·스크린샷 반복·재개 허가 질문은 하지 않는다. SDK/API/CLI 규격과 자동 기능 검증을 먼저 진행한다. UI는 공통 typed client/DTO 표시의 얇은 연결·컴파일·필요 최소 자동 회귀에 한정하며 기능 정책/저장/상태 전이를 UI에만 구현하지 않는다. 부족한 SDK 포트/뷰모델은 계약으로 구체화해 SDK 담당에게 전달한다. 전체 핵심 API가 준비된 뒤 한 번에 시각 검토할 수 있도록 기존 미검증 목록을 유지한다. 아래 과거 시각 우선 실행 계획과 수동 검토 순서는 이 최신 우선순위가 재개되기 전 실행하지 않는다. StartScreen 신규 소스 변경은 아직 0이다.

## 2026-10-19 UI 역할과 현재 실행 목표

후속 typed jobs 연결은 [영속 작업 조회 UI 계약](../../../sots/player/presentation/shared/job-inspection.md)을 따른다. SDK 등록 전에는 기존 범용 Job table을 실제 영속 작업 지원으로 간주하지 않는다. 읽기 전용 list/get부터 연결하고 cursor·선택 ID·늦은 응답·실제 상태 표시를 보존한다. 프로젝트 조회 UI 5파일은 중앙 검사 대기 상태로 유지한다.

최신 검증 분담: 기능 정확성은 SDK 테스트·player-cli.exe가 담당한다. GUI는 레이아웃/토큰/아이콘·키보드·포커스·접근성·dirty/오류/결과 표시·API 연결의 UI/UX 검토에 집중한다. 버튼 입력이 필요한 경우도 기능 판정과 연결/사용성 관측을 구분하며 SDK 고정 fixture·CLI 결과를 표시 비교 자료로 사용한다. GUI 없이 가능한 변환/저장/CAS/손상/정확도 기능을 반복 검증하지 않는다. 기존 r168의 GUI 조작 기록은 이 분류로 정정했다.

최신 사용자 지시로 실제 조작 검증은 사용자가 볼 수 있는 메인 모니터에서 수행한다. 소유 가상모니터 ID9 제거 후 물리 DISPLAY1 단독·96 DPI와 동일 r164 창/편집/채택 상태의 메인 이동을 확인했다. [S2 관측 기록](../../../sots/verification/presentation/document-source-ui-review-2026-10-19.md)을 따른다. 드라이버/인증서·DPI·VM은 변경하지 않았다. 과거 가상모니터 증거를 메인모니터 검증으로 소급하지 않는다.

최신 사용자 조율 우선순위: S2 실제9 자료의 원문 비교/수정/명시 채택을 먼저 실제 인수한다. 이후 승인된 전체 UI의 기능 연결·상태/입력/저장/오류·네이티브 중심 UX를 기존 자료와 명시 unsupported 계약으로 계속 구현·검증한다. 새 AI 추론/성능 실측과 모델 품질 승인을 기다리지 않는다. 원문/생성 원본 보존, 공통 SDK 정책, 전체 시각·키보드·패널 품질과 소유권 경계는 유지한다.

현재 thread goal은 실제 player의 시청·제작 전체 시각 일관성과 키보드·패널 검증이며 active다. 범위를 축소하거나 완료로 바꾸지 않는다. 사용자 최신 역할 지시에 따라 승인된 native Rust UI를 공통 SDK에 연결하고 실제 실행으로 검증한다. 우선 교정·카드 문구·하이라이트의 원문 비교/작업별 수정/명시 채택/충돌 입력 보존/receipt를 연결한다. [공용 편집 정본](../../../sots/player/presentation/shared/proposal-editor.md)과 [SDK 계약](../../../sots/player/application/api/proposals.md)을 따른다. application/API/store/main은 SDK 소유, UI는 presentation/typed client adapter 소유다.

구현→fmt/test/clippy→격리 fixture 실제 GUI→관측/.spec 기록을 계속 진행한다. 공유 native target의 gate는 SDK 소유 중이며 UI가 중복 실행하지 않는다. 주요 통합 완료 또는 실제 차단만 PM에 보고한다. 모델 품질 검토는 제품 연결 뒤 연구 세션 범위다. Pages/VM/DPI/새 장시간 시험/사용자 앱 종료는 수행하지 않는다. 소유 검토 창과 사용자 실행을 구분한다.

기준일: 2026-10-18. 최우선은 `.spec/artifacts/design`와 실제 Rust GUI의 일관성이다. 공통 SDK 구현 완료·테스트 통과·이미지 자산 생성은 전체 시각 검증 완료의 대체 근거가 아니다.

## 확인된 최신 근거

- [StartScreen 구현 계약](../../../sots/player/presentation/viewing/start-screen.md): r163 표식4배·r166 실제 중심 재배치와 참조 프로젝트의 Taffy 조합을 근거로 수동 spacer 이관·공통토큰·caller 행동분리·미검증 치수 후보를 정의했다. SDK 순수추출 gate와 섞지 않고 lease 반환 뒤 구현한다.

- [메인 모니터 시청 패널 r166](../../../sots/verification/presentation/viewing-main-panel-review-2026-10-19.md): 고정 r164에서 제작→시청 문맥/1180폭 복귀, P 회차패널/중앙 시작면 재배치, 첫 Tab 브랜드 focus, Escape 닫기를 실제 확인했다. SDK 순수추출 gate와 UI focus 후속수정 checkpoint를 분리하므로 source 변경은 명시적 lease 반환 뒤 수행한다. 전체 재생/시각 인수는 미완료다.

- [토큰·공용 컴포넌트·가상 모니터](../../../sots/verification/presentation/tokens-inspector-virtual-review-2026-10-18.md): palette/semantic·border/radius·헤더 토큰, A2D UI projection·전용 inspector와 선택좌표 실제 전환. 가상DISPLAY5/WGC origin3500 확인. 최종 slider/LayerRow 후속 재관측 구분.

- [컴포넌트 수정·A2D 실제 관측](../../../sots/verification/presentation/component-review-2026-10-18.md): title/clock/Fit 수정, 새 player에서 pan→Fit 복귀 확인. 나머지 전체 시각 인수와 전용 inspector는 미완료.

- [시스템 화면 확대 관측](../../../sots/verification/presentation/system-elements-2026-10-18.md): 모델 accordion/검토, JOB 단계/입력, OUT 저장확인/충돌을 일반 table과 구분해 상세 계약에 연결.

- [자막 하위 화면 확대 관측](../../../sots/verification/presentation/subtitle-elements-2026-10-18.md), [자막 요소 계약](../../../sots/player/presentation/studio/subtitles-visual.md): 파형/ROI/교정/변환 구성과 현재 범용 editor 차이, 미제공 화자 시안 분리.

- [시청 요소 확대 관측](../../../sots/verification/presentation/viewing-elements-2026-10-18.md), [시청 요소 계약](../../../sots/player/presentation/viewing/visual-elements.md): 제목 분리·회차 행·닫기 glyph·시간 형식 차이 확인. 실제 새 GUI 검증 전.
- [A2D 요소 확대 관측](../../../sots/verification/presentation/assets-2d-elements-2026-10-18.md), [A2D 요소 계약](../../../sots/player/presentation/studio/assets-2d-visual.md): 도구·레이어·전용 검사기·좌표/단위 상세화. Fit offset·alpha 표시·mask layer·readonly 상태 표시 차이 추적.

- [시청 컨트롤·창·좁은 화면 실제 관측](../../../sots/verification/presentation/media-controls-2026-10-18.md)
- [Flexbox 이관](../../../sots/verification/presentation/flexbox-migration-2026-10-18.md)
- [데모 데이터 계약](../../../sots/player/presentation/shared/design-preview.md)
- [폰트 정본](../../../sots/player/presentation/shared/typography.md)

## 다음 우선순위

2026-10-19 [r158 전체 제작 순회](../../../sots/verification/presentation/studio-wide-audit-2026-10-19.md):1440 Populated의12개 workspace/카탈로그25개 탭을 같은 고정 실행본으로 관측했다. 다음 순서는 JOB 미연결 필터의 잘못된 선택 가능 표시 정정 → SEG/SUB/OBJ의 media_preview 공용화와 overlay 좌표 보존 → MODEL 미연결 pane 기본 펼침 → PROJ viewport/camera·FRAME/CARD canvas 및 전용 inspector 순환 개선이다. 새 코드·전체 상태 인수 완료가 아니며 원래13개 화면군 목표는 그대로 유지한다.

사용자 지시: 특정 한 화면의 미세 조정만 반복하지 않고 모든13개 화면군과 하위화면의 전용 구성을 함께 적용한다. 공용 프레임/토큰/컴포넌트 변경은 전체 화면을 대상으로 검토하고, 전용 구성은 SEG·SUB·OBJ·CLIP·FRAME·CARD·A2D·PROJ·SAVE·MODEL·JOB·OUT의 순환 구현/검증으로 확장한다. 범용 preview/table 상태를 시안 매칭 완료로 표시하지 않는다.

2026-10-18 전체 탭 카탈로그를 재대조해 임의의 상위 탭을 정리했다. SEG 수동편집·OBJ 분석방식·CARD 출력·SAVE 저장방식·MODEL 자원·OUT 충돌은 시안의 도구/검사기 영역에 배치하며 별도 가짜 하위화면으로 늘리지 않는다. PROJ는 시안대로 2.5D깊이/투영패키지2탭을 사용한다. 이 수정은 전용 편집기 제작 완료와 별개다.

최신 A2D 후속: 창980 미만에서 layer/source detail을1열로 적층하도록 content폭600 조건과 정본을 맞추고, 공용 LayerRow keyboard/focus·긴 label·숨김/잠금 상태를 실제 검증한다. 가상 모니터 최신 검토 복사본은 `runtime/review-runs/ui-components-20261018-r2/player.exe`다. 전체 공용 API 이관과 모든 화면의 실제 시각 인수는 미완료다.

문서 선행 상세화 현황(전체 구성 검토와 전 요소 측정/실기 인수는 별개):

| 화면군 | 요소별 상세 계약 | 다음 검토 |
|---|---|---|
| VIEW | 작성·캡션/시작/seek 확대 | 모든 상태의 실제 렌더 대조 |
| SUB | 작성·ASR/OCR/교정/변환 확대 | 화자 전용 시안 없음, typed subview/read preview 조율 |
| A2D | 작성·도구/레이어/검사기 확대 | bounds/pivot 실기·누락 inspector/readonly 표시 |
| MODEL/JOB/OUT | 각각 작성·우측 상세 확대 | 왼쪽 목록/상태/동작별 추가 대조와 typed read model |
| SEG/OBJ/CLIP | 상세 계약·범위/마스크/track/후보/일괄 확대 작성 | 전용 컴포넌트 제작·typed 계약·실제 렌더 비교 |
| FRAME/CARD/PROJ/SAVE | 상세 계약·frame/layer/rail/scene/tree/track 확대 작성 | 전용 컴포넌트 제작·typed 계약·실제 렌더 비교 |

상세 계약 작성은 구현 완료가 아니다. 위 표의 모든 화면은 전체 실제 시각 인수 미완료다.

모든13개 화면군에 요소별 시각 계약 초안을 연결했다. [구간·피사체·숏폼 근거](../../../sots/verification/presentation/segment-subject-clip-elements-2026-10-18.md), [프레임·카드·투영·저장 근거](../../../sots/verification/presentation/frame-card-projection-save-elements-2026-10-18.md)를 적용한다. UI 컴포넌트 제작과 확대/실제 검증은 시각 근거를 보유한 현재 UI 세션이 담당하며 SDK 세션은 데이터·유스케이스·정책을 조율한다. 이 책임은 다른 세션에 컴포넌트 제작을 넘기는 의미가 아니다.

2026-10-18 사용자 재지시로 구현 확장보다 전체 시안 재검토와 설계 정본화를 선행한다. [25장 검토](../../../sots/verification/presentation/design-reference-review-2026-10-18.md)와 [화면 구성 정본](../../../sots/player/presentation/shared/screen-composition.md)을 작성했다. 다음 구현 전 각 하위 화면의 요소별 좌표/치수·컴포넌트·SDK 필드·상태 매핑을 구체화한다. 전체 구성 검토 완료와 전 요소 확대 측정 완료를 구별한다.

1. manifest selected_studio의 각 화면과 실제 채워진 데모를 요소 단위로 대조한다. 프리뷰·타임라인·표의 공통 골격만으로 카드뉴스·2D·3D·모델·대기열 등의 전용 구성을 완료로 취급하지 않는다. 공통 컴포넌트와 페이지별 조합 책임을 유지한다.
2. 1180/1280 폭 경계에서 탐색 레일/전체 탐색, 고정 검사기/설정 창 전환을 실제 확인한다. 760/980 관측을 전체 폭 검증으로 확대하지 않는다.
3. 12개 화면을 Empty/Populated/LongLabels/NeedsInput으로 열어 행·열·입력·배지·선택·포커스·비활성·스크롤 및 긴 한국어/일본어를 확인한다. 페이지 간 선택 상태와 row_id 범위를 함께 확인한다.
4. 자막 변환의 손실 확인/수락, 취소된 파일 대화상자, 오류·진행 상태를 같은 공통 필드와 버튼으로 실제 관측한다. 실제 작업을 수행하지 못한 상태는 명시적으로 남긴다.
5. 재생 중 컨트롤 자동숨김·드래그·키보드/팝오버 유지, 미디어/프레임 전체 표시와 시간 알 수 없음 상태를 확인한다. 시작 화면의 무소스 테스트로 재생 seek·오디오를 입증하지 않는다.
6. 창 캡션의 기본·hover·pressed·focus, 긴 제목 말줄임 및 전체화면 복귀를 더 검토한다. 최대화/복원/최소화 실제 동작은 최신 관측에서 확인됐지만 모든 시각 상태를 포괄하지 않는다.

VM·DPI는 사용자 보류 범위를 유지한다. 전체 `.spec` 재배치는 별도 승인 전 실행하지 않는다. 에이전트가 실행한 검토용 player는 소유 PID와 실행 경로를 확인해 에이전트가 직접 종료하며 동시에 하나만 유지한다. 사용자나 다른 세션의 프로세스를 이름만으로 종료하지 않는다.

검사기 역할과 탭별 스크롤의 최신 실제 관측은 [검사기 역할 검토](../../../sots/verification/presentation/inspector-role-review-2026-10-18.md)에 기록했다. PROJ/FRAME/CLIP의 역할 구분과 FRAME 메타데이터를 확인했으며, 전체 화면·상태·폭의 시각 인수는 계속 미완료다.

## A2D 구체화 관측

공용안내/경고·확장footer와PROJpackage후속은 [notice/footer 관측](../../../sots/verification/presentation/notice-footer-review-2026-10-18.md)에기록했다. 다음은 [검사기컨트롤계약](../../../sots/player/presentation/shared/inspector-controls.md)에따라stable field ID/typed value/availability를갖춘select·toggle·scalar·color·path외형을전체화면에서순환이관하고, workspace/subview별scroll상태를분리한다. 시각placeholder를실제SDK값으로승격하지않는다.

전체 화면 전용 조합 1차와 가상 모니터 r3 순회 관측을 [전체 화면 구성 검토](../../../sots/verification/presentation/all-pages-composition-review-2026-10-18.md)에 기록했다. generic inspector의 단순 연결 필요 필드, 시안의 select/radio/swatch/template/track/mask/collision 상세 요소는 완료가 아니다. 다음 순환은 CARD/FRAME/CLIP inspector 및 SUB/SEG/OBJ의 전용 컨트롤 → PROJ/SAVE/MODEL/JOB/OUT 상세 projection과 실제 하위 탭 검토이며, VIEW/A2D 공용 프레임도 함께 재검토한다. headless 전체 탭·4상태·3폭 렌더 검사는 실제 시각 인수를 대신하지 않는다.

asset2d.png를다시직접확인했다. 현재데모의투명캐릭터/일반표만으로는원본·알파합성모드,zoom/pan,포함·제외·지우기toolrail,선택bounds8handles/pivot,layer별thumbnail/visible/locked,sourceframe참조를재현하지못한다. SDK세션에canvas/layer의읽기모델확장을제안했다. 이는실제분할·편집성공을만드는정책이아니며DesignSample계약으로만먼저표현한다. 해당회신전presentation에application타입을복제하지않는다.

