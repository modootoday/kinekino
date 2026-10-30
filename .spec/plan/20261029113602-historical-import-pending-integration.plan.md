---
id: plan:kinekino/20261029113602
title: Historical import의 Native Pending 연결 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Historical import의 Native Pending 연결 준비

2026-10-20 PM 배정에 따른 읽기 감사·변경 경계·인수 계획이다. 현재 SDK S1 application/assembly는 [최신 실행 정본](../../sots/verification/application/sdk-execution-s1-2026-10-20.md)의 worker11·assembly3·actual9/bound3task·전체420·CLI73명령·strict/fmt 근거로 반환했고 소스/API는 동결했다. UI 후보 consumer 인수, 강한 in-flight 취소/게시(S2), 실제 OS wake(S3)는 미완료다. 아래 배정 전/제안/미실행 설명은 최초 계획 이력이며 현재 지원 여부는 다음 표와 실행 정본으로 판단한다. 이 계획 자체는 실행 증거가 아니다.

## S1 반환 후 남은 인수 범위

| ID | 현재 근거와 남은 항목 |
| --- | --- |
| HP01 | Native5 실제 codec와 미설정 import Unsupported는 SDK gate에서 확인. legacy Immediate4 유지. UI 후보 consumer 등록과 전체 canonical schema/provider attestation은 별도다. |
| HP02 | invalid/oversize/포화 pre-admission, 동일 worker/context 및 Pending 결과 소유는 집중 시험 근거가 있다. thread spawn의 실제 실패 주입은 미검증이며 UI의 실제 admission/early 완료 소비는 후속 gate다. |
| HP03 | ordinary3task25회·원본9와 bound3task36회 같은 SDK worker 실행을 확인했다. project UI 호출의 worker 소비는 아직 없다. |
| HP04 | envelope3종/typed 불일치·wake 실패·원 terminal 보존·반복 pump SDK재실행0을 확인했다. 실제 UI selection/late/foreign 소비와 명시 invalid 복구/아카이브는 미완료다. |
| HP05 | queued cancel의 SDK effect0과 claimed Requested를 확인했다. claim 자체의 모든 실제 경쟁 interleaving이 검증됐다고 확대하지 않는다. release/submit 및 concurrent release 경쟁은 별도 권한 경계 시험이다. |
| HP06 | S1 실행 중 Requested→실제 결과·ACK·close와 verifier 실제 정상 회수 근거가 있다. 외부 cancel에 의한 actual verifier kill/wait/pipe ACK는 아직 미구현이다. |
| HP07 | 강한 cancellation/publication reservation은 미구현이다. 기존 jobs CAS를 일반 SDK 요청에 연결한 것으로 취급하지 않는다. |
| HP08 | 초기 typed 수용/ACK→actual scope→unbind→sameworker 재등록 및 이전 Drop 비침범을 확인했다. 문서 unbind는 worker 종료가 아니다. 최종 parent가 별도 owner close/join을 확인하며 UI partial mount/normalclose 소비는 후속이다. |
| HP09 | 원본9·bound3task의 Native Pending와 fresh SDK, 별도 CLI73명령의 project capture/freshreopen 연속성을 확인했다. UI 소비·retired selection 보호는 UI gate가 필요하다. |
| HP10 | Meari 플랫폼의 실제 parent/child·renderer resume·worker CompletionSignal→OS wake·child-first close 시험은 os-r2에서 정확1pass를 관측했다(아래 참조). 제품 SDK→UI→OS wake 통합과 렌더/시각 인수는 여전히 미완료다. headless 시험이나 빌드된 GUI를 이 통합의 증거로 삼지 않는다. |

전체 HP01~HP10 완료로 표시하지 않는다. SDK 검증 입력과 바이너리 pin은 실행 정본에 보존하며 UI consumer 슬롯 인수 전에 임의 변경하지 않는다.

