---
id: plan:kinekino/20261029113901
title: "C: 읽기3 소비자 수용·화면 투영 계약 및 source20 인계"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C: 읽기3 소비자 수용·화면 투영 계약 및 source20 인계

## Local-action 후속 분리 · Meari 계획 회신

후보 코드 동결 후 직접 대조: `runtime/development/meari-local-actions/.spec/sots/core/local-actions.md`와 `packages/meari/src/application/host_callbacks/local.rs`를 읽었다. 후보 commit은 `4b2ad301315cfbfd24af08b784271c72e909b3b8`, source408 SHA는 `7E2CDEA4D1DE34A8339CB5A64E09D0135C9224465B25F97B31DE570321047854`다. 실제 후보 서명·Arc identity 비교·typed NotLocal/Invalid·Local export lease 요구가 아래 소비 규칙과 대응한다. 이는 source 읽기 확인이며 작성 unit8/integration5의 실행 인수는 아니다. 제품은 gate/별도 lease 전 가정 연결0을 유지한다.

추가 소비 조율안(아직 동결 전): LocalActionKey의 PartialEq는 Arc::ptr_eq identity이며 invocation은 key()/matches_key(&key), arguments()/scope()를 제공할 예정이다. dispatch_local 결과는 LocalDispatchError::Invalid(PluginError)와 NotLocal을 구분하는 안으로 조정됐다. 공통 owner/scope/arguments/signature 검증을 먼저 통과한 유효 Plugin target에만 NotLocal을 반환하고, stale/malformed는 Invalid로 종료한다. 제품은 NotLocal일 때만 기존 plugin dispatch로 넘겨 재검증하고 Invalid에는 fallback하지 않는다. Local→plugin은 기존 InvalidPayload/producer·registry0이다. 이 타입과 accessor는 아직 구현 동결·검증된 공개 API로 소비하지 않는다.

PM이 정확 경로 `runtime/development/meari-observer-test-support/.spec/sots/core/local-actions-proposal.md`를 제공해 원문 직접 읽기를 완료했다. 아래 초기 경로 미확인 상태는 해소됐다. PM은 라이브러리 exact4 source-only 구현을 별도로 승인했으며 제품 retry 배선 lease는 아직 없다. 계약 코드 동결·독립 감사·실 gate 전에는 가정 API를 제품 C21에 연결하지 않는다.

소비 영향은 원 controller가 LocalActionKey를 보존하고 document drain/borrow 해제 뒤 현재 실제 stamp.owner로 dispatch_local을 실행하는 것이다. typed local intent를 처리할 때도 원 HostStamp로 try_reproject를 재검증한다. Busy는 원 PendingCallback을 제한 보존해 재검증하고 Closing/poison/disabled를 우회하지 않는다. 최종 공개 서명에서 invocation과 controller 소유 key의 안전한 identity 비교 방식, 혼합 plugin/local queue의 wrong-target과 실제 invalid arguments를 구분하는 라우팅 순서를 확인 요청했다. 이 확인은 새 API 구현 lease를 추정하지 않는다.

Meari 담당은 라이브러리 `.spec/sots/core/local-actions-proposal.md`의 최소 계획을 회신했다. 현재 제품 checkout에는 해당 문서가 없어 실제 경로를 확인 요청했다. 아래는 담당 회신 요지이며 원문 직접 검토 근거가 아니다. C21 pin038E는 변경하지 않는다.

제안 exact4는 Core `host_callbacks.rs`, 신규 `host_callbacks/local.rs`, `host_callbacks/local/tests.rs`, `tests/local_actions.rs`다. bind_local(prop,signature)는 Rust opaque LocalActionKey를, dispatch_local(handle,args,currentScope)는 LocalActionInvocation을 반환하는 additive 설계다. Binding.target은 Plugin/Local로 나누되 bindings/export/Native queue 형상은 유지할 수 있다고 회신했다.

원 owner Arc/active/lease/revision/scope/signature 검증 순서를 private 공통 경계로 재사용한다. Local은 실제 export lease가 필수이고 기존 plugin hosthandle의 leaseNone 호환은 유지한다. cross-target은 InvalidPayload로 거절하고 SDK/producer/registry 호출0이다. Rust private Arc identity와 원 controller typed intent를 연결하며 source prop 문자열은 권한 분기에 쓰지 않는다.

