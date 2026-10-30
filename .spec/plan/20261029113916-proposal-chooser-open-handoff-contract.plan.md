---
id: plan:kinekino/20261029113916
title: chooser PreparedSelection → 명시 Open 계약 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# chooser PreparedSelection → 명시 Open 계약 제안

2026-10-24 읽기/설계만 수행했다. chooser raw6/source6 `4CC6D6CFBEFBC4BCA34285A2B8B83A08D46940907837353FF36E9C158A9D4AC9`와 gate 계획은 불변이다. chooser17의 타입·실행 인수, UI W39 수리/실제 복귀 성공을 가정하지 않는다. 코드/Cargo/S/T/GUI/네트워크/stage/commit0이며 아래 API·시험은 미구현·미검증 제안이다.

읽기 기준: Meari [seam 검토](../../../../packages/meari/.spec/evidence/reviews/chooser-open-navigation-return-seams-2026-10-24.md) SHA256 `162A2A82ADCDD5DC95ECD62EDEC5F73F51639878B6A14EA38F92C6F6AE48FFDB`, 원 raw chooser/lifecycle, canonical selection/live_entry와 ShellState::request_route. Meari 검토의 navigation 후속 확인까지 읽었으며 별도 library API가 필요하다는 이전 가정은 채택하지 않는다.

## 좁은 host-only seam

현재 `chooser_matches_prepared`는 원 scope/generation/선택 Arc identity와 Prepared/noqueued/noactive를 검사한다. 이것만 따로 호출한 뒤 renderer callback을 나중에 처리하면 두 검사 사이 상태 변경을 놓칠 수 있다. `LiveSelection`의 기존 `onOpenLiveProposal` Text local key에 **host-held prepared row 한 개**를 추가하는 방식으로 연결한다. 새 SDK 정책·pump·attachment·worker는 없다.

- chooser opt-in을 원 unexported owner에 설정할 때 Open consumer도 준비하고, 같은 export에서 기존 LIVE_ENTRY_BINDING을 정확히 한 번 bind한다. 기존 TrustedLiveChoices가 있으면 같은 LiveSelection이 정적 행과 chooser 행의 구분을 보관한다. active configure_live_choices guard를 지우거나 export 후 configure를 재호출하지 않는다.
- 표시용 host 조립 API는 현재 owner의 PreparedSelection을 읽어 `PreparedOpenRow {opaque_key, prepared}`를 등록한다. renderer는 이 opaque key만 돌려준다. ID/SHA/ref/index/origin/새 Arc 선택을 등록 인자로 받지 않는다. 같은 현재 carrier 등록은 같은 행을 반환하고, 교체는 pending Open이 없을 때만 허용한다. 새 key는 owner-local checked counter와 export/chooser generation에 묶이며 wrap/재사용하지 않는다.
- 정적 행과 chooser 행 namespace를 구별한다. key 문자열이나 selection DTO 자체는 권한이 아니다. chooser 행은 원 Arc를 그대로 보존한다. candidate scope/gen이 같아도 Arc::ptr_eq가 다르면 거절한다.
- 공개 원시 API를 둘로 나누어 “검사 성공 bool”을 저장하지 않는다. admission은 같은 원 ViewerCallbacks의 배타 borrow 안에서 authentic PendingCallback 검사와 해당 row의 chooser_matches_prepared 검사를 **동시에** 수행한다. 입력은 원 scope+pending이며 candidate는 host row에서만 찾는다.

## admission과 소비의 원자 경계

| 단계 | 검사와 소유권 |
|---|---|
| Open admission | 실제 HostStamp/document Open·source scope와 원 HostCallbacks::dispatch_local의 export lease/signature/key를 검사한다. invocation.matches_key, Text key의 row membership, carrier scope/gen/Arc identity를 같은 borrow에서 확인한다. Jobs/다른 Card·capture·partial/return owner의 기존 배타 guard도 통과해야 한다 |
| 보류 저장 | 검사 성공 후에만 원 pending+동일 prepared carrier+selected Arc+Open key를 bounded pending entry 하나에 함께 보관한다. 같은 export/key/carrier의 재클릭은 coalesce, 다른 intent는 Busy다. 단순 같은 prop명/같은 target 비교로 coalesce하지 않는다 |
| 소비 직전 | 원 zero-batch turn에서 기존 live_entry_request 경로가 원 pending lease/key와 row/carrier identity를 다시 검사한다. chooser의 query/page/item 변경·queued read·active/ACK pending·cancel/close·foreign/reexport면 Stale이며 route를 요청하지 않는다 |
| route 이전 실패 | RefCell Busy/observer 부재·view.start 미관측/route guard 오류는 pending과 carrier를 먼저 take하지 않는다. 원 owner에 보존하며 동일 intent 재시도 또는 명시 취소만 허용한다. 실패가 새 inspect/자동 refresh/첫 item 선택을 만들지 않는다 |
| route 소유권 이전 | 필요한 borrow·값/entry를 먼저 준비하고, 같은 callbacks/cards/observer borrow와 zero-batch turn 안에서 최종 재검사→원 request_route→LiveOpening에 원 request+carrier+ticket 저장→기존 pending 소비 및 원 source admission 중지 순서로 완료한다. ticket 성공 후 owner 저장 사이에 새 fallible 단계·await·callback을 넣지 않는다 |
| 이전 뒤 실패 | chooser의 이전 carrier를 재게시하지 않는다. LiveOpening이 원 request/carrier·route ticket·first error와 inspection/start/mount failure owner를 계속 소유하고 기존 취소/ACK/release 경로로 정리한다. 새 Open click으로 같은 작업을 재발급하지 않는다 |

