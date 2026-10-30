---
id: evidence:kinekino/20261029114156
title: R86 정상 CLI 명시 실행 host·ACK/EOF
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R86 정상 CLI 명시 실행 host·ACK/EOF

2026-10-28 중앙 PM. --serve-execution을정상CLI에연결했다. 모드는유일해야하며다른host/endpoint와혼용하지않고기존frame_tools의완전한실행파일경로/SHA핀을요구한다. 자동도구탐색은사용하지않는다. 원SDK예산은4operation,요청/terminal각1MiB·총각4MiB다. ready는6개execution RPC와endpoint/pid를보고하고단순접수와작업결과·ACK를구분한다. 전역100명령capability/schema는아직변경하지않았다.

EOF/ready출력오류뒤원PendingExecutionLocalHost를보유하고3초동안실제close/join을관측한다. 완료면원최초I/O오류를보존하고미완료면cleanup_complete=false와원관측을보고하며pending이살아있는상태에서비정상exit한다. 이는worker분리나정리성공이아니며프로세스종료후원operation복구/재개를보장하지않는다. 실제ready출력오류/fault전수는아직미검증이다.

preimage는mirror/runtime/verification/preimage-r85-cli-execution의player-cli.rs/runner.rs다. 새CLI execution_host.rs와tests/execution_host_cli.rs를추가했다. R86 SOURCE5600F0E62DD2438FEFA8706673DDFF2B1D235F7B857E175E3EA6C025EE1931A9,inventory831A07A34B1BA17FA11A972FF22964FC1B678D911EBB316BD75F4739A65DD90F/2331. mirror/runtime/verification/input-r86. fixed-r86 정상CLI SHA C99A1E666E53A84E4541C09B1CFF207095A8DAAD43CEF38D1BA45C035B9D6734,시험exe6ABE748367CE7B8DCFA3AC81CA32C69E59DC3B51483EA63492E78AA30FC2B593/PDB보존.

로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1770/1772|fmt적용|exit0|
|1771|초기Native strict|exit0·추가시험전|
|1773|첫strict실패|공용시험helper private import의dead_code·원stderr보존|
|1774|helper를기존시험과같은pub mod로수리후strict|exit0|
|1775/1776|CLI/실제시험no-run·관측metadata수정후최종|각exit0|
|1777|host모드중복/누락/혼용/무핀거절|1pass/37filtered|
|1778|snapshot|exit0|
|1779|정상CLI접수/조회/ACK/EOF|1pass|
|1780|정상CLI미ACK/EOF거절|1pass·예상hostexit10|
|1781/1782/1783|최종Native/default strict·fmtcheck|각exit0|
|1784|최종custody|exit0·2331content/identity동일|

1779는mirror/runtime/verification/execution-cli-r86-ack-r1에원ready/request/CLI응답/원stdout/stderr/PROCESS를보존했다. inner version 작업의submit(admitted)·terminal원oktrue·list·ACK후EOF에서원pid17944 exit0/forcedfalse/originalwait/streamsjoined를확인했다. 정상producerguard/hash와원SOURCEguard를실행전후확인했다.

1780은execution-cli-r86-unack-r1에미ACKterminal을남긴EOF를보존했다. 원pid37144 exit10/forcedfalse,stderr cleanup_complete=false이며pipe/dispatch Joined·원SDKworker Joined라도TerminalAckPending1임을보고했다. 이예상거절은시험pass이며제품작업/정리성공이아니다. 시험PROCESS에는가정한cleanup완료대신실제exit에따른host_reported_success를기록했다.

도구는기존로컬FFmpeg8.1.1 SHA09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73과FFprobe8.1.1 SHAA6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2를고정했다. 이번은실제정상CLI/pipe이지만inner작업은version이며프레임추출/receipt충돌·복구는미실행이다. 다음은기존실제frame원증거/fault primitive를재사용하는정상CLI충돌복구,형식/발견schema·응답손실/소진/미해결EOF전수다.

원R82전체190/정상GUIbins/MKV수명은각원입력근거로유지하며R86성공으로승계하지않는다. 기존WIP/소유권/원실패/privateoverlay·수동GUI/WARP/추론/다운로드/push보류·품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100,code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:원실패·실제정상CLI/예상거절·부분인수와전체지원을분리했다.
