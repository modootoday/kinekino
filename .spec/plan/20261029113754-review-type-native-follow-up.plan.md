---
id: plan:kinekino/20261029113754
title: 공통 review의 유형별 후속 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 review의 유형별 후속 인수

최신은 [원본 Card/Highlight Native SDK gate](../../../sots/verification/presentation/original-review-sdk-2026-10-19.md)다. 제품 각3모듈 원문으로 독립 입력·참조2개 렌더링·stale·save/accept/refresh·새 SDK/store 재열기를 확인했고 전체27pass/0fail/0ignored·strict/fmt0이다. 아래 map 실패·미실행 준비 기록은 이전 단계다. 실제 모델 품질·freshCLI·제품 창 등록·전체13화면 시각 인수는 계속 남는다.

다음 원본 Card/Highlight의 fixture·참조2개·독립 입력·stale/save/accept/refresh 및 동결 입력·실행 순서는 [원본 SDK gate 준비](original-review-sdk-gate.md)에 모았다. top-level SDK fixture/typed props 연결은 original_review_sdk.rs에 소스로 추가했으며 아직 컴파일·실행 전이다. 소스 helper를 실제 성공으로 집계하지 않는다.

최신 제품 실제 실행은 [R3 Document 수동 수정 Native 인수](../../../sots/verification/presentation/document-repair-sdk-2026-10-19.md)다. NeedsRepair에서 명시 선택/수동 시작/입력/save/accept/refresh/같은process 새SDK·store 재열기를 확인했다. 회귀 합산24pass/원본map intake1fail/0ignored, strict/fmt0이며 전체green·root등록·freshCLI·품질승인·전체시각 인수는 아니다. 아래 실행 전 소스 준비 기록은 해당 시점 이력이다.

최신 Array kernel checkpoint는 Core47/Native18·workspace/WASM strict·최종fmt933ms0으로 인계됐고 UI가 [본체 실행 근거](../../../../packages/meari/.spec/evidence/verification/array-kernel-2026-10-19.md)를 읽었다. 원본 두 graph의 실제 다음 진단은 `components/proposal-source.tsx` span538..609, unsupported, `only string/bool literals and props.name supported; hooks/effects/expressions unsupported`다. 양쪽 array-kernel-2026-10-19-r1/array-originals/{cards,highlights}/first-diagnostic.json을 직접 확인했다. 배열 선언과 forwarding 지원 이후 keyed map 표현식이 남은 상태이며 positive 실행은 계속101이다. 아래 Boolean·배열 선언 진단은 이전 checkpoint 이력으로 보존한다. 최신 공개 enum 변경에 대한 제품 consumer 컴파일은 아직 실행 전이다.

Boolean checkpoint 이후의 실제 최초 진단은 Cards/Highlights 모두 `components/proposal-source.tsx` span91..112, code=unsupported, message=`prop type must be string, boolean or a supported callback signature`다. UI는 packages/meari/.spec/runtime/bool-props-2026-10-19-r1/array-originals/{cards,highlights}/first-diagnostic.json과 array.receipt.json을 직접 읽었다. positive array 실행은3341ms/exit101이고, Core 선택42pass는 본체 인계112052ms/exit0이다. 이후 본체 최종 인계에서 Core42/Native18, workspace·WASM strict 및 fmt가 exit0으로 확정됐고 슬롯이 반환됐다. Actions OR 선행 장애의 보완과 array/keyed map 미지원은 구별한다. 이 최종 Bool 인계는 제품 Card/Highlight Native 실행의 성공 근거가 아니다.

아래 checked-native… baseline은 Boolean 보완 전 원진단으로 그대로 보존한다. sourceRefs 배열 선언 뒤 lexical keyed map·원본 origin/span/identity의 범용 인수도 별도로 남으며, 원본 TSX와 refs2개를 바꾸지 않는다.

최신 실제 original graph baseline은 Meari 담당 array_intake에서 확보됐다. 컴파일 뒤 실행1개가 실패101(18756ms)했으며 Cards/Highlights 모두 최초 진단은 `components/proposal-actions.tsx` span377..430, code=unsupported, message=`only string/bool literals and props.name supported; hooks/effects/expressions unsupported`다. UI는 두 first-diagnostic.json을 직접 읽어 대조했다. 이 boolean 표현식 선행 장애를 Array 진단으로 바꿔 적지 않는다.

원증거는 packages/meari/.spec/runtime/checked-native-presentation-2026-10-19-r1/array-originals/{cards,highlights}/first-diagnostic.json이다. 대조 시 제품 proposal-actions.tsx SHA256은 `1b85327fe66ff0c39918a3c6de4da68ccad57bee9a9105f28406be6b1ca0760f`, proposal-source.tsx는 `8e2145deafa7ebafc8dc797fd17e5c56c1f30bd3cb32d23575d5e7d2dfb0ee70`였다. 이는 본체 실행 baseline이며 UI의 독립 intake 실행이나 Native 저장/채택 결과는 아니다. 본체가 같은 원본으로 boolean→후속 array/keyed map 장애를 순서대로 보완하며 제품 source는 바꾸지 않는다.

