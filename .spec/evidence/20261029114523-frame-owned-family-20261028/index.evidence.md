---
id: evidence:kinekino/20261029114523
title: "R50: Windows 프레임 프로세스의 원 family 정리"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R50: Windows 프레임 프로세스의 원 family 정리

후속 [같은 R50의 지정190/정상bins gate](source-sdk-r50-integrated-regression-20261028.md)를 실제 통과했다. 아래 gate 미실행 기록은 이 후속으로 해소되며 나머지 fault/취소/pending 복구·전체 기능 인수 한계는 유지한다.

2026-10-28 중앙 PM. [R47의30.04초 실패](frame-descendant-deadline-failure-20261028.md)를 수정했다. 같은 후손 fixture가 최종 R50에서2.02초에 통과했다. 제품 구현은 Windows frame helper에 한정하며 실행 중 SDK 취소·불완전 정리의 명시 복구·전체 기능 인수는 남는다.

## 구현 경계

기존 owned-process의 suspended 생성/Job 할당/resume와 Supervisor의 borrowed cleanup/receipt API를 소비한다. 공통 패키지나 다른 담당의 owner 구현은 수정하지 않았다. 원 Supervisor는 private Session과 같은 SDK 호출 stack에 남고 기존 SDK worker가 호출을 계속 보유한다. 미완료 owner를 ApiError와 함께 버리거나 registry/분리 thread로 옮기지 않는다.

stdout/stderr는 원 RawFile identity를 가진 private regular files로 받고 별도 크기 상한을 관측한다. Windows reader thread/join을 제거했다. 원 child exit·family Empty·최종 receipt를 확인한 뒤 bounded bytes를 읽는다. regular file 전송을 pipe EOF 관측으로 표시하지 않는다. 파일 크기는 poll로 검사하며 hard filesystem quota나 OS 호출의 실시간 상한을 주장하지 않는다.

원 작업 cutoff와2초 cleanup reserve는 준비/spawn 전에 고정한다. 후속 poll은 연장하지 않는다. receipt가 실패하면 최초 시도와 원 오류·owner를 pending으로 유지하며 자동 재시도 파일을 만들지 않는다. 이 상태의 명시 복구는 미구현이고 동기 호출은 pending 동안 반환하지 않는다. [계약](../../contracts/production/frame-process-family.md)·[결정](../../../adrs/20261028-frame-native-process-custody.md).

## 원본·중간 입력

- 원 process.rs: `preimage-r47-frame-owned-family/process.rs`, SHA `BC6F2A207AE65BCAF49547FCFB0A339DFB574ED18DADB22FB23D084B7217420C`.
- receipt 재시도 제한 전: `preimage-r48-frame-receipt-attempt/mod.rs`, SHA `3EA5AFE1569A03399E5AB7AA373061591608BC918E6F96698AF30D3409272E9C`.
- launch Timeout 매핑 전: `preimage-r49-frame-launch-timeout/mod.rs`, SHA `E30CD7F172CE07C5353F2AEDB39D47694A09186686344420E8671D55D1C62415`.

R48 SOURCE `33D4D5515DFA92BC28C9214596DEBBC96F326DE2C132807CFB9710D50C891705`, inventory `833CDF98D0D9310C895D2759102A5B5531BFE18470CAD77503E40985DB17F716`/2305.1388 Native strict0/22946ms,1389 no-run0/152119ms였다.

R49 SOURCE `45EFA7FB07772BE9CA440884AB617A688C008C0559E980AC09C7CEB30F1F2751`, inventory `FE31549A9E410907699785F46E035CFEC97DDE81DA0EA04707A799A3A76F1230`/2305.1390 no-run0/132967ms,1391 후손1pass/2.02s,1392 단일 child1pass/2.03s,1393 실제 host1pass/2.36s였다. 원 고정 producer와 raw를 보존한다.

1394의 실제1ms CLI 요청은 launch의 assign_verify_resume 단계 기한 초과를 engine_failed/exit8로 반환했다. 이는 기존 Timeout/exit10 계약의 회귀다. 원 응답·receipt를 보존하고 TimedOut 분류를 수정했다. R49의성공3시험으로 이 회귀를 숨기지 않는다.

## 최종 입력·producer

R50 SOURCE `276BEDEBEC604C0DEB2418BA9C16BE8F82A978F034190F9BD35DCBCBECB51636`, inventory `3A48AAF21807F99ED2588F353CEB1527979140C57BDA0A923A2E6611556BF83B`/2305. source는 각 compile/snapshot 시작 후 수정하지 않았다. AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 유지. 전용 C-only cache/tmp·heavy1/CPU2/BelowNormal·locked/offline/-j1을 유지했다.

