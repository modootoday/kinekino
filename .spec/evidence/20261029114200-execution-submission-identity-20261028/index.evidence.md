---
id: evidence:kinekino/20261029114200
title: R91 공통 SDK 제출 식별·재접수 방지
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R91 공통 SDK 제출 식별·재접수 방지

2026-10-28 중앙 PM. [제출ID계약](../../contracts/production/execution-submission-identity.md)에따라HostExecution의typed submit_correlated/inspect_submission을구현했다. 원livehostscope·bounded submission_id·typedinner Request의serdeJSON SHA256과원ordinal을기록한다. 동일ID/동일요청은원참조를replayed로반환하고다른요청은RevisionConflict다. ACK이후도기록을남겨acknowledgedtrue와원참조를반환하며재실행하지않는다. ACK/조회상태는같은host잠금으로갱신한다.

기록128개는자동evict하지않고상한시새ID를거절한다. 기존ID조회/정확반복은상한뒤에도허용한다. 원요청은기존SDK요청예산으로크기측정후해시하며별도전체복사기록을남기지않는다. 제출ID문자열은기존RequestId문자/길이정책을따른다. hostScope/OperationId/receipt복구token권한을재구성하지않는다. wire/CLI입력/ready/schema는아직연결하지않았다. 기본무ID submit에새자동재시도보장을부여하지않는다.

원host.rs/tests.rs및관련연결preimage는mirror/runtime/verification/preimage-r90-submission-identity. 새host/submission.rs와tests/submission.rs를추가했다. R91 SOURCE9D911527C80E57082E7FA36A4DBCB2CDA1F8AFCA4D77A2D744F7E6DC85BEAEAF,inventory57B03BE624D711365632A02101143075AAEE16EA4AC64726574EC0BCA30D6AD2/2335. mirror/runtime/verification/input-r91. fixed-r91 lib시험exe SHAF625AE657AACFD3FCB7D6B4AE3D5512474F3051CC9E0B57E793E3B7FAA958D3E/PDB보존.

원로그 runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1837/1839|fmt적용|각exit0|
|1838/1840|Native strict·ACK잠금수정후최종|각exit0|
|1841|lib no-run|exit0|
|1842|제출식별/128기록상한선택|2pass/1576filtered|
|1843|기존host권한/RPC복구회귀|2pass|
|1844|snapshot|exit0|
|1845/1846|default strict·fmtcheck|각exit0|
|1847|최종custody|exit0·2335content/identity동일|

첫시험은합성Counting client의최초접수반환값을버린뒤inspect→동일요청반복으로원operation을조회하고실행count1을확인했다. 다른요청재사용·foreignscope를거절하고원terminal ACK후에도ID조회/정확반복은acknowledgedtrue,peek은SourceExpired,실행count1이다. 두번째시험은128개의서로다른ID를차례로terminal/ACK하고129번째새ID를ResourceExhausted로거절했다. 첫ID기록이그대로남고실행count128이며원workerclose를확인했다.

이것은합성SDK에서반환값을버린상황이며실제pipe응답유실·정상CLI/실제파일중복방지시험이아니다. 프로세스재시작을넘는내구적exactly-once도보장하지않는다. 다음은hostscope발견·명시IDsubmit/inspect wire/schema·실제CLI의유실응답/중복/충돌/ACK후·상한검증이다. notify재진입·전송fault/동시성전수는별도미검증이다. 원R88전체190/정상bins·R90실제wire상한소진은각원입력의근거로보존한다.

기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·사용자품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:합성응답폐기/실제전송손실·원실행count/기록상한·typed공통API/CLI지원을분리했다.
