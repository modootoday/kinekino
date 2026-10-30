---
id: plan:kinekino/20261029113810
title: R3 opt-in chooser read port 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R3 opt-in chooser read port 계약

상태: 설계 제안, 구현·타입 검사·실행 검증 전. 관측 기준은 list11 채택 root `dd27fd6c1bf33cab9f52cac949a180a3b3f309ef`, Meari library `b532fe0eed06d8526b755b11bc36416070244ef7`, Cargo.lock SHA256 `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`이다. 이전 [SDK handoff 검토](proposal-chooser-sdk-handoff-review.md)의 목록 미채택 관측은 당시 상태이며, 이 계약은 채택된 typed list를 전제로 한다.

이번 범위는 이 문서 한 개다. 코드·Cargo·S/T·store scan·GUI·네트워크를 실행/변경하지 않았다. 목록 SDK/CLI 인수는 [목록 검증 정본](../../../sots/verification/application/proposal-list-sdk-cli-2026-10-24.md)에 두며 chooser 연결 성공의 근거로 확대하지 않는다.

## 책임과 exact6

| 경로 (`src/assembly/meari/` 기준) | 담당 책임 |
|---|---|
| 신규 `viewer_callbacks/chooser.rs` | opt-in 설정, typed 요청 조립, host-held 페이지/문서/키, 원 ticket·결과·ACK 상태, 선택 준비 carrier |
| 신규 `viewer_callbacks/chooser/tests.rs` | 동일 owner의 상태·키·부분 실패·수명 계약 시험. 새 worker나 GUI 우회 없음 |
| `viewer_callbacks/mod.rs` | chooser 모듈 노출과 최소 typed 연결 |
| `viewer_callbacks/codec.rs` | 기본 READS3 및 opt-in union manifest/binding을 같은 plugin·pump에 조립 |
| `viewer_callbacks/lifecycle.rs` | 원 attachment/registry/registration/callbacks/pump에 연결, admission·terminal·ACK·release 통합 |
| `viewer_read_actions/mod.rs` | chooser 전용 bounded local intent와 원 callback key 검증. 기존 ReadKind/READS3 인덱스 의미 유지 |

페이지/panel/TSX·표시·Open·route/mount는 UI 소유다. 알려진 UI fixture 경로인 viewer_native_launch/no_media 및 tests, viewer_native_probe, player_shell_host/delivery, viewer_host/live_entry·mount_proposal·return_viewing 및 관련 tests와 위 exact6는 겹치지 않는다. fixture12의 최종 실제 manifest와 비교한 쓰기 배정은 구현 시작 전에 PM이 확정한다. 이 문서는 UI fixture12 파일을 수정하거나 그 실행 결과를 전제하지 않는다.

## 하나의 attachment와 pump

`ViewerCallbacks`가 기존 ExecutionHandle/context에서 얻는 **하나의 ExecutionAttachment**를 계속 소유한다. codec은 선택한 profile로 **하나의 TypedAsyncSdkPlugin·AsyncSdkPump**, 하나의 registration을 만든다. chooser 때문에 별도 attach, 병렬 plugin/pump, raw `original_execution.submit`, worker, ExecutionOwner, filesystem scan을 만들지 않는다. handle clone은 submit 권한을 추가하지 않는다.

기본 profile은 현재 READS3(`capabilities`, `playback.engine`, `playback.status`), empty ReadInput, callback 및 contract 의미를 유지한다. opt-in profile만 이 세 read와 typed `ListProposals → ProposalListPage`, `InspectProposal → StoredProposal` binding의 합집합을 사용한다. manifest·grant·schema·provider fingerprint는 adapter와 export가 **같은 선택 profile 값**에서 얻는다. chooser를 켜고도 기본 manifest로 grant/export하는 혼합 상태를 허용하지 않는다. 추가 profile fingerprint에는 chooser 구현과 typed list/inspect 계약의 의존 파일을 포함한다.

opt-in은 원 unexported owner에서 설정한다. export 이후 profile 변경은 Stale/Busy로 거절한다. `configure_live_choices`의 scope/closing/selection guard는 그대로 유지한다. chooser 내부 결과 갱신은 이 API를 다시 부르거나 scope를 지워서 수행하지 않는다. resume도 원 profile을 보존해 재등록하며, 등록 성공을 route/frame commit으로 판정하지 않는다.

기존 전체 pending 8, terminal 256KiB, 전체 codec 예산을 함께 사용한다. chooser는 동시에 SDK 요청 하나와 대기 local intent 하나까지만 가진다. 같은 원 intent는 coalesce하고 다른 요청은 Busy로 돌려준다. 기존 READS3 ACK retry/refresh 네 동작과 별도 namespace를 쓰며 ReadKind의 세 index에 chooser를 끼워 넣지 않는다. renderer 입력을 empty ReadInput에 숨겨 전달하지 않는다.

## host-held 데이터와 키

