---
id: plan:kinekino/20261029113838
title: Viewer 세 작업 연결 — 원 owner 재사용·최소 단계 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewer 세 작업 연결 — 원 owner 재사용·최소 단계 제안

R1 최종 repair4 source3FD29712는 실제 신규8/영향15 검증·보호·반환 후 rootaa88706에 채택됐다. R2 successor1 C5BD3B83은 선행 동결본이며, PM 승인 수리 후 최종 source14 5007ED36의 headless 고유74 근거를 검증·반환했다([정본](../../../sots/verification/presentation/live-mount-r2-2026-10-24.md)). 원 exact12와 추가 기존 시험2를 구분하고 canonical 채택은 PM 소유다. 현재 실행 슬롯은 반환됐으며 추가 fixture11/R3/R4/창 실행은 미배정이다. 캐시2 보호 실패 예외도 반환 정본에 포함한다. 별도 task3/scoped9 gate와 route 실행 권한을 혼동하지 않는다. 문서 끝의 가상 모니터 검토도 현재 DPI/VM 보류 범위에 속하며 자동 재개하지 않는다.

초기 제안은 2026-10-23 root89f59f5d7ea9301dae63cef72759b6775d66994c/libraryb532의 소스 읽기에 귀속한다. 아래 R0 정정은 root72c8132a01ca4168fdec8a0678121b67b4224b45/libraryb532에서 SDK의 [선택 공급·captured 편집 계약 대조](proposal-live-selection-r0-sdk-review.md)를 읽고 반영했다. R0 정정을 위한 코드/Cargo/S/T/CLI/GUI 실행은0이다. 아래 exact 경로는 각 후속 배정을 위한 제안이며 현재 작성 권한이 아니다. 실행 권한은 execution-leases.md만 소유한다.

## 입력 계보와 현재 사실

채택된 UI49의 실제 JOB 왕복/원 cleanup 근거는 jobs-retained-source49-2026-10-23 정본에 귀속한다. task source3 SHA7058FDCF49482C3DDC0F589768F3332E652D97A653967C3B0D827D44ED1E7AA3은 실제 새6/Card4 검증 후 root72c8132에 채택됐다([검증 정본](../../../sots/verification/presentation/project-task-choices-2026-10-23.md)). scoped source9 SHAF227A690B28A7A2C9A96F9DFBDB2E66131B120B8B571A08270A5995EC20F97F9는 이 task3 baseline에서 실제 새12/기존 영향11을 검증하고 보호·명시 반환한 뒤 PM이 rootf4f678af1b2f370efea80a2bcc13842a44400142에 채택했다([검증 정본](../../../sots/verification/presentation/scoped-task-adapter9-2026-10-23.md)). 동결 before9의 mod predecessor는 이제 채택된 task3와 같지만 원 R1 source547은 root89 preimage의 역사적 동결본이고, 최종 repair4 source3FD의 실제23 결과와 rootaa 채택은 별도 [R1 정본](../../../sots/verification/presentation/live-entry-r1-2026-10-24.md)에 귀속한다. 각 단계 시작 때 실제 새 root와 exact preimage를 다시 고정한다. 이 두 gate는 R1/R2의 actual 사용자 진입·route·frame 관측 인수가 아니다.

기존 무인자 Player는 eframe/Glow 경로이며 이미 세 작업의 원문 비교·수정·저장·별도 채택을 공통 SDK로 소비한다. explicit --native-viewer는 별도다. application/launch/native_viewer.rs는 --native-proposal-request 혼합을 금지한다. source3/9가 생겼다는 이유로 이 기본 진입/옵션 정책을 바꾸거나 Native 기본 사용 가능으로 표시하지 않는다.

현재 CardRoute는 active/rejected bridge, inspection/start failure/mount partial, capture surface와 return을 원 close owner가 함께 본다. ActiveCardBridge는 실체가 ProposalWindowBridge<ScopedProposalLease>다. 동일 generic bridge 하나를 세 live task가 재사용할 수 있다. JobsLedger를 새 task마다 복제하거나 별도 endpoint/SDK worker/close loop를 만들 이유가 없다.

