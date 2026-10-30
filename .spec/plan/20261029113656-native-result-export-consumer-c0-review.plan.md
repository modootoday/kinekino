---
id: plan:kinekino/20261029113656
title: C0 소비 계약 읽기와 다음 최소 배정안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C0 소비 계약 읽기와 다음 최소 배정안

최신 상태: PM 배정 exact9의 r2 소비 코드를 독립 overlay에서 구현하고 중앙 gate의 실제 고유55·양strict/fmt·보호/반환을 인수했다. [검증 정본](../../sots/verification/application/native-export-consumer-2026-10-21.md)을 따른다. wire 지원false·실제producer/전체출력/CLI/P1/P2는 잔여다. 아래 최초 후보/계약 읽기는 이력으로 보존한다.

2026-10-21. PM의 root626dca0 exact8 채택 후 문서만 준비했다. [공식 C0 index](../../sots/contracts/playback/native-result-export/index.md)와 codec/metadata/lifecycle/projection 본문, [SDK 단계 계획](native-result-export-c0-producer.md)을 읽었다. 공식 C0가 기존 wire 제안의128/256bytes·오류 순서 후보보다 우선한다. SDK P0 인수 전 소비 소스/Cargo/shared/root/index 수정0이다.

## 소비자가 그대로 사용할 계약

### P0 source-only 핀 정합 읽기

SDK 인계 `runtime/coordination/sdk-native-export-c0-p0-r1/source5.json` SHA256 **EE8B1724C873A8F9D8A2AC44178CD81C1341A878DD9035CAF7C7A1B8434A90B0**와 source/ 정확5의 bytes/hash를 직접 대조해 일치했다. handoff와 공개 타입/메서드 본문을 읽었다. 이것은 소비 연결 계획 정합 확인이며 독립 전체 감사·typecheck·작성15 실행 인수가 아니다.

소비 import 기준은 `application::playback_session::v1`이다. NativeIssuerScopeV1/NativeResultRefV1은 공식 parser, NativeExportOriginInputV1→NativeExportOriginV1 및 NativeExportAnnotationInputV1→NativeExportAnnotationV1은 검증된 값 변환을 사용한다. OriginInput의 server registry generation/full binding은 host 원 값에서 조립하며 클라이언트가 권한을 발급하는 입력으로 쓰지 않는다. SDK 공개 값 DTO가 있다는 이유로 host strict payload에 같은 모든 필드를 무조건 노출하지 않는다.

crate 내부 소비자는 `PreparedNativeResultExportV1::new(origin, annotation)`을 사용하고 actual target을 넣지 않는다. `ExportGuardedSeekV1 { seek, export }`와 Sessions의 `native_export_issuer`, `observe_native_guard_for_export(&issuer)`, `submit_exported_guarded_seek_v1`을 연결한다. `ExportedGuardedSeekV1::into_parts()`의 원 submission/reference/metadata를 보존해 기존 core 후처리 오류로 ref를 잃지 않는다. 동일 이름의 소비자 대체 타입/codec/ref 문자열을 만들지 않는다.

고정 NativeExportIssuer의 scope/list/peek/consume, SDK NativeExportItem/List/Failure/CoreError/ResultV1과 `encode_native_export_v1`을 그대로 소비한다. NativeExportCoreErrorV1::from_api를 사용하고 임의 details/remediation을 소비자가 다시 투영하지 않는다. encoder는 bounded compact JSON entry이며 pretty/framing/final envelope와 CLI writer는 아직 공유 통합 잔여다. generic Serialize encoder에는 유한 공식 DTO/유한 host envelope만 전달해야 한다.

P0는 실제 GUID/row resolver/Weak adapter/actual target stamp와 P1/P2/wire CLI가 없다. Engine 기본 Unsupported를 legacy fallback으로 우회하지 않고 지원false를 유지한다. retained/최대값 회귀15는 작성 상태이며98299/98304/item1024×64를 실제 검증 완료라고 기록하지 않는다.