아래 이름은 adapter 내부 타입 제안이며 새로운 SDK 정책/권한 DTO가 아니다.

| 값 | host에 보존할 내용과 공개 경계 |
|---|---|
| ChooserSession | 원 scope, profile, checked monotonic generation, query, 원 request context, 상태·ticket·terminal/ACK |
| PageSnapshot | typed page 전체, query/continuation/dataset/snapshot witness, 각 metadata 행. renderer에는 bounded 표시 필드와 opaque DocumentChoiceKey만 제공 |
| PreparedDocument | 선택 metadata와 typed StoredProposal 전체, 원 inspect request/terminal 결과의 식별·ACK 상태. 실제 payload/ref는 host가 소유 |
| PreparedSelection | 전체 문서에서 명시 선택한 실제 item으로 만든 NativeProposalSelection, session/document generation. R1 진입 권한이나 검사 성공 표시는 아님 |

DocumentChoiceKey와 ItemChoiceKey를 구별한다. 키는 owner-local 비재사용 counter와 scope/session/page/document generation에 묶고, 실제 LocalActionKey 및 pending invocation 일치 검사를 통과해야 한다. counter overflow는 명시 오류로 닫으며 wrap하지 않는다. renderer가 보낸 proposal ID/SHA/ref/index/path/URL을 선택 authority로 받지 않는다. 키는 SDK cursor나 capability token이 아니다.

페이지는 누적하지 않고 한 페이지 최대200행만 host에 보관한다. 새 query·refresh·next-page 시작 때 generation을 올리고 이전 문서/item 키와 prepared carrier를 retire한다. 이전 페이지를 오류 표시용으로 남겨도 선택은 비활성이다. 성공 응답은 예산·schema·원 request/query 대응을 확인한 뒤 한 번에 교체한다. next_cursor는 host의 원 snapshot에서만 가져오고 renderer cursor를 받지 않는다. 이전 페이지 복귀가 필요하면 UI가 명시 재조회 intent를 내며 임의 cursor cache를 추가하지 않는다.

문서 선택 시작 때 item generation을 올리고 이전 full document/item 키를 retire한다. ACK 실패 중인 원 terminal은 그와 별도로 보존한다. 같은 현재 키의 재클릭은 원 진행 상태를 반환하고 재inspect하지 않는다. 다른 선택은 진행 요청이 해소될 때까지 Busy다. captured 행은 이 live table에 넣지 않는다. create/import 즉시 handoff도 이번 list/inspect read port에 묵시적으로 포함하지 않는다.

## typed 목록과 선택 준비 inspect

1. 명시 List/Refresh intent에서 host가 typed ListProposals를 구성한다. limit/query/cursor 유효성·store 정책은 SDK가 판정한다. page의 `current_evidence=not_checked`, `selection_authority=none`을 유지한다. metadata에는 실제 item/ref가 없으므로 행만으로 NativeProposalSelection을 만들지 않는다.
2. 원 페이지의 유효 DocumentChoiceKey를 고르면 host-held proposal ID로 InspectProposal을 같은 attachment/pump에 제출한다. 준비 inspect에는 아직 target이 없으며 CardCurrentInspection을 raw submit으로 재사용하지 않는다.
3. typed StoredProposal의 validate와 proposal ID/task/draft revision/draft state/source ID·revision·SHA/typed item count를 선택 metadata에 정확히 대조한다. 하나라도 달라지면 Stale이고 item 키를 게시하지 않는다. list 이후 발생한 변경은 기존 행의 권한으로 수용하지 않는다. SDK의 historical verifier 실패도 그대로 보존한다.
4. 성공한 전체 결과의 실제 item index와 cue ID/DocumentTextRef를 host가 열거해 bounded ItemChoiceKey를 만든다. item 수가0이거나 R1 미연결 DocumentRepair이면 명시 empty/미연결 상태이며 첫 item/ref를 발명하지 않는다. 현재 wire 예산에 맞지 않으면 ResourceExhausted로 끝내고 일부 결과를 선택 가능하게 만들지 않는다.
5. 명시 item 키 선택으로 기존 NativeProposalSelection을 구성한다. 원 결과 ACK가 끝나기 전에는 prepared carrier를 외부 Open 대상으로 내보내지 않는다. metadata 준비 inspect 성공은 current evidence나 편집 활성의 증명이 아니다.
6. 실제 Open은 원 Viewing read drain/ACK/release 뒤 R1의 **target-bound current inspect와 project_initial의 full identity/target membership 검사**를 그대로 수행해야 한다. 준비 inspect와 R1 사이 변경 가능성이 있으므로 R1 검사를 생략하지 않는다. R2 mount/proof·입력 활성은 별도 UI gate다.

SDK 목록의 SourceExpired/RevisionConflict는 세션 키를 retire하고 명시 재조회·재선택을 요구한다. silent refresh, 다른 dataset 이동, 첫 행 fallback은 없다. ResourceExhausted/Unsupported/검증 실패는 원 오류로 표시한다. 반환된 next_cursor가 없으면 Next intent를 admit하지 않는다. SDK의 모든 운영 한계가 실측됐다고 주장하지 않는다.

