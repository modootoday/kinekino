---
id: plan:kinekino/20261029113814
title: 제품 TSX consumer의 Meari 보완 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 TSX consumer의 Meari 보완 인계

## 2026-10-20 다음 UI 배정 준비 — 단일 프레임에서 전체13화면으로

이번 절은 PM 요청에 따른 기존 source/evidence 읽기와 후속 작업 분할이다. 제품 코드·동결 입력·빌드·GUI는 변경/실행하지 않았다. SDK가 최종 검사와 단일 heavy를 소유하며 실제 최종 핀 인계 뒤 아래 순서로 이어간다. 별도 사용자 승인 대기나 새 mockup 제작 단계가 아니다. 아래 이전 미지원 표는 당시 이력이며 현재 지원을 일괄 부정하지 않는다.

### 현재 연결의 정확한 끝

SDK의 `runtime/quality-review/product-native-main-r1/actual-six-r2.stdout`은 새 실제 exe의 6입력 startup/frame/close/store 불변 시험1개 통과를 기록한다. `styles-native-r2.stdout`의 시험2개는 전체 제품 CSS/TTF3 전달 및 chrome 대표 pixel을 확인한다. 현재 SDK 검사가 진행 중이므로 이 원 실행을 최종 source/binary checkpoint로 확정하지 않는다. UI4의 [helper 인계](../../../sots/player/presentation/product-native-window-helper.md)와 원 실패/미검증 이력도 유지한다.

현재 `product_window/modules.rs`의 고정 inventory는 CARD/Highlight/Correction panel 중 entry1개와 source/actions/text-comparison 공용3개다. StudioFrame·WindowChrome·navigation·queue·router·ViewingPlayer·실제 video child를 포함하지 않는다. 따라서 제품 CSS와 폰트가 전달된 성공은 전체 shell이 이미 연결됐다는 뜻이 아니다. `product_styles.rs`의 고정 CSS4는 tokens/components/studio/subtitles이며 viewing/segments/shorts 전용 CSS의 해당 제품 경로 소비는 별도 연결 항목이다. source에 존재하는 전용 CSS를 임의 동적 경로나 추가 파일 읽기로 우회하지 않는다.

`routing/routes.json`은 status=product-route-contract-not-runtime-registered다. 12제작 workspace+VIEW의13화면군, studio20 하위 경로(이미지19+화자 추가1), 선택 이미지25(studio19+viewing6)는 서로 다른 집계다. routes의 null은 해당 full workspace source 미제작/미등록이며, 예를 들어 CARD panel source가 존재하는 것과 CARD 전체 canvas/rail workspace가 존재하는 것은 다르다. [구성 정본](../../../sots/player/presentation/shared/screen-composition.md)과 [시안 색인](../../../artifacts/design/screens.md)을 기준으로 유지한다. 색인의 오래된 review-required 문구로 최상위 AGENTS의 사용자 승인 상태를 되돌리지 않는다.

### 구현 순서와 파일 소유

최종 제품 인수는 **기본 `player.exe` 런처를 동일 configured client·단일 PotPlayer 공통 frame·native 영상 경로로 전환하는 것**까지 포함한다. 현재 SDK5의 명시 Native proposal mode와 별도 proposal 창/기능 fixture는 이 기본 제품 전환의 완료가 아니다. P1a/P1b 공통 프레임, P2 router·탭, P3 media child 계약/실제 연결 이후 SDK 소유 기본 launcher 조립과 UI 전체13화면의 실제 소비를 같은 제품 경로에서 인수한다. 기본 legacy UI를 남겨 둔 채 proposal 경로 성공만으로 전체 완료를 표시하지 않는다. 이 항목은 최종 인수 범위를 보존하는 계획이며 현재 caller5 freeze·추가 제품 source/빌드0을 변경하지 않는다.

| 순서/작업 단위 | UI 소유 변경 대상과 결과 | SDK/Meari 경계 및 인수 |
|---|---|---|
| P0 최종 입력 인수 | 기존 UI21/fixed10과 SDK의 실제 exe/source/styles/assets/6입력 원 receipt를 연결하고 이전 white 기능fixture와 분리 | SDK 최종 핀·heavy 반환을 인수한다. 기존 partial pixel을 전체13 시각 승인으로 승격하지 않는다. 이번 준비에서는 재실행0 |
| P1 단일 PotPlayer 공통 프레임 | `web_ui/studio/frame.tsx`, `components/window-chrome.tsx`, `studio/navigation*.tsx`, `queue-footer.tsx`, `inspector-dock.tsx`, `styles/{tokens,components,studio}.css`, `web_ui_adapter`의 얇은 화면조립. 제목줄32/탐색180/검사기300/간격12/접힌큐40 토큰, caption42×32 hit/glyph10~12·최대화/복원 별도·hover/focus/여백을 공통 적용 | SDK는 `assembly/meari/product_native_launch.rs`와 `infrastructure/meari/product_styles.rs`의 고정 style/asset 조립을 소유. UI는 platform API 직접 호출 금지. Meari 문서상 ReactNode slots는 아직 unfinished이며 현재 frame의 node slots/inspector children을 그대로 처리하는 port가 선행되어야 한다. shell 복제/flatten으로 우회하지 않는다 |
| P2 router·탭·편집 채택 결합 | `routing/routes.json`, navigation callback, `studio/subtitles/tabs.tsx` 및 공통 Rust presentation route/selection 상태. 같은 frame 안에서 content를 바꾸고 기존 proposal session의 edit/save/accept/refresh·dirty/stale/error/fullref/selection을 보존 | callback/typed input/keyed string map 전체가 미지원이라는 옛 표는 폐기하지 않고 역사로 유지하되 현재 consumer-integration 지원과 대조한다. `onNavigate(workspaceId)`의 인자 capture·Tabs selection·Dialog focus·node slot은 원 source별 정확 재현으로 Meari에만 보완 요청한다. SDK CAS/수락/저장 정책 복제 금지. URL에 draft/로컬 경로/토큰 금지 |
| P3 공통 native playback 영역 | `viewing/player.tsx`, `start-screen.tsx`, `components/transport-bar.tsx`, `styles/viewing.css`와 공통 media viewport/transport adapter. 로드/시간/seek/볼륨/회차/전체화면·open/오류/unsupported 상태를 실제 DTO로 표시하고 제작 preview에 재사용 | application playback_session와 SDK/CLI의 native 계약을 소비한다. 기존 `presentation/native_controls.rs`, `native_audio.rs`는 행동/이관 비교 자료이며 LegacyAudioHandoffPorts/browser 직접 호출을 새 TSX presentation으로 복제하지 않는다. 기존 CLI 재생·제어·시간/frame/close 성공을 TSX 영상 child 조립 완료로 보지 않는다. Meari NativeChild/host와 mpv owned session의 parent/resize/input/focus/close 수명 접점은 별도 좁은 계약으로 인수. 임의 새 mpv·SDK·loop 생성 금지 |
| P4 전체 전용 화면 순환 구현 | 아래13군을 같은 frame·토큰·공용 컴포넌트에서 하나씩 확장하되 특정 panel만 반복 polish하지 않는다. source 작성→동일 target 변환/입력→실제 렌더 확대 대조를 화면/상태별 기록 | UI는 컴포넌트·표시 인터페이스·입력/a11y를 소유. SDK는 지원 capability/DTO/use case·파일/작업/출력 정책, Meari는 원 source의 renderer/graph/widget/asset 포트를 소유. 미제공 값은 상태로 표시하고 fake waveform/ROI/모델/시각 근거를 만들지 않는다 |