2026-10-19 Native Correction23 인수 후의 범위 대조다. SDK 실제 미디어 슬롯 중에는 제품 원본 graph/계약을 읽고 인계하며 새 consumer build는 다음 공통 checkpoint에서 실행한다. Native 성공 범위를 소급 철회하거나 다른 유형으로 확대하지 않는다.

| 유형 | 현재 canonical 구조·근거 | 실제 Native에서 남은 항목 |
|---|---|---|
| Cue Correction | cue_id/text, source revision/SHA, 시간 Option. 기술 cue-a/revision7의 원본 Review Native 문서→SDK/store 회귀 통과 | 실제 미디어 cue binding·선택 이동·source 변경, 전체 correction inspector graph·시각/키보드 |
| DocumentCorrection | canonical DocumentTextRef의 asset SHA/outer pointer/extraction/inner pointer. 기존 actual9 R3 SDK/CLI 및 공통 projection/fields source 유지 | R3 companion 선택·DocumentSourceSnapshot·NeedsRepair/import_error/raw 분리·명시 수동 typed draft 시작→Native 편집/채택·provenance 오류 표시 |
| Cards | front/back/source_refs, 실제9 중 card SDK 사례·공통 ReviewItem/TextInput/공용 동작 보존 | 원본 card-proposal graph의 앞/뒤 독립 입력·다중 참조·선택 변경·stale, Native 저장/채택/refresh. CardDocument canvas/preset/이미지 출력과 구별 |
| Highlights | title/source_refs, source cue 시간 None 보존, 실제9 중 highlight SDK 사례·공통 ReviewItem 유지 | 원본 highlight-proposal graph의 제목·참조·시간 미상·Native 저장/채택/refresh. ClipDocument의 실제 구간 확정/재생/내보내기와 구별 |

## 입력 시작과 provenance 누락

검토 대상·evidence/baseline/ModelCondition 의미와 원 candidate decision 비전이 규칙은 [제품 등록 후속 계약](proposal-product-registration.md)의 해당 절을 따른다. actual source 선택/NeedsRepair→명시 edit/accept→fresh CLI+project binding→같은 원 candidate/item decision 재조회 순서를 별도로 인수하며 현재 기능 gate의 추가 선행조건은 아니다.

현재 Native 기술 case는 valid typed draft로 시작한다. NeedsRepair에서 draft=None이면 text_fields가 없으므로 이를 첫 cue/card를 임의 선택하거나 raw JSON을 몰래 복구해 우회하지 않는다. 실제 source 참조의 명시 선택과 사용자 수동 draft 시작을 제품 입력 계약으로 연결해야 한다. UI는 working draft를 소유하지만 canonical parsing/CAS/저장은 SDK가 소유한다. 해당 repair 시작 경로는 이번 Native 인수에 없다.

후속 소스는 `studio/subtitles/document-repair.tsx`, 공통 ReviewSession·intent queue·callback adapter와 `tools/player-ui-sdk-bridge/tests/document_repair_sdk.rs`에 준비했다. 한 개 원문을 묵시적으로 고르지 않고 사용자가 입력한 1부터 시작하는 항목 번호를 canonical DocumentTextRef로 해석한다. 별도 시작 동작은 빈 working DocumentCorrection만 만들며 원문·generated raw·historical import_error 및 저장 상태를 변경하지 않는다. 실제 R3 companion/candidate를 기존 SDK reader로 읽어 NeedsRepair/draft=None부터 원본 두 모듈 Native→선택→수동 시작→입력→save/accept/refresh→새 SDK/store 재열기를 검증하도록 구성했다. 아직 컴파일·실행 전이며 기술 입력은 사람의 품질 승인이나 fresh CLI 검증이 아니다.

원문/생성 raw/저장 draft/작업 중 draft/receipt·historical import_error는 계속 구별한다. quality_decision_known/generation_provenance_bound=false를 다른 품질 문서에서 임의 true로 합치지 않는다. Timeout fixture 하나로 provenance NotFound/corrupt/ref mismatch 표시 인수를 대신하지 않는다. 필요한 SDK 정본 오류/식별 확장은 SDK/AI 담당과 조율한다.

## 변경하지 않을 원본 graph