- scope/ref는 SDK strict v1 codec의37/54 ASCIIbytes다. 소비 DTO가 임의 문자열 생성·정규화·대소문자 보정·Debug id 추출을 하지 않는다. typed shape와 codec InvalidArgument를 context 검사보다 먼저 처리한다.
- 주입되는 opaque OriginalIssuer Weak handle을 고정한다. scope/ref 문자열은 lookup 값이며 권한이 아니다. router는 신뢰된 ParentRouteHandle로 캡처한 원 handle만 소비한다. 현재 route나 현재 Active를 조회해 과거 결과를 retarget하지 않는다.
- 제출만 expected_revision 양수 decimal을 요구한다. read/list/consume의 revision·idempotency·operation timeout은 Unsupported다. 원 오류순서와 code/retryable을 재분류하지 않는다.
- origin의 request revision, 선택 annotation revision/verified revision/timebase, SDK actual target은 서로 다른 값이다. 실제 target은 원 typed SubmitInput/row에 SDK가 stamp하며 소비자가 JSON·나중 segment 조회·position0 placeholder로 만들지 않는다.
- 원 SDK Ok ref/receipt를 core 후처리 오류 전에 보존한다. admitted_with_error와 no_submission_returned를 구분하고 후자를 미접수/seek0으로 일반화하지 않는다. list의 복구 범위는 원 native 결과이며 저장하지 않은 core 응답 전체가 아니다.
- session_state는 session/revision/pass만 출력한다. 전체 SkipSnapshot/segments Vec·ApiError.details/remediation을 신규 결과에 복사하지 않는다. bounded 원 error projection과 omitted 표시를 SDK 계약대로 사용한다.
- row/read/consume는 고정 issuer facade를 사용하고 application 결과 map·독자 pending/terminal64·eviction/tombstone을 만들지 않는다. pending consume은 NeedsInput/불변, terminal consume 응답 유실은 at-most-once이며 전달 ACK가 아니다.

## 선행 소비 exact9 후보 — 편집 권한 아님

| 경로 | 최소 책임 |
|---|---|
| src/application/skip_segments/native_port.rs | additive v1 호출 상태·원 issuer pre-observe·원 submission/ref 보존 |
| src/application/skip_segments/host/registry.rs | 같은 registry/session/binding/revision 잠금 범위와 원 origin·유한 상태 조립 |
| src/application/skip_segments/host/contracts.rs | strict 신규4 DTO와 tagged 결과, 기존 DTO literal 호환 |
| src/application/skip_segments/host/dispatch.rs | 정본 오류순서·기존 core reuse·SDK facade 전달 |
| src/application/skip_segments/host/schema.rs | 추가4 strict schema/지원 상태 계약, 기존 segments.skip 보존 |
| src/application/skip_segments/host/mod.rs | additive capability/원 issuer 주입 경계 |
| src/application/api/client/skip_segments.rs | 같은 공식 DTO/command 이름의 typed client |
| src/application/skip_segments/service.rs | bounded 선택 metadata와 최소 상태 getter만, 기존 정책 변경0 |
| tests/native_skip_cli.rs | Rust 소비 회귀, 실제 CLI 통합 배정 전 지원 인수 아님 |

기존 소비10에서 CLI 조립2를 제외한8+getter1=9다. SDK P0 계약5와 교집합0을 유지하며 새 타입/메서드의 실제 명칭·가시성은 P0 pin을 인수한 뒤 맞춘다. 현재 문서에서 독자 대체 SDK 타입을 선언하거나 공식 서명으로 확정하지 않는다. 추가 파일이 필요하면 exact를 PM에 먼저 보고한다.

## getter의 동시성·거절 순서

직접 읽은 service.rs::inspect는 segments 전체 Vec를 clone한다. skip은 같은 &mut self에서 evaluate→revision/attempted 예약→원 port submit을 수행한다. export metadata 때문에 inspect를 먼저/나중 호출해 큰 snapshot을 두 번 만드는 방식은 피한다.

getter는 같은 registry/session 소유 잠금과 같은 요청의 revision/binding 아래 최소 state와 선택1개 scalar metadata만 반환해야 한다. 원 선택이 없거나 미검증인 경우에도 metadata 준비가 기존 evaluate/cancel의 거절 순서를 앞질러 새 오류를 만들면 안 된다. 선택 metadata의 부재와 실제 v1 submit 준비 실패를 분리하며, annotation 형상 검증은 정본대로 실제 제출 경로에서 한다. core evaluate/예약/취소/override를 getter나 host에 복제하지 않는다. submit 뒤 상태는 갱신된 revision/pass만 읽고 과거 metadata를 재조회해 재구성하지 않는다.