현재 `window-chrome.tsx`의 최소화/최대화/닫기는 disabled이며 `transport-bar.tsx`의 재생/seek/음량/회차/전체화면도 연결 필요 상태다. 현재 읽은 NativeWindowWake에는 request_close가 있으나 최소화/복원/drag/fullscreen 전체 public host adapter의 지원 여부는 이 관측만으로 단정하지 않는다. P1/P3에서 필요한 정확 port와 소유권을 먼저 확인한다. close는 같은 정리/dirty 정책을 거치며 caption action이 SDK 정리를 우회해서 창을 파괴하지 않는다.

### P1 caller source 작성 checkpoint — SDK 반환 뒤, Meari P1a heavy 중

#### Node-slot 공개계약 초안 읽기 인수

PM 허용으로 `packages/meari/.spec/sots/frontends/node-slots.md` 초안을 읽었다. 아직 Web/strict/최종 핀 이전이므로 아래는 공개계약 대조이며 API 최종 인수·새 구현/검사 결과가 아니다. caller5/SDK13 동결·추가 source/빌드/GUI0을 유지한다.

- bootstrap은 `ModuleProgram::mount_with_node_slots`의 borrowed factory closure 안에서 `factory.component(key, ComponentReference, typed_props)`로 실제 선언 graph의 editor/inspector 값을 발급한다. 현재 application7 외에 panel과 공용 source/actions/text-comparison 등 공급 graph가 필요하다. `Value::NodeSlot(NodeSlot::Content(...))`는 authoring IR이며 runtime 권한이 아니다. raw Content/Captured/JSON 복원으로 실제 node 권한을 우회하지 않는다. 명시 Empty만 노드가 없는 공급이다.
- 기존 Proposal **SDK session·ReviewSession/controller·registry/context·prepared fullref·worker**를 유지하는 것과 이미 mount한 별도 Native runtime/document를 node로 넣는 것은 다르다. factory는 graph component+정확 props를 받아 새 opaque owned slot을 발급하며 기존 Native runtime/OS handle을 받지 않는다. 별도 editor 문서를 mount한 뒤 외부 lease로 삽입하는 구현은 foreign owner가 되므로 하지 않는다. 초기 같은 ModuleProgram에 editor graph를 조합하고 기존 정책/controller를 얇게 연결한다.
- 현재 factory의 panel initial_props 대조와 bridge의 callback export는 editor projection에 결합돼 있다. shell root bindings와 editor typed projection을 책임별로 나누되 기존 fullref/selection/props 검증을 생략하지 않는다. 공개계약 동결 뒤 그 최소 UI adapter 변경 범위를 인수한다. 현재 root StudioApplication의 Boolean/P2 진단을 줄이려고 원 source를 수정하지 않는다.
- live 공급은 Native document의 현재 getter/read guard에서 발급한 뒤 expected stamp 기반 try_reproject와 기존 **atomic callback export 교체**로 commit한다. read/factory borrow를 유지한 채 mutable update하지 않는다. registry/SDK mutation을 node getter 안에서 실행하지 않는다. Core node_slots와 Native/Web getter를 중복 node registry로 감싸지 않는다.
- owner는 snapshot lineage다. staged Clone은 독립 document가 아니며 별도 mount/문서의 같은 ID도 foreign이다. 현재 active lease 또는 현재 generation 발급만 사용하고 사용하지 않은 이전 generation/폐기 lease는 다시 넣지 않는다. 같은 placement+key+reference의 props 갱신만 state/useId/DOM/dirty/focus 보존을 기대한다. 명시 교체는 이전 subtree/export를 폐기하고 무관 shell을 보존한다. key/nonce/generation·예산/overflow last-good는 Meari 계약을 소비하고 UI가 별도 수치/ABI를 만들지 않는다.

초안의 실제 StudioApplication 후속 진단 `[546,559)`은 Boolean text callback signature이며 P1b의 남은 입력이다. P1a Core/Native 근거를 원 caller 전체 실행·P1b Dialog/P2 router·기본 native launcher 전환으로 확대하지 않는다. 최종 Meari source/API/Native·Web/strict 핀 인계 이후 UI 소비 배정을 기다린다.

PM이 SDK 최종 source13 `4F4AE9DCEB4A222F89B5730E3990C08CFE2DED6690F2A9144DF52B25376B0F0E`와 binary `6610D2FFD801020882E62B46F8BE26244CB72202F1A91E972E3B8C34146FDC6F` 인수를 알리고, Meari P1a 단일 heavy 동안 겹치지 않는 UI caller/source 준비 작성을 배정했다. 아래 이전 생성0/SDK검사중 문구는 이 배정 전 이력이다. 현재 라이브러리 gate와 겹치는 Rust adapter/API/Native/Core/Cargo 및 기존 UI21은 변경하지 않았다. 빌드·parser 실행·GUI0을 유지한다.

before7 hash/사본과 새 경로5개의 이전 부재를 `runtime/quality-review/studio-caller-source-r1/`에 먼저 기록했다. 새 선언5개는 다음과 같다.

- `studio/application.tsx`: 원 StudioFrame을 호출하고 원 StudioNavigation, QueueFooter, InspectorDock을 named JSX node attributes로 공급한다. 실제 workspace/inspector는 trusted declared graph subtree 입력이며 원 InspectorDock의 children으로 전달한다. 제품명/context·선택 workspace·관측 narrow/open·실제 queue label·typed callbacks를 props로 받고 업무/SDK 정책을 만들지 않는다.
- `studio/application.source-graph.json`: 현재 caller와 원 공통 source의 import closure7개를 명시한다. **외부 workspace/inspector 공급자까지 고정된 완전한 제품 graph나 실제 실행 인수가 아니다.** Meari public slot binding을 인수한 뒤 실제 UI4 panel/session의 공급 graph를 추가하며 외부 JSON으로 임의 JSX/HTML을 실행하지 않는다.
- `verification/studio-frame-empty.tsx` 및 대응 graph: 원 StudioFrame의 네 named slot에 명시 `null`을 공급하는 기술 계약 입력이다. 실제 제품 empty 화면/새 mockup이 아니며 숨긴 span/복제 frame/가짜 editor로 대체하지 않는다. null의 bounded empty 지원·최초 거부 Diagnostic은 Meari gate에서 인수한다. 누락을 empty로 추정하지 않는다.
- `verification/studio-shell-comparison.json`: 1440 wide,760 narrow open/closed의 고정 논리 입력·slot identity/dirty/focus/stale 비교·SDK 고정 근거·phase 의존성을 기록한 메타데이터다. 실행 가능한 SDK 정책/새 검증기나 생성 데이터가 아니다. source_authored만 true이며 parser/runtime/visual은 false다.

현재 caller는 원 navigation의 capture(P2), InspectorDock의 Boolean/controlled Dialog(P1b)를 그대로 포함한다. 미지원 표현을 삭제하거나 narrow=false로 숨기지 않았다. 따라서 P1a만으로 product caller 전체 성공을 기대하거나 전체 frame 완료로 표시하지 않는다. 실제 SDK session의 node lease 생성/교체·callback registry 연결은 공개 slot 계약 인수 후 UI Rust adapter로 이어가며 지금 임의 node ABI를 만들지 않았다. 신규 SVG/폰트/style 변경은 없다.

기존 UI21과 frame/dock/nav/queue/chrome/기존 graph7의 직접 재해시는 차이0이다. 새 source5 inventory·사본과 고정 비교 입력을 PM/Meari 인계에 묶으며, 실행/필수검사/실제 주 모니터 확대 대조는 Meari 반환 뒤 UI 슬롯에서 진행한다.