현재 ShellState::request_route는 guard/route lookup/checked generation 오류를 상태 변경 전에 반환한다. ticket 반환 뒤에는 기존 LiveOpening 저장과 intent 소비가 이어진다. successor도 이 반환 경계를 유지해야 하며 panic/OOM의 transactional rollback까지 입증했다고 주장하지 않는다. 원 pending을 먼저 take하거나 route ticket을 얻은 뒤 저장 실패 carrier를 잃는 구현은 허용하지 않는다.

route 소유권 이전 후에는 기존 stop_admission/prepare_route_release가 chooser key/carrier를 retire해도 된다. 이때 LiveOpening의 보존 carrier는 provenance/진단·cleanup용이며, 폐기된 Viewing callback을 다시 dispatch하는 grant가 아니다. 이전 전 pending Open이 존재하면 query/read/item 등록 변경은 기존 has_live_entry 배타 guard로 Busy 처리한다. 취소/close가 먼저 일어나면 pending/row를 retire하고 늦은 Open은 Stale다. mutation을 막지 못한 변경이 있더라도 소비 직전 동시 재검사가 차단해야 한다.

ACK 전 Open 금지는 chooser의 현재 PreparedSelection 부재/active 상태와 원 row 검사로 판정한다. READS3의 다른 pending ticket이 있다는 이유만으로 새 current inspect를 병렬 제출하지 않는다. route 이전 뒤 기존 Draining이 READS3+chooser 모든 원 ticket의 terminal/ACK와 attachment.release를 마친 후, 같은 handle/context로 기존 R1 target-bound current inspect 및 project_initial의 full identity/target membership 검사를 수행한다. 준비 inspect 성공은 R1 대체·편집/저장 권한·frame proof가 아니다.

## concrete exact 후보와 소유권

아래 기존5의 **별도 successor preimage/manifest**를 PM이 배정하는 제안이다. raw4CC6의 기존 파일을 덮어쓰지 않으며 실제 UI gate 뒤 최신 채택 baseline과 다시 대조한다.

| 경로 (`src/assembly/meari/` 기준) | 책임 |
|---|---|
| `viewer_callbacks/selection.rs` | 같은 Open local key의 정적/chooser row 구분, bounded prepared row와 authentic pending 보존, checked key/동일 intent coalesce/retire |
| `viewer_callbacks/lifecycle.rs` | 원 pre-export Open bind 조립, host-only row 등록, admission·소비 직전 carrier와 authentic pending 동시 검사, query/cancel/close/reexport의 retire·배타 연결 |
| `viewer_host/live_entry.rs` | LiveEntryRequest에 정적/chooser 출처를 명시하고 chooser carrier를 원 Arc 그대로 보존, 기존 zero-batch/route ticket→LiveOpening→소비 경계 연결. R1 SDK/cleanup 정책은 그대로 사용 |
| `viewer_callbacks/chooser/tests.rs` | scope/gen/Arc+lease 동시검사, ACK 전 금지, foreign/replay/변경/Busy·coalesce 및 이전 전 실패 보존 시험 |
| `viewer_host/live_entry/tests.rs` | 원 request/carrier의 route 이전·실패/Draining·inspect1·terminal ACK/release·취소 후 중복0 시험 |

`LiveEntryRequest` literal은 현재 selection.rs의 생성과 live_entry.rs의 Clone 두 곳만 확인했다. 선택 출처는 예를 들어 `Preconfigured | Chooser(PreparedSelection)`의 adapter enum으로 표현하고 selected Arc와 chooser carrier의 Arc 일치도 검증한다. 새 origin/store 권한 필드로 해석하지 않는다. frozen chooser.rs의 기존 matches_prepared seam으로 검사가 가능하므로 지금 그 파일의 추가 변경은 제안하지 않는다. 실제 구현에서 추가 seam이 필요하면 정확 경로/이유를 다시 제안한다.

실제 버튼/표시·source 선언/제품 caller는 UI의 별도 reachable R3 배정이다. source navigation의 local_signatures→기존 session key/export→PanelLocalIntentPort→zero-batch typed return 조립과 실제 Viewing Some/return proof도 UI 소유다. selection/Open adapter 성공으로 return 연결·실제 renderer/OS·수동 시각 인수를 완료 표시하지 않는다. Meari 본체·독자 router·새 domain 정책은 추가하지 않는다.

## 후속 검증과 스킬 적용

현재 chooser17 gate를 먼저 별도 인수하고, successor는 새 정확 source/타입/list/시험·원 owner cleanup 근거를 확보해야 한다. 예상 시험은 invalid/foreign lease·동일 scope/gen의 새 Arc·이전 row replay·ACK 실패/미완료·새 queued read·same/different intent·Jobs 경쟁·소비 전 Stale·route 요청 전 Busy/오류의 carrier 보존·이전 뒤 inspection 실패/취소·재export old key 폐기다. 성공 Open도 실제 R1 current inspect1/ACK/release와 별도 R2 표시/입력 proof로 구분한다. 작성된 시험/실행 성공으로 세지 않는다.

modoo.today에서 앞선 동일 작업에 탐색·읽은 `modootoday/software-qa/code-change-review-verification` 본문 SHA9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b/필수 verdict 참조9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9와 `test-fixture-and-mock-build` 본문a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100을 재사용했다. “prepared bool만으로 Open 가능”, “consume 먼저 가능”, “준비 inspect가 R1을 대체” 주장을 원 호출/반환 경계에 대조해 반증하고, atomic successor/실제 source 연결은 미구현으로 남겼다. 타 언어 구현이나 GUI 기능 우회는 도입하지 않았다.
