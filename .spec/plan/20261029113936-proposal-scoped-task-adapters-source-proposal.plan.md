---
id: plan:kinekino/20261029113936
title: 교정·하이라이트 scoped adapter — 공통 session 연결 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 교정·하이라이트 scoped adapter — 공통 session 연결 제안

## 승인 후 private 작성 상태

PM exact9 승인 뒤 runtime/development/ui-scoped-task-adapters-r1에 기존5/new4를 작성·동결했다. source9 SHAF227A690B28A7A2C9A96F9DFBDB2E66131B120B8B571A08270A5995EC20F97F9, before9 SHAB5192A130FA58043465322286573D29668AF9C2ABFA7294B18194F5F660D90D2. proposals/mod.rs만 승인된 private task3 predecessor를 쓰며 canonical과 다른 preimage를 명시한다. 나머지8은 root89 preimage/부재다. mismatch0, 작성시험12/compile0/actual0/direct leaf9 fmt/check 통과. HANDOFF에 exact 시험·재사용·실패 provider 보존·후속 gate/미연결 범위를 기록했다. task3와의 묵시 합성·지원 인수 주장은0이며 source9 단독 overlay 실행을 하지 않는다.

2026-10-23 root89f59f5d7ea9301dae63cef72759b6775d66994c/libraryb532 기준 읽기 설계다. UI49는 PM이 채택했고 root index0/push0이다. 현재 heavy는 SDK effective8-r2 소유다. 본 작업은 새 코드·Cargo·S/T·창·수동 시각 검토0이다. explicit task source3는 별도 gate 대기이며 이번 제안과 묵시 합성하지 않는다.

## 관측한 공통 구현과 실제 차이

기존 NativeProposalSelection/BootstrapSelection은 Card/Highlight/CueCorrection/DocumentCorrection/DocumentRepair의 ID·source SHA·revision·index·cue/ref를 이미 보존한다. bootstrap::project_initial은 full identity/target membership를 확인한다. panel_model::PanelSelection/prepare_panel은 Card와 Highlight를 이미 지원한다. correction_panel::CorrectionSelection/prepare_correction은 cue/document correction과 명시 repair 선택을 이미 지원한다. document_factory는 이 둘을 구분해 같은 ReviewSession과 session_assembly::create_session에 연결한다. 새로운 SDK 수정·채택 정책은 필요하지 않다.

현재 scoped_panel::bind_card_panel은 Card만 허용한다. 그러나 원 registry/scope 검사, ReviewSession, 원 등록, create_session, partial session/parent retention, quiescent unbind는 범용이다. window_bridge::mount_scoped의 소유 타입도 ScopedCardPanel에 묶여 있고 correction label provider를 mount 전에 요구하지 않는다. 반면 ProposalWindowBridge의 실제 후속 재투영은 이미 CorrectionSelection/prepare_correction과 with_correction_labels를 지원한다. 이 공통 경로를 재사용하고 Card 구현을 복사해 새 두 binder를 만들지 않는다.

assembly::scoped_card_session::ScopedProposalLease는 동일 ExecutionHandle에 등록만 붙이며 원 worker를 shutdown하지 않는다. CardCurrentInspection의 요청·terminal·ACK/release 코드는 target에 따른 새 SDK 정책 없이 기존 pending consumer를 사용한다. 이름이 Card라는 이유로 inspect worker를 새로 만들지 않는다. ProductFrameBlueprint는 현재 Viewing/Cards/Jobs만 갖고, Viewer의 CardRoute에는 capture 전용 소유가 섞여 있다. 따라서 adapter 지원과 Viewer SUB/CLIP route 지원은 별개 인수다.

## 첫 구현 배정 제안 exact9 — 기존5/new4

아래는 제안이며 아직 편집하지 않았다. 기존 source3와 proposals/mod.rs 등록 경로가 겹치므로 source3 인수·채택 상태를 PM이 확정한 뒤 preimage를 다시 고정한다.

