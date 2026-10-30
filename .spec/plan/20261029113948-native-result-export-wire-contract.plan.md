---
id: plan:kinekino/20261029113948
title: Native result export C · additive CLI wire 계약안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native result export C · additive CLI wire 계약안

2026-10-21 C0 후속: 공식 [v1 책임별 계약 정본](../../sots/contracts/playback/native-result-export/index.md)이 형식/오류순서/예산에서 아래 제안보다 우선한다. scope/ref는 엄격한 v1의37/54 ASCIIbytes, JSON envelope98299/전체 wire98304bytes다. 원 issuer는 미리 캡처한 고정 opaque Weak handle이며 router current route로 query를 대신하지 않는다. 수치는 구현 시 Rust max-bound 검증할 설계 계약이고 실제 codec/CLI 지원 통과가 아니다. 기존 결과 tag·origin/actual target·복구 요구는 유지한다.

상태: active 제안, 2026-10-21. [phase2 C](native-result-export-phase2.md)의 상세 wire 책임만 구체화한다. SDK A9 구현/원 row/cell 인수와 B의 read-owner 완료 조건이 선행하며 C 코드·시험·Cargo·공유 변경0이다. source5 ED3A961B…75C692와 기존 segments.skip/ActionResult는 변경하지 않는다. 새 명령명·수치 cap은 검토용 후보이며 지원 true를 승인한 문서가 아니다.

SDK 직접 조율: A9는 trusted standalone issuer/원 read-cell resolver까지이며 현재 wire codec·새 metadata 필드는0이다. SDK는 session revision과 annotation/verified revision·실제 target/timebase 분리, 원 row 별도 bounded metadata·SDK submit target 일치·byte charge 필요에 동의했다. scope/ref128/256·전체96KiB/item1KiB는 공식 codec 확정 전 승계하지 않는 제안이다. metadata 필드/codec/원 target 검증의 production exact는 PM이 별도 배정하며 현재 A/원 row7 입력에 혼합하지 않는다. admitted_with_error와 no_submission_returned의 한계도 합의한 설계 경계이며 실제 지원/실행 인수가 아니다.

## Outcome — 목표

같은 명시 endpoint에서 fresh CLI가 명시 skip export를 제출하고, SDK 원64 row 안에 보존된 원 결과를 list/peek/consume한다. submit 응답을 잃어도 exported row와 원 provenance로 복구하고 추가 seek를 만들지 않는다. 접수·core 후처리 오류·native queue/write/ACK·별도 landing을 구분하며 guard나 native ticket을 JSON으로 직렬화하지 않는다.

## Authority — 결정 경계

| 관심사 | 소유와 선행 계약 |
| --- | --- |
| native issuer scope/ref codec·row/provenance·read/consume | SDK. 같은 원 row/cell과 read-owner mode를 검증하며 별도 application id/map을 만들지 않음 |
| skip provenance/명시 target | skip registry/core. session revision은 기존 context 권한, annotation/verified revision과 실제 target은 원 상태/원 평가에서 별도 기록 |
| strict DTO/client/schema/등록·출력 projection | C 담당의 후속 exact 배정. 기존 mutation/read 일괄 분류와 분리 |
| standalone/routed 지원·CLI flag | PM/SDK의 단계별 실제 인수. A만 인수된 동안 C8 routed export를 지원으로 표시하지 않음 |
| 자원/실행 | 중앙 단일 배정. 본 문서는 새 소스/실행 권한이 아님 |

## Evidence — 읽은 현재 계약

현재 skip dispatch는 모든 기존 skip 명령의 idempotency_key/timeout_ms를 거절하고 mutation에 context.expected_revision을 요구한다. 기존 segments.skip은 ActionResult와 원 ControlReceipt만 반환한다. read-only NativeResultReference는 현재 Rust opaque ticket/provenance일 뿐 공식 wire codec/독립 serializable scope가 없다. producer A가 만든 cell을 두 번째 submission이나 request_id/receipt로 복원하지 않는다.