PM이 새caller/graph를 읽어 Meari에 중계했다. 중계한 원 StudioFrame baseline의 최초 진단은 file `studio/frame.tsx` span `[0,39)`의 type-only import, `plain source-owned component imports required`다. prop type 거부보다 앞선 이 원 실행 진단을 이전 source anchor 분석과 구분한다. 새 application/empty graph는 아직 이 세션의 parser 실행0이다. source5 inventory SHA `08C08286ADB5DB8A646FA8FD63EE305F8897568547B3ED2070F147A074168023`와 사본/고정 비교 입력/잔여 계약을 마감하고 Meari public API/slot 반환 인계를 기다린다. root node 공급/P1b/P2 의존성은 유지하며 Rust node ABI는 만들지 않는다.

### P1 원 source별 port 조율 — 이전 읽기 관측

PM이 P1 단일 PotPlayer 공통 프레임을 다음 UI 배정으로 지정했다. SDK 최종 핀/슬롯 반환 전 코드·빌드·GUI0, 동결 source 유지다. Meari 담당 thread `01a14d89-141a-7332-950a-b0775e1972c0`에 아래 원본 hash/span과 현재 source 근거를 직접 전달해 최신 공개 API/인수 근거를 요청했다. 아직 담당 회신 전이며 다음 span은 실제 Diagnostic이 아닌 원 UTF-8 bytes의 0-based half-open anchor다. parser 실행/target 지원 여부를 새로 검증했다고 표시하지 않는다.

| 원본/고정 SHA256 | 정확 anchor | 필요한 타입·port와 보존 조건 |
|---|---|---|
| `studio/frame.tsx` / `E73A4AB56355E9315AF70973688078FCC9E240586025D21E9BF87F76F570AE00` | ReactNode import `[0,39)`; navigation `[158,179)`, workspace `[181,201)`, inspector `[203,223)`, queue `[225,241)` | type-only ReactNode import와 네 named node prop. 문자열/HTML 주입이 아닌 선언 graph node를 전달하고 원 module/instance/callback/state/useId 소유를 유지 |
| 같은 frame | `{props.navigation}` `[448,466)`, workspace `[501,518)`, inspector `[524,541)`, queue `[547,560)` | node-valued JSX child projection, 부모 static element와 node slot 혼합. workspace 변경 시 shell/inspector의 무관 입력과 focus identity를 보존. frame 복제/flatten 금지 |
| `studio/inspector-dock.tsx` / `8812D6EFEC80D51A8D539D727DAE5E2690DB84D3E1564D48A7E18BD413CF6AB2` | `children: ReactNode` `[256,275)`; `{props.children}` `[1110,1126)` | 공통 node slot과 동일 children 소유를 wide/narrow 전환에도 유지. children을 새로 복제해 dirty/focus를 잃지 않음 |
| 같은 inspector | boolean callback signature `[277,314)`; forwarded onOpenChange `[438,471)` | `(open:boolean)=>void` typed callback/signature·payload validation·actual host dispatch. string callback으로 boolean을 임의 인코딩하지 않음 |
| 같은 inspector | open predicate `[382,416)`, modal `[417,437)`, asChild 첫 위치 `[553,567)`, forceMount `[747,764)`, hidden predicate `[765,801)` | boolean expression 자체, Dialog controlled state, trigger/close asChild, forceMount/hidden, Escape/Tab trap/trigger 복귀를 별도 target별 확인. 등록된 Dialog 이름만으로 이 동작을 인수하지 않음 |

기존 `verification/frame.source-graph.json` SHA `7324754F151E298F672F815D47281929BE3E85C648E6C73161D9AB40E69E2DAA`는 frame+window-chrome2파일만 선언하며 inspector의 재현/지원 증거가 아니다. inspector와 children 제공 caller의 원 graph inventory는 Meari와 명시 조율할 항목이다. 이번 단계에서 재현용 새 제품 source나 도구를 만들지 않았다.

현재 읽은 `packages/meari/packages/meari/src/infrastructure/tsx/prop_type.rs` SHA `FB908A4B629E48F8E0BE764F608A09E3659D24ECE45382DB4C883D3825DDB938`는 string/bool/callback/Array를 분기하고 다른 type reference는 진단한다. `tsx/callbacks.rs` SHA `602BB90DFE8C124B261D5AFA1A357767A2D789E4D5E70EED9713221C9AF69B86`는 Action 또는 required string parameter Text signature를 분기한다. tsx.rs의 react import 특례는 읽은 경로에서 useState/useId이고 JSX expression child는 Text(value) 또는 repeat로 내려간다. 이 구체 경로에서 node slot/boolean callback의 별도 처리를 확인하지 못했으므로 최신 소유자 확인을 요청했다. 다른 신규 public port의 부재나 전체 Meari의 전역 미지원을 단정하지 않는다. consumer-integration.md의 최신 Action/Text·keyed 지원과 과거 unfinished 설명을 구분한다.

Meari 회신에서는 지원되는 정확 public type/entry·target별 증거·미지원 진단 및 작은 보완 소유 범위를 받는다. UI는 원 frame/inspector source와 제품 조합을 소유하고, Meari는 parser/type/graph projection/renderer/widget port를 소유한다. SDK는 이후 style/assets/main assembly 소유를 유지한다. 공개 인계 전 Native/Core 수정, 새 loop, shell 복제/flatten으로 문제를 감추지 않는다.

#### 확정 P1a/P1b/P2 분리와 presentation 설계

PM은 현재 source의 ReactNode named slots/children 명시 거부를 확인했고 SDK 반환 뒤 필수 Meari 보완을 단일 heavy로 먼저 배정하기로 했다. UI가 직접 parser/Native/Core를 수정하지 않는다. PM 중계 Meari [현재 source 검토](../../../../packages/meari/.spec/evidence/reviews/studio-frame-slots-callbacks-2026-10-20.md)와 [확정 분할 계획](../../../../packages/meari/.spec/plans/native-document-callbacks.md)을 실제 읽었다. 최신 확정 라벨은 **P1a typed slots/children 기반 → P1b InspectorDock 포함 공통 프레임 및 필요한 Boolean controlled callback/Dialog semantics → P2 navigation capture/router·탭**이다. 이전 간략 P1 slots→P2 callbacks 설명보다 이 확정을 우선한다. 직접 회신 권한 해석으로 인계 교환은 이후 PM이 중계한다. 실제 public API 구현/실행 인수는 아직 아니다.

