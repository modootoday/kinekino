---
id: evidence:kinekino/20261029114823
title: R93 정상 CLI 제출 식별 거절 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R93 정상 CLI 제출 식별 거절 경계

2026-10-28 중앙 PM. 제품코드·정상CLI는원R92와동일하게두고제출ID의wire거절을실제CLI시험에추가했다. 원harness는mirror/runtime/verification/preimage-r92-submission-cli-rejections에보존하고새tests/execution_host_cli/submission_rejections.rs를추가했다. 기존128기록상한은원R91공통SDK시험근거를사용하며381개이상CLI프로세스의중복상한재검증은이번범위에추가하지않았다. 실제CLI상한전수미실행을보존한다.

R93 SOURCE07D836401A481E065B8C8235EAA7B02C928572E7B05FA97FD2537DFF6EF07A78,inventory2B38C5EE4001A28B7BC6F9070C04C5E20AFE933FA6A8EC3B86A05F0AA7526E3B/2336. mirror/runtime/verification/input-r93. fixed-r93시험exe SHABF45EE70A4B4956BB1F2EC30FE7FB32FBD43795B8916D5C09862CB40E2452808/PDB보존. 정상CLI는원R92 5ECE053CD866500B21F3808A76503A43C421724F09B961D44B7565ED51621BAE이며이번실행은새원로그로구분한다.

원로그 runtime/verification/source375-datalab-b0-r1-logs:1866fmt적용0·1867Native strict0·1868실제CLI시험no-run0·1869snapshot0·1870실제CLI1pass·1871defaultstrict0·1872fmtcheck0.

최종1873 custody exit0·2336입력 content/identity동일을확인했다.

1870 raw는mirror/runtime/verification/execution-cli-r93-correlated-rejections-r1. 각거절의request.json/stdout.raw/stderr.raw/process.json을보존했다.

|사례|예상오류·실제확인|
|---|---|
|동일제출ID/다른inner명령|revision_conflict|
|foreign hostscope조회|source_expired|
|foreign hostscope접수|source_expired|
|미접수ID조회|not_found|
|빈ID접수|invalid_argument|
|제어문자ID접수|invalid_argument|
|identity에추가PID필드|invalid_argument|
|inspect에추가path필드|invalid_argument|
|execution.*중첩접수|invalid_argument|

모든거절CLI는nonzero/okfalse/정확errorcode를확인했다. 거절후원submission.inspect의operation/request_sha256이최초조회와같고retained작업수1임을확인했다. 원terminal/ACK후조회/명시같은요청반복·빈작업목록·정상EOF도통과했다. 10개ready/capabilities/schema대조도같은host에서확인했다. 원본없는별도검증프로그램이나제품정책을PowerShell에구현하지않았다.

남은부분쓰기/권한fault·전송partialwrite별분류·동시성/notify재진입·프로세스재시작관리복구·독립JSON Schema실행검증은계속미완료다. 다만공통SDK/실제pipe/정상CLI의기본복구·기록상한·핵심거절은각원근거로연결됐으므로다음중앙배정은전체제품CLI의기능군/명시구성/실제인수근거미매핑을정리하고다음제품구현공백을선정하는것이다. 복구하위시험을전체제품목표로대체하지않는다.

목표현황표를R92통합이후의현재표와역사R77표로분리했다. 현재지정190/정상2bins·GUI미실행·전체CLI/GUI율미산정·품질0/9를구분한다. 제품코드변경이없으므로원R92gate를R93재실행으로표현하지않는다. 기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:예상거절·원기록보존·중복시험회피·남은실제범위를구분했다.
