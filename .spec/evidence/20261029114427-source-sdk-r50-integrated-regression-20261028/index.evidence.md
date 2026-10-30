---
id: evidence:kinekino/20261029114427
title: "R50: 원 family 수정 뒤 지정190·정상bins 회귀"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R50: 원 family 수정 뒤 지정190·정상bins 회귀

2026-10-28 중앙 PM. [프레임 원 family 수정](frame-owned-family-20261028.md)을 포함한 R50 입력으로 지정190을 실제 재실행했다. 이번 gate의 제품 소스 변경은 없다. 전체 SDK/CLI·GUI·AI 품질 완료나 canonical 채택으로 확대하지 않는다.

## 입력과 조건

SOURCE `276BEDEBEC604C0DEB2418BA9C16BE8F82A978F034190F9BD35DCBCBECB51636`, inventory `3A48AAF21807F99ED2588F353CEB1527979140C57BDA0A923A2E6611556BF83B`/2305 유지. AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C`, mirror lock `955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5`, Meari lock `23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE`를 확인했다.

Root HEAD `d13cbfca140e41863b4d0bd27930be3f57a1123f`, Meari `0844f6c24caf224617ca8abbb2dd75695a971b99`. stable MSVC cargo `AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F`, rustc `BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72`를 재확인했다. 고정190 plan SHA `082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464` 유지.

실제 C-only cache 문맥·locked/offline/-j1·heavy1/CPU2/BelowNormal을 유지했고 앞 handle의 terminal 뒤 다음 heavy를 배정했다. 원본/WIP·기존 실패·원 D 정책/증거는 변경하지 않았다.

## 실제 gate

|로그|범위|실제 결과|
|---|---|---|
|기존1400/1401/1402|같은 R50의 Native/default strict·fmt|각 actual0 유지; 이번 반복 실행으로 표시하지 않음|
|1405|Native lib/jobs no-run|terminal0/535ms,원 producer freshtrue|
|1406|Product|145/145,terminal0/19545ms|
|1407|Meari Core|18/18,terminal0/1200ms|
|1408|Meari Native|26/26,terminal0/5354ms|
|1409|자동 OS|1/1,terminal0/1335ms|
|1410|정상 Native player/player-cli|terminal0/484ms,profile.test=false,freshtrue|
|1411|최종 입력 custody|terminal0/41805ms,2305 content/identity unchanged|

각 group-return은 execution_result=Ok·producer_integrity=Ok와 위 R50 SOURCE 핀을 반환했다. 등록/ignored 일치 및 각 원 raw·spawn·전후 producer 식별은 `runtime/verification/source-sdk-r50-exact190-r1/{product,core,native,os}`에 보존했다. 원 runner 로그는 `source375-datalab-b0-r1-logs/1405~1411-source-sdk-r50-*`다. 이번 gate의 새 실패는 없다.

독립 Meari packages 전체 및 workspace Cargo.toml과 mirror는 각각 native git diff exit0였다. 기존622/624의 원 Cargo producer를 사용해 실제 시험을 재실행했다. Core/Native/OS 새 producers-before.tsv는 R46과 각각 diff0로 hash/FileID/시간 동일을 확인했다. 새 Meari 컴파일로 주장하지 않는다. 자동 OS1은 원 창 veto/completion/timer-owner 시험이며 제품 수동 GUI/시각 인수가 아니다.

freshtrue는 유효한 현재 Cargo 산출물 재사용이다. 새 link나 과거 성공을 새 실행으로 설명하지 않는다. 실제190 실행은 이번 group raw로 입증한다. R47의30.04초 실패와 R49의1ms 오류 분류 회귀는 원 기록에 남긴다.

## 보존 파일

|fixed-r50 파일|SHA256/bytes|
|---|---|
|native-bins/player.exe|6E7E4B20D92B244F0D0448AAEF47E671B4D9D74C1409B176A784B3FC45B9E644/163601920|
|native-bins/player-cli.exe|CC7489ED6B1AB1DD711EB2E91C83B775A192D8FA3891ADBFB19EE96B736DFB88/37004288|
|frame-process-tests.exe|1E6BCB004DC235F78E58DDB1F364D35AFB25B6D9654D244F308875664C472DB6/191432192|
|frame-process-tests.pdb|6FE48F8B8A4512FD6DB4CD108EEC813CC2ACEB517CC86388DAB0F82CC208EC3B/1177382912|
|jobs.exe|47E53C8792C0957CC7E87794565A2FC51D50BF998C384F5300F13B989FC7ABC2/28844032|
|jobs.pdb|63D5C6BFFAC3574B7FD796168C77185661AC7221F63793AED3C6591132F1483A/618852352|

복사본 hash가 이번 원 producer TSV와 일치함을 확인했다. 복사본의 FileID를 원 producer 식별에 사용하지 않는다. 정상 CLI는 앞서 실제 frame/Timeout을 검증한 fixed-r50/player-cli.exe와 동일한 bytes다.

## 남은 범위

지정190/정상bins gate 통과이며 전체 Cargo/CLI 기능·GUI/AI 승인은 아니다. receipt 실패의 명시 복구, 부분 launch/termination/receipt fault 전수, 실행 중 SDK 취소·활성 frame EOF·발행 경합과 panic, 나머지 미지원 SDK/CLI와 GUI 비교/수정/판정/이력·실행 시각 인수가 남는다.

source375 private overlay·canonical 미적용·원 실패/미실행·소유권/WIP·AI 사용자 품질0/9와 추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 배포는 대상 밖이며 설치/전역설정/네트워크/stage/commit/push 변경은 없다. 이 gate만으로 goal을 닫지 않는다.

modootoday/software-qa/test-fixture-and-mock-build SHA `A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100`와 code-change-review-verification SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`의 원 producer/실제 재실행·역사 실패·부분 인수 경계 구분을 적용했다.