## 상태·부분 실패·ACK·해제

정상 상태는 Idle → Listing → PageReady → Inspecting → DocumentReady → Prepared이다. 요청별 TerminalAwaitingAck는 표시 상태와 별도로 원 ticket/generation에 붙는다. Closing/Released는 새 intent를 받지 않는다. 각 전이는 owner가 원 request와 scope/generation을 확인한 후 수행한다.

| 상황 | 보존/처리 계약 |
|---|---|
| plugin 생성·등록·callback export 일부 실패 | 현재 lifecycle처럼 먼저 보유한 attachment 및 성공한 pump/registration/callbacks를 원 owner에 남긴다. 실패했다고 필드를 초기화해 자원을 잃지 않고 같은 close 경로로 회수 |
| 제출 전 오류/Busy | ticket을 제조하지 않음. 원 local pending을 명시 실패/재시도 상태로 끝냄 |
| 제출 후 decode/schema/identity 오류 | 실패 terminal과 원 ticket을 보존하고 ACK 대상에 포함. 성공 page/item 키는 게시하지 않음 |
| 이전 generation의 늦은 결과 | 표시/키 갱신 없이 원 결과를 terminal 소비·ACK. 취소 요청이 성공했다고 완료를 가정하지 않음 |
| terminal acceptance/ACK 실패 | 기존 원 ticket/generation의 제한·backoff retry 정책 사용. 같은 list/inspect를 재제출하지 않으며 원 결과와 retry 상태를 보존 |
| queued/running 취소 | 기존 queued Cancelled/running Requested 의미 유지. 실제 terminal 도착·ACK/합류 전 자원 회수 완료로 표시하지 않음 |
| route release/close | 새 intent 차단→키/carrier retire→원 callbacks invalidate once→같은 pump begin_close→모든 원 ticket terminal/ACK drain→원 registration unregister→원 attachment.release 순서 |

chooser 요청도 기존 `tickets()`/consume/read drain/ACK retry 관측에 들어가야 하며 READS3만 비었다고 release하지 않는다. tickets 또는 ACK retry가 남으면 원 owner를 반환/보존하고 다음 drain을 기다린다. release 실패 시 원 registration/attachment를 유지한다. 최종 worker close/join은 기존 ExecutionOwner 책임이며 chooser가 별도 join owner를 만들지 않는다. 재export 시 이전 키는 복구하지 않고 새 generation을 발급한다.

## exact6 밖의 handoff와 검증 배정

exact6의 산출 경계는 typed list/준비 inspect 및 host-held PreparedSelection까지다. 현재 pre-export LiveSelection을 활성 chooser 결과로 갱신하거나 Open에 연결하는 성공은 이 여섯 경로만으로 주장하지 않는다.

추가 경로가 필요한 최소 후보는 `src/assembly/meari/viewer_callbacks/selection.rs`다. 이유는 기존 LiveSelection이 소유하는 선택 table/키와 준비 carrier의 **명시 generation 교체·Open intent 전달 seam**이 필요하기 때문이다. UI/shared 소유로 별도 배정하고 configure guard를 지우지 않는다. `viewer_host/live_entry.rs`까지 변경이 필요한지는 이 seam 검토 후 결정하며, 현재 UI fixture와 겹치므로 SDK가 임의 수정하지 않는다. panel/TSX/product caller의 기존 R3 exact 집합도 그대로 별도 UI 배정이다.

구현 배정 뒤 Rust 시험으로 기본 READS3 profile 불변, opt-in union 단일 pump, list/inspect typed roundtrip, foreign/stale/replayed key 거절, query/page/document retire, metadata mismatch/빈문서/repair/초과예산, 단일 intent Busy/coalesce, stale terminal 소비, acceptance/ACK 실패 원 ticket 재시도, export 부분 실패 및 취소/close/재export cleanup을 확인한다. 같은 pump의 READS3 pending과 chooser pending을 섞어 drain 누락도 검사한다. 구현 후 배정된 자원에서 cargo fmt --check/test/clippy와 공통 SDK·CLI 검증을 실행한다. 실제 store/CLI 결과, UI callback/route/frame, 수동 시각 인수는 각각 별도 근거로 기록한다. 현재 이 시험은 미작성·미실행이다.

## 스킬 적용 근거

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b`, 필수 verdict 참조 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`를 앞선 동일 작업에서 탐색·읽은 근거를 재사용했다. 실제 codec/lifecycle/선택 계약과 대조하여 단일 owner, 원 terminal 복구, active configure guard 및 exact 경계로 적용했다. `test-fixture-and-mock-build` 본문 SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`의 fixture/관측 분리를 적용해 위 시험 후보와 실제 미실행 상태를 구별했다. 타 언어 정책·시험 구현은 도입하지 않았다.
