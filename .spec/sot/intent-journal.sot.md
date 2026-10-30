---
id: sot:kinekino/intent-journal
title: 일괄 작업의 고정 입력과 intent journal
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 일괄 작업의 고정 입력과 intent journal

R125 [계약 검증](../../../verification/application/job-batch-journal-20261028.md)7시험·기본strict/fmt·baseline190/정상2bins·2371custody 완료. 초기생성/고정키/원오류/관측Job·완료산출물 보존의 synthetic 계약이며 실제 batch producer/파일store/CLI 인수는 아니다. 아래 진행 중은 최초 작성 시점이다.

저장 전 필수 연결: 현재 BatchDocument는 입력을 메모리에서 capture하는 내부 DTO/계약이며 실제 BatchStore 구현과 wire create는 없다. 실제 persistence 전에 bound executor의 원 normalization을 재사용해 지원 operation/strict payload·민감한 임시 입력 경계를 검증해야 한다. raw SubmitJob을 그대로 native journal로 쓰는 어댑터를 만들지 않는다. core에 normalized 값을 재전달할 때의 normalization idempotence와 descriptor 동일성도 실제 adapter별로 확인한다. 이 capture gate는 다음 orchestration 단계의 미완료 조건이다.

R124/R125 기반 구현. TI09의 성공 보존/실패만 재시도를 기존 Jobs 위에서 제공한다. 기존 worker/state/retry gate를 바꾸거나 두 번째 executor를 만들지 않는다. 원 Jobs는 normalized request/key를 조회한 뒤 새 prepare를 하며 retry는 revision CAS·original retained attempt·publication_started를 검사한다. batch는 이 계약을 재사용해야 한다.

domain BatchPlan은 trusted Jobs/store binding의 scope SHA·batch ID·순서 있는 item ID/request SHA를 고정한다.1..128개/고유ASCII ID/소문자SHA를 검증한다. 제출 키는 versioned namespace+scope+batch+item+고정 request SHA의 hash로 결정되며 재열기/응답 유실 뒤에도 같다. 이 request SHA는 capture한 typed SubmitJob의 fingerprint이며 Jobs의 normalized request SHA나 실제 source blob/media SHA를 대체하지 않는다. wire에서 scope를 선택하게 하지 않는다.

application BatchDocument는 원 SubmitJob과 Waiting/SubmitPending/SubmitError/Observed/RetryPending을 저장한다. JobRecord는 기존 검증기를 통과한 관측이며 worker 상태의 새 정본이 아니다. SubmitError는 admission이 발생하지 않았다는 보장이 아니다. 같은 intent/key로 원 Jobs를 조회/재제출해 해소하며 새 키를 임의 생성하지 않는다. RetryPending은 원 job/attempt/revision/input을 보존하고 성공 Job은 재시도 대상으로 바꿀 수 없다. publication_started의 재시도 허용을 batch가 새로 판단하지 않는다.

첫 생성은 validate_initial(scope일치/revision1/모두Waiting)을 통과해야 한다. successor는 고정 plan/request·연속 revision·항목 하나의 변경을 요구한다. 첫 SubmitPending 없이 Job 관측으로 건너뛰기/Job ID·input 바꾸기/역행/same Job revision 내용 바꾸기/완료 Job·output receipt 바꾸기/미완료 retry intent의 CAS pin 바꾸기는 거절한다. 원 submit error의 code/message/remediation/retryable/details/request_id를 보존한다. 완료 기록은 현재 파일 가용성이나 AI 품질 승인이 아니며 삭제된 성공 출력의 자동 재생성을 의미하지 않는다.

BatchStore/BatchLease 포트는 동일 trusted Jobs root에 결합하고 원 writer lease 아래 load→intent의 durable CAS publish→Jobs 호출→관측 publish를 요구한다. create-only 최초 record와 immutable event history,partial 현재 revision 노출 금지를 지킨다. 전체 input512KiB/document24MiB 한도는 core JobRecord128KiB×128과 오류/입력 보존을 위한 상한이다. 실제 어댑터는 per-item event/parent hash로 기록해 전체 큰 snapshot을 매번 복사하는 구현을 피한다. 열기/오류/host 재시작만으로 실행을 자동 재개하지 않는다.

R124는2130/2131fmt·2132snapshot2371/inventory4C784BF43E1D285FDB4744A9B5BDB0213D672324D26AE1D64BA6C09F4FFC7132/SOURCE.mdE6C56A30743DB2BD6A0503C47FE0DD882602635538A172DD01DDCE7512E3983D를 보존했다. 컴파일 전 초기 생성 guard를 보강한 R125는 preimage-r124-batch-initial-guard와2133fmt/2134새snapshot을 보존했다.2135core no-run 진행 중. 테스트는 원 JobRecord 도메인 메서드를 사용하는 synthetic journal 계약7개이며 실제 worker/파일발행/CLI 인수가 아니다.

남은 구현은 lease를 사용하는 submit/refresh/retry_failed orchestration,scope/FileID에 묶인 native event store,기존SDK/CLI host 등록,실제부분실패→성공보존/실패만retry·재열기/ACK미관측 인수다. 수정된 입력으로 fork하는 경로·cancel·부분실패 UI도 별도 추적한다. 현재 미구현을 batch 지원으로 계상하지 않는다. 최신 완료제품gate R123/원AI9 NeedsRepair·품질승인0/9·기존소유권/WIP/private overlay·WARP/추론/다운로드/수동GUI/push보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 synthetic 저장계약과 실제 producer/작업 인수를 분리한다.