- **P1a 프레임 조합 필수:** 원 StudioFrame 그대로 type import·named4 node prop·node-valued JSX child/attribute·혼합 static/node·원 graph instance identity를 연결한다. `children`은 같은 node 타입 계약이다. shell/state/useId/callback의 소유를 유지하고 host가 임의 HTML/string을 주입하는 대용 API로 축소하지 않는다.
- **P1b 원 InspectorDock 포함 공통 프레임:** boolean onOpenChange 및 controlled Dialog 계약은 P1a slots 첫 묶음과 분리하되 P1b의 필수 조건이다. 원 InspectorDock 전체 graph를 P1a만으로 실행했다고 주장하지 않는다. 실제 inspector/editor subtree를 전달하며 Dock 복제품으로 우회하지 않는다. narrow=false 같은 값으로 signature/children 진단을 숨기거나 원 source를 삭제하지 않는다. controlled open/modal/trigger/close/asChild/forceMount/hidden과 focus 정책의 port는 Meari, 실제 narrow/resize/Tab/Escape/복귀·dirty 유지 검토는 UI 소유다. 현재 bool predicates 및 open/defaultOpen/modal focus 일부 구현은 존재하므로 Dialog 전체가 미지원이라는 판정은 하지 않는다. 최신 읽기 근거에서 controlled migration 진단, markup의 modal skip/aria-modal=true, asChild/forceMount 전용분기 미확인이 있으므로 조용한 prop 무시나 generic attribute 출력으로 인수하지 않는다.
- **P1c 창 프레임 동작 필수:** 제품 원 WindowChrome의 최소화/최대화·복원/닫기와 hit/glyph/hover/focus·drag 경계를 실제 공개 host port로 연결한다. 기존 Action callback 포트는 재사용하고 플랫폼 호출은 infrastructure/Native host가 소유한다. minimize/maximize 등의 현재 port를 확인하기 전 임의 API를 가정하지 않는다. 닫기는 동일 SDK/Native dirty·정리 수명을 거친다.
- **P2 후속:** navigation-item의 `onNavigate(props.workspaceId)` 인자 capture, route 등록·하위탭 상태/선택·history semantics는 P1a node type gate와 분리한다. 이미 인수된 Action/Text callback이나 keyed 지원 자체를 미지원으로 되돌리지 않는다. P1의 고정 workspace 상태를 routing 완성으로 세지 않는다.

Meari source 검토의 `source_modules/program.rs:388`은 local component의 nonempty children을 typed slot contract 필요로 명시 거부한다. 이 관측은 intrinsic/widget의 children 전체를 부정하지 않는다. 기존 `parse_module_graph`/`ModuleProgram::mount/project/update_props/activate/input`과 `InteractiveDocument::from_module_program_with_callbacks`를 재사용하며 scalar Value를 임의 node/HTML로 취급하지 않는다. string navigation signature 자체와 props.workspaceId를 보내는 zero-parameter arrow capture도 구별한다.

P1 slot 공급 요구는 UI 관점에서 다음과 같이 PM을 통해 인계한다. 실제 타입명/함수명은 Meari 공개 인계로 확정한다.

- caller는 선언 inventory의 실제 컴포넌트 subtree로 네 required named slot을 공급한다. 네 slot과 children은 현재 source에서 각각 한 번만 삽입된다. JSON/문자열로 임의 JSX·HTML을 받아 실행하지 않는다.
- 필드 누락과 명시 empty slot은 구별한다. UI에 일반 ReactNode의 모든 null/bool/iterable 의미론은 필요하지 않으며 bounded 명시 empty 표현의 문법/API는 담당이 확정한다. P1은 같은 owned subtree를 여러 위치에 복제하는 사용을 요구하지 않는다. 중복 삽입은 명시 진단 가능하며 state/useId/callback owner를 몰래 복제하지 않는다.
- 같은 slot identity의 props 갱신은 dirty editor/input·state/useId/callback을 유지한다. 다른 workspace로 명시 교체하면 이전 slot의 event export를 폐기하고 stale event를 거부하되 frame/chrome/nav/queue 및 무관 inspector state는 유지한다. route 변경/떠나기·SDK 작업 보존 정책의 실제 연결은 P2이며 slot 교체가 SDK owner를 drop/재생성하는 구현을 만들지 않는다.
- caller lexical callback/prop/ID의 소유는 callee의 삽입 위치와 구별한다. foreign/stale node owner와 cycle·깊이/규모/bytes 초과는 bounded 진단으로 거부하며 budget 수치와 generation/lease schema는 Meari의 public 계약을 인수한다. UI가 독립적인 renderer registry나 node ABI를 만들지 않는다.

caller graph 준비의 정확한 상태는 다음과 같다. 현재 제품 source에는 네 slot을 실제 공급하는 root caller가 없으므로 기존 frame.source-graph2개를 완성된 caller로 부르지 않는다. SDK 반환 뒤 UI 소유의 제품 조립 entry와 선언 inventory를 고정하고 최초 실제 file/span 진단을 남긴다. 새 mockup을 공급자로 만들지 않고 UI4의 실제 제품 panel/session과 원 공용 source를 조합한다.

| caller edge | 공급 source/형태 | 현재 남는 경계 |
|---|---|---|
| 제품 조립 entry → StudioFrame | 후속 UI entry의 JSX element-valued named attributes 4개 → 원 `studio/frame.tsx` | entry/manifest는 아직 생성0. Meari P1a public slot 계약 인수 후 실제 product props와 조립 |
| navigation | 원 `studio/navigation.tsx` → `navigation-item.tsx`를 node attribute로 전달 | transitive `onNavigate(props.workspaceId)`는 P2 capture에 걸리므로 **원 navigation까지 포함한 전체 caller 실행은 P2 의존**이다. 단순 graph listing이나 무관 node로 채운 기술 fixture를 실제 제품 탐색 인수로 표시하지 않음 |
| workspace | UI4의 실제 correction/cards/highlight panel 및 기존 공용 source/actions/text-comparison graph와 같은 SDK session | 범용 panel을 P4 전용 전체 workspace의 완성으로 승격하지 않음. 실제 active workspace slot 공급과 필요한 prop mapper는 UI 소유 |
| inspector | 원 `studio/inspector-dock.tsx`의 JSX nested children으로 실제 editor/inspector subtree 전달. `correction-inspector.tsx` 등 기존 source를 실제 read model에 대응 | P1a children 타입뿐 아니라 P1b Boolean/Dialog 의미가 필요. 없는 미디어/cue/waveform 값을 새로 꾸며 넣지 않으며 실제 typed capability와 상태를 구분 |
| queue | 원 `studio/queue-footer.tsx`를 node attribute로 전달, 실제 요약/상세 prop | local bool functional updater/hidden/aria 상태는 현재 지원과 원 source를 대조. placeholder job count를 영속 SDK 조회의 증거로 사용하지 않음 |

Meari 계획은 동일 slot의 다중 placement에 독립 instance/ID를 주고 안정 placement/key만 재사용하는 후보다. UI의 현재 caller는 slot별1회만 요구하므로 이를 축소·부정하지 않는다. 다중 placement를 지원한다면 독립 state/ID와 budget·stale 규칙을 인수하며 같은 owned instance를 몰래 alias하지 않는다는 조건을 유지한다. 임의 부모 간 이동의 상태 보존은 요구하지 않고 같은 placement의 props/resize 재투영만 보존한다.

라이브러리 gate 동안의 겹치지 않는 UI 설계는 다음처럼 준비한다. 아래 파일명은 후속 UI 소유 후보이며 아직 생성/구현한 API가 아니다.

| 책임/후속 파일 후보 | presentation 상태와 입력 | 금지/인수 경계 |
|---|---|---|
| `web_ui_adapter/shell/view_model.rs` | 제품 식별 설정에서 받은 표시명/제작 문맥, 현재 workspace ID, 공용 notice·queue 요약·지원 상태를 읽기모델로 투영 | 제품명 중복 상수·SDK capability 추정·업무 정책 금지 |
| `shell/panel_state.rs` | inspector open/선택/포커스 복귀 대상, host 관측 viewport로 결정되는 narrow 상태, queue expanded | wide/narrow 전환에 inspector instance/무관 편집 입력을 보존. 임의 DPI/VM 변경·독립 resize 정책 금지 |
| `shell/composition.rs` | 기존 frame/chrome/nav/queue/inspector와 현재 workspace를 선언 inventory로 조합. Meari의 실제 node-slot 공개 port를 인수한 뒤 사용 | 원 source flatten/복사·private renderer 의존·JS 런타임·서로 다른 shell로의 중복 mount 금지 |
| `shell/window_intents.rs` | 닫기/최소화/복원 등 명시 Action을 기존 host port에 전달하고 실제 반환 상태를 표시 | OS API 직접 호출·새 loop/SDK/worker·닫기 autoACK/dirty 폐기 금지 |
| 기존 `proposals` session 연결 | 기존 runtime/registry/context/fullref·pending/stale/error·수정 입력/명시 채택 상태를 slot 내 editor에 유지 | route나 resize 때문에 동일 SDK 작업을 재전송하거나 session을 몰래 재생성하지 않는다. 떠나기 정책은 기존 정책과 조율하며 새 UI 전용 정책을 만들지 않음 |

