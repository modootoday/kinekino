---
id: plan:kinekino/20261029114004
title: SDK native 수신 관측·guarded seek 최소 계약 후보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK native 수신 관측·guarded seek 최소 계약 후보

## 현재 판정과 읽기 순서 · 2026-10-21

- A7은 root fc22d8c에서, B15 r3는 root4568aecac967a52dc2c6a9a8db755d42f844ecae에서 중앙 인수됐다. A7 집중9/영향48, B15 신규24/고유67 및 각각 양 strict/fmt0은 실제 Rust 실행 근거다. 실제 IPC/mpv/production issuer와 C owner 연결은 미완료다.
- C A11 r3 단독 gate의 guarded35/영향78은 [A11 정본](../../sots/verification/application/native-guarded-observation-ca11-2026-10-21.md)을 따른다. 후속 B7 r2+C8 union23은 fixture1 보정 r2로 양 구성 실제119(helper2 포함/행동단언117), 양 all-target strict/fmt0을 마쳤다. [owner/context 검증 정본](../../sots/verification/application/native-guarded-owner-context-union23-2026-10-21.md)을 따른다. production issuer/CLI wire/실제 media/skip 소비는 미완료다.
- [C 정확 범위/단계 표](#current-c-scope)의 A10→B6→C5/최대22는 최초 범위 이력이다. 최신 추가 승인은 A11(fence1), B7(test-only scope 접근1), B7 r2 보조3, C7(read-context 보조2)다. 각 manifest와 중복 경로로 합류하며 숫자 합산을 실행/인가 완료로 표시하지 않는다.
- [2026-10-20 초기 제안 이력](#phase-one-history) 아래의 exact11/exact8·시험 작성 전/Cargo0 표현은 당시 상태다. 현재 A7/B15 인수나 C 전체 완료를 뜻하지 않는다. 원 내용은 보존했다.

**최신 후속 gate(2026-10-21):** A7 인수 뒤 root47e3825b 기준 B15 r3를 신규24/고유67pass 및 양 all-target strict/fmt0으로 마감하고 PM에 슬롯을 반환했다. 실제 source 수리0·receipt13 terminal0/기록26PID 부재·root949/snapshot1306 보호 delta0이다. [B15 검증 정본](../../sots/verification/application/native-guarded-protocol-b15-2026-10-21.md)을 따른다. production issuer/C 실제 owner/같은 호출2단계 인가·실 mpv media는 아직 미완료이며 지원false다. 아래 source-only/pass0 표기는 해당 시점 이력으로 구분한다.

**현재 검증 상태(2026-10-21):** 1차 source11을 포함한 최종 combined30 r5에서 native 전체627pass/29ignored, default 전체523pass/24ignored 및 양 strict/fmt0을 확인했다. 실제 IPC/NativeActive/deferred stale 통합과 2차 인가·제출은 미완료이며 observation_authorized=false/guardNone/submit·resultUnsupported를 유지한다. 아래 source-only/실행0 서술은 설계·인계 당시 이력이다. 세부 실행 근거는 [combined 정본](../../sots/verification/application/native-surface-observation-combined-2026-10-20.md)을 따른다.

exact30의 중앙 인수는 **a8efdb6b6b1f674ad9bce1bf06e88b78684b2b44**다. 신규 집중40과 전체 두 구성의 중복을 더하지 않는다. heavy/shared snapshot은 PM에 반환됐고 현재 UI 소유다. 아래 2단계 결정안은 아직 제품 코드로 구현하지 않은 제안이며 이 gate의 실제 성과와 구분한다.

## 2단계 공통 계약 결정안 · root a8efdb6 기준 읽기

중앙 인수 commit은 a8efdb6b6b1f674ad9bce1bf06e88b78684b2b44다. UI가 heavy/shared snapshot을 소유하므로 SDK는 아래 설계/소스 읽기만 했고 새 코드·Cargo·실제 OS 실행0이다. 아래는 PM의 별도 구현 배정용 **결정안**이며 기존 true/false 권한을 이미 변경한 것이 아니다. 이전 terminal128/동작 미정 제안 대신 이 절의 한도와 순서를 사용한다.

### A7 소스 읽기 인수 · 실행 gate 대기

2026-10-21 A7 exact7은 별도 checkout에서 작성했고 PM이 source7 `C84156393F2959FF5DDDCE68849B0CDE8E10130DD728A011B10DFFF2A44E0AB7`의 hash/길이 delta0·권한 우회/투영 의미 변경0으로 읽기 인수했다. 집중 시험은 총9=신규8(guarded5/projection3)+기존 default Unsupported1이며 실행 pass 수가 아니다. direct rustfmt/check·diff check0, Cargo/test/clippy0이다. production issuer/registry/8KiB 구현0이고 실제 지원은 계속 닫혀 있다. `runtime/coordination/sdk-guarded-a7-r1/handoff.md`의 existing5와 registration2는 중복 등록을 포함하는 대체 검토 단위이므로 연속 apply하지 않는다. B/C 구현은 별도 배정 전이다.

### A7 실행 마감 · 슬롯 반환

2026-10-21 A7 원 source7만 별도 gate에서 집중 고유9/영향 고유48 및 default/native all-target strict/fmt0으로 마감하고 PM에 슬롯을 반환했다. 최종 integrated7은0ACF5252716C86E795984D6CFF167ECE1ABE11FBCAA70698E72C32392C77A62D다. 원mtime에 따른 첫 strict 캐시 재사용 가능성은 이력 보존 후 exact7 touch·실제 Checking으로 보완했다. before기존5/부재2, root940/snapshot1300/library405/SDK27·source7 delta0, receipt17 terminal0/기록37PID 부재를 확인했다. deferred 필터0selected와 metadata 목록은 성공 수0이다. 세부 근거는 [A7 검증 정본](../../sots/verification/application/native-guarded-contract-a7-2026-10-21.md)과 runtime/quality-review/sdk-guarded-a7-r1/handoff.md다. B15는 미포함이며 production 지원은 계속 닫혀 있다.

### B15 별도 소스 후보 · 중앙 인수/실행 전

2026-10-21 PM의 B13+application2 배정으로 별도 후보를 작성했다. `runtime/coordination/sdk-guarded-b15-r2/handoff.md`가 정확 인계이며 원 B13 manifest5AC4491B93412300CDBD3B004A36BFEAE4E6C9697C6A7F973566CC63447F52B0, 추가2 manifest1003EF5DFBD7371FF8AF7DEAA3E4D913CB8B51A71CCCD99445152FDC7A7992FD다. A7과 합친 고유20 manifest3504FB4E6846C246C09AA900066D88387057D719054F22D275E7BCE26E887D97이며 A7 원 핀은 바꾸지 않았다. 신규22 author 시험은 미실행, direct fmt/check·diff0, Cargo/test/clippy0이다. 결과 read adapter만 추가했고 production observation/control issuer와 C 실제 owner 연결은 없다. legacy FIFO는 유지하되 두 channel 사이 전체 접수 순서는 미정의이고 barrier 사이 legacy write가 가능하므로 실제 지원 개방 전 감사 대상이다. 중앙 root의 false/Unsupported를 완료로 승격하지 않는다.

### B15 r2 독립 감사 후속 · r3 소스 수리/미실행

최신 상태: A7 검증 반환과 root fc22d8c88a6dd1b07d6196f5d1e42d2cb1f3ad5b 인수 뒤 별도 staging에서 r3를 작성했다. 정확 인계는 runtime/coordination/sdk-guarded-b15-r3/handoff.md다. source13=0315FFE6B35CFEAB2729A82670C29FD2ABD051B0D77B847BBDF8EBB99B8068CC, 추가2는r2와동일, union20=8528990BF1375DD059D85338F203B74420158F0BBC8008E620EB4AB2904E537A다. 변경6은 ipc와 guarded_command/{mod,fence,queue,results,tests}이며 legacy/초기구독/guarded write가 같은 기존 tick·pinned write 경계를 쓴다. 원 registry가 Queued를 포함한 pending deadline을 보관하고 write 도중 새 접수도 확인한다. 신규2 포함 총24 author 시험, Cargo/test/clippy0, direct fmt/check·diff0이며 실제 liveness는 미검증이다. 다음 문단은 이 수리를 요구했던 r2 감사 이력이다.

PM 소스 감사에서 P2를 확인했다. ipc.rs의 legacy write_all.await가 계속 Pending이면 같은 worker의 guarded deadline/live/read 처리가 멈춰 Queued/WrittenAwaitingAck가 종료되지 않을 수 있다. OS 재현이 아닌 소스 반례다. r2 핀과 diff를 보존하며 A7 원 검증과 혼합하지 않는다. 별도 r3에서는 legacy에도 기존 tick 기반 bounded pinned write와 실패 시 연결 폐기·원 guarded terminal 처리를 적용하고, guarded pending+legacy writer 계속 Pending 회귀를 작성한다. FIFO와 기존 enqueue receipt는 유지한다. 현재 A7 gate를 우선하며 r3 Cargo는 배정 전0이다.

또한 B worker가 BarrierUpdate::Complete 표본을 버리고 seek commit하는 현재 형태는 동일 호출 observe→공통 core 평가→원 guard 제출의 완성이 아니다. C의 최소2단계 계약에서 표본 전달·opaque issuer·소비 시점을 확정해야 하며 B만 연결해 같은 barrier 권한이 완성됐다고 하지 않는다. production 권한은 계속 false다.

### B15 r3 독립 소스 감사 인수 · 실행 대기

2026-10-21 PM 재감사에서15사본/4manifest 일치·변경6/추가2 불변을 확인했고 r2의 Pending legacy write 반례 수리를 소스로 인수했다. 추가 확정 문제는 없었다. 신규2는 generic writer 회귀이며 실제 IPC loop/OS 시험이 아니다. 작성24/pass0이고 실행은 LocalHost→UI 이후 별도 배정 전까지 보류한다. 기존100ms lifetime tick에서 timeout/close를 확인하므로 deadline 시각의 즉시 중단이나 이미 기록된 bytes 취소를 보장하지 않는다. r3 원 소스와 핀은 동결하며 C2단계 인가 공백·productionfalse를 유지한다.

### C 최소2단계 계약 재설계 · 읽기 제안/미배정

PM 독립 연구를 반영한다. 아래 C18은 구현 배정 요청용 후보이며 source 변경0이다. 기존 C11 표는 이2단계 연구 전 후보로 남기며 최종 범위는 다시 배정받는다. B15 r3 liveness 수리는 이 인가 공백을 닫지 않는다.

| 단계 | 원 데이터와 소유 | 허용/거부 경계 |
| --- | --- | --- |
| 목표 없는 Observe | 기존 bounded channel에 Observe envelope를 넣고 원8RPC barrier의 Sample을 응답한다. private issuer가 원 host/fence/generation/engine/resolution/connection/lifecycle/entry/barrier nonce/deadline과 immutable view를 묶은 move-only guard를 만든다. | public 진단 숫자/authorized bool에서 발급하지 않는다. Complete 표본을 버리거나 아직 평가하지 않은 target seek를 함께 수행하지 않는다. |
| 공통 application 평가 | 같은 호출이 view와 guard를 받아 기존 skip core를 정확히1회 평가한다. worker는 core 정책을 복제하지 않고 평가를 기다리는 Active/registry lock을 잡지 않는다. | 기존 예약/cancel/revision/override 순서를 그대로 소비한다. skip 불필요·평가 실패·취소면 guard를 포기하고 seek0이다. UI/WndProc에서 RPC 완료를 기다리지 않는다. |
| 동일 guard Submit | 원 move-only guard를 소비해 그 issuer/nonce의 원 grant만 제출한다. 원 queue 접수 receipt와 read ticket을 반환하고 worker는 같은 fence의 commit 순서로 기록한다. | 새 barrier를 발행해 표본을 묵시 교체하지 않는다. 관련 원 scope/세대/표본이 바뀌거나 만료되면 거부하고 호출자가 새 관측·평가를 수행한다. 자동 retry/새 core 평가0이다. |

Observe 응답의 수신 포기/receiver drop, core의 guard 포기, deadline, cleanup을 모두 다룬다. 총64 슬롯은 observation grant와 queued/in-flight/terminal command가 공유하는 한 예산이며 별도 무제한 guard map을 만들지 않는다. Observe 접수 전 슬롯을 예약하고 실패 시 원 예약을 되돌린다. 성공한 Submit은 같은 슬롯을 command로 전환하며 terminal 결과는 명시 소비 전 퇴출하지 않는다. 포기한 observation grant만 해제할 수 있으며 이를 terminal command eviction으로 확대하지 않는다. 응답/표본/진단의 고정·가변 저장을 기존64×8KiB와 barrier 임시 예산에 다시 산정한다.

guard Drop/receiver drop은 UI callback에서 대기하지 않는 포기 신호와 기존 wake로 전달하는 안이다. 기존 worker가 bounded64 슬롯을 확인해 원 observation 예약을 회수하고 deadline/host close가 최종 정리한다. 새 thread/timer/queue를 추가하지 않는다. 원 host는 제어/관측 admission을 닫되 이미 발행한 결과 ticket의 read scope는 별도 보존한다. SDK 동기 observe의 대기는 기존 SDK 실행 worker에만 허용하며 presentation과 LocalHost 공통 수명 계약을 함께 검토한다.

이 방식도 관측되지 않은 외부 mpv 변경을 포함한 atomic compare-and-seek를 증명하지 않는다. Manual-only sampled 허용 여부와 설치된 mpv/CLI result wire/고정 미디어 인수 전에는 production 권한false를 유지한다.

정확 C18 후보는 기존 C11(실제 owner 연결 표)과 다음 추가7의 합이다: application/playback_session/{guarded,mod}.rs, infrastructure/mpv/ipc.rs, guarded_command/{queue,barrier,results,tests}.rs. private authority/응답/원 슬롯 전환과 실제 owner lifecycle를 함께 검토하는 범위다. skip 소비/API·CLI wire는 별도 담당 범위이고 여기에 무단 합류하지 않는다. 추가 source가 필요하면 작성 전 다시 보고한다.

<a id="current-c-scope"></a>

### C 생성 경로 재대조 · 필수21/조건부1의 최대22 후보

harness r2 인계 후 root4568aec 기준으로 읽기만 했다. 생산 권한 코드/실행0이며 아래 목록은 아직 구현 배정이 아니다. 앞 C18은 생성자/책임 분리 전 이력이다. 실제 두 생성 지점은 mpv/mod.rs의 spawn_process→MpvControlHandle::new와 owned_startup.rs의 spawn_with_plan→동일 new다. 기존 new(sender)를 standalone/disabled wrapper로 유지하고 새 private scoped 생성자를 추가하면 legacy spawn_process 자체의 기능 변경은 필요하지 않다. 따라서 control.rs는 필수 추가1, scope/issuer 책임 모듈 신규2가 필요해 **기능 필수 하한21**이다. root mod.rs는 C 실제 연결 뒤 B의 광범위 dead_code expect를 정리하는 조건부1로만 둔다. 숫자를 맞추기 위해 legacy 생성 동작을 바꾸지 않는다.

| 단계 | 정확 경로(src 기준) | 한 책임 |
| --- | --- | --- |
| A | application/playback_session/guarded.rs | private authority/read-scope 계약과 원 Sample view 결합. public 필드/constructor/Clone guard/serde 복원 추가0 |
| A | application/playback_session/mod.rs | 필요한 crate 내부 계약 export |
| A | infrastructure/mpv/control.rs | 기존 new(sender) 호환과 private new_scoped; 원 host scope를 ConnectionState에 주입 |
| A | infrastructure/mpv/guarded_command/mod.rs | 기존 worker의 Observe/Submit2단계 조립과 실제 Sample 전달. worker에 skip core를 넣지 않음 |
| A | infrastructure/mpv/guarded_command/queue.rs | 기존 bounded channel envelope와 동일 grant 슬롯→command 전환/실패 rollback |
| A | infrastructure/mpv/guarded_command/barrier.rs | 목표 없는 Observe의 entry 발견·원8RPC Sample 반환. Submit에서 새 barrier0 |
| A | infrastructure/mpv/guarded_command/results.rs | observation grant/command가 공유하는64 슬롯, 원 read issuer·terminal 보존 |
| A | infrastructure/mpv/guarded_command/scope.rs(신규) | Active 밖 원 host marker/fence/registry 수명과 trusted scope 주입 계약 |
| A | infrastructure/mpv/guarded_command/issuer.rs(신규) | 원 barrier/nonce/fence/gen/identity/deadline에 결속한 opaque guard 발급·소비·포기. 단순 진단 숫자로 발급하지 않음 |
| A | infrastructure/mpv/guarded_command/tests.rs | 기존 B seam을 Observe8→SDK 평가 구간→동일 guard Submit1로 변경하고 budget/abandon 회귀 |
| B | infrastructure/mpv/ipc.rs | 같은 worker/단일 allocator/tick에서 응답·실제 수신 변경·close를 원 scope에 전달 |
| B | infrastructure/mpv/session_engine.rs | Active 밖 원 scope 보관, admission 후 관측 대기 전 Active lock 해제, 제어/원 결과 read 분리 |
| B | infrastructure/mpv/owned_launch.rs | 고정 launch 경로에 원 scope 전달, 임의 command/새 실행 정책0 |
| B | infrastructure/mpv/owned_startup.rs | scoped constructor 사용과 원 startup 실패 owner/진단 보존 |
| B | infrastructure/mpv/session_engine/replacement.rs | 같은 host scope를 replacement에 전달하고 옛 표본/결과를 새 engine으로 remap하지 않음 |
| B | infrastructure/mpv/guarded_command/lifecycle_tests.rs(신규) | 실제 owner 조립 seam·원 scope 전달/close/read/cleanup drain 회귀 |
| C | infrastructure/mpv/parent_binding.rs | 반환 전 같은 fence close. callback은 atomic 신호만, 원 admission/cleanup 계약 보존 |
| C | infrastructure/mpv/parent_binding/tests.rs | revoke/commit 순서·원 permit count·late bind·lost wake 회귀 |
| C | infrastructure/mpv/parent_router/mod.rs | prepare에서 active 이전 원 scope 결속 및 guarded3 forwarding; read는 원 issuer를 검증 |
| C | infrastructure/mpv/parent_router/tests.rs | foreign/stale route와 원 ticket 결과 조회의 제어 권한 비복원 회귀 |
| C | infrastructure/mpv/deferred_engine.rs | observe/submit의 원 admission과 ticket read의 별도 권한 경계 |
| 조건부 | infrastructure/mpv/mod.rs | C 연결 뒤 광범위 dead_code expect 제거/좁은 예외 정리 필요 시만 변경. 기존 legacy constructor 호출 호환 유지 |

단계 A10→B6→C5=필수21이며 마지막 조건부1까지 최대22다. 각 단계의 정확 manifest와 기본 Unsupported 회귀를 따로 고정하며 source 조립을 실제 실행/인가 완료로 바꾸지 않는다. 새 public 필드나 필수 trait 메서드로 기존 literal/외부 ParentOwnedEngine 구현을 깨지 않는다. 기존 public constructor·launch_local/legacy Mpv 경로는 그대로 두고 원 NativeEpisodeEngine/owned startup 내부만 scoped 경로를 소비한다.

시험 seam은 실제 generic worker/response cell/registry/fence를 사용한다. Observe 전 슬롯 예약 실패 packet0,8RPC 완료 후에도 seek0, 같은 view로 기존 application core1회 평가 뒤 Submit만 seek1, 평가 중 lifecycle/관련 수신 변경·revoke·deadline·foreign scope에서는 write0을 확인한다. guard/receiver drop과 observation 포기만 grant 예약을 회수하고 미소비 command terminal은 해제하지 않는다. caller의 SDK worker가 응답을 기다릴 수 있어도 IPC worker·UI callback·Active/registry lock은 기다리지 않는다. per-request 응답 cell도64 슬롯 및 전체 예산에 포함하고 새 worker/timer/command queue를 추가하지 않는다.

추가 필수 회귀는 Active 종료 후 원 ticket read, host 종료 후 SourceExpired, 다른 router/issuer의 ticket 거부, parent revoke 반환 전 같은 fence 닫기, commit 선승 원 permit drain, startup/replacement/실패 복구의 같은 scope 유지다. native 기본값과 광고되는 실제 host seek 지원은 계속 false다. 설치 mpv/실미디어·CLI wire·Manual sampled 안전 판단은 별도 인수 전이며 외부 mpv 변경의 atomic compare-and-seek를 보증하지 않는다. 기존 skip core/CLI 소비 코드는 다른 담당 범위로 분리하고 추가 경로가 필요하면 작성 전에 다시 보고한다.

### 소비4항을 닫는 API 형태

| 공백 | 채택 제안 | 기존 호환 경계 |
| --- | --- | --- |
| seconds→us/status | playback_status/projection에 기존 micros의 finite/nonnegative/floor/checked 범위를 한 번만 둔다. NativeStatusPublisher와 guarded snapshot 생성이 같은 순수 투영을 사용한다. guard의 private view를 `into_guarded_parts()`로 status/binding/seekable view와 move-only guard로 분리한다. | 기존 PlaybackStatus 필드와 published_age 의미는 그대로다. raw property는 계속 Unverified이고 source PTS로 부르지 않는다. |
| ticket-only submission | 기존 GuardedSeekSubmission의 ticket 필드는 유지하고 `receipt()`는 **실제 enqueue 성공에서 받은 원 ControlReceipt**를 빌려 준다. 그 receipt는 ticket의 private immutable admission record에 보관하며 `into_parts()`는 원 receipt clone과 원 ticket을 반환한다. | ticket 존재나 engine 숫자로 receipt를 재생성하지 않는다. public struct 필드를 더해 literal을 깨거나 ControlReceipt를 write/ACK 완료로 바꾸지 않는다. deferred startup coalescing은 guarded 제출로 인정하지 않는다. |
| move-only pending query | 관측 guard는 여전히 noClone이다. 별도 **read-only NativeSeekTicket에만 Clone**을 추가해 owned query 입력을 반복 구성한다. consume_terminal=false는 snapshot 조회, true는 terminal만 명시 소비한다. | ticket 복제는 새 seek guard/권한을 만들지 않는다. query 도중 pending 소비는 NeedsInput이고 원 ticket은 caller에게 남는다. |
| private Result | private 저장은 유지하고 `state()`, `failure()`, `original_identity()`, `write_commit()`의 typed getter와 bounded snapshot view를 추가한다. | 성공한 조회와 seek 성공을 구별한다. 기존 Result 타입은 외부 literal 생성이 불가하므로 private 필드 교체로 구현한다. |

새 guarded status는 private guard 안의 **동일 barrier 표본**에서 만든다. raw NativeReceiveObservation public 필드를 확장해 외부 literal을 깨거나 last-publication 값을 fresh 근거로 섞지 않는다. volume/track 같은 부가 상태는 마지막 관측이라는 의미를 유지하며 skip 판단에 필요한 position/duration/pause/idle/seekable만 barrier provenance를 요구한다. guard가 없으면 guarded parts를 반환하지 않는다. native raw 진단 값만으로 bool true를 만들지 않는다.

### 파일 barrier와 실제 허용 범위

제안하는 지원 수준은 **명시 Manual 요청의 observed-owned-epoch guard**다. 자동 tick/detect/Auto 제출은 별도 지원하지 않는다. phase2 Rust protocol+실제 CLI 결과 조회+고정 미디어 인수까지 끝난 trusted host만 이 모드를 켠다. 그 전 현재 false/Unsupported를 유지한다. 이것은 외부 mpv 클라이언트까지 포함한 atomic compare-and-seek 보증이 아니다.

[공식 mpv IPC data flow](https://mpv.io/manual/stable/#data-flow)는 요청 상관 ID와 응답 순서를 설명하지만 event/외부 상태와의 원자적 snapshot을 보증하지 않는다. [playlist ID](https://mpv.io/manual/stable/#properties)는 core 수명 내 entry ID와 start/end-file의 playlist_entry_id 대응을 제공한다. 이는 stable 문서 읽기 근거이며 설치된 고정 mpv의 실제 응답 지원은 후속 실측 대상이다.

구체 barrier는 한 번에 하나, 같은 연결에서 **playlist 전조회→time-pos/duration/pause/idle-active/seekable/partially-seekable→playlist 후조회**의 상관 요청8개다. before/after playing entry ID와 관측 lifecycle이 같고 중간 end/start/disconnect/오류가 없으며 모든 필수 값이 알려진 경우만 sampled barrier가 된다. raw property-change는 파일 ID가 없으므로 이 표본으로 승격하지 않는다. 잘못된 값·ID 부재·redirect/전환·generation 변경은 새 guard0이다. 첫 모드는 partial seek=false/seekable=true를 요구한다.

독립 B/C 감사 반영: `BeforeIssued→BeforeReplyAccepted→PropertiesIssued→AllSixRepliesAccepted→AfterIssued→AfterReplyAccepted`의 상태기계를 쓴다. **전조회 상관 응답 후에만6개를 발행하고, 필수6개 응답이 모두 끝난 뒤에만 후조회**를 발행한다. 8개 동시 발행은 허용하지 않는다. late/duplicate/wrong-ID 응답은 원 barrier generation에만 귀속하며 이미 닫힌 barrier를 되살리지 않는다.

barrier는 최대1개/상관 map8개·고정 scalar 상태로 제한한다. 추가 보관 상태 목표 상한4KiB와 실패 진단8KiB를 결과64×8KiB와 별도 budget으로 둔다. playlist reply는 기존 최대1MiB IPC frame/parser의 일시 메모리를 사용하되 배열을 row별로 복제하지 않고 현재 entry ID/playing 여부만 추출한다. 초기 지원 playlist는64 entries까지로 제한하는 안이며 초과/다중 playing/없는 playing은 guard0이다. parser의 Value overhead를1MiB라고 주장하지 않는다. 이 임시 예산은 B 구현 전 구조체/파서 경계로 다시 확정한다.

일반 구독 이벤트와 상관 query 응답을 별도 provenance로 보존한다. 관련 request ID가 없는 ACK는 표본0이다. before/after 조회 사이의 시점 일관성과 관측하지 못한 외부 전환은 제거되지 않는다. guard strength는 `SampledSameObservedEntry`로 명시하고 ExactFile/atomic landing이라고 부르지 않는다. 이 제한으로 Manual조차 허용할 수 없다는 제품 판단이면 구현 중 상수 Supported로 바꾸지 않고 별도 in-engine Rust 비교 동작/검증된 libmpv 경계를 새 배정해야 한다. 권고안은 위 제한을 명시한 Manual 경로를 먼저 인수하는 것이다.

barrier 완료를 기다리는 곳은 기존 SDK worker다. UI/WndProc·parent callback에서 기다리지 않는다. native active lock은 원 identity/capability를 얻은 뒤 대기 전에 놓고 완료 시 다시 검사한다. barrier 실행 예산은 trusted host 설정의 bounded operation deadline이며 event freshness TTL이 아니다. timeout은 요청 상관을 폐기하고 late reply가 다음 barrier를 완성하지 못하게 한다. 새 worker/poll timer는 추가하지 않는다.

### revoke와 write의 한 순서

현재 ParentBinding.revoke_admission은 atomic bool 신호이고 이미 admit된 작업은 이후 끝날 수 있다. worker가 그 bool을 두 번 읽는 것만으로 write 시작과의 원자 순서를 만들 수 없다. 제안은 원 scope가 공유하는 **하나의 checked atomic admission word**에서 closed/generation/in-flight write count를 관리하는 것이다. revoke는 closed bit를 세우고 write-commit은 같은 word의 CAS로 expected generation을 검사하며 count를 예약한다.

- revoke가 먼저 이기면 writer poll0/RejectedBeforeWrite다.
- write-commit이 먼저 이기면 그 요청은 원 owner의 in-flight다. 실제 OS 바이트 전송이 revoke 뒤에 일어날 수 있으며 철회/실행 안 됨으로 표시하지 않는다.
- write-commit은 물리적 첫 바이트 시각이 아니다. 상태에서 이를 분리한다. 기존 cleanup은 이 permit이 끝날 때까지 원 owner를 보유하고, UI revoke는 기다리지 않는다.
- worker permit Drop의 count/drain 알림과 cleanup의 조건 대기는 동일 mutex/Condvar 규약으로 결합해 lost wake를 막는다. mutex guard를 async write 동안 보유하지 않는다. callback은 atomic/wake만 유지한다.

engine open/close·connection/file lifecycle 변경은 같은 scope generation을 checked 증가시킨다. parent scope는 trusted prepare에서 **active engine이 생기기 전에** 연결하고 성공 route activation 전에는 guard를 발급하지 않는다. standalone은 별도 원 SDK scope를 사용한다. parent scope의 HWND/lease를 worker/ticket으로 보내지 않는다. 기존 ParentChild cleanup driver를 추가하거나 원 engine join proof를 바꾸지 않는다.

revoke/generation 갱신은 기존 in-flight count를 지우지 않는다. old permit Drop은 새 current scope가 아니라 원 Arc scope의 count를 회수한다. 같은 fence 닫기는 ParentBinding.revoke_admission 반환 **전에** 완료되어야 하며 별도 bool을 다시 읽는 것으로 대체하지 않는다. committed write는 원 owner/drain에 남고 generation 변화로 cleanup에서 누락시키지 않는다.

관측·guarded mutation은 Deferred의 binding.admit(epoch)와 Router의 원 route admission을 거친다. root 읽기에서 ParentEngineRouter는 새3메서드 override가 없어 현재 기본 Unsupported임을 확인했으므로 forwarding 연결도 필수 범위다. 이전 read-only recovery와 마찬가지로 **원 ticket 결과 조회에 한해서만 identity 검증 read permit**을 새로 제안한다. 부모 revoke 후에도 같은 owner의 저장된 결과를 읽되 current route로 remap하거나 control 권한을 되살리지 않는다. 이것은 phase1의 무조건 admit forwarding에 대한 명시적 후속 변경안이며 PM 인수 전에는 바꾸지 않는다.

### 요청 수명·상관·메모리·오류

guarded/legacy/query/구독 command의 wire request_id는 기존 worker의 **단일 positive i64 checked allocator**로 발급하고 연결/파일/소비 후 재사용하지 않는다. observe_property의 observer ID와 wire request_id는 별개다. public ticket은 wire 숫자가 아니라 원 issuer/owner/request에 결합한 private capability다. 세션 교체 뒤 현재 engine에 옛 request를 전달하지 않는다.

결과 slot은 원 host 범위 총64다. Queued/WriteCommitted/WrittenAwaitingAck도 pending에 포함하며 terminal과 합해64를 넘지 않는다. queue enqueue와 결과 슬롯 선예약이 둘 다 성공해야 ticket/ControlReceipt를 반환한다. 실패하면 슬롯을 되돌리고 packet0이다. 미소비 terminal eviction0, consume 뒤 lookup은 NotFound다. scope가 다른 clone ticket은 RevisionConflict이며 host 자체가 사라지면 SourceExpired다. ticket은 parent/engine/process owner를 강하게 붙잡지 않는 read capability이고 result registry만 host가 보유한다.

registry는 `NativeEpisodeEngine::Active` **외부**의 같은 host/issuer 수명에 둔다. close/교체 후에도 원 ticket과 저장 결과를 읽을 수 있어야 한다. 조회 권한은 current-route admit과 분리한 원 read scope이며 다른 engine으로 remap하지 않는다. pending consume=true는 row/권리 불변으로 거부하고 terminal consume 경쟁은 정확히 한 번만 성공·나머지는 NotFound다. 이 저장·경쟁의 실제 구현은 B/C이며 A7의 readticket Clone만으로 인수하지 않는다.

원 결과 단계는 Queued→WriteCommitted→WrittenAwaitingAck→CommandAccepted 또는 Failed/OutcomeUnobserved다. commit 전 거부는 RejectedBeforeWrite다. 부분 write/timeout은 실행 여부 미관측이며 connection을 폐기해 다음 JSON frame을 이어 쓰지 않는다. 원 seek 자동 재전송0이다. ACK success는 CommandAccepted일 뿐 landing/정확 seek 완료가 아니다. close는 미commit 요청을 거부하고 이미 commit된 요청은 ACK/원 오류 또는 OutcomeUnobserved로 끝낸다. late ACK는 이미 소비/종료한 요청을 새 slot 결과로 바꾸지 않는다.

write·ACK deadline 검사는 기존 IPC lifetime tick/runtime를 재사용한다. pinned write future를 유지한 채 tick으로 deadline을 확인하며 tick마다 write_all을 취소·재시작하지 않는다. 새 timer thread/worker는 없다. deadline은 command 결과 보관 규약이며 source event freshness 인가 TTL과 구별한다.

8KiB는 **한 결과 row의 가변 payload 전체** 한도다. raw OS error code/kind/단계, bounded 문자열 prefix·원 byte 길이·SHA/truncated를 보존하고 unbounded anyhow/JSON/source chain을 결과64개에 복제하지 않는다. 외부 API의 ApiError에는 원래 std Error source chain이 없으므로 있다고 하지 않는다. 결과64 payload 최대512KiB 외에 fixed row overhead·기존 IPC frame buffer·legacy queue는 따로 계산한다. 응답 snapshot도8KiB 안에서 제공한다.

Busy=NeedsInput retryable, invalid/null=NeedsInput 또는 InvalidArgument(요청 형식), stale=RevisionConflict, revoked=Cancelled(현재 ParentBinding 기존 코드 유지), capacity/counter=ResourceExhausted, pipe/ACK=EngineFailed, poison=Internal이다. 원 stored failure 조회는 API query 성공일 수 있으므로 결과 state/failure로 seek outcome을 판단한다. 앞의 PermissionDenied revoked 제안 대신 현재 Cancelled를 재사용한다.

### 구현 배정 후보: A7 → B13 → C11 → 소비/CLI 별도

| 단계 | 정확 경로 후보 | 완료 조건 |
| --- | --- | --- |
| A 계약/공용 투영 exact7 | application/playback_session/{mod,guarded,guarded_tests}.rs; application/playback_status/{mod,observed,projection,projection_tests}.rs(projection2 신규) | 소비4항의 Rust 계약·같은 us 투영, ticket 재조회/terminal consume/error 읽기 회귀. native submit은 아직 활성화하지 않음. |
| B protocol/결과/fence exact13 | infrastructure/mpv/{mod,control,ipc}.rs + control/options.rs + native_observation/{state,tests}.rs(기존6); guarded_command/{mod,queue,barrier,results,fence,tests,fence_tests}.rs(신규7) | generic Rust reader/writer fault seam으로8 query barrier·단일 allocator·슬롯64/8KiB·partial write/ACK·CAS 순서/late reply 검증. 기존 legacy wire/receipt 불변. |
| C 실제 owner 연결 exact11 | mpv/session_engine.rs; owned_launch.rs; owned_startup.rs; parent_binding.rs; parent_binding/tests.rs; parent_router/{mod,tests}.rs; deferred_engine.rs; session_engine/replacement.rs; guarded_command/mod.rs; guarded_command/lifecycle_tests.rs(마지막 신규) | 같은 scope가 startup/replacement까지 전달되고 원 route 조회/제어 분리·parent cleanup drain·close 결과 보존을 검증. source-only 단계에서 실제 OS 성공 주장0. |

A/B/C 합계는 guarded_command/mod.rs 중첩1을 빼 고유30 후보이며, 이미 검증한 **기존 input30과 다른 인벤토리**다. 실제 서명/호출 경로를 구현 직전에 재대조하고 추가 경로가 필요하면 먼저 PM에 보고한다. 한 번에30파일을 승인했다고 취급하지 않고 단계별 독립 source pin·읽기·Rust gate를 요청한다.

자막팀은 SkipPort/native_port와 service의 기존 평가/예약/cancel 정책 공유만 소비한다. 새 공개 `segments.skip_guarded`/후속 result 조회는 기존 SkipResult/ControlReceipt에 필드를 몰래 넣지 않는 additive wire 후보이며 정확 CLI/SDK/schema/host 범위는 자막팀이 별도로 제안한다. 동일 호출의 view와 guard를 전달하며 last-observation cache를 두지 않는다. native result 조회 wire와 실제 fixed mpv/local media gate 전 host seek_submission은 false다. Auto/detection/tick은 이 단계의 완료 대상이 아니다.

필수 Rust 회귀는 wrong/late barrier8 응답·entry 변경/redirect·동일값 반복·ACK무관성, revoke-before-commit writer0/commit-before-revoke drain·cleanup lost wake, 큐 full/슬롯64 전예약 rollback/8KiB 초과진단, read ticket clone 재조회와 terminal consume 경쟁/late ACK·scope mismatch·close 뒤 원 결과 읽기다. mock 결과와 실제 installed mpv wire 지원은 분리하며 후자는 PM의 별도 중앙 슬롯에서만 인수한다.

### B13 구현 전 책임 분할 · 배정 대기

다음은 앞 표 exact13 내부 책임이며 아직 구현하지 않았다. A의 production 생성자가 없는 현재 상태에서 B가 application private 필드에 직접 접근할 수 있다고 가정하지 않는다. 실제 issuer 연결에 A 경로 추가가 필요하면 구현 전에 PM에 추가 범위를 보고한다.

| 경로(src/infrastructure/mpv 기준) | 담당 책임 |
| --- | --- |
| mod.rs | guarded_command 내부 등록과 host 수명 registry 보관 지점 준비. Active 외부 보관 계약을 유지한다. |
| control.rs | 기존 제어와 guarded queue envelope 진입, 슬롯 선예약/실패 rollback 경계. |
| control/options.rs | trusted bounded operation deadline과 한도 입력. 사용자 wire로 권한을 열지 않는다. |
| ipc.rs | 기존 worker의 단일 request allocator·reader/writer·lifetime tick 연결. |
| native_observation/state.rs | connection/file generation 변화와 표본 무효화 전달. |
| native_observation/tests.rs | raw event가 barrier 권한을 만들지 않는 회귀. |
| guarded_command/mod.rs | 내부 계약 조립. C의 실제 owner 연결과 공유한다. |
| guarded_command/queue.rs | packet/slot admission 순서와 legacy receipt 보존. |
| guarded_command/barrier.rs | before 응답→6개 응답→after 발행 상태기계와 generation 상관, 별도 임시 예산. |
| guarded_command/results.rs | host registry64/row8KiB, pending 소비 불변·terminal 단일 소비·원 결과 보존. |
| guarded_command/fence.rs | 같은 atomic word의 commit/revoke/generation/count와 원 permit drain. |
| guarded_command/tests.rs | generic fault seam으로 barrier/queue/결과/부분 write/ACK 회귀. |
| guarded_command/fence_tests.rs | revoke 선승 writer0·commit 선승 drain·old permit/count 보존 회귀. |

C에서 ParentBinding.revoke 반환 전 같은 fence 닫기, startup/replacement scope 전달, Active 종료 뒤 원 registry 조회를 연결한다. B 단독 mock 통과로 이 실제 수명주기 통합을 완료 처리하지 않는다. 신규 경로·공개 issuer 계약·생성자 소비 변경 필요가 발견되면 exact13 범위를 임의 확대하지 않는다.

### 별도 후속: LocalHost 접수 종료·원 실행 수명

2026-10-21 PM 소스 연구 인계: LocalHost server.rs Drop은 pipe thread만 join하며, 이미 dispatch된 동기 SDK 실행은 Drop 뒤 완료될 수 있다. 공개 admission close/in-flight quiescence receipt API는 아직 없다. 따라서 LocalHost Drop이나 timeout을 SDK 작업 join/정지 증거로 전달하지 않는다. 후속 SDK 범위에는 새 접수 차단, 이미 접수한 worker의 원 owner 수명, 종료 미완료 시 owner 반환과 명시 receipt 계약을 제안한다. C21 InitialExecutionParent와 독립된 공통 정리 경계가 필요하다. 정확 경로는 PM 연구 최종본과 별도 배정 후 확정하며 이번 B15에 합류하지 않는다.

<a id="phase-one-history"></a>

## 2026-10-20 초기 제안 이력 · 현재 상태 아님

다음 절들은 당시의 설계/선행 인계 기록이며 위의 a8efdb6 인수·실제 전체 gate 및 2단계 결정안을 대체하지 않는다. 특히 Cargo0·source19 이전 상태와 과거128/FIFO·revoked PermissionDenied 제안은 현재 실행 결과나 최신 정책안이 아니다.

2026-10-20 PM 배정 읽기·설계 제안이다. source19 E90B42ADDE493E00978DF8B2439CFE9FEC9FBD242C8A068268B4E6D2D2BEFC2B는 동결한다. 코드/Cargo/OS/shared snapshot 변경0이며 독립 freshness 검토와 합류한 후 구현 범위를 확정한다. [skip 관측 공백](native-segment-skip-observation.md)의 core/SkipPort 소비는 자막팀, 여기의 native 계약·수신·worker 결과는 SDK 담당이다.

현재 공통 SDK 계약 후보는 **이 문서 하나**로 인계한다. 자막팀의 observe_seek/seek_result는 각각 아래 observe_native_guard/guarded_seek_result의 이전 제안 이름이며 별도 중복 메서드를 만들지 않는다. 최종 이름과 구현 scope는 PM 인수 전이다. probe/source19·parent_binding/router/cleanup·NativeChild·unique lease/outer proxy 변경은 여기의 범위 밖이다. 새 필요가 생기면 별도 exact 배정한다.

독립 freshness 검토에서 cached snapshot 재publish가 published_age를 갱신하는 사실은 확인됐다. 그러나 현 주석/UI는 이를 게시 후 시간으로 정확히 표시하고 skip도 Unknown/false이므로 현행 권한 우회 결함으로 표시하지 않는다. 원 published_age 의미·표시를 유지하고 새 native 수신 근거만 별도 제공한다. 모든 수신 메시지 뒤 cached 전체를 publish해도 무관 ACK가 position freshness를 만들지는 않는다.

## 이력 · 단계별 정확 경로 후보

1차는 **exact11 후보**다. 기존6은 src/application/playback_session/mod.rs와 src/infrastructure/mpv/{mod,control,ipc,session_engine,deferred_engine}.rs다. 신규5는 application/playback_session/{guarded,guarded_tests}.rs와 infrastructure/mpv/native_observation/{mod,state,tests}.rs다. 책임은 additive 계약, 실제 IPC 수신 근거 보유, 조회/deferred forwarding이며 실제 guarded 제출은 계속 Unsupported다. 기존 events.rs의 NativePlayback 표시 정책과 status DTO는 그대로 둔다. ipc 수신점에서 별도 관측 state를 갱신하고 기존 events::apply를 계속 호출한다.

2차는 **exact8 후보**다. 기존4는 mpv/{mod,control,ipc,session_engine}.rs, 신규4는 mpv/guarded_command/{mod,queue,results,tests}.rs다. Sender<Value>/Receiver<Value>의 내부 queue envelope를 분리하고 legacy 제어는 동일 wire/receipt를 유지한다. 실제 alias/생성자 소비 때문에 host.rs 등 추가 변경이 필요하면 구현 전 정확 범위를 다시 보고한다. 1차와 합치면 고유15 후보이며 아직 배정된 소스 목록이 아니다.

## 이력 · 공개 default Unsupported 서명 후보

EpisodePlaybackEngine에 object-safe 기본 메서드3을 추가한다.

```rust
fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError>;
fn submit_guarded_seek(&self, input: GuardedSeek) -> Result<GuardedSeekSubmission, ApiError>;
fn guarded_seek_result(&self, input: GuardedSeekResultQuery) -> Result<GuardedSeekResult, ApiError>;
```

모두 기본 구현은 ErrorCode::Unsupported이며 새 필수 trait 메서드가 아니다. EpisodePlaybackSessions가 검증과 forwarding을 맡고 DeferredEpisodeEngine은 기존 binding.admit(epoch)를 통과한 동일 engine에만 위임한다. 기존 observation/control/ControlReceipt/public struct literal은 바꾸지 않는다. 새 계약은 독립 타입이며 새 CLI/wire 명령은 이번 범위에 없다. SkipPort는 새 계약의 지원 여부를 소비하되 실제 매체 인수 전 Unknown/observation_authorized=false/seek_submission=false를 유지한다.

NativeGuardObservation은 값/진단과 **발급자만 만들 수 있는 호출별 opaque guard**를 묶는다. guard는 원 engine/resolution·native 수신 instance·connection/file generation·원 route epoch·속성 receive sequence/barrier를 결합하고 public constructor/Clone/serde/외부 복원을 제공하지 않는다. UI lease/HWND는 포함하지 않는다. GuardedSeek는 그 guard를 소비하며 목표 position_us를 함께 전달한다. public 숫자·JSON·기존 snapshot에서 권한을 재구성하지 않는다. 장수 SkipPort의 마지막 관측 캐시에 guard를 넣지 않고 같은 skip 호출 스택에서 observe→기존 core 평가→submit으로 전달한다.

submission은 queue acceptance와 별도 opaque request ticket만 보고하고 완료를 뜻하지 않는다. result query는 동일 issuer/engine/connection/request ticket과 consume_terminal 옵션을 받는다. pending에 consume=true를 주면 거부하고 terminal의 명시 소비만 보관을 해제한다. 과거 요청을 새 engine에 remap하지 않는다. 구체 private 생성 경계와 구독/barrier의 실제 인과 보장이 인수되기 전 guard를 발급했다고 지원으로 표시하지 않는다.

## 이력 · 실제 수신과 세대

관측 state는 ConnectionState에 하나만 보유하고 IPC read loop만 변경한다. observe/status 재publish·일반 조회·무관 ACK는 receive sequence/시각을 바꾸지 않는다. property-change의 동일 값 반복도 새 실제 수신이므로 sequence가 증가한다. time-pos/duration/pause/seekable의 null·부재·형식 오류·비유한 값은 각각 Unknown으로 남기고 연결/파일 전환 때 이전 값을 무효화한다. 모든 counter는 checked이며 소진은 권한을 닫는다.

engine/resolution은 active 소유자와 결합하고 connection/file generation은 실제 연결/파일 lifecycle에서 증가시킨다. start-file/end-file/disconnect가 기존 표본을 폐기한다. 일반 property-change 자체에 mpv file generation이 들어온다고 가정하지 않는다. 재조회도 서로 원자적인 표본이라고 가정하지 않으며, 상관 ID가 있는 응답을 어디까지 동일 파일 표본으로 인수할지는 freshness 독립 검토의 필수 미확정 항목이다. 구독만 추가해 native 권한을 열지 않는다. surface Ready는 이 인가의 근거가 아니다.

임의 TTL로 seek 성공을 보증하거나 값이 반드시 변해야 한다는 조건을 만들지 않는다. queue admission과 worker dequeue 시 기대 engine/resolution/connection/file 및 근거 sequence가 그대로인지 확인하는 보수적 계약을 우선한다. 새 위치 수신 때문에 stale이면 명시 재관측/재평가를 요구하고 자동 submit loop는 만들지 않는다.

## 이력 · queue·write·ACK·보존 한도

기존 command capacity64를 재사용하고 새 worker/timer는 만들지 않는다. guarded envelope는 원 기대 세대와 요청 ID를 보유한다. 기존 active lock에서 admission을 검증하고 worker가 꺼낼 때 동일 observation gate를 다시 검사한다. revoke/close/교체와 pipe write의 선형화 경계는 freshness 검토 후 구현 전 명시해야 한다. 단순 두 번의 atomic read만으로 write 도중 외부 파일 전환까지 차단했다고 하지 않는다.

결과 상태는 Queued, RejectedBeforeWrite, WriteFailed, WrittenAwaitingAck, AckSucceeded, AckFailed, ConnectionLostAfterWrite, ExpiredOrUnavailable을 구별한다. 실제 write 일부 성공 후 오류는 실행 안 됨으로 단정하지 않는다. 성공 ACK도 목표 위치 도달을 증명하지 않는다. 요청별 원 error와 관측 단계·identity를 보존하고 전역 ACK 카운터는 사용하지 않는다. subscription ID와 guarded request ID를 분리하고 checked counter로 충돌/재사용을 막는다.

통합 한도 후보는 **결과 slot 총64**다. pending 최대64/terminal 최대64이지만 합계도64이며 admission 때 slot을 먼저 예약한다. 완료 전이를 위해 추가 할당/퇴출이 필요하지 않다. 미소비 terminal과 pending은 FIFO/TTL로 eviction하지 않는다. 꽉 차면 ResourceExhausted로 접수 전에 거부하고 명시 terminal consume 뒤에만 재사용한다. 최초 문서의 terminal128/FIFO 제안은 이 계약으로 대체한다. 명시 소비 뒤 조회는 NotFound로 구분하고 옛 요청 ID를 재사용하지 않는다.

관측 state는 속성마다 최신 표본1개만 둔다. 결과 slot의 진단 UTF-8 payload는 **8KiB** 이내, 최대64 slot으로 payload 상한512KiB다(객체 overhead와 native 원 오류 객체는 별도). 원 request별 오류 kind/code와 원 단계는 보존한다. 과대 오류 문자열은 prefix+원 길이+SHA256+truncated=true로 명시하며 전체 원문을 보존했다고 하지 않는다. 외부 API에는 미디어 URL/비밀정보를 담은 request wire를 노출하지 않는다. 기존 IPC frame bound는 그대로 유지한다.

현재 ApiError는 media-contracts의 구조체로 std::error::Error source chain이 없다. 따라서 기존 공개 source chain을 보존한다고 주장하지 않는다. native 원 io 오류는 worker 내부 typed 진단으로 보유하고 공개 ApiError 매핑과 bounded 표시를 분리한다. 진단 객체의 내부 추가 할당까지 bounded가 되도록 구현 시 하나의 슬롯 budget을 적용하며 무제한 anyhow chain/JSON details를 복제하지 않는다.

close는 해당 owner가 남아 있는 동안 pending을 명시 종료/미관측 terminal로 바꾸며 같은 request로 조회 가능하게 한다. engine owner 자체의 강제 Drop 뒤 복구는 없다. timeout/취소는 미관측 결과로 남기며 새 timer를 요구하지 않는다. 별도 CLI 결과 조회 wire/typed client/schema 인수 전 실제 host 제출false를 유지한다.

| 실패 | 기존 ApiError code / typed reason 후보 |
| --- | --- |
| 기본 기능 미지원 | Unsupported |
| null/Unknown/미수신·연결 준비 전 | NeedsInput / observation_unavailable |
| 일시 try-lock busy | NeedsInput, retryable=true / busy |
| poisoned 상태 | Internal / observation_poisoned, 인가 닫힘 |
| foreign/stale generation·근거 불일치 | RevisionConflict / stale_guard |
| 폐기 route/parent epoch | PermissionDenied / revoked_guard |
| queue/result slot/counter 소진 | ResourceExhausted / 세부 원인 |
| 사라진 engine·명시 소비된 ticket | NotFound |
| pipe write·요청별 ACK 오류 | EngineFailed / 원 단계·원 진단 |

worker dequeue 이후의 거부/실패는 이미 반환한 queue receipt를 수정하지 않고 request result에 기록한다. read/submit의 Busy를 새 ErrorCode로 추가하지 않는다. write 뒤 취소/연결 끊김은 실행 안 됨을 뜻하지 않는다.

## 이력 · 결정적 Rust 회귀 후보

- cached observe/re-publish는 position event age를 감소시키지 않고 무관 ACK는 position 수신 sequence/시각을 갱신하지 않음, 동일 실제 값 반복은 수신 sequence 증가.
- missing/null/잘못된 값/false/Unknown, disconnect/start-file/end-file와 지연된 옛 응답이 인가를 열지 않음.
- engine/resolution/connection/file/sequence 불일치, stale binding/revoked parent, poison/busy/counter 소진에서 enqueue/write0.
- queue full/closed와 pending 한도에서 접수 실패, worker dequeue 이전 교체에서 RejectedBeforeWrite·writer0.
- partial write/원 transport 오류/요청 ACK 실패/무관 ACK/중복 ACK/unknown request는 각 결과와 원 진단 보존.
- Written/AckSucceeded와 위치 관측 분리, 미소비 terminal/pending 비퇴출·한도선예약·명시소비·late ACK·close 뒤 원 query 유지와 새 engine remap 거부.
- trait 기본 Unsupported, deferred stale epoch 거부, 기존 legacy ControlReceipt/표시 observation 회귀 유지.

위 목록은 작성 예정이며 시험 source도 아직 없다. 실제 CLI/미디어 seek·부분 seek·외부 조작 인과·파일 전환 race 검증은 별도 중앙 배정 gate다. skip core의 one-pass/cancel/override/revision을 이 모듈에 복제하지 않는다.

## 이력 · 1차 source 읽기 인수 상태

1차 exact11은 source11 manifest6A90A9D65B70F0F57B3BB34F219508BB8F042AAA7792B4C9B0535EB6292909D9로 작성·동결했고 PM 독립 읽기에서 새 확정 결함0이다. 세부 정본은 `runtime/coordination/sdk-native-observation-r1/handoff.md`다. receive state8/default trait1을 작성했으며 Cargo/OS 실행0이다. 위의 전체 결정적 회귀 목록과 2차 제출 계약이 구현됐다는 뜻은 아니다.

현재 수신 Instant는 parser 후 상태 적용 시각이다. mpv property 발생/pipe 도착/source PTS와 구별한다. connection/lifecycle counter 직접 overflow와 전체 property null/타입/비유한 조합, 실제 IPC/NativeActive/deferred stale 통합은 검증 공백이다. 해당 공백을 인가false/Unverified로 유지하고 source19 gate 이후 별도 입력·배정에서 실행한다.
