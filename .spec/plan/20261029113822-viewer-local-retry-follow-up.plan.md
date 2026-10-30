---
id: plan:kinekino/20261029113822
title: UI-local retry 후속 연결 계획 · 구현 전
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI-local retry 후속 연결 계획 · 구현 전

최신 실행 인수: `runtime/quality-review/sdk-row-ui-retry-union20-r1/handoff.md`에서 원 UI13 pinD20C와 SDK7를 Native canonical로 검증해 고유212(SDK161/UI51, 각 신규10 포함)·양strict/fmt0을 확인하고 PM에 슬롯을 반환했다. 수리0/원핀 불변이다. UI51은 local actions6·callback8·표시4·controller8 등과 명시 pipe/CPU host1을 포함한다. default runtime0·실 OS 입력/시각·전체 i18n 인수0을 구분한다. 아래 source-only 이력은 이 실행 기록으로 보완되며 원문을 덮어쓰지 않는다.

최신 작성 상태: 승인된 exact13은 `runtime/coordination/ui-local-retry-r1/revision-r2/handoff.md`의 source pin `D20C6E942EEDFA231BDA0089517F7FA5B49573A2841832A8E21B9830E2DD285A`로 동결했다. 기준을 검증 C21r3 root82f605d로 올리고 원 중간본/C21r2·r3핀을 보존했다. 신규 active10 작성·실행0, Rust 파일별fmt/check·원문11 해시/patch 검사만 확인했다. 라이브러리408 원bytes를 독립 입력으로 사용하며 Cargo/shared/root제품/index/GUI0이다. 아래는 선행 계획 이력이며 구현 후보의 실제 범위·미검증은 handoff를 따른다.

2026-10-21 PM 배정은 source-only 설계다. C21r2 pin `038E0D5000EC62BC8515F9841AEBF6C1A7D4BD5CC79D11A3CE05068818D55CDF`를 보존한다. 아래13은 다음 제품 lease 후보이며 구현·새 API 채택·Cargo/shared/root/index/GUI 변경0이다. Meari Core 실행 성공과 제품 연결/시각 검토는 별도 인수다.

## 선행 입력과 경계

Meari 후보408은 commit `4b2ad301315cfbfd24af08b784271c72e909b3b8`, source SHA `7E2CDEA4D1DE34A8339CB5A64E09D0135C9224465B25F97B31DE570321047854`다. 실제 후보 정본은 `runtime/development/meari-local-actions/.spec/sots/core/local-actions.md`다. bind_local/dispatch_local 및 Invocation.matches_key·typed NotLocal/Invalid 서명을 소스에서 확인했다. 현재 Core gate가 진행 중이므로 제품이 이를 검증된 API로 채택하지 않는다.

제품 구현 배정 전 중앙이 채택한 라이브러리 commit·source pin·정확 실제 gate 결과와 기존 plugin/queue 호환을 확정해야 한다. 제품 Cargo/lock/gitlink는 이 후속13 범위에서 바꾸지 않는다. 향후 승인 후 C21 위에 별도 overlay/delta/pin으로 작성하며 라이브러리 source4나 C21 원문을 덮어쓰지 않는다.

## 최소 exact13 후보

| 경로 | 책임 |
| --- | --- |
| src/assembly/meari/mod.rs | local intent 소비 모듈 연결 |
| src/assembly/meari/viewer_read_actions/mod.rs (신규) | 원 controller가 소유한 local key→typed intent 대응·bounded 원 callback 큐 |
| src/assembly/meari/viewer_read_actions/tests.rs (신규) | key/scope·큐/중복 동작·SDK0 회귀 |
| src/assembly/meari/viewer_callbacks/lifecycle.rs | 기존 callback owner에 local binding 추가·typed target dispatch·ticket별 명시 resume/상태 조회 |
| src/assembly/meari/viewer_callbacks/tests.rs | mixed export/실 lease·stale/교차 target·원 ACK budget |
| src/assembly/meari/viewer_callbacks/codec.rs | 새 actions 구현 원문을 기존 provider source pin에 추가, SDK capability3 유지 |
| src/assembly/meari/viewer_host/mod.rs | admission/after_dispatch의 두 단계 local 소비와 원 document commit |
| src/assembly/meari/viewer_host/tests.rs | 실제 CPU runtime·close/poison·수용/투영 재시도 경계 |
| src/presentation/web_ui_adapter/player_shell/read_observation.rs | 표시 가능 동작·지역화 key/오류 code props |
| src/presentation/web_ui_adapter/player_shell/read_observation_tests.rs | 미관측/실패별 버튼 상태·표시와 권한 구분 |
| src/presentation/web_ui_adapter/player_shell/document.rs | local CallbackReference와 props 준비 |
| src/presentation/web_ui/viewing/read-panel.tsx | 기존 panel에 수신 처리 재시도3·화면 갱신1 버튼 선언 |
| src/presentation/web_ui_adapter/player_shell/preparation_tests.rs | 동일 source graph에서 모든 callback reference 준비 |