P1 후속 인수는 같은 source graph와 고정 styles/assets로 occupied shell을 실제 실행한 뒤 공통 frame·caption·inspector·큐를 확대 대조한다. 전체13화면은 동일 frame의 소비자로 남는다. 이 설계 준비에서는 파일 생성·코드 변경·검사·GUI 실행 없이 SDK의 최종 gate/슬롯 소유를 유지한다.

### 전체13화면군의 남은 전용 구성

| 화면군 | 현재 TSX source와 구체적 미연결/미제작 영역 |
|---|---|
| VIEW | player/start/transport 선언은 존재. 실제 media surface·작품/회차 목록·탐색·seek/volume/전체화면 overlay·focus/자동숨김 연결 필요 |
| SEG | segments workspace/inspector/candidate-table 선언. 실제 preview·OP/ED 범위 handle·썸네일 시간축·구간 수정/저장·undo 연결 및 sourcePTS 대조 필요 |
| SUB | speech/correction/conversion 및 cue/inspector 선언, correction panel 실제 SDK 연결은 있음. 전체 tabs/workspace에 편집 session 재사용, waveform/media/cue 선택 동기화 필요. OCR/화자 full body route는 null이며 ROI/전용 상태 제작 필요 |
| OBJ | objects route source null. 도구 rail·원본/마스크/합성·안정 피사체/track·검사기/시간축 전용 컴포넌트와 DTO 연결 필요 |
| CLIP | highlight proposal panel/recommendation-item은 있음. 수동 crop·추천 full 후보목록/preview·회차 일괄 계획3개 workspace route는 null. proposal 수락을 실제 영상 출력/게시로 해석하지 않음 |
| FRAME | 원본/썸네일2 route null. 실제 PTS filmstrip·crop/text canvas·레이어/전용 inspector·원본/편집 저장을 구분해 제작 |
| CARD | 텍스트 proposal panel 실제 연결. full workspace route null. 세로 thumbnail rail·카드 canvas·페이지/레이어·플랫폼 파생·alt/출력 UI 별도 제작 |
| A2D | route null. checker/mask canvas·도구/레이어·pivot/bounds·전용 inspector 및 실제 좌표 연결. 기존 egui 구현/관측을 TSX 실행 증거로 전용하지 않음 |
| PROJ | depth/package2 route null. viewport/camera·상대 depth·scene/tree·geometry/투영 package 전용 구성 및 capability 구분 |
| SAVE | route null. 소스/트랙 목록·현재 선택/계획·다운로드/저장 검사기·실제 job/파일 receipt 연결 |
| MODEL | route null. HF revision/license/precision·설치/활성/검토/자원 상태 전용 목록·검사기. 새 모델 실행/다운로드는 이번 UI 범위 아님 |
| JOB | workspace/job-row/inspector/queue 선언 존재. 실제 영속 list/get·필터/cursor/안정 ID/pending/오류·취소/재시도 capability 연결. 6상태를 가짜6탭으로 복원하지 않음 |
| OUT | route null. 실제 결과/저장 경로·충돌/호환 미검증·확인/실패 상태·최종 저장 inspector 연결. 저장 정책은 SDK 소유 |

### 증거와 후속 인수의 구분

SDK/CLI가 증명한 부분은 proposal 원문 보존·edit/CAS/명시 accept/refresh/receipt, native playback의 각 고정 검증 범위, 이번 새 exe6입력의 startup/frame/close/store 불변이다. 이를 GUI에서 같은 정책으로 다시 구현하거나 버튼 클릭 반복으로 기능 정확성을 재증명하지 않는다. CLI/SDK 자료를 UI 표시의 고정 비교 입력으로 소비한다.

OS/실제 렌더가 필요한 부분은 단일 frame·모든 하위탭의 실제 크기/여백/색/폰트 줄바꿈/SVG stroke·caption hit/glyph·grid/flex overflow·canvas/preview 좌표·선택/hover/focus/disabled·한글/일본어 IME·키보드/Tab/Escape/focus복귀·narrow inspector·dirty/오류 표시·video child overlap·fullscreen/overlay/input/close다. 대표 chrome pixel과 3TTF 전달은 전체 glyph·mask·폰트 weight·레이아웃 지원의 증거가 아니다. 시안 요소 확대와1:1 대응 검토는 [시각 잔여 계획](visual-fidelity-follow-up.md)의 전체13군 gate를 그대로 따른다. 이번 준비로 새 GUI/목업/Pages/AI/VM/DPI 작업은 실행하지 않았다.

## 최신 실행 목표와 다음 gate

2026-10-20 최신 인수는 [Pending 문서·실제 공통 SDK consumer83](../../../sots/verification/presentation/pending-document-sdk-2026-10-20.md)이다. correction/cards/highlights 원본 ModuleProgram·공통 SDK worker·typed terminal 수용/ACK·사용자 수정 저장·명시 채택·재투영·headless close를 실제 확인했고, 초기 준비/ACK·부분 조립 복구 및 기존 Immediate/bootstrap을 포함한83pass/0fail/0ignore·strict/fmt exit0를 인수했다. UI source16=5b6515510b154805e5d8d199fdf778f3d439486a0fd6f7a15113def151d17a2f를 동결하고 단일 heavy 슬롯을 반환했다. 아래 controller16/초기 미지원 표와 prepared 기록은 당시 이력이며 현재 세 패널의 검증된 callback/입력/SDK 소비가 미구현이라는 뜻이 아니다. 반대로 이 세 패널 성공을 모든 라우팅·slots·keyed 목록·전체13화면 인수로 확대하지 않는다.

현재 SDK typed close 구현·집중/root/고정 consumer 호환 gate는 SDK 담당이 소유한다. 다음 제품 조립은 [notifier→Native wake NW01~08](native-sdk-wake-gate.md)과 [pre-document startup wake 결정](../../../adrs/player/presentation/native-async-startup-wake.md)을 따른다. Native startup 공개 port·Busy retained dirty·host/child deadline 합성은 아직 보완·인수 전이다. 최신 API/pin을 인수하기 전에 UI source를 미리 변경하지 않으며, 기존83을 새 입력의 통과로 상속하지 않는다. 전체13화면/모든 하위탭·공통 토큰/SVG/Pretendard·입력/키보드/a11y·시각 일치 및 실제 제품 런처/미디어 child 연결 범위는 유지한다.

### 이전 checkpoint 이력

[controller16 gate](../../../sots/verification/presentation/proposal-controller-2026-10-19.md)는 현재 Rust source의 fmt/offline locksync/test16/strictClippy를 통과했다. 실제SDK controller와 edit/accept 각각 inspect실패→명시refresh/동일mutation재전송0을 입증했고 슬롯을 반환했다. CARD/CLIP 패널 원본·최소graph·refresh Action 요구는 인계했다. 다음은 Meari callback lowering/renderer guard/dispatch 공개 인계 뒤 같은 TSX를 실제 consumer로 재검증하는 것이다. 아래16준비 기록은 당시 이력이다.

