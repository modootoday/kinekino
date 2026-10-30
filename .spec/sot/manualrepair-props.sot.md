---
id: sot:kinekino/manualrepair-props
title: ManualRepair 실행 props 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# ManualRepair 실행 props 계약

2026-10-25. [공통 요소 계약](../shared/element-review-contract.md)을 따른다. 이 문서는 현재 후계의 구현 계약이며 실행 성공이나 GUI·AI 품질 승인 문서가 아니다.

## 현재 소스와 검증 상태

제품 소스 정본 후보는 [전용 projection source10](../../../../../runtime/development/product-manual-repair-projection-r1/HANDOFF.md), manifest SHA256 `8A88587A8E3EA8F9E85C3DE4B3B7991E7EAF408D49E58E00EC0DCA598E8D4C3D`다. 선임은 source262 `9DFB56A10982D97D454A4A32BA078BFD0ACF317DC38A7A9BF11F6B59E1224E6D`, 초기 source3 `38EAA2A5AD4E6B9A1D4F5FB247A28A68FF97E11E2E3B87442A20B24BDA8816B7`의 제한적 수리를 구조적으로 대체한다. canonical 제품 파일에 반영됐다고 표시하지 않는다.

| 단계 | 현재 사실 |
| --- | --- |
| 설계 | 전용 명시 builder·원본 binding·컴포넌트별 feedback 경계 정의 |
| 소스 | source10에 구현; 미소비 필드 삭제 누적/typed props 상속/자동 retain 없음 |
| 선임 실검증 | source262는 fmt/type/clippy/no-run 성공 뒤 신규 manual exact 첫 시험 실패: 0 통과/1 실패/15 미실행. 구체 typed 오류 수집 누락으로 원인 확정 불가 |
| 후계 실검증 | projection267 manifest `12294FDFD027B4137F6ADEA0023171FBFD4E090BB95646BFE8AD693296579377`: fmt 성공, 첫 type E0560 실패. clippy/no-run/exact27 미실행 |
| 후속 소스 | [IntentPins 시험 생성자 source1](../../../../../runtime/development/product-manual-repair-projection-pins-fix-r1/HANDOFF.md) `10ECF98FF63CDF8A65E125CA97A56E443D4FD462C008B4030BD4C12AD573035B`: 실제 contract/provider/request/reply 필드로 수정 |
| 최신 중앙 실행 | pinsfix 포함267r2 source `39E0FC7E023906C1E42934647493ABFD3FC3D699EFC0D2D755D00A6E8127A880`: SDK 실보고 기준 fmt/type exit0(타입53.410s), strict Clippy exit0(46.921s), Root jobs no-run exit0(252.340s). exact12 통과 후 13번째 ManualRepair callback 기대 목록 불일치로 중단, 14 미실행. 신규 상태3task·원 live mount3task 포함12 통과는 GUI 승인 아님 |
| 다음 기대 계약 수정 | [callback expectation source1](../../../../../runtime/development/product-manual-repair-callback-expectation-r1/HANDOFF.md) `D5F40468E2FC1071785C9E2523363395B2331A28A33DE207E1816A92D2FCB5EE`: source10 선언/실사용에 맞는 Text2+Action1을 원 exact expected 맵에 추가. assert_eq 유지. fmt/check 성공, 중앙 재실행 대기 |
| 최신 완료 CPU gate |267r3 source `720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137`: SDK 실보고 fmt/type/Clippy/no-run 및 영향27개 각각 실제 통과. 위 r2 실패는 역사 이력이며 현재 callback36 기대 계약도 통과 |
| GUI·품질 |267r3 정상 GUI에서 실제 Correction 모델 raw와 ManualRepair 구조 표시, 마운트 후 AwaitingFrame에서 정지. 최종 원 오류 Slot(NotVideo), scene15/admission revoked. 실제 활성화·전체 원문/후보/이력·3task GUI 인수와 사용자 AI 품질 승인 미완료. [실행 근거](../../../verification/presentation/normal-gui-projection267-r3-awaiting-frame-2026-10-25.md) |

## 선언 키와 공급 책임

source10의 `src/presentation/web_ui/studio/manual-repair.tsx`와 Rust `repair_panel/{projection,bindings,feedback}.rs`가 아래 계약을 함께 소유한다. props는 필수이며 값이 없을 때도 명시적인 빈 문자열·typed 빈 배열·비활성 상태를 제공한다. 아래는 독립된 기능 정책이 아니라 표시/입력 경계다.

