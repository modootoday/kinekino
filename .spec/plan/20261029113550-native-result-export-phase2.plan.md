---
id: plan:kinekino/20261029113550
title: Native result export phase2 단계별 연결 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native result export phase2 단계별 연결 계획

2026-10-21 PM 배정: 소스 읽기·계약/계획만. 코드/Cargo/shared 실행0. union20 실행은 UI 소유다. 1차 source7·현재 감사 및 gate는 `runtime/coordination/sdk-row-export-r1/`를 따르며, [생산자 계획](native-seek-row-export-producer.md)·[skip 소비 계약](native-segment-skip-observation.md)을 연결한다. 아래 exact는 단계별 편집 후보이며 새 코드 배정이 아니다. 각 단계 시작 시 최종 채택 base·원 before·보호값과 필요한 경로를 확정한다.

## 현재와 목표의 차이

1차는 원 row/cell·metadata 보존 기반과 Unsupported/Unavailable 경계다. 실제 trusted standalone Engine/Sessions 연결, C8 준비된 read owner, wire reference의 공식 변환·복구·consume는 없다. Native 첫 compile exit0도 이 기능의 실제 지원 증거가 아니다. 단계 인수 전 registry/제품 지원 flag를 true로 바꾸지 않는다.

### A. Trusted standalone Rust API 연결

최종 gate는 [A9 검증 정본](../../sots/verification/application/trusted-standalone-export-a9-2026-10-21.md)을 따른다. 원 strict101을 보존하고 동일 함수 위치만 옮긴 r2 source9 C7509337…79D07에서 Native174/helper2·양strict/fmt 통과 후 운영 반환했다. 아래 source-only 기록은 원r1 작성 시점의 이력이다. root 채택·실미디어·B/C는 별도다.

PM의 A 정확9 source-only 배정으로 독립 `runtime/development/sdk-row-export-standalone-r1`에 작성·동결했다. root5a40+원row7 합성 before9와 원핀은 `runtime/coordination/sdk-row-export-standalone-r1/handoff.md`에 기록한다. source9 `1E9545E8F66118DF2D4CBC56B4949AED46D11454315899764F7F7816FBC0B1EE`, 신규회귀13/directfmt·diffcheck0이며 typecheck/test/Clippy0다. B/C·wire·실미디어 인수로 승격하지 않는다.

| 정확 경로 후보 | 책임 |
|---|---|
| `src/application/playback_session/native_result_export.rs` | 명시 read-owner 준비·조회/목록/consume 입력·출력, issuer identity와 provenance 타입 분리 |
| `src/application/playback_session/guarded.rs` | crate-private authority의 명시 export submit과 준비된 원 read cell 연결; default Unsupported 유지 |
| `src/application/playback_session/mod.rs` | Engine 기본 unsupported 조회 계약·같은 Sessions 전달만 |
| `src/infrastructure/mpv/guarded_command/{exports,results,issuer,scope}.rs` | trusted standalone 준비 token·원 issuer 검증·동일 row export·목록/resolver·원 consume |
| `src/infrastructure/mpv/session_engine.rs` | 기존 verified standalone scope/closed/issuer 검사로 명시 opt-in; 현재 Active로 결과 재선택0 |
| `src/infrastructure/mpv/guarded_command/mod.rs` | 회귀/계약 등록만 |

이 단계는9 경로 후보다(brace 확장4 포함). constructor가 검증한 기존 scope가 standalone 모드를 발급하며, context None은 발급 근거가 아니다. 기존 disabled/new/start와 parent 입력 경계는 유지한다. prepared 값은 native issuer 권한과 별개인 검증된 skip provenance를 받는다. 원 ticket/admission/OnceLock을 한 번 만들고 row/caller에 공유하며 try_send 뒤 fallible 보존을 추가하지 않는다.

resolver는 원 issuer를 검증한 뒤 원 row를 찾아 준비된 원 ticket을 얻고 lock을 풀어 read한다. pending consume은 NeedsInput/권리 불변, terminal consume은 원 mutex에서1회 제거한다. row lock 아래 ticket.read를 재진입하지 않는다. 목록은 metadata만 내보내며 read 허가와 별개다. native read는 현재 skip revision·현재 Active·새 engine을 요구하지 않는다. host close 뒤 원 owner가 살아 있는 동안의 결과 조회와 실제 owner 만료 SourceExpired를 구별한다.

