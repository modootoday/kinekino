---
id: evidence:kinekino/20261029114411
title: R88 실행 host 지원 조회·schema·실제 CLI
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R88 실행 host 지원 조회·schema·실제 CLI

2026-10-28 중앙 PM. 명시execution host에version/capabilities/schema를추가하고9개목록(발견3/RPC6)을ready/지원조회/schema에공유했다. 기본SDK의100명령목록에는execution RPC를추가하지않았다. supported는host RPC처리범위이며내부SDK작업전체지원이나기능완료율이아니다.

schema는strict payload·outer context의null제약·innerSDKcontext분리·hostscope/ordinal·원결과/접수/복구/ACK의의미를기록한다. 공통SDK의정확u64범위/원참조권한검사를대체하지않는다. 모든JSON Schema를독립실행기로검증한것은아니며해당근거는미완료다. shape/status/examples조회와실제Rust거절/동작시험범위를구분한다.

원host.rs/wire.rs/CLI실행host/시험은mirror/runtime/verification/preimage-r87-execution-discovery에보존했다. 새host/schema.rs와연결을private mirror에추가했다. R88 SOURCEED7F1A3AB6A70C3C4900988F2756795D1D7DED5C64C49FE001A6FA76A233F493,inventoryA907340D2CCF5743B5A7F2BC37687783845A3366295E68E6AF2EF43B93A9C1B9/2333. mirror/runtime/verification/input-r88. fixed-r88 CLI SHA6660232B94AD056337A4E2D790A9937E02ADFD6DDAFDF4A81F016A639779C2F3,시험E70C92110D611D1F40A2E48831D7F1E561DC05441B73EDA55CCBED9D63DA573A/PDB보존.

원로그 runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1795/1797/1800|fmt적용|각exit0|
|1796/1798|Native strict·시험추가후|각exit0|
|1799/1801|lib/실제CLI시험no-run·최종|각exit0|
|1802|host전용발견/unknown schema필드거절·terminalfault|2pass/1574filtered|
|1803|snapshot|exit0|
|1804|새CLI실제probe receipt복구및발견/schema대조|1pass|
|1805|새CLI version작업/ACK/EOF및발견/schema대조|1pass|
|1806|새CLI미ACK EOF거절및발견/schema대조|1pass·예상hostexit10|
|1807/1808/1809|최종Native/default strict·fmtcheck|각exit0|

실제각host에서ready/capabilities의9항목·status·각schema.command/host_protocol/request.command.const를대조하고outer version의execution-host/1을조회했다. 원request/stdout/stderr/process는mirror/runtime/verification/execution-cli-r88-{frame-fault,ack,unack}-r1에보존했다. frame-fault는원R87과같은외부Rust충돌로새원프로세스/receipt를검증했으며같은첫EngineFailed·sentinel·원미디어SHA/guard·PNG미발행·최종receipt·ACK/EOF를확인했다. unack은cleanupfalse/exit10이며제품작업성공으로계상하지않는다.

같은입력후속[지정190/정상bins](source-sdk-r88-integrated-regression-20261028.md)를따른다. 남은범위는wire반복/16세대소진·부분쓰기/권한fault·유실submit상관관계/응답손실·강제crash/미해결recoveryEOF·형식검증전수·전체CLI/provider/GUI/AI품질이다. 이전실패·WIP/소유권/privateoverlay·보류·품질0/9유지. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:발견/schema조회·실제CLI·전체기능인수를분리했다.
