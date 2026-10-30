---
id: plan:kinekino/20261029113522
title: 원 native64 row export 생산자 최소 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 native64 row export 생산자 최소 계약

후속 연결 단계와 정확 경로 후보는 [phase2 계획](native-result-export-phase2.md)에 분리한다. 현재 union20 실행·원 source7 pin과 별개인 읽기/계획 문서이며 새 코드 배정이 아니다.

2026-10-21. PM 후속 배정은 소스 읽기·설계 구체화다. 코드 구현/Cargo/shared 변경0. SDK7 `07251ea`, skip5 `b89682ab44f9bb31dbf263890a8c95cb5d53a480` 채택 뒤의 후속이며 [union12 검증](../../sots/verification/application/trusted-standalone-skip-union12-2026-10-21.md)을 승계한 새 실행 증거는 없다. skip 소비 요구는 [원64 export 선행 계약](native-segment-skip-observation.md#원64-row-내-export-record-권고--sdk-선행-계약)을 따른다.

## 소스 관측과 변경 경계

`guarded_command/results.rs::Registry::admit_observation`은 grant id를 그대로 command row로 바꾸며 원 mutex 아래 try_send 후 Queued row를 넣는다. `issuer.rs::GrantAuthority::submit`이 그 뒤 `Ticket::submission`을 호출한다. `playback_session/guarded.rs::native_seek_submission`은 매 호출 새 admission Arc/read-context OnceLock을 만든다. row/caller별로 두 번 만들면 C8의 `parent_binding.rs::EngineAdmission::bind_result` 장식을 공유하지 않는다.

생산자 1차 exact 후보는 아래7이다. 아직 편집 배정이나 검증된 파일 수가 아니다. host/Sessions/route 공개 조회·wire는 2차 exact로 분리한다.

| 경로 | 한 책임 |
|---|---|
| `src/application/playback_session/native_result_export.rs` 신규 | 비직렬화 준비값·불투명 read reference·고정 provenance 값 계약 |
| `src/application/playback_session/mod.rs` | 신규 계약 등록/재공개만 |
| `src/application/playback_session/guarded.rs` | 원 submission 생성/공유와 명시 export submit 계약, 기존 submit 호환 |
| `src/infrastructure/mpv/guarded_command/exports.rs` 신규 | private row export record·검증/예산·원 reference 해석 |
| `src/infrastructure/mpv/guarded_command/results.rs` | 원 row 소유·grant→command 전환과 export 결합·조회/소비 |
| `src/infrastructure/mpv/guarded_command/issuer.rs` | export 준비를 원 admit에 전달, submission 사후 재생성 제거 |
| `src/infrastructure/mpv/guarded_command/mod.rs` | export 모듈·인라인 회귀 모듈 등록 |

시험은 신규 exports.rs의 cfg(test) 하위 모듈로 시작한다. 별도 시험 파일이 필요하면 exact를 갱신한다. 실제 standalone host와 binding/router에서 공식 조회를 제공하는 연결, skip 준비값 생성·consumer 및 CLI는 이7로 완료됐다고 표시하지 않는다.

## API 초안과 admission 순서

이름은 구현 전 검토용이다. application은 infrastructure 타입에 의존하지 않는다. `PreparedNativeResultExport`는 검증된 bounded provenance와 원 host 세대를 소유하는 이동값이며 guard·새 command slot을 만들지 않는다. application 내부의 신뢰된 skip 조립만 생성하고 외부 caller 문자열을 그대로 권한으로 받지 않는다. provenance에는 원 skip registry generation/session/pass/segment/revision의 제한된 식별값만 보존한다.

- 명시 export submit은 기존 `NativeObservationGuard`를 소비하고 준비값을 함께 전달한다. 성공은 원 `GuardedSeekSubmission`과 read-only `NativeResultReference`를 함께 반환한다. 기존 submit의 결과형/동작은 유지하고 기본 미지원 엔진은 Unsupported다.
- 생산자 내부 `prepare_submission`은 원 Ticket와 receipt에서 NativeSeekTicket를 정확1회 만든다. row의 export record와 caller는 이 ticket의 Clone을 받는다. `same_request`만으로 context 동일성을 대신하지 않고 admission Arc/OnceLock 공유를 회귀로 확인한다.
- 공식 resolver는 원 owner 안에서 reference를 검증해 원 ticket Clone을 반환하는 책임이다. Debug.id 파싱·receipt/request_id 역산·현재 engine/route 선택은 하지 않는다. reference는 원 issuer/host 세대와 원 row를 식별하는 제한된 opaque 값이며 submit/guard 권한은 없다. restart/foreign reference는 원 owner 확인에 실패한다.

원 mutex 진입 전 typed provenance/길이·출력 형상 검증과 bounded 할당을 끝낸다. 원 mutex 아래 grant/revision/deadline/fence를 기존처럼 검사하고, 같은 id의 row payload·ticket/read cell·optional export record를 준비한 뒤 try_send한다. 실패하면 원 grant 포기 규칙과 임시 record 정리를 따른다. 성공하면 원 grant→command를 전환하고 준비된 submission을 반환한다. 성공 후 별도 fallible export insert나 두 번째 ticket 생성은 두지 않는다. worker는 같은 mutex를 획득해야 row를 읽으므로 Queued row 전환 전 관측하지 못한다는 기존 순서를 유지한다. 프로세스 OOM/panic까지 복구되는 트랜잭션을 약속하지 않는다.

C8 bind_result는 성공 후 동일 OnceLock을 장식하므로 row의 Clone도 장식을 본다. generic decorator의 post-admission 오류는 여전히 가능하다. export row를 보존하더라도 아직 장식되지 않은 ticket을 plain read로 우회하면 안 된다. 1차 producer의 회귀는 이 경우를 미완료/원 오류로 남기고, 2차 routing 연결에서 준비된 원 read owner를 admission 전에 결속하거나 명시 read-unavailable 상태를 보존하는 방법을 확정한다. 모든 Err=admission0이나 receipt에서 context 복원은 금지한다.

## 읽기 owner 준비의 추가 제약

skip 담당의 직접 검토를 반영한다. `read_context=None`은 정상 standalone뿐 아니라 routed ticket의 장식 전에도 가능하므로 plain read 허가 조건으로 쓰지 않는다. export 준비에는 SDK가 검증한 별도 read-owner mode와 완료 상태를 보존한다. 내부 후보는 `StandaloneOriginal`, `RoutedBindingRequired`, `ReadUnavailable`이며 이름 자체는 미확정이다. standalone 모드는 원 SDK 권한으로만 확정하고, routed 모드는 원 binding/router 결속 완료 전 read를 거절한다. 장식 실패 뒤 열거 가능한 provenance/참조 metadata와 실제 읽을 수 있는 capability를 구분한다. metadata 복구가 read 권한 복구를 뜻하지 않는다.

사전 read-owner 결속을 선택한다면 기존 C8 `bind_result`가 같은 OnceLock에 다시 set하지 않도록 route 전용 준비 token을 단1회 소비하는 조립 계약이 필요하다. 이 경계는 허가된 같은 원 owner가 이미 결속됐음을 확인하고 원 submission을 그대로 전달한다. generic `bind_read_context`의 중복 set 거절 규칙을 완화하거나 임의 같은 값 재결속을 허용하지 않는다. 원 route 부재·owner mismatch는 읽기 불가로 남기며 현재 route로 대체하지 않는다. 이 token/조립 경계는 2차 routing exact에서 확정하고 1차7만으로 완성됐다고 표시하지 않는다.

SDK의 native host/issuer generation은 guard와 원 owner의 권한 타입이다. skip registry generation/session/pass/expected_revision은 소비자가 기존 정책으로 검증한 provenance 타입이다. 서로 다른 타입과 생성·검증 책임을 유지하며 같은 숫자/문자열 비교로 대체하지 않는다. provenance는 검색·귀속 근거이고 native read/submit 권한을 발급하지 않는다.

필수 회귀에 정상 standalone None/장식 전 routed None의 구별, 장식 실패 뒤 metadata 열거 가능·read unavailable, 원 C8 double-bind 거절 유지, 원 route 부재·owner mismatch 거절, row/caller Clone의 동일 context 공유를 추가한다. 이 추가는 문서 검토 결과이며 코드·시험 작성·실행은0이다.

## 수명·예산·복구

원 rows→export→NativeSeekTicket→AdmissionRecord→private Ticket→Weak rows/Weak HostScope 구조만 허용한다. C8 ReadOwner도 Weak binding/router다. export record가 Registry/HostScope/engine/parent admission을 strong으로 담지 않는다. 조회 중에만 원 owner를 upgrade하며 idle export는 프로세스/host 수명을 연장하지 않는다. 원 row lock을 잡은 채 ticket.read/resolver 재진입을 호출하지 않는다. 조회용 Clone을 꺼낸 후 잠금을 풀고 원 read 경계를 통과한다.

원 grant+command 합계64와 id/slot1:1을 유지한다. export는 기존 command row의 optional 필드이며 별도 map·추가64slot·무제한 tombstone은 없다. 기존8KiB row payload 안에서 provenance/reference/context 고정 metadata와 오류 prefix 몫을 나눠야 한다. 현재 PREFIX_BYTES=8192−512는 추가 metadata 비용을 반영하지 않았으므로 그대로 더하지 않는다. 구현 때 size_of/동적 문자열 cap/공유 Arc 보존 그래프와 JSON worst-case escaping을 Rust로 검증해 고정 상한을 정한다. 지금 숫자 상한을 실행 검증한 것으로 쓰지 않는다. 오류의 원 bytes/hash/truncated를 유지하며 목록은 최대64 reference/요약만 반환하고 오류 본문을 복제하지 않는다. 목록 응답 별도 byte cap은 wire 담당과 확정한다.

submit 응답 유실은 같은 살아 있는 host와 검증된 원 provenance의 export row 열거/peek로 복구하며 seek 재제출0이다. 현재 skip revision/Active 교체는 원 결과의 귀속을 바꾸지 않는다. pending consume은 NeedsInput과 원 row/권리 보존, terminal consume은 원 row mutex에서1회 제거 후 NotFound다. 읽기 Clone이 남아도 소비된 row를 재생성하지 않는다. consume 응답 유실은 at-most-once 해제이며 exactly-once delivery/ACK를 주장하지 않는다. EOF·revoke는 원 close/write 경계를 유지하고 owner 만료 후 SourceExpired, restart 복원0이다. foreign 식별 오류와 원 row 부재 오류의 최종 코드/정보 노출 순서는 공식 resolver 구현 시 기존 issuer 검증과 함께 고정한다.

## 다음 확인 순서

PM 단계 결정과 source-only 진행: C8·일반 엔진의 신규 export API는 기본 Unsupported를 admission 전에 반환하며 legacy submit으로 fallback하지 않는다. 실제 standalone Engine/Sessions 연결과 C8 사전 결속은 각각2차다. 1차 정확7은 독립 b89682a staging에서 작성·동결했으며 인계는 `runtime/coordination/sdk-row-export-r1/handoff.md`, source7 `DC3025DEA114DCA92DCD7CA57DEEA479BA573121279984FD0B8B44917010ED5B`다. 작성회귀10/direct rustfmt·diffcheck0이며 Cargo/typecheck/실제시험/Clippy0다. 이번 구현은 read availability를 Unavailable로 유지하므로 export wire/생산읽기 완료가 아니다.

1. PM·skip 담당이 1차7과 2차 routing/read API 분리를 대조한다. 특히 장식 실패 후 read-unavailable/사전 결속 선택을 먼저 확정한다.
2. 코드 배정 뒤 원 cell1회/Clone 장식 공유·enqueue 전 준비·full64/try_send 실패·post-admission 후처리 오류 보존을 Rust 회귀로 작성한다.
3. Weak cycle/Drop/owner 만료·foreign/restart·원 revision 변경 후 조회·동시 terminal consume1회·pending 권리 보존을 검증한다.
4. 원8KiB/escaping/목록 cap을 실측하고 fixed runner의 별도 배정 gate에서 fmt/test/양 strict를 실행한다. 이번 문서는 시험 작성/실행0이다.

미인수 구현을 되돌릴 때 기존 submit/false 지원 상태를 유지하고 export 연결만 제거한다. 이미 접수한 seek를 취소·재전송하거나 원 실패 증거를 지우지 않는다.