기존 SkipSession::edit는 Vec에 segment를 넣고 inspect는 이를 clone하며 개수 상한을 두지 않는다. 따라서 신규 응답에 전체 NativeSkipOutcome.snapshot/SkipSnapshot을 그대로 포함해 고정 전체 출력 cap을 보장할 수 없다. C는 최소 상태 projection을 쓰며 기존 segment 정책/inspect DTO를 몰래 제한하지 않는다. 이 읽기에는 실제 wire 실행/출력 크기 측정이 없다.

## Proposed commands — 검토할 additive 이름

| 명령 후보 | strict payload | RequestContext.expected_revision |
| --- | --- | --- |
| `segments.skip_export` | issuer_scope, session:기존 SessionRef, input:기존 SkipInput | 기존 session revision의 양수 decimal 필수 |
| `segments.export_list` | issuer_scope, origin:ExportOrigin | null/부재만 허용 |
| `segments.export_peek` | issuer_scope, reference, origin:ExportOrigin | null/부재만 허용 |
| `segments.export_consume` | issuer_scope, reference, origin:ExportOrigin | null/부재만 허용 |

각 DTO는 Rust deny_unknown_fields와 JSON additionalProperties=false를 대응시킨다. API version/request_id의 기존 검증을 재사용한다. 신규4도 idempotency_key/operation timeout_ms는 부재/null만 허용하며 transport deadline은 별도다. consume는 row 해제이지 skip 정책 mutation이므로 기존 expected_revision mutation 묶음에 넣지 않는다. 명시 cancel은 기존 SkipInput.cancelled 의미를 사용하고 재생 seek 철회로 바꾸지 않는다.

issuer_scope와 reference는 **SDK 공식 codec의 bounded read-only 값**이다. 임시 계약 목표는 각각 ASCII128/256bytes 이하이며 형식/버전/체크는 SDK가 소유한다. 현재 구현된 문자열 형식이 있다는 뜻이 아니다. scope는 submit 전에 capabilities 등 같은 endpoint의 공식 Rust 결과에서 얻을 수 있어야 하며 응답 유실 뒤에도 원 issuer를 지정할 수 있어야 한다. SDK가 다른 bound/형식을 필요로 하면 이 계약과 크기 시험을 같이 갱신한다. application이 주소/Debug.id/독자 난수 command ID를 만든 뒤 이를 SDK 참조라고 부르지 않는다.

ExportOrigin은 `{session: SessionRef, pass_id: DecimalU64, request_revision: DecimalU64, segment_id: String}`의 원 요청 근거다. request_revision은 submit 당시 context.expected_revision이며 **현재 session revision 요구가 아니다**. registry_generation은 server의 고정 원 registry 값으로 결속하며 client가 session id 문자열을 파싱해 발급하지 않는다. 조회는 현재 session entry가 close/변경됐다는 이유만으로 원 row를 새 세션으로 복원하거나 거절하지 않는다. origin과 native issuer는 별도 검증 책임이다.

list는 같은 원 issuer/provenance와 일치하는 exported row의 최대64개 요약을 반환한다. 유효한 issuer/origin에서 일치 row가 없으면 빈 목록이다. 별도 cursor/히스토리/tombstone은 만들지 않는다. 목록은 bounded 현재 관측이며 원자적 영구 snapshot이나 소비 권리가 아니다. peek/consume의 row 부재는 NotFound다.

## Stored provenance — session/annotation/target 분리

원 row에 보존할 값은 native issuer/원 engine+resolution identity와, registry generation/session/pass/segment, request session revision, **원 annotation revision·verified revision·실제 target_us·engine_playback timebase**다. session revision을 annotation revision으로 복사하지 않는다. target은 caller JSON으로 받지 않고 기존 core가 선택해 원 SDK에 실제 제출한 값이다. 원 grant submit target과 export metadata target이 일치하도록 SDK 계약이 보장해야 한다.

producer r1 provenance에는 session expected_revision만 있고 annotation/target 필드가 없다. 따라서 실제 annotation/target을 내보내려면 A/C 경계에서 준비 DTO와 원 row metadata 예산을 명시 확장해야 한다. 현재 segment를 나중에 읽어 과거 값으로 재구성하는 대안은 금지한다. annotation 확인이 끝나지 않은 NotSubmitted/Cancelled에는 존재하지 않는 target을 만들지 않는다.

## Response DTO — 유한 projection과 원 오류 소유