HP10 플랫폼 선행 근거는 [Meari os-r2 receipt](../../../packages/meari/.spec/runtime/native-window-driver-2026-10-20-r1/os-r2.receipt.json)와 같은 prefix stdout이다. SDK 담당이 원 receipt의 terminal=true/exit0/68966ms와 정확1pass/0ignored·본문0.59s 및 실제 lifecycle 시험 source를 읽었다. 최초 os-first는 AccessKit adapter를 visible window의 resume에서 재생성해 실패했고 로그를 보존했다. 이 플랫폼 수정의 최종 strict/소스 인수는 Meari/PM 소유다. renderer suspend/resume은 시스템 절전 시험이 아니며 SDK verifier·실제 proposal UI의 wake 배선이 이 시험에 포함된 것도 아니다.

## 현재 공개 API에서 확인한 사실

| 근거 | 읽은 계약과 한계 |
| --- | --- |
| `packages/meari/packages/meari/src/application/plugins.rs` | PluginDispatch::Pending, PluginEventKind::{Completed,Failed,Cancelled}, CancelReceipt::{Requested,Completed,AlreadyTerminal}과 adapter.shutdown 공개 계약이 있다. adapter 호출은 owner thread이며 비동기 event는 registry.accept_event로 전달한다. |
| `application/plugins/registry.rs` | dispatch가 adapter 반환 뒤 pending을 등록한다. accept_event는 실제 ticket/owner/scope와 typed reply/error를 검사한 뒤 terminal을 제거한다. cancel Requested는 pending을 남기고 cancellation_requested를 기록한다. unregister는 Closing을 먼저 표시하고 pending이면 Busy다. drain/shutdown 성공 전에 등록을 제거하지 않으며 scope 교체도 pending이면 Busy다. |
| `packages/player-ui-plugins/src/infrastructure/typed_sdk_plugin.rs` | 현재 binding은 typed decode→동기 SDK 호출→Immediate다. cancel은 항상 AlreadyTerminal이고 shutdown은 Ok다. 이 구현에 Pending만 추가하면 worker 소유와 취소/종료 확인이 없다. |
| `src/assembly/meari/proposals.rs`·`proposal_manifest.rs` | 기존 동일 ApplicationClientHandle과 문맥 factory를 쓰는 Native4 adapter다. historical import capability는 아직 없다. |
| `src/presentation/web_ui_adapter/proposals/registry_host.rs` | typed ReviewHost의 동기 relay다. Pending을 비동기 결과로 보존하는 controller 계약이 없다. |
| `window_bridge.rs` | callback의 예상 밖 Pending은 ticket을 보존하지만 정상 asynchronous SDK admission은 아니다. 기존 callback queue/drain과 SDK worker terminal을 혼동하지 않는다. |
| `meari-native/src/infrastructure/window/wake.rs` | NativeWindowWake는 owner ID·atomic wake bits와 proxy만 전달한다. wake의 coalesced bool은 완료/렌더 ACK가 아니다. worker에 document/registry/GL을 전달하는 API가 아니다. |
| `src/application/api/client.rs` | ApplicationClientHandle은 Arc<dyn ApplicationClient>, trait는 Send+Sync다. SDK service·plain DTO/context를 worker로 전달할 수 있으나 Rc registry/context slot을 보내지 않는다. |
| `packages/media-jobs/src/application/ports.rs` | ExecutionControl은 cancellation_requested/deadline/reserve_publication/begin_verification을 제공한다. reserve_publication은 durable CAS와 request_cancel의 선형화 경계다. 일반 proposal SDK 호출은 현재 이 control을 받지 않는다. jobs의 실제 구현을 연결했다는 근거 없이 같은 취소 보장을 주장하지 않는다. |

Meari 담당의 [API 읽기 감사](../../../packages/meari/.spec/evidence/reviews/plugin-pending-sdk-boundary-2026-10-20.md)를 실제 nested 저장소 경로에서 읽고 위 Core/제품 adapter 관측과 대조했다. 제품 루트 .spec의 파일로 오인하지 않는다. Core API 존재를 제품 worker/실제 OS wake 배선의 완료로 승격하지 않는다.