현재 [공통 proposal 읽기 모델/입력 계약](../../../sots/player/presentation/shared/proposal-view-input.md)은 이전11회귀 통과 이후 field mapper·확인된 mutation/refresh 상태·ReviewController·동일 Meari registry용 ProposalReviewHost source까지 진행했다. fmt/check0, 후속 총16검사는 SDK frame gate 뒤 중앙 슬롯 배정을 받아 실행한다. correction inspector는 공용 ProposalActions의 save/accept/명시 refresh source를 소비하며 신규 자산·스타일 정책은 추가하지 않았다. 실제 TSX callback과 화면·시각 인수는 미검증이다. 전체13화면 범위는 유지한다.

최신 [proposal 실 SDK gate](../../../sots/verification/presentation/proposal-sdk-plugin-2026-10-19.md)는 bridge test7/실제9/strictClippy/build와 고정CLI3task×6=18호출을 완료했다. 원문/NeedsRepair/sourceSHA·명시 기술수정·stale CAS·automaticfalse·재열기를 입증했다. 슬롯은 반환했다. 다음 source는 correction/CARD/CLIP의 공유 Rust read model/입력/pending-stale 표시 연결이며, 현재 Meari callback/ReactNode/keyed 목록 미지원은 정확 재현으로 인계한다. GUI 수동검토·품질승인·자동적용은 보류 상태를 유지한다. 아래 prepared/미검증 기록은 당시 상태다.

독립 [plugin preflight 검사](../../../sots/verification/presentation/proposal-plugin-preflight-2026-10-19.md)는 fmt/test기존6/strictClippy/WASMlib 모두 exit0이다. direct serde 의존성만 offline workspace lock에 동기화했고 슬롯을 반환했다. root SDK 연결·실제9·bridge7·고정CLI는 이 결과에 포함하지 않는다. SDK의 jobs/activation 성공 checkpoint/freeze를 선행해 전체 제안 묶음을 실행하는 순서는 유지한다. 준비한 source는 동결하고 test의 ParentDir 경로 수정만 별도 before/after 근거로 남겼다.

proposal plugin/host/CLI source 준비와 fmt check 완료, 신규 실행은 아직0이다. TypedSdkPlugin의 typed binding에 root canonical DTO와 공유 proposal host를 주입했다. 같은 probe에 per-Request context/CAS와 `--proposals STORE REQUEST` CLI를 연결했고, 실제9 raw를3task 시험으로 create/freshinspect/명시 기술edit/stale CAS/accept/재열기하는 source를 작성했다. 원문/SHA·NeedsRepair·automaticfalse·품질승인없음/unknown provenance를 유지한다. 예약 검사는 plugin6+bridge7 시험·strictClippy·portable lib WASM check·고정CLI task별 재현이다. root SDK/Meari API freeze와 중앙 슬롯 배정 후 실행한다.

실 SDK bridge [r1 완료 근거](../../../sots/verification/presentation/sdk-plugin-bridge-2026-10-19.md): 공유 제품 host+실 ApplicationClient/SubtitleFiles를 같은 Meari registry로 연결해 fmt/실파일4시험/strictClippy/build가 모두 exit0이었다. 고정 CLI는 새 파일0/충돌5/손실동의필요3을 반환했고 원본·충돌출력·root lock SHA를 보존했다. 슬롯을 반환했다. 제품 root 등록/TSX source IR/Native GUI/Web 권한·renderer/전체 시각은 여전히 미완료다. 다음 구현은 [proposal host 계약](../../../sots/player/presentation/shared/plugin-proposal-host.md)의 기존 실제9 source 비교·사람 수정·명시 채택이다. provenance API 부재를 전면 선행조건으로 확대하지 않고 unknown 표시와 현재 canonical SHA/revision을 유지한다. 아래 준비/예약 기록은 당시 상태다.

실 SDK bridge source/fmt 준비: PM 승인 구조로 tools/player-ui-sdk-bridge가 root SDK와 plugin sibling을 조립하고 `src/assembly/meari/subtitle_conversion.rs` 제품 host한벌을 재사용한다. 실제파일/SDK오류·명시loss동의·context보존·동시host/재생성문맥4시험과 고정 CLI 실행을 다음 슬롯에 예약했다. root module/Cargo/SDK를 변경하지 않았고 신규 compile/test는 아직 미검증이다. 현재 예약 순서는 SDK resolver 반환→Meari graph builder14회귀→본 bridge다. 제품 최종등록은 SDK/PM 소유로 인계한다.

최신 제품 플러그인 [r2 실제 registry 검사](../../../sots/verification/presentation/product-plugin-registry-2026-10-19.md)는 기존3+등록3=6회귀/fmt/strictClippy/WASM library check 모두 exit0이다. core 공개 서명으로 PluginAdapter를 구현했고 target enum별 메모리 변환 성공·정본 오류·grant/schema·terminal/unregister를 검사했다. 슬롯은 반환했다. 아래의 draft/pre-freeze 기록은 당시 상태이며 현재 공개 API 미인수를 이유로 막혀 있지 않다. 다음 gate는 root 실 ApplicationClient/context/파일 결과·오류를 같은 registry에 연결하는 공통 SDK/CLI 시험이다. root Cargo/등록 소유는 SDK/PM과 조율하며 source callback·renderer·OS/Web 권한·전체 시각은 미완료다.

Meari plugin 후속 source 읽기에서 오류 전용 schema/Error direction/Immediate Result와 registry capability grants가 추가된 것을 확인했다. 기존 누락 피드백은 source에서 보완됐지만 public freeze/실제 회귀 인계 전이므로 제품 dependency/등록 구현을 앞당겨 섞지 않았다. [host 연결 계약](../../../sots/player/presentation/shared/plugin-conversion-host.md)에 정확 경계와 다음 실제 registry gate를 갱신했다.

첫 제품 플러그인 source는 `packages/player-ui-plugins` 독립 workspace다. canonical 자막 DTO codec과 typed host closure만 구현했고 Meari registry는 복제하지 않았다. [r1 검사](../../../sots/verification/presentation/product-plugin-codec-2026-10-19.md)에서 fmt/test3/strictClippy가 모두 exit0이었다. 시험은 메모리 저장소를 사용하는 실제 canonical 변환 usecase, SDK 오류 보존, 잘못된 DTO/transport 미지원 범위다. 슬롯은 반환했고 Meari 등록·root SDK host assembly·Native/Web 소비 실행은 미연결/미검증이다. SDK request context/ID와 ApplicationClient는 root host가 소유한다. 루트 Cargo/lock/SDK 파일은 수정하지 않았다.

ASR의 speech-workspace와 공용 cue-edit-toolbar source를 추가했다. 6버튼/그룹/구분선과 기존SVG semantic role6개를 공통 토큰으로 대응했으며 실제 입력/renderer/asset 검증은 하지 않았다. [확대 source 대응 기록](../../../sots/verification/presentation/web-speech-source-2026-10-19.md)을 보존한다. route source 경로는 선언 목록일 뿐 Meari Router 등록/화면 완성을 뜻하지 않는다.

최신 실제 근거: [consumer r2](../../../sots/verification/presentation/meari-product-graph-2026-10-19.md)는 fmt/test/strictClippy/build와 원본4파일 ModuleProgram 투영을 통과했다. 부모+자식3instance만 입증하며 렌더·자산·SDK·시각은 미검증이다. PM 슬롯은 반환했다. 후속 제품/환경 플러그인은 [공통 계약 초안](../../../sots/player/presentation/shared/meari-product-plugins.md)과 [배포 ADR 제안](../../../adrs/player/meari-product-plugin-deployment.md)으로 분리했다. Meari 본체 SDK/registry는 본체 담당, 제품 Rust adapter와 TSX binding은 UI 담당이며 공개 API 인계 전 자체 registry를 복제하지 않는다. 다음 gate는 실제 Meari plugin SDK 및 builder graph/asset 연결 인계 후 동일 consumer·실제 SDK로 재검증하는 것이다.