독립 인수 source는 tools/player-ui-sdk-bridge/tests/review_graph_intake.rs와 support/typed_review_case.rs에 추가했다. SDK 등록 경로나 제품 presentation source는 수정하지 않았다. direct rustfmt/check만 통과했고 실행은 SDK 슬롯 뒤 공통 checkpoint에서 예약한다. 기존23+원본 graph intake1의 예상24이며 이 추가1은 두 원본 graph의 최초 진단 검사다. 새로운 Native 유형 성공이나 합성 SDK 검증 수로 집계하지 않는다.

intake는 Card/Highlight 원본 각3파일을 그대로 parse_module_graph에 전달하고 원본 bytes/hash·canonical seed·첫 ModuleDiagnostic JSON(file/code/span 포함)을 각각 보존한다. 실제 parser 실패에만 실패하며 parser 성공도 Native/SDK/시각 status는 미검증으로 남긴다. 아직 진단을 실제 실행하지 않았으므로 예상 span을 만들어 적지 않는다.

ReviewCase의 공통 수직 함수는 실제 ModuleProgram/Native constructor(scope factory 주입), 기존 제품 owner용 유형별 field binding, Native text 입력→fresh props 재투영→명시 save/accept/refresh를 준비한다. Cards는 앞/뒤 두 field, Highlights는 title이며 source_refs는 cue-a/cue-b 두 개를 유지한다. 실제 SDK/client/store 및 post-accept inspect 실패 주입은 기존 실문서 fixture 경계에서 주입해야 하고 이 함수 자체가 별도 SDK 정책을 만들지 않는다.

준비 당시 공개 Core PropType은 String/Bool/Callback이고 Value에 typed array 표현이 없었다. 최신 Array kernel은 typed array와 forwarding을 제공하므로 sourceRefs:string[] 공급 port는 그 공개 계약에 맞춰 후속 연결한다. mount는 sourceRefs prop 누락을 거부하며 가짜 Value variant/단일 문자열/첫 참조로 대체하지 않는다. keyed map 최초 진단은 위 최신 실행 근거를 따른다. 공개 데이터 adapter 연결과 원본 Native 실행 전에는 Card/Highlight 저장/채택 성공으로 표시할 수 없다.

R3 gate 반환 후 저자원 소스 준비로 ReviewCase의 reference_value를 공개 runtime::IntoValue/Vec<String> 변환에 연결했다. mount는 cue-a/cue-b 두 참조의 순서까지 일치하는 typed value를 요구하며 intake는 그 값을 별도 JSON으로 보존하도록 준비했다. 이 후속 변경은 마지막24pass/1fail gate 이후이며 아직 컴파일·실행하지 않았다. 원본 graph와 SDK 정책은 변경하지 않는다.

PM의 읽기 전용 연구 인계는 string[]뿐 아니라 ProposalSource의 keyed .map lexical item 표현도 현 lowering 미지원임을 확인했다. Array DTO 추가만으로 원본 지원을 완료하지 않는다. 범용 frontend/IR·module forwarding·key identity·원본 origin/span 처리는 Meari 소유이며 제품은 원본2 inventory/TSX와 refs2개를 유지한다. 읽기 근거와 다음 consumer 실행의 최초 실제 ModuleDiagnostic은 구분한다. map/참조를 제거하거나 flatten하여 통과시키지 않는다.

- verification/card-proposal.source-graph.json: CardProposalPanel→ProposalSource/ProposalActions, onFrontChange/onBackChange와 explicit save/accept/refresh.
- verification/highlight-proposal.source-graph.json: HighlightProposalPanel→같은 공용 컴포넌트, onTitleChange와 같은 명시 동작.

두 원본은 sourceRefs:string[]와 keyed map, details/summary, callback forwarding을 포함한다. 현재 이 제품 graph의 parser→renderer 실행은 미검증이다. 다음 gate에서 같은 원문을 소비해 최초 실패 file/code/span을 기록하고 Meari 본체 보완 후 같은 graph로 재검증한다. 단일 고정 참조로 flatten하거나 목록을 제거해 완료로 만들지 않는다. 원본 CSS·폰트·SVG·실제 시안 확대 비교는 기능 인수 이후 별도 중앙 범위다.

## root 등록·CLI 동등성

독립 consumer 성공은 player.exe root 등록이 아니다. SDK 소유의 [제품 조립 인계](proposal-product-registration.md)에 따라 actual document registry/SDK client·store/context와 모듈/Cargo를 연결한다. UI는 root Cargo나 공통 SDK 정책을 직접 수정하지 않는다.

각 유형은 같은 application DTO/유스케이스·source identity/expected revision·API 오류·immutable receipt를 GUI/CLI가 공유해야 한다. Native23의 재열기는 같은 test process의 새 SDK/store 객체다. 별도 CLI process 인수는 기존 근거와 정확히 연결하거나 다음 제품 gate에서 확인하며 GUI 클릭으로 CLI 검증을 대체하지 않는다. 전체13화면군·모든 하위탭은 이 review matrix만으로 완료되지 않는다.