SDK는 추가로 [취소/publication source 감사](../../evidence/reviews/sdk-historical-cancellation-boundary-2026-10-20.md)를 수행했다. 기존 StoreExecutionControl은 실제 durable job/attempt에 묶인 private 구현이며 일반 SDK import에는 아직 연결되지 않았다. 게시 후 inspect의 오류도 무변경 취소로 해석할 수 없다. 이후 강한 취소 구현은 이 공백을 해소하는 별도 공통 application gate를 필요로 한다.

## 제안하는 책임 경계

제품은 caller가 전달한 기존 SDK/client/store와 명시 verifier 설정을 그대로 사용한다. Native historical capability의 입력은 canonical ImportHistoricalProposal이며 UI/웹 payload에 실행 파일·pin·workspace 설정을 추가하지 않는다. 기존 request context 검증·ticket correlation과 source/raw/reference 검증을 공통 SDK에서 재사용한다.

SDK application 책임은 bounded admission·실행 상태·취소와 publication 경계·terminal 보존이다. 한 document parent의 worker owner를 panel마다 중복 생성하지 않는다. Worker에는 plain canonical DTO와 UI에서 캡처·검증한 RequestContext, 같은 Send+Sync SDK service만 보낸다. actual PluginTicket/registration·Rc document/registry/context·GL은 UI ledger가 보존한다. worker key를 직렬화한 handle에서 재구성하거나 opaque owner identity 대신 사용하지 않는다.

assembly/plugin 책임은 Native manifest·typed request/reply/error codec, dispatch admission→Pending, owner cancel/shutdown 위임이다. 기존 immediate binding의 cancel/shutdown 의미를 조용히 바꾸지 않는다. 별도 owned asynchronous binding/adapter가 필요한지 제품 plugin 담당과 확정한다. 단순 wrapper가 policy를 복제하거나 두 번째 SDK/store를 만들지 않는다.

UI/controller 책임은 pending ticket과 실제 selection/scope의 연결, registry borrow 밖 completion drain·accept_event·재투영·close 재시도다. SDK worker가 registry를 직접 호출하지 않는다. 먼저 terminal을 owner mailbox에 보존하고 wake를 보낸 뒤 UI가 실제 ticket으로 accept_event를 호출한다. event 검증이 실패해도 원 결과와 미완료 소유를 버리지 않고 보고/정리한다. OS window driver 연결은 별도 wake gate에서 검사한다.

## 경쟁·취소·종료의 인수 조건

1. worker가 dispatch 반환보다 먼저 끝나도 UI는 registry의 Pending 등록 뒤에만 terminal을 반영한다. owner ledger/mailbox에는 먼저 저장하고 정확1회 drain한다. queue full·thread 시작 실패는 SDK counter0인 pre-admission 실패이며 숨은 worker를 남기지 않는다.
2. terminal queue 포화·wake coalescing/실패·채널 단절·invalid reply·late/foreign/중복 완료에서도 결과/owner를 보존한다. SDK mutation을 재실행해 결과를 복구하지 않는다. 같은 actual ticket의 stale 검사는 Meari에 위임한다.
3. queued 작업이 claim되지 않았음을 원자적으로 확인하고 제거한 경우만 즉시 취소 완료를 반환한다. 이미 실행 중이면 cancel 요청과 실제 terminal ACK를 구분한다. SDK publication을 완료한 작업을 Cancelled로 포장하거나 rollback을 주장하지 않는다. registry가 지원하는 cancellation_requested=true의 Completed/Failed도 실제 결과대로 보존한다.
4. 강한 in-flight 취소는 아직 없는 application execution-control 연결이 필요하다. existing jobs port의 publication reservation/CAS 의미를 재사용할 수 있는 경계를 먼저 확정한다. verifier kill/wait/reader 종료 확인과 publication 선형화 전 취소를 실제 Rust 시험으로 증명하기 전에는 flag 변경이나 queue 제거만으로 실제 작업 취소를 주장하지 않는다. 기존 proposal timeout/idempotency 거부 규칙도 몰래 제거하지 않는다.
5. 문서 unbind는 Closing→새 admission 금지→pending terminal drain/취소 ACK→adapter.shutdown의 quiescent attachment release→registration 제거 순서다. 동일 owner로 mount를 재시도할 수 있으며 worker를 종료하지 않는다. 최종 parent close는 unbind 뒤 별도로 worker와 verifier 종료·join 영수증을 확인한다. Busy/borrow 실패 때 parent handle/owner를 남겨 명시 재시도하며 같은 teardown을 다시 시작하지 않는다. timeout/Drop은 정상 close 영수증이 아니다.
6. scope/source 전환은 기존 request가 quiesce한 뒤 registry.rebind_scope로 수행한다. 다른 scope로 결과를 적용하거나 retired worker의 SDK 변경을 현재 UI 상태에 상속하지 않는다. 저장이 이미 발생했다면 실제 proposal ID/revision receipt와 화면 적용 여부를 별도로 보존한다.