P0와 조율할 점은 bounded getter의 실제 반환형을 어느 기존 모듈에 둘지, 선택 식별자/metadata Optional 상태가 not_submitted/cancelled와 어떻게 대응하는지, 원 application 오류의 bounded projection을 소비자가 직접 재구현하지 않고 호출할 공식 entry다. 이는 설계 연결 항목이며 현재 SDK 결함 판정이 아니다.

## 마지막 통합과 출력 경계

P0 핀을 읽으며 추가 확인한 기존 clone 경계: host registry의 `with_session`도 binding 비교에 `entry.session.inspect().binding`을 사용해 전체 segments를 clone한다. 따라서 export handler에서 결과 snapshot만 빼는 것으로 유한 경로가 되지 않는다. 이미 후보에 포함한 service.rs의 bounded binding/state getter를 같은 with_session 비교에 재사용하는 연결을 후보 registry.rs 책임으로 둔다. binding 비교/오류순서/current_required 의미는 유지하고 getter 안에 core 정책을 복제하지 않는다. 새 파일 범위 확대는 필요하지 않지만 실제 구현은 별도 PM 배정 뒤다.

CLI native_host/playback_host 조립2는 생산/소비 핀 인수 뒤 마지막에 결합한다. capabilities.rs/api mod/local_host mod/server/client/CLI runner 공유6은 SDK·통합 담당을 PM이 지정한다. host 문자열 목록만 등록하고 전역 schema/capabilities/client에 빠뜨리거나 소비자가 SDK 소유 파일을 함께 수정하지 않는다.

신규4의 JSON envelope98299/전체98304, list item1024×원64, error prefix1024는 설계 목표이며 실제 출력 크기 통과가 아니다. compact/pretty·request_id128 Unicode chars·escaping·최대 decimal/식별자·fallback·header/newline을 같은 bounded writer로 Rust 실측한다. DTO→무제한 Value→truncate 순서를 쓰지 않는다. LocalHost client도 decode/allocation 전 명령별 cap을 적용해야 최종 지원 인수가 가능하다. consume 뒤 출력 실패를 row 항상 보존이라 표시하지 않는다.

최소 회귀는 원 stale/closed/cancel/override 순서, 같은 원 annotation/actual target, post-admit core error ref 보존, current-route retarget0, pending/terminal consume·응답 유실 복구/추가 seek0, strict DTO/context/foreign/expired 우선순위, 최대 출력과 지원false/전역 등록 일치다. 실제 CLI는 같은 최종 pin/원 endpoint에서 별도 배정한다. fake Native 결과와 실제 mpv/ACK/landing은 분리한다.

## 검증도구 다음 최소 범위 후보

same-endpoint exact8은 root626dca0 채택·최종 고유32로 닫았으나 legacy13의 blocking Host/NativeOwner가 남아 있다. 가장 작은 다음 후보는 기존 Fixture finite 요청을 하나의 no-media 소비자에 additive 연결하는 것이다. broad13 기계 이관·실제3category 실행을 동시에 묶지 않는다. 실제 대상과 필요한 경로 수는 PM 우선순위 배정 뒤 원 caller/오류 보존을 읽고 고정한다. 지금 새 source/test/build/OS 실행0이다.

실제3category/37요청은 아직 ignored 종단 잔여이며 CLI wiring·pin/환경·mpv/브라우저·중앙 media slot을 별도로 인수해야 한다. 현재 no-media EOF 성공을 실제 광고차단/재생 성공으로 확대하지 않는다.

적용 근거: 기존에 본문을 읽은 modoo.today plan/plan-authoring 및 code-change-review-verification/verdict-table의 책임·authority·관측/제안/미검증 분리 원칙을 재사용했다. 원 C0 수치/서명과 실제 구현·검증을 구별하고 PM 배정 전 소스를 편집하지 않았다.