신규 submit data는 모든 분기에서 작은 `session_state {session,revision,pass_id}`를 제공한다. 전체 segments snapshot은 포함하지 않는다. 원 core Result/실제 SDK 반환 submission의 조합에 따라 strict tagged union을 사용한다.

- `not_submitted`: 원 Decision, reference 없음/seek0.
- `cancelled`: 원 CancelStatus/갱신 revision, reference 없음. TooLateOrUnobserved는 철회 성공이 아님.
- `admitted`: 원 receipt·reference·저장된 provenance. receipt completion은 Unobserved이며 native 완료와 다름.
- `admitted_with_error`: 위 원 receipt/reference/provenance에 core 후처리 원 오류를 추가. 원 ticket을 버리고 outer error만 반환하지 않음.
- `no_submission_returned`: 원 오류·갱신 session_state·recovery origin. ticket을 SDK에서 실제 받지 못한 상태이며 **미접수/seek0을 일반 보장하지 않음**. 원 exported row가 있으면 list로 복구한다.

이 tagged action 결과의 outer ok=true는 정상적으로 typed 결과를 반환했다는 뜻이며 seek 성공이 아니다. strict decode/context/사전 owner 검증 실패는 기존 outer ApiError다. 서버 응답 encoding/transport 실패는 domain 결과와 별도이며 원 row를 지우지 않는다.

list item은 reference/원 provenance·identity/read availability/state 요약만 갖고 전체 오류 본문을 반복하지 않는다. peek/consume data는 같은 원 reference/provenance와 typed native 결과의 queue/write/ACK state·write_commit·bounded 원 failure를 제공한다. landing은 별도 관측 없이 필드 true를 만들지 않는다. failure는 SDK 원 code/stage·prefix/original_bytes/hash/truncated를 보존하는 공식 bounded projection을 사용한다. 임의 ApiError.details Value를 무제한 복사하지 않는다. 그 projection이 없으면 새 handler 안에서 오류 정책을 복제하지 말고 SDK 계약 공백으로 남긴다.

## Error order — 외부 입력에서 원 row까지

1. 기존 transport/Request/api_version/request_id 검증을 재사용한다.
2. 명령별 strict shape·숫자/문자열 한도를 검사한다. 잘못된 형식/unknown field는 InvalidArgument.
3. 명령별 context를 검사한다. submit의 missing/invalid session expected_revision은 InvalidArgument, list/peek/consume의 expected_revision 또는 지원하지 않는 idempotency/timeout은 Unsupported. 기존4외 명령의 오류 순서는 변경하지 않는다.
4. 해당 기능이 실제 주입/인수되지 않았으면 Unsupported. 모든 schema와 capability도 동일하다.
5. SDK 공식 scope/ref codec과 원 issuer/owner를 검증한다. foreign/mismatched issuer는 원 RevisionConflict, 식별된 원 Weak owner 만료는 SourceExpired. row lookup 전에 권한을 확인한다. endpoint가 사라진 transport 실패를 SourceExpired API 응답으로 꾸미지 않는다.
6. submit만 기존 session lookup/full binding/revision/annotation·core 평가/예약을 수행한다. stale/closed/busy·SDK guard/생성자 오류는 원 코드와 의미를 유지한다. 별도 무조건 retryable/NeedsInput 재분류0.
7. read는 현재 session revision/Active/새 route와 비교하지 않는다. 원 row의 provenance mismatch는 원 계약의 RevisionConflict, row 부재는 NotFound, read owner Unavailable은 Unsupported. pending consume은 NeedsInput이며 row/조회권 불변. 세부 foreign/expired/absent 우선순위는 SDK resolver의 인수된 순서와 일치시킨다.

## Entire output cap — 제안 수치와 실패 의미

신규4개 응답의 **UTF8 최종 envelope+JSON escaping+마지막 newline 전체를96KiB(98304bytes) 이하**로 제한하는 안이다. 기존 transport MAX_RESPONSE_BYTES4MiB보다 작은 명령별 계약이며 그 기존 전역 값을 바꾸지 않는다. list item은 직렬화 후1KiB 이하·최대64개, issuer/ref128/256bytes, 원 failure display prefix1KiB의6배 escaping까지 포함하는 목표다. 현재는 실행 검증 전 수치이며 SDK 공식 codec/provenance 확정 뒤 Rust max-bound 직렬화 시험으로 인수해야 한다.