| 책임 | 필수 키 / 타입 |
| --- | --- |
| 원본/수정 텍스트 | string: proposalId, sourceChoice, sourceText, generatedRaw, importError, resultLabel, front, back, title, workingText; string[]: sourceOptions |
| 수정 상태 | boolean: selectionDisabled, startDisabled, cardDisabled, highlightDisabled, correctionDisabled, canSave, canAccept, busy, refreshRequired, isCard, isHighlight, isCorrection |
| 수정 입력 | Text: onSourceChoice, onFrontChange, onBackChange, onTitleChange, onDraftChange; Action: onStartManual, onSave, onAccept, onRefresh |
| 원 후보 텍스트 | string: rawReviewPackage, rawReviewItem, rawReviewNote, rawReviewSource, rawReviewCandidate, rawReviewStatus, rawReviewError, rawReviewVerdict, rawReviewSaved, rawReviewModel, rawReviewSelection, rawReviewRetainedIdentity, rawReviewRetainedResponse, rawReviewAttempt, rawReviewRecovery |
| 원 후보 배열 | string[]: rawReviewCriteria, rawReviewItems, rawReviewIssues, rawReviewHistory |
| 원 후보 상태 | boolean: rawReviewCanInspectRetained, rawReviewCanHistoryRetained, rawReviewCanReleaseRetained, rawReviewFieldsDisabled, rawReviewCanRead, rawReviewCanRecord |
| 원 후보 입력 | Text: rawReviewOnPackageChange, rawReviewOnItemChange, rawReviewOnNoteChange; Action: rawReviewOnInspectRetained, rawReviewOnHistoryRetained, rawReviewOnReleaseRetained, rawReviewOnOpenPackage, rawReviewOnOpenItem, rawReviewOnReadSource, rawReviewOnReadCandidate, rawReviewOnHistory, rawReviewOnRecord, rawReviewOnUsable, rawReviewOnNeedsChanges, rawReviewOnDeferred |
| 채택 출력 텍스트 | string: outputReviewStatusLabel, outputReviewErrorLabel, outputReviewNote, outputReviewReviewerLabel, outputReviewVerdictLabel, outputReviewReviewerId, outputReviewClaimedTime, outputReviewSourceText, outputReviewGeneratedRaw, outputReviewDraftText, outputReviewAcceptedText, outputReviewEvidenceLabel, outputReviewAttemptedLabel, outputReviewConfirmedLabel |
| 채택 출력 배열 | string[]: outputReviewHistoryRecords, outputReviewTargetLines, outputReviewCriteriaLines |
| 채택 출력 상태 | boolean: outputReviewCanRecord, outputReviewCanRetry, outputReviewFieldsDisabled, outputReviewAvailable, outputReviewBusy, outputReviewHasInspection, outputReviewSelected, outputReviewResultUnknown, outputReviewHistoryNotChecked |
| 채택 출력 입력 | Text: outputReviewOnNoteChange, outputReviewOnReviewerLabelChange, outputReviewOnReviewerIdChange, outputReviewOnClaimedTimeChange; Action: outputReviewOnInspect, outputReviewOnHistory, outputReviewOnSelect, outputReviewOnUsable, outputReviewOnNeedsChanges, outputReviewOnDeferred, outputReviewOnRecord, outputReviewOnRetry |

원 후보와 채택 출력의 prefix 계열은 기존 전용 projector/state가 공급한다. 넓은 output 계약의 target/criteria/원문/모델/저장 초안/채택 출력/시도·확인/재시도 필드를 버리지 않고 TSX에서 실제 표시·입력에 사용한다. 모든 입력은 기존 ReviewIntents 한 queue/node와 원본 action/input token을 공유한다. 필드 편집 뒤 바뀐 draft generation의 token을 다시 사용한다.

## 상태와 후단 경계

초기에는 수정본을 생성하거나 원문을 자동 선택하지 않는다. 사용자의 명시 source 선택과 BeginManualRepair 뒤에만 기존 SDK/ReviewSession의 수정 필드가 나타난다. Cards front/back, Highlights title, Correction workingText와 참조 원문을 직접 구성하고 사용하지 않는 작업 필드는 빈 값·비활성으로 유지한다. 다른 typed editor의 sourceIdentityLabel, sourceRefs, qualityLabel, revisionLabel 및 전용 disabled props를 가져오지 않는다.

저장/채택/CAS·검증 오류·refresh·pending 여부는 원본 API/session 사실을 표시한다. 공통 ReviewFeedback의 rendered revision/result/error 문구를 재사용하되 ManualRepair는 선언된 resultLabel만 생성한다. bootstrap과 bridge 후단도 이 전용 경계를 사용하며 revisionLabel을 재삽입하지 않는다. typed Card/Highlight/Correction의 기존 apply 키·문구는 유지한다.