| 경로 | 책임 |
|---|---|
| 신규 src/presentation/web_ui_adapter/proposals/scoped_task_panel.rs | 공통 ScopedPanelInput/Panel/Failure/Metadata와 bind_task_panel. 원 Card binder의 scope/owner/partial failure 계약을 한 구현으로 이동. Card/Highlight는 원 prepare_panel, cue/document correction은 원 prepare_correction을 사용 |
| 신규 src/presentation/web_ui_adapter/proposals/scoped_task_panel/tests.rs | 명시 second item/원문·generated·수정안 분리/full cue/ref·stale/scope/label/등록 실패·partial cleanup 및 Card wrapper 영향 시험 |
| 기존 src/presentation/web_ui_adapter/proposals/scoped_panel.rs | 기존 ScopedCardInput/Panel/Failure public 이름은 공통 타입 re-export로 보존. 기존 bind_card_panel은 Card-only 검사를 유지하는 얇은 wrapper로 위임. 기존 테스트·호출자 struct input 필드는 유지 |
| 기존 src/presentation/web_ui_adapter/proposals/mod.rs | 기존과 같은 Windows/native-proposal-window cfg의 새 공통 모듈 등록만 |
| 신규 src/assembly/meari/scoped_proposal_session.rs | 원 ScopedProposalLease와 공통 bind_scoped_task 등록 조립. 동일 원 ExecutionHandle/AsyncProposalRegistration/create_session을 사용하고 endpoint/store/worker 생성·shutdown 정책을 소유하지 않음 |
| 기존 src/assembly/meari/scoped_card_session.rs | 공통 lease re-export 및 기존 bind_scoped_card의 Card-only wrapper. 원 등록/해제 의미를 유지 |
| 기존 src/assembly/meari/mod.rs | 같은 cfg의 공통 scoped_proposal_session 등록 |
| 기존 src/presentation/web_ui_adapter/proposals/window_bridge/scoped.rs | 공통 panel을 원 checked handle/zero callback suffix에 설치. 기존 Card mount API를 보존하고 새 typed task mount 진입에서 correction provider를 commit 전에 필수 검사. 기존 source/props commit port/원 receipt 및 실패 input 보존 |
| 신규 src/presentation/web_ui_adapter/proposals/window_bridge/scoped/task_tests.rs | 원 NativeHostRuntime/원 SDK owner에 연결할 headless 통합 후보. correction provider 누락·foreign scope·남은 callback·commit 실패 소유보존, controlled text→save 응답/ACK→별도 accept→reopen·explicit unbind를 구별 |

새 공통 Input은 기존 Card input의 필드를 유지한다. CorrectionLabels는 새 bind 함수의 별도 명시 인자로 받고, 후속 bridge에는 CorrectionLabelProvider를 mount 전에 공급한다. Card 호출자에 Option 필드를 무조건 추가해 구조체 리터럴 전체를 고치는 방식을 피한다. 기존 Card panel 이름은 같은 실제 공통 타입의 re-export로 유지해 bridge 소유를 복제하지 않는다. 새 target 종류를 소스 callback 문자열로 역직렬화하지 않는다.

최초 범위는 Card/Highlight/CueCorrection/DocumentCorrection의 이미 명시된 item이다. DocumentRepair의 별도 source 선택 UI는 source3가 만들지 않으므로 자동 ref·첫 텍스트를 넣지 않고 이 adapter 단계에서는 명시 unsupported로 거절한다. 기존 독립 NativeProposal repair 지원은 그대로 보존한다. repair를 나중에 포함하려면 원 choose_document_text/CorrectionSelection::repair 계약의 별도 연결을 제안한다.

## 원문·수정·명시 채택과 권한 경계

stored는 원 실행의 실제 inspect 응답을 소비·ACK한 결과여야 한다. project_initial에서 원 selection identity와 payload membership를 검증하고 ReviewSession이 source/generated/working draft/acceptance를 분리한다. cue는 cue_id와 원 text, document는 정확 DocumentTextRef와 원 asset SHA, highlight는 title와 source_refs를 보존한다. title/source refs로 실제 시간 범위·seek·자르기 기능을 추정하지 않는다.

local text change는 원 controlled callback→ReviewIntents→기존 pending controller로 간다. Save와 Accept는 서로 다른 원 application 요청이다. 실제 저장 receipt/후속 inspect·ACK 전에는 저장 완료로 표시하지 않고, 저장 성공만으로 accept/원문 적용을 하지 않는다. CAS 충돌·오류·취소·stale는 원 API 응답과 입력 보존 규칙을 사용한다. 공통 품질 판정은 기존 capability/ProposalOutputReview 범위이며 자동 사람 승인으로 취급하지 않는다.

project_task_choices의 captured row/live_inspection_selection은 표시 항목을 가리키는 inspect 입력이며 편집 권한이 아니다. fresh inspect 결과가 같은 ID/SHA라 해도 captured origin을 live editable로 승격하지 않는다. 이 첫 adapter는 원 host가 이미 승인된 live entry와 원 SDK terminal을 연결하는 내부 조립 계약에 한정한다. 향후 route origin은 LiveExplicit와 CapturedReadOnly를 구별해야 하며 구체 SDK 정책/authority 발급을 UI에 새로 만들지 않는다. 기존 guarded captured Card edit 계약은 Card에 유지한다. 자막/Highlight captured mutation 지원은 별도 SDK 계약이 없으면 미지원으로 남긴다. SDK 담당에게 이 경계를 source-only로 조율 요청했다.

모든 callback은 원 frame generation/opaque local key·scope/owner를 확인한다. 동일 generation을 독립 프로젝트에서 재사용한 source3 key 자체는 origin proof가 아니다. adapter 진입은 queue provenance와 원 registry scope를 함께 요구하고 URL/query에 원문·draft·로컬경로·권한을 넣지 않는다.

