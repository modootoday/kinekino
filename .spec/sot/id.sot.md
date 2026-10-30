---
id: sot:kinekino/id
title: 명시 실행 제출 ID
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 명시 실행 제출 ID

원host의제출응답유실을다루기위한공통SDK계약이다. R91은typed HostExecution의submit_correlated/inspect_submission을구현하며wire/CLI지원은별도미완료다. 기존무ID submit은그대로이며자동재시도안전성을새로부여하지않는다.

SubmissionIdentity는host_scope와submission_id다. scope는원livehost와정확히같아야하고ID는기존RequestId의문자/길이정책으로제한한다. 새host가같은문자열ID를받았다고과거실행을재개하지않는다. 아직CLI ready에서scope를발견하거나wire로조회하는것은구현되지않았다.

첫접수전에원inner Request의context/요청예산을검증하고typed Request의serde JSON을SHA256으로고정한다. 같은host의동일ID/동일SHA는원operation과replayed=true를반환한다. 같은ID/다른전체요청은RevisionConflict다. outer전송request_id는원요청hash에섞지않으며wire연결시별도RPC경계에서처리한다. 원요청의inner request_id/context도SHA에포함하므로수정하면충돌이다.

ACK는원operation참조를회수하지만제출기록을지우지않는다. 조회와동일요청반복은acknowledged=true를반환하며새작업을실행하지않는다. peek의기존ACK후SourceExpired정책은유지한다. ACK/제출조회는같은host잠금으로일관되게관측한다.

기록은host당128개이며ID·SHA·ordinal만원본요청과별도로보유한다. 자동LRU/eviction/만료가없으므로과거ID가새작업으로바뀌지않는다. 상한이차면새ID접수를ResourceExhausted로거절하고기존ID조회/정확반복은유지한다. 새host시작은명시결정이며현재host의미완료작업/정리/ACK를대신하지않는다. admission실패는기록/ordinal을예약하지않는다.

이보장은동일원livehost내접수상관관계다. 프로세스재시작을넘는내구적exactly-once·임의client자동재시도·응답손실의실제pipe검증을뜻하지않는다. 원실행owner/worker·receipt복구권한은기존계약그대로다. 다음wire단계에서는scope발견/명시IDpayload/조회schema·유실응답및중복/충돌/ACK후·기록상한의실제CLI인수가필요하다.

R92 연결: ready가host_scope를제공하고execution.submit의선택payload submission에{host_scope,submission_id}를받는다. 생략/null은기존무ID접수다. 명시ID는원submission receipt(operation/request_sha256/acknowledged/replayed)를반환한다. execution.submission.inspect는동일submission객체로원기록을조회하며조회자체가접수/재실행하지않는다. host전용발견목록은10항목으로늘고기본SDK100명령목록은별도다. 같은livehost에보존된기록에대한명시반복만허용하며자동전송재시도/새host이전/프로세스재시작보장을추가하지않는다. 실제pipe/CLI검증완료는별도근거를따른다.