ManualRepair는 원 후보 읽기/쓰기 표시를 자체 소유해 raw_review=false다. exact NodeSlot 검사, 원 inspection ACK/release, callback export, 동기 receipt/retained proof와 실제 observed-route admission gate를 유지한다. 헤드리스 가짜 frame으로 편집 gate를 열지 않는다.

## 필수 회귀와 판정 범위

`presentation::web_ui_adapter::proposals::repair_panel::tests::manual_repair_state_projections_match_exact_component_contract`는 합성 scratch의 실제 SDK 응답으로 세 task의 초기/선택/working/편집/검증·stale 오류/저장/confirmed inspect 오류/current refresh/채택/출력 판정·이력을 조립한다. 각 상태는 실제 FrameProjection/NodeSlot 조립을 사용하며267r2에서 이 시험은 통과했다. 전체 exact 묶음은 별도 graph 기대 목록 실패로 완료되지 않았다.

| 경계 | 현 manual 시험의 구현상 검사 | 현재 실행 근거 |
| --- | --- | --- |
| 실제 TSX + production props | 실제 ManualRepair graph와 production projector를 FrameProjection.mount로 조립하고 project 호출 |267r2 해당 exact 시험 통과 |
| required 누락 | sourceChoice를 제거한 조립의 오류를 단언 |267r2 해당 exact 시험 통과 |
| 임의 extra | unknownPrivateExtra를 추가한 조립의 오류를 단언 |267r2 해당 exact 시험 통과 |
| 잘못된 값 타입 | 실제 값의 정상 조립은 검사하나, wrong-type 값을 주입하는 별도 negative 없음 | 현 시험이 명시적으로 보장하지 않음 |
| callback signature | production binding.signature와 TSX 정상 조립을 사용하나, 잘못된 서명 주입 negative 없음 | 현 시험이 잘못된 서명 거부를 독립 입증하지 않음 |
| 외부 node 콜백 | 원 queue/node/token을 사용하나 foreign-node 콜백을 주입·등록하는 negative 없음 | 현 시험이 외부 node 거부를 독립 입증하지 않음 |
| 실제 live admission | 이 시험은 SDK 상태의 headless projection 조립이며 실제 observed frame이 아님 | GUI 입력·포커스·live gate 통과를 입증하지 않음 |

Meari 담당의 조사상 기존 strict API로 전체 consumer의 실제 TSX·production props·callback signature를 연결하는 공통 conformance gate 구성이 가능하다. 이는 향후 공통 검증 설계 가능성이고 현재 구현·전 consumer 실행 완료를 의미하지 않는다. 위 누락 negative를 새 소스로 추가하지 않은 상태에서 보장 범위를 넓혀 보고하지 않는다.

267r2의 graph 실패는 source10에서 콜백 선언/producer/실사용을 확장하면서 원 exact callback expected 목록을 같은 변경 묶음에 갱신하지 않은 누락 사례다. 실제36/기대33의 차이는 outputReviewOnReviewerIdChange/Text, outputReviewOnClaimedTimeChange/Text, outputReviewOnRetry/Action이다. 후계는 정확한3개만 추가하고 equality·서명·capability 검사를 유지한다. 관련 선택 소스 검색에서 다른 ManualRepair exact 기대 목록은 발견하지 못했으며 다른 typed consumer 기대값은 변경하지 않았다.

원 후보 verdict/history 사례는 명시적인 합성 UI state이며 원 후보 SDK 영속화 성공을 대신하지 않는다. 실제 원 후보 영속화·복구는 기존 SDK 회귀를 재사용한다. 합성 출력 Deferred 기록도 scratch 시험이며 사용자 품질 승인이 아니다. initial live-entry 시험은 원 prepared Open/current inspection/release/mount 소유권까지만 검증하고, 이후 화면 편집 인수를 주장하지 않는다.

다음 변경은 TSX 선언+명시 producer+binding 서명+전체 상태 exact 회귀를 동일 후계로 인계해야 한다. 문서/소스 검토 통과를 중앙 타입·시험이나 GUI 승인으로 승격하지 않는다. [독립 감사](../../../verification/presentation/manual-repair-state-props-source-audit-2026-10-25.md)는 선임 결함의 역사적 근거이며 현 후계 구현 목록을 대신하지 않는다.

기존에 읽은 modoo.today defect-triage-evidence/test-fixture-and-mock-build를 적용해 실제 오류·선임/후계·합성 상태·실행 증거를 분리하고, 원본 SDK 정책 회귀와 UI 계약 회귀의 책임을 구별한다.