CardCurrentInspection은 원 ExecutionHandle/같은 context의 pending Inspect/terminal/ACK/release를 이미 소유한다. open_card의 captured_editor_unavailable와 finish_inspected_card는 captured selection을 editable로 자동 승격시키지 않는다. 이 guard를 제거해 새 live 진입으로 쓰지 않는다. 원 captured Card의 onCapturedCardEdit/SDK mutation guard와 legacy read-only는 각각 보존한다.

## 연결할 사용자 흐름과 권한

사용자가 명시한 live 문서 항목→원 frame local key/scope/generation 확인→원 route guard→현재 Viewing/Jobs owner의 terminal drain·ACK·attachment release→동일 SDK/current inspect→원 full identity/target 검증→inspect ACK/release→공통 scoped panel→실 props commit/원 proof→actual checked frame→관측 완료→편집 활성의 순서다. 원문/source refs와 원생성·사람 working draft는 ReviewSession에서 분리하며, text 변경/Save/별도 Accept/reopen은 원 controller/application만 호출한다.

captured 행은 source3로 세 종류를 표시하더라도 현재 Viewer의 일반 선택 경로에서는 CapturedReadOnly다. 같은 ID/SHA의 current inspect가 돌아왔다는 사실, ProposalStoreOrigin 문자열, URL/query, 로컬 key 자체는 live mutation 권한이 아니다. ProposalStoreOrigin은 원 코드가 명시한 provider dataset identity이며 permission이 아니다. 현재 별도 guarded edit의 UI 소비자는 CaptureCardModel/Card 경로다. 이는 SDK가 Card만 지원한다는 뜻이 아니다. 공통 SDK의 EditCapturedProposal은 ProposalPayload를 받고 Correction/DocumentCorrection/Cards/Highlights의 task/source ref 검증과 원 writer·origin·CAS guard를 적용한다. 교정/Highlight captured UI는 이 generic 계약을 소비하는 입력·연결·종류별 검증이 남았으므로 현재 읽기 경로를 유지하며 SDK Unsupported로 재분류하지 않는다.

LiveExplicit/CapturedReadOnly는 설계상 출처 구분이며 구현된 신규 SDK enum이 아니다. 원 SDK import/create 결과 또는 기존 trusted local NativeProposalLaunchRequest로 host가 이미 보유한 live 선택을 전달하는 것이 첫 연결 입력이다. renderer는 host가 보유한 선택 key만 보낼 수 있고 ID/SHA/ref/origin을 새로 조립하지 않는다. source3의 generation은 원 caller가 화면/프로젝트 교체마다 갱신하고 원 opaque key+scope 검증을 함께 한다.

## R0 — 기존 typed 결과 handoff 우선·generic captured 계약 구분

SDK의 [주장별 소스·원 실행 대조](proposal-live-selection-r0-sdk-review.md)를 R0 근거로 연결한다. 현재 configured SDK에는 proposals.list가 없다. 반면 create_proposal은 ProposalDocument, import_historical_proposal은 StoredProposal을 반환하며 PendingRequest::Create/Import, PendingOutput::Created/Imported 및 기존 async 등록 포트가 있다. 필요한 typed 재료가 없다는 주장은 정정한다. 현재 기본 Viewer가 이 결과를 live chooser에 전달하고 있지는 않으므로 handoff 연결은 별도 UI 후속이다.

첫 선택 공급은 이 원 create/import 결과 또는 기존 trusted local NativeProposalLaunchRequest를 우선 재사용한다. 원 host가 전체 typed 결과와 출처를 보유하고 사용자가 정확 item 또는 명시 repair source ref를 선택하면 기존 NativeProposalSelection/Target으로 전달한다. repair ref가 없으면 Unselected로 남긴다. 원 결과 terminal의 소비/ACK/release를 원 ExecutionHandle owner가 마친 뒤 같은 current inspection을 연결한다. renderer는 bounded opaque key만 제출하며 scope/generation/원 owner 검증, 교체 시 generation 갱신, stale·foreign·초과·중복 정책을 적용한다. 첫 행 자동 선택이나 captured→live 권한 변환은 없다. 새 SDK payload/선택 권한 발급 포트·worker를 만들지 않는다.