## 구현 배정 때 확정할 파일과 gate

| 담당 | 후보 변경 경계 |
| --- | --- |
| SDK application | `src/application`의 새 책임별 worker/admission/terminal/control 모듈과 필요한 기존 ai_provenance/proposals 계약. 동기 CLI 유스케이스와 같은 정책을 사용하고 새 실행 semantics는 공통 API/CLI에서도 검사 |
| SDK assembly | `src/assembly/meari/proposals.rs`, proposal_manifest/registration 및 필요한 하위 typed async adapter/owner. 기존 context validation과 동일 SDK 주입 유지 |
| 제품 plugin | `packages/player-ui-plugins`의 typed async binding/codec 소유를 담당과 조율. Immediate 기존 tests/의미를 유지 |
| UI | registry_host의 비동기 admission/result 계약, controller/document_session/window_bridge의 pending/close/reprojection. UI bootstrap 반환 뒤 해당 담당과 경계 확정 |
| infrastructure | 실제 worker/process owner의 취소·회수, application notifier에 actual NativeWindowWake를 연결하는 얇은 adapter. Meari core/native 구현을 임의 변경하지 않음 |

우선 deterministic Rust headless 시험으로 early/late terminal·full/failed admission·queued/in-flight cancel·실제 mutation receipt·invalid typed reply 보존·unregister Busy/Closing drain/재시도·scope 교체·timeout/channel failure·Drop 잔여를 인수한다. 기존9 raw와 source pins로 fresh CLI/proposal/project 회귀를 유지한다. 다음 실제 창 gate에서만 worker terminal→wake→OS Completion→accept_event→suffix 정확1회 admission/재투영과 사용자 close ACK를 확인한다. 기본 fmt/test/strict와 root C/consumer D target 분리·단일 heavy 정책을 유지한다.

현재 확정한 것은 읽은 API와 계획뿐이다. 새 capability/worker/Pending/cancel source나 실행을 추가하지 않았고, UI source/manifest/SDK Cargo 입력도 변경하지 않았다.

## 배정 전 구체 파일 소유와 shared 계약

아래 경로와 port는 제안이며 아직 존재/지원하는 API로 설명하지 않는다. UI bootstrap gate 반환 뒤 PM이 파일 경계를 확정한다.

