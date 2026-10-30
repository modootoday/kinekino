---
id: evidence:kinekino/20261029114434
title: R84 공통 실행 host RPC 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R84 공통 실행 host RPC 경계

2026-10-28 중앙 PM. [RPC계약](../../contracts/production/execution-host-rpc.md)을 공통 application HostExecution에 구현했다. 원SDK attachment/OperationId·세대token은host내부에보유하며strictpayload의submit/list/peek/cancel/retry_receipt/acknowledge를해석한다. 상위RPC의작업context혼용·unknown필드·중첩execution submit을거절한다. ACK전bounded목록조회와원결과보존을추가했다. OS pipe·명시정상CLI host·전역capabilities/schema는아직미연결이다.

원host.rs와tests/host.rs는mirror/runtime/verification/preimage-r83-host-wire, 새host/wire.rs와수정본은private mirror에유지한다. SOURCE45946D50F4AF24148C9E4A525CE1EB318A3B8AC946ADD47A32AFBF9E9C7C0815,inventory9A1F61EDFB4C1F35CA617FF62F280C460EB6C5C9A32806A29B6E24D719CA413D/2327. mirror/runtime/verification/input-r84에고정했다. fixed-r84의lib시험exe SHA28220007D5A09C14A151F43A939A3A6A386E6F2CCE8D60993604E488C5FACABE/PDB보존.

로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|실제범위|결과|
|---|---|---|
|1746/1750|fmt적용|exit0|
|1747/1751|Native alltargets strict·추가시험후최종|각exit0|
|1748/1752|lib no-run·추가시험후최종|각exit0|
|1749|추가fault시험전host선택|2pass·최종입력시험으로승계하지않음|
|1753|최종host소유권/RPC선택|2pass/1571filtered|
|1754|terminal응답크기/측정/형식fault선택|1pass/1572filtered|
|1755|snapshot|exit0|
|1756|default alltargets strict|exit0|
|1757|fmtcheck|exit0|
|1758|최종custody|exit0·2327content/identity동일|

RPC시험은running중조회/목록·strict거절·2세대명시retry·원EngineFailed의nested terminal응답·명시ACK·후속참조거절/목록회수를검증했다. fault시험은정확측정이없는상태/응답예산초과에서response_available=false/원결과보존, bounded원실패응답과ID불일치envelope_valid=false를검증했다. 합성SDK/API시험이며실제프로세스receipt·pipe동시수명·CLI실행성공이아니다. 일반조회를위해원작업오류를성공으로변경하거나자동ACK하지않는다.

다음미완료:hostscope의안전한발급·현재사용자pipe보안과서버identity·원hostowner의EOF/미해결정리/ACK/조인·CLI flag/typedclient/schema·응답손실및실제frame충돌복구. list만으로유실submit과원요청의동일성을보장하지않으며자동재시도하지않는다. 전체190/정상bins·실제MKV수명은원R82근거로유지하며R84재실행으로계상하지않는다.

기존WIP/소유권/원실패/raw/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·품질승인0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:합성RPC/실제전송·원결과/조회응답·일부회귀/전체기능의경계를구분했다.