generic captured mutation 계약과 현재 UI/CLI 근거도 구별한다. EditCapturedProposal의 공통 ProposalPayload와 domain 검증은 Correction/DocumentCorrection/Cards/Highlights를 처리하며 원 guarded writer 아래 origin·source/generated/historical immutable bytes·전체 state CAS를 확인한다. 표시 identity, supplied CaptureExpectation, ACK 자체는 권한이나 Accept가 아니다. 원 guarded open이 origin 없는 legacy를 거부하고, expected 생략 시 stale capture를 current로 자동 갱신하지 않는 정책을 그대로 소비한다. project working_draft/프로젝트 CAS 저장은 live proposal Accept와 별개이고 cross-root transaction을 보장하지 않는다.

종류별 실제 CLI 근거는 SDK 대조 문서의 원 Correction exact1(2026-10-21 이전 기준, 실제 standalone edit/edit/accept/retry 및 fail-closed)로 귀속한다. 이 조사에서 Cards/Highlights/DocumentCorrection captured exact CLI 양성을 확인한 것은 아니며 현재 root 재실행도0이다. 현재 Card UI 소비가 있다는 사실과 generic 계약이 있다는 사실로 다른 종류의 CLI/UI 인수를 대체하지 않는다. 미확인 근거를 SDK 미지원으로 바꾸지도 않는다.

저장소 전체 live chooser의 proposals.list는 별도 SDK 읽기 후속이다. UI가 filesystem을 열거하지 않는다. SDK 제안은 configured dataset/RequestContext에 종속된 bounded metadata page→명시 inspect→원 host-held item 선택의 두 단계다. 목록에 원문/generated/draft/파일 경로/권한이나 새 mutation permission을 넣지 않으며 정확 item/cue/ref는 원 inspect 결과에서 선택한다. scan·정렬·응답 상한, cursor/dataset·동시 수정·stale·취소/close 정합과 exact SDK 경로는 SDK 대조 문서의 제안에 귀속하며 아직 구현·인수하지 않았다. create/import handoff만으로 전체 저장소 chooser가 완성됐다고 표시하지 않는다.

## R1 — 명시 live 진입과 같은 owner 연결 seam, exact9(기존7/new2)

승인 후 runtime/development/ui-live-entry-r1에 작성·동결했다. source9 SHA5479218AD0639821E23130827E0D6A11C34AAE02C559B414182F4B5A2881C607/before9 SHA09F9540B145A0F0297C3244135BB70594A5F30348EDD3CB281AEA08CC52190E6, existing7/new2/preimage mismatch0. R1 exact9는 source3/9와 파일 중첩0이며 모두 root89 preimage/부재다. private 선행 후보는 별도로 기록하고 합성/인수하지 않았다. 이는 최초 작성8/compile0/actual0 동결 시점 기록이다. 이후 fixture leaf만 repair1–4로 수정했으며 최종 source3FD29712의 실제 신규8/영향15 통과·보호·반환 후 rootaa88706에 채택됐다. 원 실패 로그와 수정 계보는 R1 정본에 보존한다. actual 사용자 진입·editor mount/route 관측/return은 계속 R2 이후 미구현이다.

전제: task3/scoped9의 인수 상태와 R0에서 사용할 원 live 선택 공급자를 확정한다. 사용자 화면·편집 mount 완료를 이 단계에서 주장하지 않는다.

| 경로 | 최소 책임 |
|---|---|
| 신규 src/assembly/meari/viewer_host/live_entry.rs | 원 host-held 명시 선택·현재 scope·route ticket의 한 live opening 상태. 기존 CardCurrentInspection과 cards.inspection/active/partial 소유를 재사용. 별도 ledger/worker 없음 |
| 신규 src/assembly/meari/viewer_host/live_entry/tests.rs | 선택 출처/정확 cue·ref·generation·old key/중복/foreign·entry 중 close/원 terminal·ACK/release 영향 계약 |
| 기존 src/assembly/meari/viewer_native_launch.rs | 기존 ViewerChoices와 같은 원 pre-export 조립에 명시 live choices variant/별도 진입을 연결. 기존 ViewerLaunchInput 필드를 무조건 늘려 모든 호출자를 바꾸지 않음. 기본/Standalone/Captured 모드 보존 |
| 기존 src/assembly/meari/viewer_callbacks/lifecycle.rs | 원 HostCallbacks에 bounded task entry key/intent 연결, SDK 정책0. 같은 원 original_execution/context와 read release 재사용 |
| 기존 src/assembly/meari/viewer_callbacks/selection.rs | host-held live/captured 출처를 구분해 key→원 선택만 resolve. 기존 Card captured queue/guard를 재사용·보존 |
| 기존 src/assembly/meari/viewer_host/mod.rs | 원 turn/batch barrier 뒤 한 live entry driver 조립. 기존 ActiveCardBridge public 이름을 보존하며 같은 bridge 소유를 세 task가 공유 |
| 기존 src/assembly/meari/viewer_host/cards.rs | 같은 active/inspection/rejected/partial owner slot에 live task metadata 추가, captured와 상호배타. captured 전용 owner를 새 task에 복사하지 않음 |
| 기존 src/assembly/meari/viewer_host/close.rs | 원 OrderedClose가 동일 ledger의 live opening/inspection/partial/active를 끝까지 관측하도록 연결. 새 close 순서/loop 없음 |
| 기존 src/assembly/meari/viewer_host/jobs/route.rs | live opening/active와 JOB 진입 상호배타만 보강. 원 SDK calls/JobsLedger·continuation·효과 정책은 보존 |