2026-10-19 사용자 goal 갱신 지시에 따라 get_goal을 조회했으며 반환은 goal=null이었다. 진행 중 objective를 교체하는 update 도구는 없지만 이번에는 기존 goal 객체가 없으므로 최신 역할 목표를 create_goal로 한 번 등록했다. 별도 token budget은 설정하지 않았다. 과거 native 시청·제작 전체 시각/키보드/패널 범위는 아래 최신 목표에 포함하며 완료로 간주하지 않는다.

목표는 승인된 PotPlayer 스타일/native-first 제품의 React/TSX UI를 Meari parser/generator/builder/runtime의 실제 consumer로 제작하고 공통 Rust SDK에 연결되는 얇은 presentation을 완성하는 것이다. 전체13화면군/모든 하위탭·공통 프레임/토큰/SVG/Pretendard·세부 상태/입력/접근성/키보드·패널·시각 일관성을 유지한다. 정책·SDK·개발 도구는 Rust로 유지하고 TSX는 사용자 지정 UI 선언 입력이다.

현재 작성은 viewing/player·start-screen·window-chrome·transport-bar, 제작 navigation/navigation-item/queue-footer, JOB workspace/row/inspector·공용notice와 CSS 토큰 projection이다. 탐색은 기존12작업+VIEW 카탈로그를 유지하고, queue 상세는 useState·aria-expanded·hidden으로 선언했다. inspector drawer와 나머지 전용화면·하위상태, 라이브러리 부족 기능 보완, 실제 SDK intent 연결, asset/token 전달과 실제 변환/실행/시각 인수는 남아 있다. 비활성 목업·임의 flatten·별도 JS 런타임·기능 정책 복제로 범위를 줄이지 않는다.

다음 gate는 Meari local-module composition 최소 재현 보완→동일 두 입력 및 실제 제품 source graph 변환 재검증이다. 이후 widget/asset/host intent 각각의 진단·실행 인수를 진행하고, 중앙 슬롯에서 실제 Web 렌더/확대 시안 비교·상태/키보드/패널을 검증한다. 기능 정확성은 공통 SDK/CLI가 선행하며 UI 관측과 구분한다. 현재 parser 인계본/rootSDK 소유 및 정상 검사 source freeze를 유지한다. Pages/VM/DPI/추론/새 장시간 시험은 보류다.

modoo.today design-to-code-fidelity-diff의 실제 본문 적용 근거는 [토큰 대응 검토](../../../sots/verification/presentation/web-ui-token-projection-2026-10-19.md)에 남겼다. 문법/위젯/에셋 부족은 아래 인계표를 보완하고 같은 consumer 재검증으로 닫는다.

2026-10-19 사용자 직접 지시로 제품 Web UI를 React/TSX 선언으로 제작한다. UI를 flatten하거나 별도 JS 런타임으로 우회하지 않는다. 본체 구현은 Meari 담당 소유이며 제품 소스는 src/presentation/web_ui다. [consumer 안내](../../../../packages/meari/docs/consumer-integration.md), [화면 구성](../../../sots/player/presentation/shared/screen-composition.md)을 따른다.

| 경계 | 보존할 최소 입력/기대 | 현재 근거·상태 | 영향과 인수 |
|---|---|---|---|
| 로컬 공용 컴포넌트 | viewing/player.tsx의 WindowChrome/ViewingStart/TransportBar import와 props 전달 | Core9 graph API로 실제4파일 parser 단계 종료0. 단일소스API와구분하며 Web document/Builder graph는미완료 | 모든13화면. 원본 source graph를 Rust IR→Web에 유지하고 shell 복사 없이 실제 표시 |
| 중첩 Button+SVG 표시 | window-chrome.tsx의 Button 안 아이콘 span | native intrinsic validator의 nested widget 제한, Web recipe 지원 범위 별도. 변환/타깃진단 미실행 | caption/toolbar/탐색 전부. hitbox와 glyph를 독립 유지·키보드 이름/포커스 검증 |
| font/SVG asset | styles/tokens.css @font-face 및 components.css mask URL | builder asset/module delivery 미완성 안내 확인 | Pretendard/JP·OFL·기존 SVG를 bundle에 포함. 런타임 CDN/폰트문자 대체 없이 실제 로드·색/stroke/크기 검증 |
| CSS/레이아웃 | CSS 변수·Flex/Grid·minmax·mask·focus-visible | CSS 원문 모드 존재. 실제 소비 검증 대기 | 전체 shared frame. 경로/토큰 손실 진단, 760/1180/1440의 실제 overflow/정렬 검증 |
| 탭/팝업/검사기 | Tabs/Dialog/Select의 선언과 keyboard 동작 | Web shadcn composite runtime 미완성 안내 확인 | 제작 하위20화면과 좁은 검사기. 실제 선택·숨김·Escape·focus복귀와 원본 input 보존 |
| SDK intent | 명시 host action→동일 Rust ApplicationClient, capability와 오류 표시 | source onClick은 bounded state setter이고 product intent bridge 미완성 | 파일/재생/저장/AI. 임의 product 함수 실행 대신 명시 typed host 계약을 Meari/SDK와 인계 |
| 편집 입력 | 제안 비교/경로/검색의 controlled text·checked | Web typed source input 미완성 안내 확인 | 사용자 입력·dirty/오류 보존, 동시 갱신 시 무관 입력/선택 유지 |
| keyed 목록 | verification/keyed-jobs.tsx의 typed rows와 rows.map/key | 현 Props primitive-only/조건·반복 미완성 소스근거. 실제 진단은 중앙/Meari 재현 대기 | JOB/회차/cue/레이어/후보 전부. 안정 ID로 선택·포커스 유지하며 SDK page를 여러 행으로 표시. 단일 고정 행으로 최종 범위를 축소하지 않음 |
| UI 라우팅 callback | studio/navigation-item.tsx의 onNavigate(workspaceId)와 navigation.tsx의 callback prop 전달/selectedId 비교 | 현 primitive Props·호출·attribute expression 제약에 해당. 원본 source 인계/실제 진단 미실행 | 12작업+VIEW 탐색. pure presentation 이동을 제품 명령과 구분하고 현재 선택/포커스 유지 |
| 접힌 큐의 상태/접근성 | studio/queue-footer.tsx의 useState toggle·aria-expanded·hidden={!expanded} | 기존 state setter와 별도로 attribute unary식·aria/hidden lowering은 인수 필요 | 표시/접근성 상태가 같이 바뀌고 Enter/Space·숨긴 내용 focus 접근을 실제 확인 |
| 공통 frame slots | studio/frame.tsx의 ReactNode navigation/workspace/inspector/queue와 inspector-dock children | 현재 primitive Props를 넘어서는 typed node slot·type import·컴포넌트 조합 필요. 담당에 원본 인계 | 모든 제작 화면의 shell 중복 없이 동일 source와 state identity 유지 |
| responsive inspector | inspector-dock.tsx controlled Dialog/modal/asChild/forceMount·boolean expression props | Web Dialog 미완성과 source 표현식/children 제약. 실제 진단/실행 대기 | Rust observed host의 narrow prop과CSS1280경계 일치. 한 inspector 인스턴스·wide nonmodal/grid·narrow Escape/Tab trap/trigger복귀·닫기/resize 입력보존 |