## 후속 Viewer route 연결의 후보 경계 — 첫 exact9에 포함하지 않음

SDK 담당은 source-only 조율에서 captured/current inspect 일치로 live 권한을 발급하지 않는 경계와 원 scope/registry/execution/ACK된 StoredProposal+명시 선택 소비 계약을 확인했다. 새 origin API를 즉석 추가하지 않으며 실제 필드·수명 차이가 발견될 때 정확 계약/경로를 별도 조율한다. 위 LiveExplicit/CapturedReadOnly는 이 구분을 설명하는 설계 명칭으로, 구현된 공통 SDK enum/API 또는 새로운 UI 권한 정책이 아니다.

첫 adapter와 source3의 독립 gate 뒤 별도 exact 변경을 배정한다. 다음 책임 위치를 후보로 고정하되 실제 preimage·규모는 그때 다시 제시한다.

- 신규 src/assembly/meari/viewer_host/proposal_route/{mod.rs,selection.rs,opening.rs,active.rs,return_viewing.rs,tests.rs}: task 선택/원 inspect drain·ACK/release/공통 bridge/원 viewing 복귀를 분리한다. 기존 CardRoute의 capture 상태를 복사하지 않고 범용 live panel owner 하나만 보유한다.
- 기존 src/assembly/meari/viewer_host/{mod.rs,close.rs} 및 viewer_callbacks/lifecycle.rs: 기존 Viewing/Jobs/Card와 함께 원 owner를 순서대로 poll/close한다. 한 turn당 commit1·callback batch barrier·원 continuation/wake를 유지하며 SDK 기능 정책을 추가하지 않는다.
- 기존 src/assembly/meari/product_viewer_launch/project.rs: 원 프로젝트 choices/source3의 명시 선택과 origin 전달만 조립한다. 첫 항목 선택·capture origin 자동 upgrade0.
- 기존 src/presentation/web_ui_adapter/player_shell/document/product_frame.rs와 src/presentation/web_ui/product/frame.tsx: 원 closed inventory에 교정/Highlight entry를 합성하고 typed workspace slot을 추가한다. 기존 FrameComposition/제품 토큰·폰트·공통 shell을 재사용한다.
- 기존 src/presentation/web_ui/routing/routes.json: 실제 구현·인수 뒤에만 studio.subtitles.correction 및 studio.shorts.recommendations의 source/지원 상태를 연결한다. 현재 null/명시 지원 차이를 설계만으로 완료 처리하지 않는다.

Route ticket 성공은 실제 commit/proof와 원 checked frame 관측을 연결해야 한다. JOB의 retained proof를 붙여 쓸 때도 실제 lineage/receipt를 보존하고 metadata/None으로 완료시키지 않는다. 기존 Card strict full-stamp 정책을 일괄 완화하지 않는다. 진입 실패와 return 실패는 원 owner/SDK terminal/partial session/receipt를 보유한 채 명시 retry·close로 정리한다.

## 기대 검증·미검증

현재 새 작성·compile·actual0이다. 첫 exact9 배정 시 pure target/projection와 원 session 소유 계약을 Rust로 작성한다. 기대 범주는 explicit second Highlight, cue/ref mismatch, source SHA/revision drift, missing correction labels before registration, foreign owner/registry, scope release 중 원 worker 보존, partial bind/commit 실패와 동일 owner retry, 원문·dirty/CAS·save 후 별도 accept/reopen이다. Card wrapper가 non-Card를 계속 거절하고 원 Card 회귀를 통과하는지도 포함한다. 등록 성공 전/후 실패 carrier를 분리하고 synthetic frame을 actual로 세지 않는다.

실행은 source3 gate/PM 자원 배정과 exact9 독립 source review 이후에만 한다. 실제 SDK save/accept 결과는 같은 공통 SDK·CLI 기존 검증과 연결하며 GUI로 기능 검사를 반복하지 않는다. headless가 통과한 뒤 실제 Native 연결은 별도 bounded child로 원 HostRuntime/checked receipt/cleanup을 확인한다. 수동 GUI·13화면 시각 인수·미디어/모델·새 워커·새 router·새 launch default는 이 첫 범위 밖이다.

적용 출처는 기존에 탐색·본문을 읽은 modoo.today modootoday-free/design-delivery-plus/frontend-first-journey(공통 원문/typed ports/기존 컴포넌트 재사용과 지원 상태 구분), modootoday/software-qa/test-fixture-and-mock-build(원 setup/body/cleanup/error carrier·실제/합성·권한 경계 분리)다. foreign-language 예제를 도입하지 않고 새 구현/시험은 Rust, TSX는 승인 UI 선언 범위에 한정한다.