| 소유자 | 정확 후보 파일 | 공유 경계 |
| --- | --- | --- |
| SDK application | 신규 `src/application/sdk_execution/{mod.rs,admission.rs,worker.rs,terminal.rs,tests.rs}`와 책임별 tests 하위; 부모 `application/mod.rs` | canonical Request/Response·plain operation ID·bounded owner 상태. Meari/PluginTicket/registry/renderer에 의존하지 않음. 기존 ApplicationClientHandle 하나를 사용 |
| SDK 취소/게시 | 기존 `application/ai_provenance/import.rs`, `application/proposals/mod.rs`, 필요 시 application/projects·infra historical_evidence/process/proposals와 해당 tests | 강한 취소를 구현할 별도 배정 범위. 현재 동기 SDK에는 control 인자가 없으므로 worker flag만으로 취소 완료를 구현하지 않음 |
| SDK assembly | 기존 `assembly/meari/{proposals.rs,proposal_manifest.rs,proposal_registration.rs,request_context.rs}`와 신규 async owner/terminal bridge 하위 | 실제 ticket↔application operation 상관 연결. context 검증/correlation은 기존 helper 재사용. 원 ticket은 UI owner에 보존 |
| UI 제품 plugin | 신규 `packages/player-ui-plugins/src/infrastructure/typed_async_sdk_plugin/{mod.rs,codec.rs,lifecycle.rs,tests.rs}`와 해당 infrastructure 부모 | generic I/O codec 및 start/cancel/shutdown delegation. root DTO는 SDK assembly가 type parameter로 주입. 패키지의 root 역의존 금지. worker/취소/게시 정책을 패키지에 복제하지 않음 |
| UI presentation | 기존 `presentation/web_ui_adapter/proposals/{registry_host.rs,controller.rs,document_session.rs,window_bridge.rs}`·factory/bootstrap의 필요한 공개 입력과 tests | 동기 ReviewHost를 Pending으로 조용히 바꾸지 않음. 별도 admission/result 계약·view ticket ledger·completion drain·재투영·close 진행. UI가 verifier 경로/pin을 고르는 payload를 추가하지 않음 |
| SDK infrastructure/조립 | 신규 completion notifier adapter와 필요한 main의 소유 조립만 | application notifier를 actual NativeWindowWake에 연결. SDK application이나 generic plugin에 Meari-native/GL/OS 의존을 넣지 않음. actual window 부분은 UI/Meari와 별도 배정 |

application의 worker 결과가 canonical Response라고 해서 generic plugin이 root를 import하지 않는다. SDK assembly가 generic binding callback에 canonical 요청 제출과 typed I/O 변환을 주입한다. application worker ledger는 실행/자원/terminal을 소유하고, assembly의 actual ticket 상관 연결과 UI의 selection/view ledger는 각기 다른 책임이다. worker는 registry.accept_event를 호출하지 않는다. registry가 typed terminal을 수락한 뒤 owner ACK로 terminal을 소비한다. invalid reply는 원 결과/실행 receipt를 보존하며 SDK mutation 재실행 없이 typed failure/명시 정리 경로를 확정해야 한다.

## 전체 capability와 단계별 지원 상태

| 대상 | 현재 | S1 worker/typed Pending 배정 후보 | S2 실제 취소/게시 control |
| --- | --- | --- | --- |
| proposals.create | Native4 Immediate, 일반 source도 store IO 발생 | 같은 create 유스케이스를 worker에서 실행 | 새 control을 연결했다는 증거가 있을 때만 강한 취소 지원 |
| proposals.inspect | Immediate; bound이면 verifier process IO | 일반/bound 모두 worker. unsupported reader도 canonical ApiError로 반환 | 읽기 verifier의 실제 취소/pipe ACK 검증 |
| proposals.edit | Immediate; bound 검증+immutable CAS 저장 | 같은 edit/CAS를 worker에서 실행 | 취소와 publication reservation 경쟁 검증 |
| proposals.accept | Immediate; bound 및 receipt 재요청도 검증 | 최초/재요청 모두 worker. 자동 retry하지 않음 | publication 이후 실제 결과 보존·취소 ACK의 선형화 |
| proposals.import_historical | CLI/common SDK 지원, Native capability 없음 | explicit Native typed capability 추가·같은 import 유스케이스 worker 실행. verifier 미설정은 Unsupported이며 UI가 실행 경로를 지정하지 않음 | 원본 검증/실제 verifier 정지와 최초 publication의 취소 경쟁 |
| project capture/reopen | common SDK 동기이며 bound capture 검증 | UI가 project 호출을 소비한다면 같은 worker 경로에 포함해야 함. 포함 전 project UI 무차단은 미지원으로 유지 | 실제 project 게시·saved capture와 취소의 별도 공통 gate |