제품은 document borrow를 푼 뒤 현재 원 stamp.owner로 dispatch하고 Busy이면 원 PendingCallback을 보존해 재검증한다. 반환 intent로 closing/poison/disabled를 우회하지 않는다. 검증된 intent에서만 기존 Rust consumer/projection 재시도를 호출하는 제품 후속 lease를 별도로 동결한다. 공개 API/코드 lease/실행 인수는 아직 없으며 UI 재시도 버튼 완료로 표시하지 않는다. Core export/queue·혼합 target·stale/reexport/SDK0 회귀와 Native consumer gate도 구분한다.

최신 r2는 `runtime/coordination/ui-read-projection-c-r1/revision-r2/handoff.md`의 exact21/pin `038E0D5000EC62BC8515F9841AEBF6C1A7D4BD5CC79D11A3CE05068818D55CDF`다. 원20을 보존하고 provider1·locale/code·회귀를 보강했다. 감사 수리로 ACK 재시도 admission을 원 ticket별 총3회/실패 간 최소25ms로 옮겨 Open/Completion/close가 공유한다. None은 예산0, 명시 재개는 해당 원 ticket만 검증한다. remaining_callbacks batch의 문서 commit도 보류한다. 아래 r1 기술 중 host counter만으로 ACK 예산을 제한한다는 설명은 대체됐다. 신규 active9/ignored1 작성, 실제pass0이다. UI-local retry API는 Meari 확인 중이며 API/제품 기능 확대0이다.

2026-10-21 최종 PM 지시: 제품 exact20, 공통 pump API 변경0이다. 중간 sink API2/acquire closure 제안은 철회됐다. 조회 결과 ACK는 소비자 수신 확인이며 사람 품질 승인과 무관하다. 정상 흐름에 별도 사람 ACK 버튼은 없다. AI 산출물 채택 정책은 변경하지 않는다.

## 실제 API와 처리

기존 AsyncSdkPump::accept는 terminal 검증→registry Core 수락→pump entry.accepted 원 event 저장→SDK ACK→TerminalAcceptance 반환 순서다. 이 원 event 저장을 수신 소유 근거로 사용한다. ViewerCallbacks는 결과를 받을 projection borrow를 먼저 확보하고, 반환 event/owner_ack를 원 UI owner에 보존한 뒤 기존 typed DTO로 decode한다. ACK 성공 뒤 typed 변환/문서 투영 실패는 보존 event를 재decode/재투영하며 SDK 재요청/재ACK0이다. ACK 실패는 pump가 원 event/ticket을 보존하므로 같은 ticket으로 재시도한다.

기존 NativeStartupCompletionNotifier(startup.wake_handle)→Completion host turn→after_dispatch에서 bounded read consumer를 호출한다. 새 worker·공개peek·SDK/common port API를 만들지 않는다. close도 같은 consumer를 최대 기존 budget8 ticket에 대해 한 번 drain한 뒤 원 종료 driver를 관측한다. 정상 수용은 ticket을 남기지 않고, 실제 미처리/검증/ACK 실패만 CallbackPending·NeedsTerminalAck로 보존한다. 실제 terminal 미관측을 완료로 표시하지 않는다.

## 상태와 입력·출력

- 원 ticket/capability는 assembly owner만 소유한다. capability별 outstanding 하나를 거절하며 기존 registry budget8을 유지한다.
- ReadPhase는 NotRequested/Pending/Received/Failed/AckFailed다. ReadPanel은 이전 보존값·진단·projection_error를 별도로 소유한다. 원 event/ACK 결과는 최대 세 capability의 최신 entry에 보존한다.
- 조회 버튼3→기존 실제 callback/receipt→Pending→consumer transaction→Received/실패 표시다. CapabilitiesReply/EpisodePlaybackObservation/PlaybackStatus의 schema로 decode하며 unknown/None은 그대로 보존한다. 화면 JSON은 typed 데이터 표시이며 실행 코드가 아니다. 두 읽기를 원자 snapshot으로 합성하지 않는다.
- ACK 실패는 같은 원 event/ticket을 재시도한다. host turn 재시도는 최대3회/25ms이며 소진하면 owner 진단과 실패 상태를 보존한다. 무한 retry loop0이다.
- 문서 투영은 같은 NativeDocumentHandle과 전체 HostStamp의 try_reproject·update_module_props_with_callbacks를 사용하고 실제 commit receipt만 반환한다. 같은 scope에서 새 export lease만 생성하며 새 attachment/register0이다. preflight 실패는 이전 export/표시/candidate 유지, own_nodes invariant 실패는 poisoned 상태 보존이다.
- projection 실패도 같은 제한 내에서 재시도한다. Rust retry_read_projection은 보존 event 재decode·dirty 표시만 수행하며 SDK 호출0이다. 실제 다음 host turn은 호출자가 원 runtime 경로로 진행해야 한다.