진입과 중복은 원 opaque key 검증 후만 admission한다. 동일 current intent는 새 SDK inspect/commit 없이 coalesce할 수 있어야 하고, Capacity로 무한 남겨 zero-batch와 서로 기다리게 하지 않는다. 서로 다른 작업 요청은 원 bounded queue/Busy 정책으로 명시 거절한다. failed inspection/cancel/ACK 미완료는 원 결과/owner와 함께 남으며 다른 SDK attachment를 붙이지 않는다. captured 기존 open_card/mount의 금지 guard는 이 단계에서 바꾸지 않는다.

## R2 — 공통 mount·실프레임·복귀, exact12(기존10/new2)

Meari 담당의 [b532 API 읽기 대조](../../../../packages/meari/.spec/evidence/reviews/r2-live-entry-retained-api-2026-10-23.md)를 읽어 아래 계약에 반영했다. 현재 library API로 충분하며 새 library/API 추가 제안0이다. 최초 exact12 작성 뒤 PM은 별도 headless gate와 exact 수리들을 배정했다. 최종 effective14의 fmt/strict/CargoJSON 및 고유74 근거는 R2 정본에 귀속한다. actual Some/사용자 화면·시각 양성은 인수하지 않았고 현재 실행권은 반환했다.

R2 원 source6A0A5864를 보존하고 successor1 sourceC5BD3B83/delta53C8F784에서 채택 R1 fixture만 상속했다. 이는 초기 successor 동결 시점의 다른11 불변·신규14 작성/compile0/actual0 기록이다. 이후 수리 계보와 effective14/74근거는 R2 검증 정본을 따른다. [최소 headless 실행계획](live-mount-r2-headless-gate.md)과 [별도 실제 frame fixture 제안](live-mount-r2-fixture-gates.md)을 구분한다. R1 입력을 predecessor로 명시한다. 신규 ProductProposalMount를 두고 기존 Card mount는 호환 wrapper로 위임한다. 세 panel controller/factory/등록·worker를 다시 구현하지 않는다.

