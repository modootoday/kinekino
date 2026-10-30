---
id: sot:kinekino/host-rpc
title: 명시 실행 host RPC 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 명시 실행 host RPC 경계

R84 공통 application 소비자의 입력/출력 계약이다. 아직 OS pipe·정상 CLI host·전역 capabilities/schema에 연결된 지원 기능이 아니다. [구현계획](../../../plans/player/production/receipt-recovery-wire-implementation-20261028.md)의 원owner보유/submit-peek경계를 따른다.

|RPC|엄격한 payload|응답 의미|
|---|---|---|
|execution.submit|request: 원 전체 SDK Request|operation참조·completion=admitted. 작업 성공 아님|
|execution.list|빈 객체|현재 host가 ACK 전 보유한 bounded operation참조|
|execution.peek|operation: host_scope/ordinal|queued/running/terminal과원결과·복구관측. 조회 성공과작업성공별도|
|execution.retry_receipt|operation, expected_generation(u8)|requested/already_requested·completion=unobserved·operation_reexecuted=false|
|execution.cancel|operation|원SDK queued_cancelled/requested/이미ACK disposition|
|execution.acknowledge|operation|원terminal의명시ACK와host참조회수. 이후참조거절|

각payload는unknown field를거절한다. 원OperationId/Arc복구토큰을외부값으로재구성하지않고host보유값으로해석한다. 바깥Request의expected_revision/idempotency_key/timeout_ms는거절한다. 내부SDK Request는자기context를유지한다. execution.*중첩submit은거절한다. submit의전송결과불확정시자동재시도하지않는다. list는원보유작업을관측할수있지만유실응답의요청과자동으로동일시하지않는다. 정확한응답손실/상관관계시험은wire연결후미완료다.

terminal은원response와envelope_valid/response_available/original_retained/cancellation_requested/queued_cancelled를구분한다. SDK의정확한크기측정이응답예산(전송4MiB에서wrapper8192bytes예약)이내일때만원response를복사한다. 큰응답이나측정fault는원결과를owner에보존하고response_available=false다. 원오류를성공으로바꾸거나자동ACK하지않는다. 실제최종wire인코딩의전체상한과오류회귀는별도필수다.

RPC에는임의명령실행/PID/파일경로복구권한·자동재시작·강제정리성공이없다. close/join은host조립의원owner보유책임이며아직RPC로노출하지않는다. hostscope는assembly가매번새불가예측값으로생성하고기존현재사용자인증/서버검증을사용해야한다. 공통소비자만으로인증·실제네트워크/CLI지원완료를표시하지않는다.

R85 infrastructure/local_host/execution_host 조립은기존보안endpoint생성기의OS CSPRNG nonce를새hostscope로사용하고별도실제pipeendpoint를LocalHost가보유한다. RpcClient는짧은execute_rpc만처리하며원작업은HostExecution의원ExecutionOwner에서실행한다. begin_shutdown은원SDK정리진행과기존pipe/dispatchworker조인을독립관측하는PendingExecutionLocalHost를반환한다. 완료는두계층의실제조인및미ACK0을모두요구한다. 미ACK/pending은성공종료가아니며원owner를보유한다. OS pipe시험·CLI기능인수는각검증근거를따르고아직CLI flag를추가하지않았다.

R86은명시--serve-execution CLI를연결했고R87은실제probe receipt충돌복구를검증했다. R88은host내version/capabilities/schema를추가해9개항목을한COMMANDS목록에서ready/지원조회/schema에사용한다. 기본SDK명령목록에는execution RPC를추가하지않는다. supported는이host RPC처리가구현됐다는뜻이며inner SDK작업전체지원/의미적성공은아니다. inner SDK명령의지원·작업오류는원terminal로조회한다. schema의ordinal/expected_revision 문자열은공통SDK의정확한u64범위/원참조검사를계속적용하며schema문서만으로권한을부여하지않는다. JSON Schema 실행기로모든example을검증한증거는별도로필요하다.
