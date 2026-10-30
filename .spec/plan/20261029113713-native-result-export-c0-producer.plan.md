---
id: plan:kinekino/20261029113713
title: C0 정본과 단계별 정확 경로
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C0 정본과 단계별 정확 경로

2026-10-21. [C0 v1 계약 정본](../../sots/contracts/playback/native-result-export/index.md)이 이전 후보보다 우선한다. P0 정확5는 source-only 배정과 단일 gate를 거쳐 최종r3 F536…94E3B/Native199·양strict/fmt·보호 대조 후 운영 반환했다([검증 정본](../../sots/verification/application/native-export-c0-p0-2026-10-21.md)). root 적용/채택은 PM 책임이다. 아래 최초 경로 분해는 이력이며 **미인수 P1/P2·소비/통합 경로의 편집 권한을 부여하지 않는다.** CLI flag/OS·실미디어는 이번 변경 범위 밖이다.

## 확정한 설계와 관측 근거

- codec: `nsi1.`+nonce hex32=scope37bytes, `nsr1.`+nonce32+`.`+원 row hex16=ref54bytes. 엄격한 v1/ASCII/소문자/폭/nonzero roundtrip. 원 scope별 GUID1회와 기존 grant/row id를 사용하며 새 command ID/map0.
- 원 issuer는 고정 opaque Weak handle이다. router는 명시 원 ParentRouteHandle에서 캡처하며 current route로 문자열 scope를 해석하지 않는다. CLI host는 같은 Sessions와 고정 handle 하나를 조립한다. 원 issuer 지정 없는 router 전체 list는 Unsupported 유지.
- 원 annotation/verified revision/timebase는 선택된 같은 세션 상태에서 전달하고 actual target/engine/resolution은 SDK 실제 제출 값에서 stamp한다. 기존 Rust row에 wire metadata를 소급 만들지 않는다.
- list/peek/consume·오류 순서·응답 유실/at-most-once·EOF/foreign·bounded projection은 정본의 책임별 문서에 있다. JSON envelope98299/총98304bytes·item1024/최대64는 구현 시 Rust max-bound 시험으로 인수할 설계 상한이다.
- 소스의 `issuer.rs::submit_exported`는 Registry가 만든 position0 packet을 closure에서 실제 position으로 덮는다. 따라서 **기존 공통 producer4만으로 v1 actual target/metadata 일치를 구현할 수 없고 issuer.rs가 필수**다.
- B NativeEngine/Engine/Sessions에 새 v1/expected issuer Observe 계약을 전달해야 한다. C8 capture/forwarding의 parent_binding/deferred/router 경로도 빠뜨리지 않는다. 기존 public struct literal에 필드를 추가하기보다 additive v1 입력/메서드를 사용한다.

## SDK 소유 생산자14 · 세 단계

P0 정확5 source-only 배정으로 rootc485 독립 overlay에 동결했다. 원 before/source와 인계는 `runtime/coordination/sdk-native-export-c0-p0-r1/handoff.md`, source5 `EE8B1724C873A8F9D8A2AC44178CD81C1341A878DD9035CAF7C7A1B8434A90B0`다. 신규15/directfmt·diffcheck0, typecheck/test/Clippy0이며 실제 OS issuer·row/target packet·pretty/CLI/전역capability는 후속이다. 예산 상한은 max-bound 시험 실행 전 설계값으로 유지한다.

| 단계 | 정확 경로 후보 | 책임 |
|---|---|---|
| P0 계약5 | `src/application/playback_session/native_result_export.rs` | 기존 legacy 계약 보존·v1 타입/opaque issuer facade·하위 모듈 등록 |
| P0 | `src/application/playback_session/native_result_export/codec.rs` 신규 | strict lookup 값/고정 codec; parser는 권한을 mint하지 않음 |
| P0 | `src/application/playback_session/native_result_export/metadata.rs` 신규 | 별도 origin/annotation/실제 target 결과 타입·bounded shape |
| P0 | `src/application/playback_session/native_result_export/projection.rs` 신규 | 공식 bounded Native/core error·list/result DTO·출력 상한 |
| P0 | `src/application/playback_session/mod.rs` | 재공개·Engine default Unsupported/Sessions 전달 서명 |
| P1 원 row6 | `src/infrastructure/mpv/guarded_command/{exports,results,scope,issuer}.rs` | nonce/원 Weak issuer handle·same row/ref·원metadata/target packet 결속·lookup/list/read/consume |
| P1 | `src/application/playback_session/guarded.rs` | v1 authority/guard 전달·B 준비 token 보존·기존 submit 호환 |
| P1 | `src/infrastructure/mpv/session_engine.rs` | 원 scope 획득/기대 issuer pre-observe·v1 submit 전달·현재 Active로 결과 retarget0 |
| P2 C8 전달3 | `src/infrastructure/mpv/parent_binding.rs` | 고정 epoch/원 scope handle 캡처·기대 issuer 확인·원 read-owner |
| P2 | `src/infrastructure/mpv/deferred_engine.rs` | fixed binding의 issuer/observe/v1 전달 |
| P2 | `src/infrastructure/mpv/parent_router/mod.rs` | 명시 원 route handle capture·신규 submit admission의 기대 scope 확인·원 context 전달 |

14는 서로 다른 파일 수다. 역사적 공통4(application 계약1+exports/results/scope3)에 필수 issuer1=5, 책임별 계약 helper3=8, guarded와 appmod/native engine 및 C8전달3을 더해14다. 신규 helper3은 기존 `.rs`의 하위 모듈로 추가하며 전체 파일 재배치를 하지 않는다. 테스트는 각 책임 모듈의 cfg(test)로 우선 작성하고 별도 경로가 실제 필요하면 before/source exact를 갱신한다.

