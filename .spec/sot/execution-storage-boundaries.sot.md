---
id: sot:kinekino/execution-storage-boundaries
title: 실행 수명과 저장 경계의 통합 기준
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 실행 수명과 저장 경계의 통합 기준

2026-10-26 PM 정규화, 기능 소유 SDK. 기존 ExecutionOwner/Job/Store 계약을 연결하며 독자 scheduler/새 전역 registry를 도입하지 않는다. 이번 source 검토는 새 제품 실행 인수가 아니다.

## 독립 수명과 완료

| 단위 | identity/owner | 완료 기준 | 다른 단위와 혼동 금지 |
|---|---|---|---|
| Request/Response | RequestContext request_id/api_version, 동일 envelope | ok/data/error·correlation·버전 검증 | 응답 생성/전송≠영속 저장/엔진 적용 |
| SDK Operation | opaque OperationId(owner Arc identity+sequence), ExecutionOwner/Handle | Queued→Running→Terminal→Acknowledged, worker join 별도 | clone handle≠owner 이관, ACK≠Job/미디어 종료 |
| Job/attempt | durable JobId/revision/input snapshot/expected_attempt | operation 실행→검증→Completed, cancel은 worker 관측 | transport/GUI 종료≠Cancelled, 재시작 Interrupted≠자동 재실행 |
| Native engine/transition | 원 engine_instance_id/session/generation·retire ticket | 원 관측/ACK/정리 및 해당 transition commit/release | process spawn·resolve complete·seek accepted≠재생/프레임 성공 |
| Project/proposal/draft | 각 immutable revision·content hash·원 capture/expected CAS | 저장소의 원자 publication과 readback/reopen | 화면이 닫힘/토스트≠저장, 기술 accept≠미디어 적용/사용자 품질 |
| Transport | 한 endpoint·request·전달 상태 | terminal response decode 또는 명시 오류 | EOF/timeout≠업무 성공/취소/미실행 |
| Document/plugin | document epoch·binding generation·plugin instance | dispatch 차단/원 listener·handle 정리 | unmount/HMR≠job cancel, stale 응답 새문서 투영 금지 |

## 취소·닫기·기한

소스 sdk_execution/control.rs에서 queued cancel은 execution_started=false인 Cancelled terminal을 만든다. Running/Terminal cancel은 cancellation_requested를 기록하고 원 response를 보존한다. Requested를 강한 in-flight 취소 성공으로 표시하지 않는다. terminal은 소비자가 validate_response 후 명시 ACK하고, close는 TerminalAckPending/WorkerExitPending/Faulted/Closed를 보존한다. loop_finished와 Joined를 구분하며 incomplete owner를 성공으로 버리지 않는다.

mutating usecase는 publish 이전 취소 관측 시 no-publication, publication 이후에는 성공 결과 또는 publication_may_exist/실행불확실을 반환한다. 모든 기존 usecase가 이 취소 port를 이미 제공한다는 뜻은 아니다. 동기 export의 미지원 취소를 UI에서 구현됐다고 표시하지 않는다. deadline은 admission/transport/task/publish 단계마다 별도이며 한 context timeout으로 합치지 않는다.

전송 전 실패는 delivery=not_sent, 전송 후 응답 불확실은 delivery=execution_unknown/automatic_retry=false라는 기존 transport 진단을 보존한다. 지원되는 exact request/idempotency/decision ID로만 재조회한다. 모호한 작업을 새 키로 자동 재시도하지 않는다. 재조회 계약이 없는 명령은 해당 원본 결과/저장 상태 확인 절차를 제공하고 확정 성공·실패로 추정하지 않는다. source289 timeout은 기존 미확정 원인으로 남긴다.

## 저장소별 정책

| 저장소 | 불변 원본/변경값 | 경쟁/재시도 | 미완료 인수 |
|---|---|---|---|
| ReviewStore | package/objects hash, immutable sample decisions | session이 package/assets와 root guard 유지; decision identity 충돌 판정 | 실제 사용자 GUI verdict·원9 무변경, 손상/권한/재열기 |
| ProposalStore | source/generated/historical binding 불변, draft/output revision | publish expected CAS, guarded capture session, exact accept 재요청 | 모든 task 기본 경로·미디어 적용은 별도 |
| Output review | fixed accepted target·criteria/evidence·immutable decision | 원 writer session+decision_id, current output revision | 불완전 publication·history current_evidence/복구 구분 |
| ProjectStore | project revision·capture 원본/locator | load/publish CAS, 저장 context key/timeout 미지원 | 설정/schema migration·모든 source reconnect/export 지원 별도 |
| Jobs store | prepared input·events/attempt/output receipt | submit key/retry revision/cancel attempt | 각 operation durable executor·재시작·부분실패 |
| User association/range draft | actual9 바깥 사용자 namespace, association/draft 독립 revision | [계약](api/review-source-draft-contract.md)의 transaction+replay | 격리 mock와 실제 filesystem/process 중단 분리 |

## filesystem publication의 한계

기존 infrastructure/proposals/publication.rs는 reservation/revision dir 아래 불완전 publication을 남겨 후속 읽기를 차단하고 복구하게 한다. output_reviews/publication.rs는 Reserved/Published/readback 단계를 보존한다. media-file-publication은 sync_all 후 같은 디렉터리 temp의 no-overwrite hard-link publication을 사용한다. 이 구현을 공통 multi-file transaction 또는 전원 장애 내구성 보장으로 확대하지 않는다.

새 durable adapter는 원 rooted-directory/file guard·writer reservation·bounded strict decode·content identity·no-overwrite primitive를 재사용한다. 서로 다른 파일 두 번 쓰기를 단일 transaction이라고 부르지 않는다. association+draft+replay처럼 같은 commit이 필요한 값은 단일 불변 record envelope에 묶는다. 예약 후 중단·부분파일·readback 실패를 성공 cache나 빈 저장소로 취급하지 않고 preserve_and_inspect/NeedsInput으로 복구한다. 완료 산출물·원 증거를 cancel/복구에서 삭제하지 않는다.

## 공통 인수 행렬

EX01 성공 terminal+ACK+join, EX02 queued cancel no execute, EX03 running cancel requested와 원 결과, EX04 foreign/stale owner·revision 거절, EX05 notifier/worker/close 실패와 원 소유 보존, EX06 transport 전후 단절, ST01 CAS 경쟁, ST02 exact key replay/changed payload conflict, ST03 publish 직전 cancel, ST04 publish 뒤 ACK 유실/reopen, ST05 프로세스 중단·예약/부분파일·손상, ST06 disk full/permission/path escape·no overwrite, ST07 동일 request GUI/CLI 저장 결과. 각 기능에서 적용/N/A 이유·실제 port/저장소·근거 종류를 기록한다. helper/mock 통과를 실제 디스크·원 HWND/mpv·전체 descendant 정리로 계상하지 않는다.

연결: [owned SDK](api/sdk-execution.md), [Jobs](jobs/state-transitions.md), [host 표면](api/host-support-boundaries.md), [설계 공백](../../../plans/player/design-completion/gaps.md).