| 경로 | 책임 |
|---|---|
| 신규 src/assembly/meari/viewer_host/mount_proposal.rs | 원 mount_current_card의 closed graph/registry/FrameComposition/partial retention을 공용 조립으로 추출. task별 기존 product module·label·scoped9 binder를 선택 |
| 신규 src/assembly/meari/viewer_host/mount_proposal/tests.rs | 실제 원 SDK inspection→ACK→같은 scope bind/controlled input·save/별도 accept·CAS/원문 보존과 rejected commit 소유 계약 |
| 기존 src/assembly/meari/viewer_host/mount_card.rs | 원 Card 입력/실패 public 내부호환 및 captured 금지 경로를 보존하는 wrapper. 기존 mount 시험은 그대로 영향 검증 |
| 기존 src/assembly/meari/viewer_host/live_entry.rs | 성공 commit의 receipt/proof를 fallible observer arm 전에 보유. one commit/turn, explicit retry와 단발 continuation/redraw. 관측 전 editor intent는 활성화하지 않음 |
| 기존 src/assembly/meari/viewer_host/live_entry/tests.rs | entry/return actual stamp advancement·원 receipt·재arm/중복/close/error 회귀 |
| 기존 src/assembly/meari/viewer_host/cards.rs | 원 partial session/mount/rejected bridge/active의 공통 타입과 task metadata 연결. 별도 소유 복제0 |
| 기존 src/assembly/meari/viewer_host/mod.rs | 이미 있는 active bridge dispatch/HostEffects 및 원 frame reflection 순서와 연결 |
| 기존 src/assembly/meari/viewer_host/return_viewing.rs | 기존 ViewingReturn을 live task의 원 출발 route에 적용. hard-coded studio.cards를 임의 raw route로 풀지 않고 원 active metadata로 검사. capture return 유지 |
| 기존 src/presentation/web_ui_adapter/player_shell/document/product_frame.rs | 한 StudioFrame을 공유하고 기존 Card/Correction/Highlight provider modules를 closed inventory에 합성. task별 slot key/명시 entry 선택. 원 ProductFrame workspace shape를 유지 |
| 기존 src/presentation/web_ui_adapter/player_shell/document/product_frame/tests.rs | 공통 frame/module 충돌·원 source fingerprint·기존 Viewing/Card/Jobs slot 영향 및 교정/Highlight editor reference 검증 |
| 기존 src/assembly/meari/player_shell_host/observation/retained.rs | 원 view.start wrapper는 유지. 새 typed live task entry 허용 route만 별도 진입으로 공통 proof 검증에 연결; 모든 route/string을 허용하는 완화0 |
| 기존 src/assembly/meari/player_shell_host/observation/retained/tests.rs | route 불일치/foreign/current epoch/new projection·None 거절 및 원 view.start/strict Card 정책 영향 회귀 |

FrameComposition은 고정 studio/frame.tsx와 실제 editor reference를 합성한다. 같은 StudioFrame 소스를 task마다 다른 구현으로 복제하지 않고 동일 module source 충돌 검사 아래 공유한다. 제품 root frame.tsx는 이미 workspace NodeSlot을 받으므로 이 단계의 새 UI root 파일은 필요 없다. closed graph에 실제 provider가 없으면 명시 실패하며 새 모듈을 런타임 파일 경로에서 찾아 넣지 않는다.

현재 arm_retained_viewing_route는 view.start만 허용한다. 새 live entry의 typed 허용값은 studio.cards/studio.subtitles.correction/studio.shorts.recommendations로 한정하는 별도 계약이 필요하다. 기존 strict Card/Jobs와 그 full stamp 검사는 유지한다. pending strict/retained의 상호배타, 원 host/owner/definition/epoch·route ticket·scene/context·monotonic notice guard는 그대로다. actual Some의 frame stamp/ticket을 notice DTO와 대조한 뒤 원 document.validate_retained_frame을 통과해야만 route 관측 완료다. None/metadata/과거 frame=current 비교/임의 receipt로 완료시키지 않는다.

mount 성공과 route 관측 성공은 다르다. commit 후 arm borrow 실패는 원 bridge/receipt/proof/최초·최신 오류를 보존하고 같은 current local intent로 재arm한다. 재commit·재inspect·재attach로 과거 성공을 만들지 않는다. 관측 전 입력은 원 presentation/route gate가 대기시키고, 실제 관측 뒤 다음 turn의 공통 projection에서 편집 상태를 표시한다. 관측을 기다리는 동안 SDK 기능 지원을 새로 판단하지 않는다.

복귀는 원 navigation_guards를 먼저 확인하고 Requested 단계의 ticket cancel만 되돌릴 수 있다. dirty/미ACK/취소 pending 중 자동 폐기0. release 이후 실패는 단계/owner/receipt를 보존해 명시 retry·close한다. 원 Viewing export 재연결→commit/proof→실제 frame 관측 뒤에만 복귀 완료로 표시한다. close는 기존 Endpoint join→Viewer callbacks terminal/release→Jobs scope→공통 Card/live scope→원 ExecutionOwner 순서를 유지한다. 기존 renderer/borrow/SDK 오류를 새 성공으로 바꾸지 않는다.

### R2 proof·동기 효과의 필수 구분

try_reproject가 mutable document borrow를 풀고 반환한 직후 원 receipt를 보존하고 retain_commit을 호출한다. 다른 fallible host binding/observer arm이나 다음 turn보다 proof 확보가 먼저다. callback/owner 연결 실패가 그 사이에 끼어 proof 발급 시점을 잃지 않도록 mount 결과 carrier를 설계한다.