새 source module TSX는 없으므로 source-graph.json/inventory 파일 추가0이다. 기존 공통 버튼/class/token을 소비해 CSS 정의 변경0을 목표로 한다. scope 확장이 필요하면 정확 파일/이유를 먼저 보고한다. 기존 read3 codec wire/schema·SDK/pump·read_projection·close driver·source graph·main·surface는 정책 변경 대상이 아니다. provider 입력을 바꾸는 codec source 특성상 request/reply hash의 기존 계산 결과도 변하므로 gate에서 함께 대조한다.

## key와 callback 계약

초기 mounted scope의 기존 HostCallbacks에 Action signature로 local4를 추가한다: onRetryCapabilitiesAck/onRetryEngineAck/onRetryStatusAck/onRefreshReadProjection. bind_local이 반환한 실제 key를 원 viewer owner에 저장하고 enum ReadLocalIntent::{RetryAck(ReadKind),RefreshProjection}에 연결한다. source 이름은 export 식별이며 권한 분기로 쓰지 않는다. 가짜 SDK capability·registration·새 worker/endpoint를 만들지 않는다. provider의 SDK allowlist는 read3 그대로다.

기존 원 export lease에 plugin/local을 함께 내보낸다. 같은 scope의 재투영은 새 export lease만 만들고 key/owner/registry/attachment를 재생성하지 않는다. 사용자의 정상 조회는 결과 도착→consumer 수용→SDK ACK→표시 흐름 그대로이며 사람 승인 절차를 추가하지 않는다.

## admission→처리→commit

1. 원 Native node/disabled/input poison/Closing guard를 통과한 PendingCallback을 받는다. 현 실제 NativeDocumentHandle stamp.owner로 dispatch_local을 호출한다.
2. Invalid는 즉시 거절하고 plugin fallback0이다. 검증을 통과한 Plugin target의 NotLocal만 기존 read3 dispatch로 넘겨 그 경계에서 다시 검증한다. LocalInvocation은 matches_key로 원 key에 대응시키고 알 수 없는 key는 거절한다.
3. local admission은 ACK/SDK/문서 갱신을 실행하지 않는다. PendingCallback 전체를 제한된 원 owner 큐에 보존하고 Accepted를 반환한다. 타입 자체는 Clone이 아니므로 원 CallbackHandle과 CallbackInvocation의 Clone으로 명시 재구성한다. 큐는 최대4 pending action을 보유하며 여유가 없으면 Capacity로 원 runtime이 미수용 callback을 보존한다.
4. after_dispatch에서 document borrow가 해제된 뒤 큐의 원 PendingCallback을 현재 stamp.owner로 다시 검증한다. Busy는 원 callback을 보존해 다음 제한 turn에서 재검증한다. stale/invalid는 진단과 함께 종료하며 SDK/문서 호출0이다. 검증된 invocation을 durable grant로 보관하지 않는다.
5. RetryAck는 해당 ReadKind의 현재 원 ticket과 ACK 소진 상태를 Rust owner에서 다시 확인한다. C21 retry_read_acceptance의 원 ticket 검증을 통해 그 ticket의 예산만 명시 재개하고 동일 bounded consumer를 호출한다. status 조회 재submit0·새 ticket0이다. terminal 미도착/Pending에는 예산을 초기화하지 않는다.
6. RefreshProjection은 보존 원 event를 재decode하고 dirty candidate를 표시하는 기존 Rust 메서드만 호출한다. SDK submit/accept/ACK0이다. 새 SDK 조회를 화면 갱신의 대체로 사용하지 않는다.
7. intent를 성공 처리하면 원 local pending 항목을 소비 완료로 제거한다. 이후 문서 preflight/commit 실패는 이 intent나 ACK budget reset을 재실행하지 않고 기존 candidate만 재투영한다. 남은 callback batch가 있으면 C21 규칙대로 commit을 미룬다.
8. 실제 same-handle try_reproject/commit receipt·이전 export 보존·stale queued handle 거절은 C21 경계를 그대로 사용한다. Closed/Closing/poison을 local API로 해제하지 않는다.

## 화면·재시도 상태

ACK 재시도 버튼은 자동 재시도 소진으로 AckFailed인 경우에만 활성화한다. Rust가 같은 조건과 원 ticket을 재검증하므로 props는 권한이 아니다. C21의 실패 총3회·실패 간 최소25ms admission을 Open/wake/close가 공유한다. local 명시 재개가 해당 ticket에만 새 제한 구간을 열며 다른 ticket의 예산/관측값은 보존한다.

