---
id: sot:kinekino/typed-api
title: typed API와 오류·복구·소비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# typed API와 오류·복구·소비

상태·인수 한계는 [C0 index](index.md)를 따른다. 아래는 실제 코드가 아닌 Rust 서명 계약이다.

```rust
fn native_export_issuer(&self) -> Result<NativeExportIssuer, ApiError>;
fn observe_native_guard_for_export(&self, issuer: &NativeExportIssuer)
    -> Result<NativeGuardObservation, ApiError>;
fn submit_exported_guarded_seek_v1(&self, input: ExportGuardedSeekV1)
    -> Result<ExportedGuardedSeekV1, ApiError>;
// NativeExportIssuer가 원 scope를 고정하므로 current engine/route를 선택하지 않는다.
fn list(&self, origin: &NativeExportOrigin) -> Result<NativeExportListV1, ApiError>;
fn peek(&self, reference: NativeResultRefV1, origin: &NativeExportOrigin)
    -> Result<NativeExportResultV1, ApiError>;
fn consume(&self, reference: NativeResultRefV1, origin: &NativeExportOrigin)
    -> Result<NativeExportResultV1, ApiError>;
```

앞3은 Engine 기본 Unsupported·동일 Sessions 전달, 뒤3은 고정 issuer handle의 read-only SDK facade다. router의 issuer 획득은 별도 trusted `original_export_issuer(&ParentRouteHandle)`로만 연결한다. C8 submit은 기존 B move-only read-owner 준비 token을 v1에 전달하고 postbind0을 유지한다. 원 result row가 보존한 Weak owner mode를 사용하며 row lock을 풀고 원 read/consume를 실행한다. 새 guard/control fallback0이다.

공식 명령4와 strict payload는 기존 wire 제안의 역할을 유지한다. submit만 context.expected_revision 양수 decimal 필수다. list/peek/consume은 expected_revision·idempotency_key·operation timeout을 Unsupported로 거절한다. consume를 skip 정책 mutation 묶음에 넣지 않는다. origin은 원 SessionRef/full binding·pass/request_revision/segment이고 server registry generation을 결속한다. read는 현재 session entry/revision을 조회해 원 origin을 재구성하지 않는다.

## 고정 오류 순서

P0 facade의 source 반환 뒤 scope/origin/ref 일치 검사는 추가 불변식 검사다. 이것만으로 consume 권한을 보장하거나 이미 제거한 row를 복구하지 못한다. 실제 P1 source는 원 scope·row·origin을 **삭제 전에** 검증해야 한다. 원 row의 origin/metadata는 불변이고 id는 재사용하지 않으므로 lock 안에서 record/selector를 확인해 clone→unlock→원 owner read→동일 원 ticket의 terminal consume 순서를 유지할 수 있다. 다른 row를 지운 뒤 상위 facade가 mismatch를 발견하는 구현은 인수하지 않는다.

1. 기존 transport/Request/api_version/request_id 검증.
2. 명령별 strict shape/숫자/길이와 **SDK 공식 codec**: unknown field·overflow·non-v1/noncanonical는 InvalidArgument. typed decode가 context보다 먼저다.
3. 명령별 context 분류: submit revision 오류 InvalidArgument, read/consume revision 및 미지원 idempotency/timeout은 Unsupported.
4. 해당 기능의 주입/구현이 없으면 Unsupported. decode가 유효한 미지원 기능을 기존 submit으로 fallback0.
5. 고정 handle/payload scope/ref embedded scope mismatch는 RevisionConflict. 그 뒤 원 Weak issuer upgrade 실패는 SourceExpired. row lookup 전에 검사한다.
6. submit만 원 session/binding/revision·선택 상태·core evaluate/예약 및 expected issuer pre-observe 검사를 수행한다. 원 오류를 무조건 NeedsInput/retryable로 재분류하지 않는다.
7. list는 원 issuer의 complete v1 export row만 원 origin으로 필터하고 일치0이면 빈 목록. peek/consume의 row 부재·이미 소비·legacy non-v1 row는 NotFound. 존재 row의 origin mismatch는 RevisionConflict.
8. 원 row의 read mode Unavailable은 Unsupported, 원 binding/router Weak 만료는 SourceExpired. **유효한 원 read owner에서** pending consume은 NeedsInput/권리 불변, terminal consume은 mutex 안에서1회 제거·이후 NotFound다. foreign/expired owner보다 pending 상태를 먼저 노출하지 않는다.

list는 최대64 원 row의 bounded metadata/state/read_mode 요약이다. read_mode는 `standalone`/`routed`/`unavailable`의 결속 형태이며 다음 read 성공·owner 생존 보장이 아니다. 현재 route 선택·독립 application 결과 map·추가 pending64/terminal64·임의 eviction0이다. list/peek 사이 경쟁을 영구 snapshot으로 보장하지 않는다.

submit typed 결과는 not_submitted/cancelled/admitted/admitted_with_error/no_submission_returned로 분리한다. 실제 SDK Ok로 얻은 원 ref/receipt를 core 오류 때문에 버리지 않는다. no_submission_returned를 모든 SDK Err=미접수/seek0으로 일반화하지 않는다. outer ok는 typed 결과 반환이며 native write/ACK/landing 성공이 아니다.

submit 응답 유실은 같은 살아 있는 endpoint에 미리 얻은 scope+원 origin으로 list/peek해 복구하며 재submit/추가 seek0이다. list/peek 응답 유실은 소비0. terminal consume 응답 유실 뒤 NotFound는 전달 ACK가 아니며 at-most-once 해제다. exactly-once delivery·무제한 tombstone·자동 재submit0. 복구되는 native 결과와 저장하지 않은 원 core 응답 전체를 구분한다.

EOF는 기존 admission close→in-flight/worker/child 정리→owner 만료 순서다. idle ref/issuer/export는 원 engine/process를 강하게 연장하지 않는다. revoke/교체 뒤 원 owner가 살아 있으면 원 read를 유지하며 현재 route로 retarget하지 않는다. host restart의 다른 nonce는 RevisionConflict이며 원 세션/결과를 자동 복원하지 않는다. endpoint 종료는 transport 오류로 남긴다.