proof가 이미 있는 observer arm Busy는 원 document/proof/receipt/오류를 보존해 정상 input/turn 뒤 같은 proof로 재arm할 수 있다. proof 발급 자체가 실패해 receipt만 남은 경우는 별개다. 다음 turn에서 과거 receipt로 proof를 새 발급하면 Stale이며 일반적인 재arm으로 복구할 수 없다. 이 경우 원 receipt/owner/발급 오류를 보존하고 명시 중단·close 또는 아직 동일 current stamp 안에서만 가능한 제한 복구로 구분한다. 새 역사 receipt proof API나 guard 완화를 요구하지 않는다.

첫 commit 성공 뒤 arm 오류가 나도 그 turn의 HostEffects에는 정확한 동기 receipt를 반환한다. 후속 rearm-only turn에는 옛 receipt를 committed로 반환하지 않는다. one commit/turn, 성공한 arm 전환의 단발 continuation/redraw, 중복 same intent의 추가 commit/notify0, close 우선을 유지한다.

opaque frame proof의 stamp/ticket과 notice DTO는 정확히 연결하지만, 정상 turn 이후 current full stamp와 과거 submitted frame full stamp의 동등성을 강제하지 않는다. 원 live lineage/epoch는 library validator와 기존 제품 guard로 검증한다. 다음 props commit은 이전 proof를 무효화하므로 editor 활성 표시 commit 전에 실제 frame 검증·route 관측 결과를 확정해 보존한다. Viewing return은 그 새 commit에서 별도 proof를 확보하고 별도 actual frame을 기다린다. entry proof를 재사용하지 않는다.

R2의 기존 exact12 시험 소유에 proof 발급 실패 carrier, 첫 commit 후 arm Busy/같은 proof 재arm, rearm turn의 옛 receipt 거절, DTO 불일치/None/새 projection/foreign owner, 다음 활성 commit 전 관측 확정, return 새 proof와 단발 wake를 포함한다. 신규14와 기존 영향60의 고유74 headless 근거를 검증·반환했다. 최종 BIN의 실행7과 비영향 이전 소스 근거67을 구분하며 실제 frame/notify·proof fault 추가 범위는 별도 fixture 계획에 남긴다.

## R3 — 기본 사용자가 볼 선택·진입 UI, exact9 후보(기존6/new3)

R1/R2가 기능 연결을 인수하고 R0 live 선택 공급자가 확정된 뒤 배정한다. 실제 사용자 경로가 없는 headless seam을 이 단계의 완료로 대체하지 않는다.

- 신규 src/presentation/web_ui_adapter/proposals/task_choices.rs 및 task_choices/tests.rs: source3 행을 공용 표시로 변환하되 host-held live origin과 CapturedReadOnly를 구분한다. 정확 source/ref/working draft는 host에 남고 callback에는 key만 제공한다.
- 신규 src/presentation/web_ui/components/task-choice-list.tsx: 세 종류 공통 목록/명시 열기/미지원·stale·빈 상태. 기존 토큰·Pretendard·공용 컴포넌트·아이콘을 소비한다. 첫 항목 자동 선택0.
- 기존 src/presentation/web_ui_adapter/proposals/mod.rs: 같은 cfg 등록.
- 기존 src/assembly/meari/viewer_callbacks/selection.rs: 실제 원 source와 local key/queue 연결. captured Card guarded 버튼은 기존 경로, 교정/Highlight capture는 별도 generic guarded UI 소비·종류별 검증 배정 전 읽기 경로 유지. SDK 미지원이라는 의미로 사용하지 않음.
- 기존 src/assembly/meari/product_viewer_launch/project.rs 및 product_viewer_launch.rs: 기존 configured SDK의 실제 선택 결과를 원 ViewerChoices에 전달. 저장소 정책/ID 임의 검색/새worker0.
- 기존 src/presentation/web_ui/viewing/read-panel.tsx: 새 공통 목록을 기존 Viewing에 연결하되 capture 편집 버튼을 다른 종류의 live 권한으로 전용하지 않는다.
- 기존 src/presentation/web_ui_adapter/player_shell/document/product_frame.rs: 공통 목록 source와 slot을 closed graph에 연결. 지원 source가 없으면 disabled 이유 표시.