## 화면과 호환

기존 callback-free ViewingPlayer·prepare_start_document 경로를 유지한다. viewer_native_launch만 enable_read_panel로 명시 interactive entry를 선택한다. 새 read-panel.tsx root가 기존 ViewingPlayer를 그대로 합성하고 조회3/상태/typed 결과/오류를 붙인다. 원 connection/time/product labels를 동일 base props로 보존한다. Rust inventory와 source-graph.json에 해당 root를 반영한다.

색·border·radius·여백·폰트는 기존 공통 토큰/컴포넌트를 소비한다. 파일열기/탐색/재생 control·창 chrome·surface·main은 변경하지 않는다. 현재 조회 label은 TSX, 상태 문구는 Rust이며 locale key 정본화는 후속 보완이다. 시각 일치 완료 근거가 아니다.

## exact20

목록은 runtime/coordination/ui-read-projection-c-r1/paths.txt, 원문은 before20.json/before/, source는 source/다. 제안된20 경로를 유지하며 변경 없는 compatibility test/기존 player 파일도 범위에 포함해 해시를 고정한다. packages/player-ui-plugins 두 파일은 편집0이며 최종 source에서 제외했다. before22-attempt.json은 중간 범위 기록으로만 보존한다.

## 작성 회귀와 아직 닫히지 않은 검증

신규 fixture는 read_consumer_retains_real_terminal_without_human_acceptance(실 SDK worker·actual callback·중복 요청 거절·수신 후 ticket0·원값 보존·재decode 후 재ACK0), actual_read_graph_reprojects_with_original_scope_and_commit_receipt(실 source graph/CPU document/원scope·commit receipt), projection_failure_preserves_candidate_for_reprojection_without_sdk_work, unobserved_values_have_no_numeric_playback_defaults, pending_preserves_previous_observation, empty_callback_snapshot_cannot_claim_ack_requirement_or_worker_completion, interactive_read_root_is_opt_in_and_retains_start_labels다. 모두 작성만 했고 Cargo/시험 실행0이다.

독립 감사에서 추가로 확인할 항목은 실제 host Completion dispatch→consumer→document commit, ACK 실패 fault adapter·제한 재시도, Core/codec 실패·원 event 재decode, actual preflight 실패 후 이전 callback export 유지, Closing의 원consumer drain·원worker join이다. 기존 read3/proposal/host/cleanup/document 회귀·양strict/fmt도 중앙 배정이 필요하다. 작성 시험을 실제pass로 세지 않는다.

## 정확한 남은 UI 재시도 연결

현재 실패 재시도는 bounded host retry와 원 Rust owner 메서드까지다. 소진 후 UI-local 재시도 버튼의 검증된 handle 라우팅은 아직 구현하지 않았다. 기존 HostCallbacks.bind는 SDK plugin capability에 결속되므로 prop 문자열만으로 임의 local action을 받거나 가짜 SDK capability를 추가하지 않았다. 이 연결이 필요하면 Meari의 scoped local-action export/검증 계약 또는 기존 동등 API를 확인해 최소 범위를 별도 보고해야 한다. 사용자 수동 ACK 버튼으로 대체하지 않는다. Closing은 문서 입력이 막히므로 UI 버튼으로 탈출한다고 주장하지 않는다.

현재 source는 독립 읽기/추가 명세 보완을 위한 후보이며, 위 실패/복구 실제 인수와 UI-local retry gap을 해결하기 전 C 전체 완료로 표시하지 않는다. 제품 root/index/shared/GUI/media/main/Cargo 변경0이다.