인수 회귀: 원 verified standalone positive/disabled·foreign·closed submit, same cell/원 issuer, lost submit reply 후 목록 복구·추가seek0, revoke/교체 후 원 결과, full64/pendingconsume/동시terminal1회, Weak owner drop/EOF, retained·transient 메모리와 escaping을 각각 검증한다. C8는 이 단계에서 기본 Unsupported를 유지한다.

### B. C8 read-owner 사전 결속

최종 [B11 검증 정본](../../sots/verification/application/routed-native-export-b11-2026-10-21.md): source7212 무수리로 Native184(helper2 포함/신규10)·양strict/fmt와 최종 보호/운영 대조 후 슬롯을 반환했다. 아래 source-only 기록은 동결 시점 이력이다. router 전체 list Unsupported/C0 wire·CLI·지원true는 후속으로 유지한다. root 채택은 PM 책임이다.

PM의 정확10 source-only 배정 뒤 typed 계약 crate 재공개를 위한 application/mod.rs1을 추가 승인받아 B11로 동결했다. 독립 c1aeb overlay와 before10/추가1/source11은 `runtime/coordination/sdk-row-export-routed-r1/handoff.md`를 따른다. source11 `7212FA808837E363521134F80EB0F906FA71980DD1159D11355116075468DC06`, 신규회귀10/directfmt·diffcheck0, typecheck/test/Clippy0이다. 아래10은 최초 후보 이력이며 현재 정확 범위에는 등록1이 더해진다. router 전체 목록은 원 issuer query 부재로 Unsupported를 유지하며 C의 공식 codec/복구목록을 현재 route로 대체하지 않는다.

| 정확 경로 후보 | 책임 |
|---|---|
| `src/application/playback_session/{native_result_export,guarded}.rs` | 이동형 trusted read-owner 준비 token과 완료 상태, 외부 mint0 |
| `src/infrastructure/mpv/parent_binding.rs` | 원 binding/epoch의 Weak ReadOwner를 admission 전 준비하고 token1회 소비 |
| `src/infrastructure/mpv/deferred_engine.rs` | 원 fixed binding의 준비 owner 전달·원 context 읽기 |
| `src/infrastructure/mpv/parent_router/mod.rs` | 원 router/binding/epoch 결속·현재 route로 결과 retarget0 |
| `src/infrastructure/mpv/parent_router/tests.rs` | foreign/replacement/원 route 부재 회귀 |
| `src/infrastructure/mpv/session_engine.rs` | standalone-only 경계와 별도로 준비된 routed owner를 명시 인수·원 context 읽기, generic fallback0 |
| `src/infrastructure/mpv/guarded_command/{exports,issuer,results}.rs` | 준비 owner를 같은 원 cell에 enqueue 전 결속, 원 row에 완료 mode 보존 |

10 경로 후보다. A engine의 standalone-only 검사 때문에 실제 routed producer에는 session_engine 연결도 필요하다는 후속 읽기를 반영했다. generic `bind_read_context`의 두 번째 set 거절은 유지한다. 새 export 경로만 준비 token 소비로 결속 완료를 확인하고 기존 `bind_result`의 사후 set을 중복 호출하지 않는다. 기존 non-export C8 동작은 유지한다. 준비 실패/원 owner mismatch는 admission 전에 거절한다. generic post-admission 실패를 모두 미접수로 일반화하지 않고 metadata와 원 오류를 보존한다. 원 owner 완성이 안 된 record는 Unavailable이며 plain read를 열지 않는다.

인수 회귀: None 두 경우, token duplicate/foreign/expired, 모든 Clone context 공유, 기존 doublebind 거절, route 교체 뒤 원 context 조회, 원 binding/router 부재, lock/revoke 경계, idle export의 strong cycle0. 이후에만 C8 export의 해당 Rust 계약 구현을 인수한다. OS parent/frame 품질과 실제 미디어는 별도다.

### C. Skip 조립과 typed wire

소비 담당의 [C additive wire 상세 계약안](native-result-export-wire-contract.md)을 연결한다. 명령/strict DTO/context 분리·전체 출력 cap·원 annotation/target provenance·응답 유실과 consume 의미의 검토안이며 아직 코드/지원 배정은 아니다. SDK 공식 codec/메모리/오류 계약과 수치 조율이 선행한다.