1395 Native lib/jobs no-run terminal0/121709ms. 고정본은 다음과 같다.

|fixed-r50 파일|SHA256/bytes|
|---|---|
|frame-process-tests.exe|1E6BCB004DC235F78E58DDB1F364D35AFB25B6D9654D244F308875664C472DB6/191432192|
|jobs.exe|47E53C8792C0957CC7E87794565A2FC51D50BF998C384F5300F13B989FC7ABC2/28844032|
|player-cli.exe|CC7489ED6B1AB1DD711EB2E91C83B775A192D8FA3891ADBFB19EE96B736DFB88/37004288|

libtest PDB도 보존했다. operator index `source-sdk-r50-frame-owned-index.json`은 proof=false이며 실제 실행/출처 증거를 대체하지 않는다.

## 최종 실제 검증

|로그|범위|원 결과|
|---|---|---|
|1396|원 후손 fixture 기한 사례|정확1pass/1553filtered,2.02s; terminal0/17701ms|
|1397|기존 단일 child 기한/파일 해제|정확1pass/1553filtered,2.02s; terminal0/17150ms|
|1398|실제 pinned frame host의 endpoint CLI·typed client·EOF|정확1pass/66filtered,2.36s; terminal0/14612ms|
|1399|실제1ms CLI 기한 초과|Timeout/exit10,terminal10/443ms,미발행|
|1400|Native all-target strict|terminal0/28363ms,기존 blitz-paint 경고 보존|
|1401|default all-target strict|terminal0/17914ms|
|1402|전체fmt|terminal0/3670ms|
|1403|CLI 계약|4pass/1.33s;terminal0/2140ms|
|1404|입력 custody|terminal0/48294ms,2305 content/identity unchanged|

원 runner 로그는 `runtime/verification/source375-datalab-b0-r1-logs/1388~1404-source-sdk-r{48,49,50}-*`에 있다. 최종 C2 증거는 `source-sdk-r50-frame-{descendant,single-deadline,host}-os1/outer`와 각 TMP의 raw/receipt다. 세 C2 outcome 모두 selected_one=true, first_error=null, ownership=complete, physical_closed=true, child_exit0, family=Empty, forced=false, stdout/stderr EOF=true였다. C2 total60000/cleanup5000을 유지했고 실제 terminal 확인 뒤 다음 heavy를 배정했다.

후손 사례 내부의 원 process receipt는 child_exit124·family Empty·forced=true·stdin_eof_requested=true다. output_transport=bounded_regular_files, pipe_eof_observed=false를 그대로 보존했다. 외부 시험 프로세스의 forced=false와 내부 종료를 혼합하지 않는다. 같은 fixture의 파일 삭제 단언도 통과해 후손의 독점 파일 handle이 남지 않았음을 확인했다.

1398은 원 portrait SHA `0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483`와 명시8.1.1 도구 핀을 소비했다. endpoint/typed PNG는 모두500000µs·720×1280·SHA `0B73162584F488C517833254E029CCB4D26F6C966D126D7DE7E63F1DD7B9DE2A`였다. 원본/출력/CLI hash를 다시 확인했다. EOF 뒤 dispatch=completed·pipe/SDK=joined, job 카운터0을 확인했다. 직접 frame API를 durable job 성공으로 세지 않는다.

1399의 원 응답에는 stage=assign_verify_resume, child_exit124·physical_closed=true·receipt_recorded=true가 남았다. frame-owned root의 원 receipt를 보존했고 timeout.png는 생성되지 않았다. forced는 Supervisor의 관측/행동 범위이며 launch 실패의 선행 정리 여부를 단독으로 추정하지 않는다.

## 남은 인수

원 실패의 후손 기한 경로와 실제 정상 frame·짧은 기한 오류 분류를 인수한다. 부분 launch/termination/receipt failure의 강제 fault 전수, pending의 명시 복구, 활성 frame EOF·실행 중 SDK 취소·발행 경합과 panic 경로는 미완료다. 비Windows helper는 이번 변경의 인수 범위가 아니다.

R50 전체190/정상2bins gate와 새 CLI의전체명령도 아직 미실행이다. R46의190/bins·R44 discovery는 각 원입력에 보존한다. source375 private overlay·canonical미적용·원실패/소유권/WIP·AI0/9·추론/다운로드/수동GUI/WARP/push 보류 유지. 설치·전역설정·네트워크 변경 없음. 이선택검증으로 전체 goal을 닫지 않는다.

modootoday/software-qa/test-fixture-and-mock-build SHA `A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100`과 modootoday/software-qa/code-change-review-verification SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`의 동일 fault fixture/실제 도구 구분·원회귀·현재 producer와 인수 범위 보존을 적용했다.