P0는 소비자와 공유할 타입/행동·default Unsupported를 먼저 고정한다. P1은 원64 row와 공식 ref/원 actual target을 검증하고, P2는 현재 route 조회로 original issuer 부재를 우회하지 않는다. 각 단계는 원 before/source pin·독립 읽기·검증 구성·미완료를 따로 인수한다. 구현하지 않은 API를 fallback/지원true로 표시하지 않는다.

## 소비10과 getter1의 분리

기존 소비10은 그대로 소유 anchor다: `src/application/skip_segments/native_port.rs`, `host/{registry,contracts,dispatch,schema,mod}.rs`, `src/application/api/client/skip_segments.rs`, `src/bin/player-cli/{native_host,playback_host}.rs`, `tests/native_skip_cli.rs`.

추가 필수 후보1은 `src/application/skip_segments/service.rs`의 **bounded 선택 상태 getter만**이다. 현 snapshot은 segments 전체를 clone하므로 export session_state와 원 annotation을 얻기 위해 전체 snapshot을 복사하지 않는다. getter는 원 binding/revision/pass와 선택1개의 annotation metadata만 반환하고 evaluate/예약/취소/override 정책을 바꾸지 않는다. consumer 정확 범위는 최초10+getter1=11 후보이며 편집 전에 PM이 배정한다.

CLI 조립2를 제외한 선행 소비9는 P0 계약 뒤 별도 overlay에서 작성할 수 있다. 독자 codec·fake ref 문자열·새 정책/결과 map0. 실제 SDK Ok로 받은 원 ref를 core 오류 전에 보존하고 session_state 최소 projection만 출력한다. 기존 segments.skip DTO/old API/지원false와 원 core policy를 보호한다. CLI2는 SDK/소비 핀을 인수한 마지막 통합에서 연결한다.

## 전역 등록·출력 공유6 누락 방지

| 추가 경로 후보 | 실제 이유 |
|---|---|
| `src/application/api/capabilities.rs` | COMMANDS 고정 배열에 신규4 등록. 동적 host schema만 고치면 전역 명령 목록/schema enum에서 빠짐 |
| `src/application/api/mod.rs` | 기존 skip fast dispatch/schema 경로와 export capability 주입. 기존 SkipHostCapability public literal을 확장하지 않고 additive capability 메서드/describe를 사용 |
| `src/application/local_host/mod.rs` | 기존 bounded encoder를 공유하고 compact/pretty의 동일 예산 인코딩 책임을 한 곳에 유지 |
| `src/infrastructure/local_host/server.rs` | 현재 전역4MiB로 encode/write한다. 신규4에만98299 JSON 예산·bounded fallback 적용 |
| `src/infrastructure/local_host/client.rs` | 요청 명령을 알고 새 응답 frame 크기를 allocation/decode 전에 같은 예산으로 제한 |
| `src/bin/player-cli/runner.rs` | 현재 to_string/pretty 뒤 newline을 쓴다. 신규4의 compact/pretty 최종 출력 예산·오류/종료 코드를 공통 bounded writer와 연결 |

이 공유6은 소비자가 생산자4를 동시에 수정하는 이유로 쓰지 않는다. SDK/통합 담당을 PM이 별도 지정한다. 명령 이름은 host의 단일 정의를 공유하고 server/runner가 제각각 문자열 목록을 만들지 않는다. 기존 전역4MiB·다른 명령 출력 정책은 유지한다. api/schema.rs는 현재 skip 전용 schema fast path와 COMMANDS 참조로 연결되므로 실제 필요 없이 추가하지 않는다. 전역/host schema·capabilities·typed client가 동일 지원 상태인지 회귀로 추적한다.

전체 최종 후보 union은 SDK14+소비11+공유6=31이며 한 번에 구현하라는 배정이 아니다. P0→SDK 생산/소비 별도 pin→공유 등록·출력6+CLI조립2 마지막 결합 순서다. 충돌하는 appmod/contract 파일은 SDK 단독 소유하고 소비자는 인수된 계약을 사용한다.

## 검증과 완료 조건

고정 pin/directfmt 뒤 구성별 all-target strict를 먼저 실행해 저비용 type/lint 수리를 마친 뒤, 같은 최종 pin의 fresh test binary와 실제 검증을1회 우선한다. 진행 중 gate를 재시작하거나 시험/최종 pin 검증을 생략하지 않는다. 실제 실행 슬롯은 PM 단일 배정이다.

필수 Rust 회귀: codec strict/roundtrip/zero/foreign/restart·scope prewarm 실패 observe0, 실제 packet target=저장target·원annotation 이후수정 불변·old rows비노출, samecell/B token·send 실패/64·원 context/Weak/consume, list lost response 복구/추가 seek0, max-bound64items·request id·escaping·compact/pretty·header/newline·bounded error/transport failure, 전역등록/schema/capability/client/CLI 동등성이다. source/시험/실제 native/지원 품질을 구분한다.

실제 fresh CLI는 같은 endpoint/원 scope로 제출·목록·peek·consume·출력 유실·EOF/foreign를 검증한다. 합성 Native/CLI와 실제 mpv/ACK/landing·GUI는 별도다. root/index/채택은 PM 책임, 원 실패·receipt/PID·입력/보호·복구와 명시 반환을 유지한다. 현재 실제 인수는 P0 pure lookup/DTO/default Unsupported까지이며 P1/P2·전역출력/CLI 통합은 미인수다. 소비9의 source-only 진행은 소비 담당 인계를 따르며 P0로 그 실행 성공을 선반영하지 않는다.