| 정확 경로 후보 | 책임 |
|---|---|
| `src/application/skip_segments/native_port.rs` | 같은 view/guard1·원 evaluate1/attempt 예약을 유지하며 준비된 export submit 사용 |
| `src/application/skip_segments/host/registry.rs` | 기존 동일 Sessions Arc에 결속, 원 skip generation/session/pass/구간/session revision의 provenance 검증 |
| `src/application/skip_segments/host/{contracts,dispatch,schema,mod}.rs` | additive DTO·명령 등록/strict decode·조회/소비 규칙; 기존 segments.skip DTO 유지 |
| `src/application/api/client/skip_segments.rs` | 동일 새 계약의 typed client |
| `src/application/playback_session/native_result_export.rs` | bounded wire reference 공개 값 계약, opaque native ticket의 serde0 |
| `src/infrastructure/mpv/guarded_command/{exports,results,scope}.rs` | 원 issuer 세대+원 row에 대한 공식 ref 변환/해석; 별도 command ID/map0 |
| `src/bin/player-cli/{native_host,playback_host}.rs` | 같은 원 Sessions와 skip owner 조립만 |
| `tests/native_skip_cli.rs` 신규 | fresh CLI 프로세스·원 host endpoint의 종단 회귀 |

14 경로 후보다. SDK wire converter가 row의 기존 issuer/host 세대와 원 row 참조를 소유한다. native host generation은 skip registry generation과 별도 타입/생성 책임이다. 원 scope에 wire용 stable issuer identity가 없다면 scope adapter에서 명시 host identity를 준비해야 하며 주소/Debug.id/request_id/receipt로 대신하지 않는다. token은 bounded read reference이며 guard/submit 권한이 없다. 형식·cap·오류 순서를 구현 전에 고정하고 새 독자 command ID를 발급하지 않는다.

명령명은 아직 미확정이다. additive export-submit/metadata-list/result-peek/terminal-consume 역할을 분리한다. list/peek는 현재 skip expected_revision을 요구하지 않고 원 endpoint/issuer/provenance를 검증한다. submit의 expected_revision은 원 session revision이며 annotation revision/target은 별도 값이다. 실제 원 annotation/target 보존이 필요하면 metadata 예산과 함께 명시 필드를 추가하고 현재 segment로 재구성하지 않는다. consume는 native 결과 row 해제이지 새 skip 정책 mutation/seek 철회가 아니다. 기존 dispatch의 일괄 mutation/read expected_revision 분류에 임의 편입하지 말고 새 계약의 요구를 분리한다.

새 host 조립은 capability를 무조건 열지 않는다. 지원 근거와 오류를 제공하는 명시 기능 범위를 먼저 인수하고 실제 미디어 gate 전에는 제품 지원의 기존 false를 유지한다. 이 계획은 기존 false를 바꿀 권한을 부여하지 않는다.

## 공통 출력·복구·수명 인수

- submit 응답 encode/transport 실패 뒤 원 exported row를 남기고 같은 살아 있는 host의 bounded list/원 provenance로 복구한다. 재submit·추가seek0. metadata 열거는 read 허가를 자동 복원하지 않는다.
- list는 최대 원64 row의 참조·요약만 반환한다. retained8KiB와 grant/tentative row/준비Clone의 transient peak를 구분한다. 원 진단 hash/original_bytes/truncated 유지, display escaping·목록 전체 bytes cap·과대입력/출력 쓰기 실패를 Rust로 측정한다. 독립 결과 map·pending+terminal각64·미소비 eviction0.
- pending consume은 원 row/조회권을 남긴다. terminal consume 경쟁은1회 해제, 이후NotFound. 응답 유실 뒤NotFound는 전달 ACK가 아니며 exactly-once delivery/무제한 tombstone/재submit으로 보상0.
- EOF는 원 admission close→in-flight/worker/child 정리→scope 만료 순서다. metadata/readcap은 owner를 강하게 연장하지 않는다. restart/foreign 참조 자동 복원0, 현재 route/Active로 재귀속0. cleanup Incomplete는 원 상태로 남긴다.
- 각 단계는 별도 source pin/독립 감사 뒤 단일 실행 배정에서 Rust 회귀와 strict/fmt를 수행한다. 구성별 실제 시험/타입 검사를 구별한다. fresh CLI 합성과 실제 mpv 인수도 분리하며, 실제 고정 media·EOF/revoke·관측/접수/write/ACK/landing 근거 전 전체 재생/skip 지원 완료를 주장하지 않는다.

실패 시 원 로그·ticket/row를 보존하고 미인수 신규 연결을 닫는다. 이미 접수한 seek를 자동 취소/재전송하지 않으며 기존 non-export submit·지원false 상태를 보존한다.