S1의 queued 취소 완료와 실행 중 Requested→실제 Completed/Failed는 지원 범위를 명시할 수 있다. 이것을 강한 in-flight 취소나 mutation rollback 지원으로 광고하지 않는다. S2가 후속 배정이면 API/CLI/host 지원 metadata와 정본에 미지원으로 남기며 전체 취소 인수를 완료로 표시하지 않는다. S3 actual window/wake·실행 시각 검토도 독립 인수다. import 하나만 비동기로 바꾸고 bound inspect/edit/accept를 UI thread에서 계속 실행하는 구성을 완료로 승인하지 않는다.

## HP01~HP10 인수 매핑

| ID | 계약·책임 | Rust/CLI 근거와 완료 판정 |
| --- | --- | --- |
| HP01 | SDK manifest+UI generic codec | Native 대상5 capability의 canonical DTO/reply/error·document3/historical1·권한·미설정 Unsupported. 기존 Native4 consumer의 명시 호환 범위를 대조하고 root 역의존/설정 payload 없음 확인 |
| HP02 | SDK admission/assembly | invalid context·queue full·worker 시작 실패는 SDK counter0. enqueue 직후 early terminal도 registry Pending 등록 후 exact1회 반영. 원 ticket/context/CAS 옵션 보존 |
| HP03 | SDK worker+UI relay | create/inspect/edit/accept/import 전부 별도 worker에서 동일 SDK 실행. bound actual verifier 또는 Rust barrier로 owner dispatch 반환·UI 입력 처리와 작업 실행 thread를 구별. project UI 소비가 남으면 미지원으로 명시 |
| HP04 | SDK terminal+UI controller | typed success/ApiError·invalid reply·duplicate/foreign/late event·mailbox/wake 실패에서 원 결과/owner 보존. registry 수락 전 ACK 금지·SDK 재실행0·selection/scope 오염0 |
| HP05 | SDK queued cancel | queued→claimed 경쟁을 원자 제어. 미실행 확인된 작업만 취소 완료·SDK counter0. 이미 claimed면 Requested이고 terminal 전 handle 제거 금지 |
| HP06 | SDK in-flight cancel/infra | S1은 Requested 뒤 실제 Completed/Failed·cleanup ACK를 입증. S2 강한 취소는 actual verifier kill/wait/pipe 종료와 공통 control을 별도 CLI/API gate로 검증. flag만 바꿔 Cancelled를 만들지 않음 |
| HP07 | SDK common publication | reserve 전 cancel 승리→publication0, reserve 후 PublicationInProgress/실제 결과, 게시 후 evidence 오류→기존 저장 보존·재조회. jobs의 실제 durable job/attempt를 연결하거나 공통 책임 추출을 승인받기 전 같은 보장을 주장하지 않음 |
| HP08 | SDK owner+UI close/plugin delegation | unregister Busy/Closing 중 drain·typed ACK→attachment release/shutdown→등록 제거와 최종 parent의 별도 worker/verifier close/join을 구분. borrow/kill/wait/channel 실패 재시도, invalid terminal 소유, Drop/timeout 미완료를 구별. teardown/SDK mutation 중복0 |
| HP09 | 공통 SDK/CLI와 UI 소비 | 기존9 원문과3task 기술 edit/accept/retry/project/fresh reopen을 같은 SDK/manifest/input pin으로 유지. 선택 전환/retired 요청이 현재 source를 덮지 않음. 새 worker 결과를 품질 승인/미디어 적용으로 승격하지 않음 |
| HP10 | UI/SDK notifier+Meari 실제 host | headless Completion을 실제 wake로 세지 않음. 별도 actual window에서 terminal 보존→wake→OS Completion→accept_event→suffix exact1회/재투영·close ACK. coalescing bool은 renderer ACK가 아니며 시각 인수는 별도 |

