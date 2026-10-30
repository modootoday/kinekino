---
id: sot:kinekino/jobs-2
title: 원 Jobs를 사용하는 명시적 일괄 명령
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 Jobs를 사용하는 명시적 일괄 명령

R129 고정:2184fmt/2185snapshot 종료0,2375파일/inventory BA25EF7E45EE92BC1CA8E22C2D0C7AD41ED74F3AEAA603454004876109CA66AE,SOURCE.md 0D4A88A7C30BD1D7D0926E5536AD9A38E284D68641DA4AE3196873B629C5865B.2186 기본 no-run 진행 중이다. 첫 fixture 실패와 입력은 R128로 보존하고 완료 시험/strict/전체190/bins/custody는 아직 계상하지 않는다.

R128 첫 시험:2182 기본 no-run 종료0(214.649초),고정lib SHA0844F8D1A02FF3D65F94C8DAB985D1F3773457942526B66FA5A4120C55C53BB6.2183 journal/orchestration11시험 중10통과/1실패. 새 완료 fixture가 원 Job.begin_verification을 누락해 complete가 거절됐다. 원 로그·입력·preimage-r128-batch-completed-fixture 보존 후 R129에서 fixture에 원 단계만 추가했다. 제품 orchestration 동작 변경은 없으며 아직11통과로 계상하지 않는다.2184fmt 검사 및 다음snapshot/재컴파일·시험이 필요하다.

입력 고정:2181snapshot 종료0,2375파일/inventory90A7A90922AFD52B72613261A651BD612EDAFA89744DF8FC7FCEFA2E255B93FF,SOURCE.md A391EF3BD02D495C1F3403F934B216AE422D8DDE81EFDD2B9B1E9B5F70532ABF.2182 기본 lib/Jobs/CLI no-run 진행 중이며 새 orchestration 시험은 아직 미실행이다. 최신 완료 기준선은 R127이다.

R128 구현 중이다. BatchAction Submit/Refresh/RetryFailed는 원 BatchStore writer lease 아래 문서를 검증하고 batch ID·trusted scope·expected revision을 확인한다. host는 같은 native root identity의 store와 Jobs를 조립해야 하며 현재 실제 native store/host 등록은 없다.

Submit은 Waiting/SubmitError에서 durable SubmitPending을 먼저 publish하고 원 Jobs.submit에 고정 request/key를 전달한다. 이미 SubmitPending이면 같은 key로 명시 재제출한다. 원 Jobs 오류는 항목에 보존하며 성공·이미 관측된 항목은 건너뛴다. journal publish 실패는 명령을 중단하고 새 명시 명령으로만 해소한다. 다른 항목은 Jobs 오류 때문에 자동 폐기하지 않는다.

Refresh는 실행을 요청하지 않는다. 완료 항목은 원 출력 영수증과 함께 그대로 보존하고, 관측 항목만 원 Jobs.get으로 갱신한다. RetryPending의 동일 revision 관측은 의도를 해제하지 않으며 내용이 바뀌면 거절한다. 조회 오류를 보존하고 이 상태에서는 retry를 호출하지 않는다.

RetryFailed는 먼저 fresh get으로 이전 ACK 미관측을 해소한다. Failed 항목만 원 job/revision을 가진 durable RetryPending 뒤 원 Jobs.retry에 전달한다. 원 Jobs가 publication_started·retained attempt·revision CAS 및 실행 허용을 판단한다. 원 retry가 실행되고 응답 저장이 실패했으면 다음 명시 명령은 증가한 revision/attempt를 관측해 재시도 호출을 반복하지 않는다. Cancelled/NeedsInput을 실패만 retry 대상으로 확대하지 않는다.

2180fmt 종료0,2181snapshot 진행 중이다. 시험은 synthetic memory lease/Jobs port의 의도 저장 실패·ACK 저장 실패·동일key replay·retry ACK 유실·조회 실패·완료 보존을 다룬다. 실제 파일저장/부분실패 worker/SDK·CLI 인수는 아니다. preimage-r127-batch-orchestration에 이전 mod.rs를 보존했다. 기존 modootoday test-fixture-and-mock-build/code-change-review-verification을 적용해 포트 fault 시험과 미구현 native 인수를 구분한다. 원AI9 NeedsRepair/승인0/9·private overlay·소유권/WIP·보류 유지.
