---
id: evidence:kinekino/20261029114519
title: R83 공통 SDK host 실행 소비자
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R83 공통 SDK host 실행 소비자

2026-10-28 중앙 PM. [wire구현계획](../../../plans/player/production/receipt-recovery-wire-implementation-20261028.md)의W0/W1 공통 application 소비자 `sdk_execution::host::HostExecution`을 구현했다. 이는 CLI/wire host 구현 또는 지원 표시가 아니다.

한원ExecutionOwner와exclusiveattachment를보유하며submit은원worker에접수후host scope/ordinal참조만반환한다. peek/cancel/recovery/retry/ACK/close는원attachment에위임한다. OperationId/Arc복구토큰을외부JSON에서재구성하지않는다. generation기대값을원pending snapshot과대조해원token으로retry하며원worker의단일소비/16상한을우회하지않는다. ACK후map참조를제거하고ordinal은재사용하지않는다. map은ExecutionBudget.max_operations로제한한다. assembly가매host새불가예측32자리소문자hexscope를공급해야하며실제scope발급/인증/wire연결은아직미구현이다.

원mod.rs/tests.rs는mirror/runtime/verification/preimage-r82-execution-host에보존했다. 새host.rs와tests/host.rs를추가했다. rootcanonical/소유권/WIP미변경·privateoverlay유지.

R83 SOURCE657579F134ED3C37B220CD3176D16AA60F4CB8B02CAFF0AD7EA46550D01EB81E,inventoryBF3D565C4AEF7C6CBA47477AC06033E0477038085C867C7C3E3E19A960595DA7/2326. mirror/runtime/verification/input-r83에고정. fixed-r83의lib시험exe SHA54B2EFCF88E96B7C9D68382773A1A25C99A52C2D328B0AA6E54FC5D105DBF51B 및PDB보존.

로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|결과|
|---|---|---|
|1737|fmt적용|exit0|
|1738|Native alltargets strict|exit0|
|1739|Native lib no-run|exit0|
|1740|새host원소유권합성시험|1pass/1570filtered|
|1741|입력snapshot|exit0|
|1742|default alltargets strict|exit0|
|1743|기존SDK recovery선택회귀|exit0|
|1744|fmtcheck|exit0|
|1745|최종custody|exit0·2326content/identity동일|

새시험은원recoverable client의2대기세대동안상태조회·명시retry·중복AlreadyRequested·세대별한번소비·foreignscope/비정규ordinal거절·queued retry거절·capacity·queuedcancel/ACK·ACK후거절·terminal첫EngineFailed보존·원workerclose를확인했다. 실제OS파일receipt실패/프레임process/CLI/wire를실행한것이아니다. 기존SDK범위의16상한회귀는1743원시험을따른다. capabilities/명령수는변경하지않았다.

다음은host scope발급·공통소비자의wire DTO/status/terminal과명시host조립이다. 미해결EOF/응답손실·ACK/close·실제frame충돌의정상CLI인수를남긴다. 전체190/정상bins는원R82근거이며R83으로승계하지않는다. 사용자품질0/9·GUI미인수·WARP/추론/다운로드/수동GUI/push보류유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1유지.

적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 합성권한시험/원SDK/실제wire지원을분리했다.