연속 클릭은 원 ticket 정합과 소진 여부를 각 intent에서 재검증한다. admission 때 원 PendingCallback과 함께 Rust owner가 관측한 ticket·재시도 세대를 제한 큐에 보존한다. 처리 시 현재 ticket/세대/실제 ACK 소진 상태가 모두 같아야 한다. 첫 intent가 재개하면 해당 세대를 소비하고 다음 세대로 올려, 같은 소진 상태에서 들어온 나머지 클릭이 후속 소진 뒤까지 지연되더라도 예산을 다시 초기화하지 못하게 한다. 세대는 Rust 내부 값이며 source props나 문자열에서 받지 않는다. 카운터 overflow는 원 소유를 보존한 실패이며 wrap하지 않는다. 새 소진 후 새로 admission된 명시 요청만 새 세대로 처리할 수 있다.

이 세대 메타데이터도 기존 최대4 local pending 항목 안에만 보관한다. 별도 work queue·무제한 역사·새 SDK operation으로 확장하지 않는다. 처리 시 불일치/미소진은 재submit/ACK/예산 reset 없이 종료하고, UI props만 보고 재개하지 않는다.

‘화면 갱신’은 Open 상태에서 사용할 수 있는 멱등 표시 동작으로 둔다. 투영 실패 때문에 새 버튼 활성 상태를 화면에 보낼 수 없는 경합을 피하도록, 보존 결과가 표시된 이전 문서에서도 사용 가능하게 준비한다. 아직 결과가 없으면 Rust에서 no-op이다. 새로운 상태 승인·결과 품질 승인 의미는 없다. raw 오류는 기존 owner에 보존하고 표시 문구/key는 공통 read_observation에서 전달한다. 전체 i18n 완료로 표시하지 않는다.

Closing/poison 이후 버튼 동작은 실행하지 않는다. 종료 중 consumer와 신뢰된 Rust 복구 경계는 기존대로 유지하며 이 계획은 창 재개/원 입력 차단 해제 기능을 제공하지 않는다.

## 정확 gate 계획

구현 후 source pin/원 C21·라이브러리 pin·before13/비중첩 보호를 먼저 고정하고 독립 읽기를 요청한다. 실행은 새 중앙 슬롯 배정 때만 한다. Meari Core 통과 수와 제품 시험 수를 합쳐 제품 연결 성공으로 표시하지 않는다.

신규 fixture 명세(현재 작성/실행0):

- mixed_queue_routes_only_verified_not_local_to_sdk: 유효 NotLocal만 기존 plugin 경로, Invalid fallback0/SDK0.
- local_retry_requires_original_export_key_scope_and_lease: 다른 key/owner/세대·old export·stale queue 거절.
- busy_local_intent_retains_original_callback_and_revalidates: 큐/재검증 및 무권한 intent 재사용 금지.
- ack_retry_resumes_original_failed_ticket_without_read_submission: 원 ticket·request 수 불변/다른 ticket 독립.
- local_retry_preserves_shared_three_attempt_twenty_five_ms_budget: Open/wake/close 동일 예산·미도착 예산0.
- repeated_local_retry_cannot_reset_the_same_exhaustion_generation: double-click·지연 queue가 같은 원 ticket 세대를 중복 소비하지 못하며 첫 재개 뒤 추가 reset0. ticket 교체/overflow도 원 소유 보존·SDK0으로 검사한다.
- projection_refresh_uses_retained_result_without_sdk_or_ack: 보존 원 event 재decode·SDK/ACK count0.
- projection_failure_does_not_replay_consumed_local_intent: 실제 CPU preflight 실패·ACK 예산 reset 중복0·old export 유지.
- closing_poison_and_disabled_local_actions_have_no_effect: 원 Native guard·local 재검증 및 engine/SDK0.
- actual_retry_panel_reexports_same_owner_and_rejects_old_queue: 실제 graph/CPU document/원 scope·new export/commit receipt.

일반 gate는 새 actions/기존 callbacks·projection·read_observation·controller·host/cleanup/document 및 기존 proposal/pump 회귀, 양 all-target strict/fmt다. pipe/SDK worker를 쓰는 fixture는 exact 이름과 별도 명시 권한으로 실행하고 전체 ignored 실행0이다. 실제 Native input/OS wake/timer/GUI/시각 검토는 CPU/Core gate와 분리한다. 원 exec/receipt·artifact·보호 대조/명시 반환을 인계한다.

다음 구현 lease 요청은 위 exact13이다. 라이브러리 최종 채택 버전·실 gate 및 제품 scope 배정이 확정되기 전 코드 연결을 작성하지 않는다.
