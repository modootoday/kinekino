---
id: evidence:kinekino/20261029114321
title: R90 정상 CLI 16세대 소진·미완료 EOF
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R90 정상 CLI 16세대 소진·미완료 EOF

2026-10-28 중앙 PM. 제품코드/정상CLI는원R88과동일하게유지하고실제시험을확장했다. 원harness/감시자preimage는mirror/runtime/verification/preimage-r89-cli-exhaustion. 기존두충돌과별도로17개불변receipt충돌을만들어generation1..16각명시retry를실패시킨다. 상한/실패보존을검증하는예상미완료시험이며정리성공시험이아니다.

R90 SOURCEFCF681AEC0A0C95E159CE0019C6E40122698BD8BE51BBDC839CE4FC0C19635DE,inventory5F841CDDFB4BDB11D319E527EA488994E2B2113E55E6ED04AAE1C6EC81F06F09/2333. mirror/runtime/verification/input-r90. fixed-r90시험exe SHA7B59BC15DFFE1D5B70C7BA9987C28FABB184297ED8054D5E32409DA6349D5F72/PDB보존. 정상CLI는원R88 6660232B94AD056337A4E2D790A9937E02ADFD6DDAFDF4A81F016A639779C2F3다.

원로그 runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1826/1828|fmt적용|각exit0|
|1827|초기Native strict|exit0|
|1829|실제CLI시험no-run|exit0|
|1830|snapshot|exit0|
|1831|실제CLI16소진/추가요청거절/미완료EOF|1pass·예상hostexit10|
|1832|두충돌정상복구회귀|1pass·hostexit0|
|1833/1834/1835|최종Native/default strict·fmtcheck|각exit0|
|1836|최종custody|exit0·2333content/identity동일|

원raw는mirror/runtime/verification/execution-cli-r90-{exhausted,repeat-fault}-r1. 소진사례원hostpid34680의원작업root는<원 호스트 저장소>/runtime/build-cache/tmp/frame-owned-34680-1793140591224126100-4. 각세대자동요청없음·이전세대거절·명시요청단일진행·원미디어guard보유·원sentinel불변을확인했다. 마지막generation16/retry_requestedtrue/retry_consumedtrue/attempt_budget_exhaustedtrue와phase=running을관측했고추가generation16요청은resource_exhausted로거절됐다. receipt0000..0016은원sentinel,최종receipt0017/PNG는없다. 새ownedframe root는원작업하나뿐이다. EXHAUSTED.json은원관측/root목록/final_receipt_recordedfalse/cleanup_completefalse를보존한다.

EOF후stderr는pipe/dispatch Joined와원SDK TerminalAckPending1/workerExitPending(loop_finishedfalse)를구분했다. host는정리미완료를보고한상태에서원pending을보유하고비정상exit10으로종료했으며시험supervisor의강제kill은없었다. PROCESS.json의originalwait/streamsjoined/forcedfalse는원host프로세스관측이다. 이를원SDKworkerjoin·최종receipt저장·작업terminal/ACK성공으로승격하지않는다. 프로세스종료후원operation의재개·관리복구는지원/검증되지않았고원최종receipt가없으므로family정리성공도새롭게계상하지않는다.

별도두충돌회귀는새원프로세스에서첫EngineFailed·원sentinel/guard·최종receipt·ACK/EOF exit0를확인했다. 최초SDK/CLI오류와상한소진·hostEOF불완료를서로구분한다. 기본9발견/schema조회도각명시host에서수행했다. 내부모델/네트워크/새미디어/GUI는실행하지않았다.

다음미완료는유실submit응답과원작업의안전한상관관계/중복입력·응답손실,부분쓰기/권한fault,소진후관리복구및강제crash전수다. 현재list참조만으로유실submit의동일성을보장하지않으며자동재시도하지않는다. 제품전체목표를이복구하위시험으로축소하지않는다. 원R88지정190/정상bins는원입력에유지한다.

기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·사용자품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:예상미완료/시험pass·원host종료/원SDK조인·guard보유/복구완료를분리했다.