현재 source의 disabled 제품 버튼은 SDK 연결 전 상태의 정확한 표현이며 완성품/최종 인수로 세지 않는다. 디자인 제작을 비활성 목업으로 끝내지 않고 라이브러리 보완 후 같은 source로 변환/실행/상태/시각 인수를 이어간다. 실제 Diagnostic/빌드/실행/화면 비교는 각각 따로 기록한다. Rust SDK 기능 인수는 계속 CLI가 선행하며 이 UI에 기능 정책을 복제하지 않는다.

## Meari 보완 전 변환 baseline 인계

### Router·변환 화면 후속

conversion-inspector.tsx에 source/output 경로·명시SRT/VTT·speaker policy·손실 확인·새 파일 실행을 controlled source로 추가했다. encoding/원본 보존은 SDK 표시값으로 받고 offset/선택 구간은 미지원 안내와 비활성 입력을 유지한다. 임의 ASS 지원·원본 삭제·덮어쓰기를 구현하지 않는다. canConvert/canBrowse/busy는 명시 host capability/state이며 실제 실행은 onConvert가 동일SDK typed request를 호출하는 후속 bridge에 맡긴다.

손실 확인은 SDK가 반환한 현재 요청의 손실 목록과 결합해야 한다. 원본/출력 경로·형식·화자 정책이 바뀌면 presentation adapter가 이전 확인 상태와 결과 context를 해제하고 새 요청을 구분해야 한다. source 파일 변경/출력 충돌/손실 정책 판단은 application/domain에 유지하며 UI가 parser/검증 정책을 복제하지 않는다. 실제 host bridge·변환 실행/저장·UI 인수는 아직 하지 않았다. useId/controlled select/text/checked/외부 callback과 boolean식 지원은 Meari 인계 범위다.

사용자 React Router 지시에 따라 [제품 route 계약](../../../sots/player/presentation/shared/web-routing.md)과 routing/routes.json을 Meari에 인계했다. 기존 callback을 독자 router로 확정하지 않는다. 과거25와 현재20의 차이는 r159의 JOB6상태탭→1페이지+본문필터 분류 정정으로 확인됐다. 승인 이미지19studio+6viewing=25와 하위화면 수를 혼동하지 않는다. 현재19studio+화자1=20,12workspace/route목록과 일치하며 범위 삭제나 새 기능 완료가 아니다. 당시 r158 감사는 보존한다.

convert.png와 실제 ConvertSubtitles/ConversionResult를 읽고 conversion-workspace/syntax-row/loss-notice source를 추가했다. 원본·출력 preview는 별도 slot이고 stable cue ID/문법/시간/본문을 받으며 교정안 열이나 완료 receipt로 변환 문법을 대신하지 않는다. 손실 목록은 SDK가 제공한 자료만 표시하고 UI가 ASS/offset/범위 지원을 시안에서 추론하지 않는다. 현재 SDK 지원은 SRT/VTT UTF-8이며 actual dry preview 계약과 명시손실 확인·실제요청 bridge는 후속이다. source 구성은 작성했지만 변환/런타임/시각 인수는 미실행이다.

최신 실제 근거는 [제품 graph 검사](../../../sots/verification/presentation/meari-product-graph-2026-10-19.md)다. consumer fmt/test명령(0tests)/strictClippy와 viewing4파일 graph 파싱은 종료0, keyed/queue/frame의 실제 file/span 진단은 같은 고정 도구로 보존했다. 아래 보완 전 baseline과 과거 대기 문구를 최신 실행 결과로 소급하지 않는다. 다음 gate는 Meari Web graph document/Builder bundle·assets·typed host bridge이며 원문/수정/명시 채택을 실제 SDK에 연결한다.

### 내장 asset catalog 후속

사용자가 공식 Rust/React SVG Icon 카탈로그와 Pretendard 공통 내장을 Meari에 요청했다. [제품 소비 경계](../../../sots/player/presentation/shared/web-assets.md)와 assets/requirements.json의29개 의미 role·기존 파일·glyph/hit·font revision/weights/fallback 요구를 인계했다. 아직 실제 catalog ID·resolver 연결을 완료한 것은 아니다. 공식 팩 provenance/license는 Meari, 제품 custom SVG는 제품이 소유하고 기존 자산 삭제/중복 취득 없이 같은 source를 재검증한다.

### 자막 source 후속

correction.png를 직접 열어 전용 중앙 구성과 검사기 비교 구조를 재확인했다. studio/subtitles/tabs.tsx는 기존 카탈로그의 음성/화자/OCR/교정/변환5탭 선언이며 나머지4탭 본문을 구현했다는 뜻이 아니다. 교정 workspace는 media/waveform/cue slots를 구분하고 TextComparison은 원문·제안·상태를 받는다. waveform·ROI·diff·근거·sourcePTS를 가짜 그림/값으로 채우지 않는다.

cue table은 안정 ID·표시 번호·formatted 시간·원문/교정안과 선택을 받아 keyed 반복으로 표현한다. inspector는 proposal/source identity/revision·미디어 binding 상태를 명시하고 controlled multiline draft와 save/explicit accept callback을 구분한다. capability/busy는 투영 props이며 CAS/채택/저장/AI 정책을 UI에 복제하지 않는다. 원문 문서 제안과 영상 cue 연결을 동일하게 표현하지 않는다. 실제 SDK intent bridge와 textarea/input·repeat·slot 지원은 Meari 담당에 원본 source로 인계했다.

CSS는 기존 font/body13/small11·label72·index48·time132·border1/radius5 토큰 projection을 사용한다. 시안 raster 색/확대 stroke/실제 줄바꿈·IME·keyboard·drawer 검증은 아직 없다. product consumer 최초 실행은 Meari Core9 자연 종료/API freeze 뒤 예약된 fmt/test/Clippy·viewing4파일 graph check부터 진행한다. 이 자막 source는 아직 변환/실행하지 않았다.

### 실제 제품 consumer 실행 경로

tools/player-ui는 독립 Rust workspace다. check 명령은 선언된 inventory를 bounded 파일 읽기로 로드하고 실제 meari::parse_module_graph에 전달한다. viewing/source-graph.json은 제품 entry와 공용3컴포넌트의 원본4파일을 지정한다. 성공 결과에도 runtime_verified/visual_verified=false를 표시한다. build 명령은 Meari Builder의 BuildRequest/execute에 위임하며 임의 HTML/JS 생성기로 대체하지 않는다. 도구의 Rust source 포맷만 종료0이고 패키지 검사·실제 제품 graph check·Native/Web build는 아직 미실행이다. Meari composition 체크포인트 뒤 중앙 검사 슬롯을 요청했다.

PM이 전달한 고정 meari-cli SHA `886239e5ddb410e218d6b50586545ff2af01e27dbeab62555a9f2feca25e8f6d`의 최소 재현은 packages/meari/.spec/artifacts/repro/consumer-composition/ 두 파일이다. viewing.tsx local import는 exit1/unsupported import/span0..58, window-chrome.tsx 단독은 exit0/schema2로 변환됐다. 원문 진단은 같은 라이브러리의 .spec/evidence/verification/consumer-composition-baseline-2026-10-19.json이다. 이는 제품의 더 큰 source 전체가 변환·실행됐다는 증거가 아니며 공용 조합 부족을 좁힌 baseline이다. 본체 보완 후 동일 두 입력과 실제 src/presentation/web_ui/viewing/player.tsx source graph를 각각 재검증한다.