전체 cap은 success뿐 아니라 error/data/null·원 request_id·metadata/요약까지 포함한다. 단순 message 길이·row count 검사만으로 통과라 하지 않는다. encode는 byte-bounded 출력 writer/버퍼를 사용하고 무제한 JSON Value를 먼저 만든 뒤 자르지 않는다. cap 초과는 bounded ResourceExhausted 응답과 원 row 보존으로 처리한다. 조용한 문자열/항목 누락은 없다. consume가 이미 row를 제거한 뒤 출력이 실패하면 아래 at-most-once 의미가 우선하며 '항상 보존'이라고 일반화하지 않는다.

기존 unbounded segments Vec/inspect clone의 내부 메모리까지 이 출력 cap이 제한하는 것은 아니다. 새 segment 개수 정책/삭제·인위적 truncate를 C에서 추가하지 않는다. 원 retained8KiB와 준비/row/clone transient peak는 SDK 메모리 계약으로 따로 검증한다.

## Recovery/lifetime — 실제 전달과 원 소비

submit 응답 유실 후 원 endpoint+issuer_scope+ExportOrigin으로 list/peek하며 재submit/추가 seek0이다. metadata 목록에 reference가 있어도 read availability가 없으면 원 read cap을 복구했다고 표시하지 않는다. peek/list 응답 유실은 row 해제0이다.

consume는 terminal 원 row를 단1회 해제한다. 그 응답을 잃으면 이후 NotFound일 수 있으며 이는 caller가 결과를 받았다는 ACK가 아니다. 무제한 tombstone/자동 resubmit/암묵 새 job은 없다. 원하는 caller는 먼저 peek를 보존한 뒤 consume할 수 있으나 exactly-once delivery를 보장하지 않는다.

EOF는 admission close→원 in-flight/worker/child 정리→scope 만료 순서를 유지한다. 원 ticket/export가 engine/process를 강하게 붙잡지 않는다. revoke/교체 뒤 아직 살아 있는 원 owner의 read는 원 계약대로 유지하고 current route로 retarget하지 않는다. host restart/foreign scope/ref는 거절하고 원 세션/결과를 자동 복원하지 않는다. C8가 인수되지 않았으면 routed 지원은 계속 Unsupported다.

## Phases/acceptance — 구현 배정 후 조건

1. SDK와 명령/codec·scope 획득/annotation·target 보존·오류 순서·96KiB/1KiB bound를 합의하고 기존 phase2 C의 exact 경로를 갱신한다. 현재14 경로는 후보이며 추가 source를 지금 편집하지 않는다.
2. Rust DTO/schema/typed client의 unknown fields·decimal overflow·context 분리·max string/escaping/큰 기존 snapshot의 최소 projection·전체 cap·원 error projection을 검증한다. old segments.skip/context/ActionResult 회귀를 유지한다.
3. same endpoint의 actual fresh CLI로 submit/list/peek/consume·native source same_request/원 receipt·원 annotation/target·구간 이후수정·current revision 불필요·foreign issuer/provenance/expired·pending consume/terminal 경쟁1회·full64·응답 쓰기 실패 복구·EOF/restart를 검증한다. 합성 native fixture와 실제 mpv/미디어는 별도다.
4. source pin/원 request-response-bytes/출력 cap·실행 파일 hash·원 handle/receipt/PID identity/미완료 owner·보호/명시 반환을 인계한다. 소스/실행 성공을 전체 제품 지원이나 사람 품질 승인으로 확대하지 않는다.

## Rollback/Unverified

새 export 연결만 닫고 기존 non-export submit/지원false를 유지한다. 이미 접수한 seek를 자동 철회/재전송하거나 원 row/실패 로그를 지우지 않는다. 이번 문서에는 코드·명령 등록·직렬화 크기 실측·fresh CLI·실제 mpv 실행이 없다. modoo.today 기존 plan/plan-authoring의 기존 계획 연결·Authority/관측/미검증/완료조건/복구 구분과 code-change-review-verification의 원 결과/오류 보존을 적용했다.