S1만 배정되면 HP06의 S2 부분과 HP07/HP10은 미완료로 남는다. HP01~HP10 전체를 작은 gate의 통과 수로 대체하지 않는다. 위 대기는 계획 준비 당시 이력이다. UI bootstrap 인수 뒤 PM이 S1 구현·SDK 단일 heavy를 배정했고 shared port를 합의해 SDK application source와 첫 집중 gate를 진행했다. UI는 generic/presentation source, Meari는 별도 Native source만 작업하며 root application 첫 gate는 Native를 소비하지 않는다.

## Meari 감사와 제품 인수 ID 대조

제품 인수는 **HP01~HP10**, Meari 감사는 **P01~P10**으로 구분한다. 서로 다른 의미의 같은 번호를 완료 근거로 상속하지 않는다.

| Meari 감사 ID | 제품 인수 연결 | 대조 결과 |
| --- | --- | --- |
| P01 실제 worker·옵션·상관 | HP02/HP03/HP09 | Send compile assertion·동일 SDK actual 실행은 후속이다. 읽은 Arc trait만으로 시험 통과를 주장하지 않음 |
| P02 Pending typed terminal | HP01/HP04 | reply/error validation과 ACK 전 보존을 generic codec/제품 ledger로 나눔 |
| P03 early/Busy/wake | HP02/HP04/HP10 | dispatch 후 pending 등록 경쟁과 actual OS wake를 별도로 인수 |
| P04 bounded 포화 | HP02/HP03 | count뿐 아니라 request/result bytes·terminal 예산도 명시. 수락 전 effect0과 수락 후 무재전송 구분 |
| P05 실제 취소/완료 경쟁 | HP05/HP06/HP07 | queued 취소와 in-flight Requested/실제 결과·게시 reservation을 구분 |
| P06 Closing/drain | HP08 | cancel/shutdown 안에서 registry 재진입하지 않음. Busy owner를 보존해 외부 pump에서 drain |
| P07 late/foreign/기억 소거 | HP04/HP09 | bounded terminal 기억 소거/unregister 후에는 AlreadyTerminal이 아니라 Stale일 수 있음. caller receipt로 재실행을 막음 |
| P08 큰 결과/codec 오류 | HP01/HP04/HP08 | UI 검증 ACK 전에 terminal slot을 지우지 않음 |
| P09 Drop/실패/timeout | HP06/HP08 | 정상 정리 영수증으로 집계하지 않고 residual owner/명시 재시도 유지 |
| P10 actual wake | HP10 | generic lifecycle17/host13/Core 회귀를 actual provider→OS wake 성공으로 대체하지 않음 |

추가로 확인한 제한은 다음과 같다. 현재 TypedSdkPlugin은 event_schema Some을 거부하지만 Pending의 Completed/Failed terminal에는 progress schema가 필요하지 않다. S1은 terminal-only로 준비하며 progress가 필요해질 때 별도 계약을 배정한다. worker 완료와 UI terminal 검증 ACK는 다르다. TerminalReady인 요청에 cancel을 받았다는 이유만으로 AlreadyTerminal을 반환해 registry pending을 지우지 않는다. 최소 owner 상태는 queued/claimed/terminal-ready/acknowledged를 구별하고, 실제 terminal을 밖에서 accept_event한 뒤에만 소비를 확정한다. count/bytes 예산과 invalid terminal 보존·Closing drain을 shared callback 계약에 포함해야 한다.

이 대조는 읽기 감사이며 새 provider/worker 실행 근거가 아니다. 현재 UI gate의 full/strict와 SDK의 이전 CLI73요청도 HP01~HP10 또는 Meari P01~P10의 통과로 자동 상속하지 않는다.