UI 선택은 원문 비교 화면으로 명시 들어가고, 뒤로 가기는 같은 Viewing 선택/원 context로 복귀한다. dirty 상태의 돌아가기/창 닫기는 기존 guard와 원 사용자 선택을 따른다. list/empty/error가 live 권한을 만들어내지 않으며 입력 실패 후 임의 다른 문서로 대체하지 않는다.

## R4 — 기본 런처·URL·실제 인수의 마지막 연결, 별도 exact5 후보

기존 src/application/launch.rs, src/application/launch/native_viewer.rs, src/main.rs, src/presentation/web_ui/routing/routes.json과 신규 src/application/launch/viewer_proposal_entry_tests.rs를 후보로 한다. 이 단계는 SDK/application의 기본 실행 정책·CLI 계약 소유와 별도 배정한다. 기존 --native-viewer/--native-proposal-request 배타 조건을 UI에서 몰래 해제하지 않는다. 현재 noargs legacy의 세 작업을 제거하거나 새 Native 기본으로 전환할 권한은 이 문서에 없다. 공통 기능/CLI gate와 전 경로 인수 뒤에만 명시 결정한다.

React Router 호환의 기존 route ID/URL을 사용하며 임의 router를 만들지 않는다. 실제 구현·관측 인수 뒤에만 correction/shorts recommendations source/지원 상태를 갱신한다. GUI 클릭으로 CLI 미지원 import/create/save/accept 기능을 지원으로 위장하지 않는다. 실제 chooser·명시 열기·원문/수정/Save/별도 Accept·reopen·복귀·close가 기본 실행에 도달하는지 마지막으로 확인한다.

## 완료 판정과 최소 실제 시험 제안

기능은 기존 공통 SDK/CLI 결과를 우선 사용한다. 새 headless는 SDK 정책 재검증보다 HostRuntime admission/원 binding·scope/attach·receipt/표시·오류·close 연결을 확인한다. 원문/source SHA·cue/ref/revision을 보존하고 Save 응답/후속 inspect ACK와 Accept receipt를 구별한다. 원 cancel/timeout/raw 오류와 partial owner를 원 정리 carrier로 남긴다.

UI 고유 양성은 R2/R3 뒤에 원 NoMedia bounded supervisor를 사용한 별도 child 후보로 제안한다: task별 actual source action→같은 원 SDK 응답 소비→actual checked entry proof→원 controller 결과 표시→명시 return→actual checked Viewing proof→원 cleanup/EOF/familyEmpty/forcedfalse. 세 task의 actual input/proof/owner를 각각 귀속하고 하나의 Card 성공을 세 종류 성공으로 세지 않는다. 기존 Card guarded/legacy read-only, JOB 왕복·중복·wake 및 close 영향 검증을 포함한다. 정확 child 파일 경로·이벤트/receipt schema는 R1/R2 계약 고정 뒤 별도 exact 제안한다. 임의 큰 timeout·직접 frame proof 제조·사용자 앱 종료0.

현재 R1은 실제 신규8/영향15가 통과하고 채택됐으며, R2는 고유74 headless 근거와 캐시2 예외를 포함해 반환했다. R3/R4 실제 시험과 R2 actual Some/사용자 화면 인수는0이다. 별도 source3/scoped9의 위 실제 gate 결과와 이 route 단계의 미실행 경계를 구별한다. headless나 checked CPU submission을 OS compositor readback/사람 키보드/13화면 시각 일치로 확대하지 않는다. 가상 모니터를 포함한 DPI/VM 작업은 현재 보류 상태이며 자동 재개하지 않는다. 수동 GUI 인수는 공통 SDK/CLI 기능 통합과 이 보류 범위의 별도 명시 재배정 뒤에만 검토한다. 사용자 demo는 사용자 소유로 유지한다.

적용한 기존 modoo.today 본문: modootoday-free/design-delivery-plus/frontend-first-journey(공통 원문·typed ports/기존 컴포넌트와 사용자 reachable·미지원 구분), modootoday/software-qa/test-fixture-and-mock-build(원 owner/terminal/ACK/partial cleanup·실제/합성·revision 증거 분리). 이번 R0 정정은 SDK 원문 대조와 문서 편집만 수행했고 Rust 외 정책/검증 코드·새 실행은 추가하지 않았다. 별도 실제 source3/scoped9 gate는 각 검증 정본에 귀속한다.
