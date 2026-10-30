---
id: evidence:kinekino/20261029114206
title: R63 제품 review 여정 후 지정190·정상bins
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R63 제품 review 여정 후 지정190·정상bins

최종1522 terminal0/41764ms·입력2312 content/identity unchanged 확인. 지정190/190·정상Native2bins·원 생성자 무결성 통과. 아래 작성시진행중 표기를 이실제종료로보완한다.

2026-10-28 중앙 PM. [제품 CLI 제어/종료 여정](review-product-controls-20261028.md)을 포함한 고정 R63 입력의 지정190을 실제 재실행했다. source375 private overlay/canonical미적용·기존 WIP/소유권/원실패를 유지한다. 전체CLI/GUI/AI품질 인수가 아니다.

입력2312 SOURCE6FCB07D47F9EC5CE281D6E08E989221C60E16BBCC96804F7B0B5FCC68A350F86,inventory525B36954D8F5B8D4123F2C7A07EB23994F9AA252EBB7E6F19BFF3BA3E866587. AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C,mirrorlock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5,Mearilock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE,190plan082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464를 재확인했다.

stable MSVC cargoAA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F/rustcBA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72를 재확인했다. C-only cache/heavy1/CPU2/BelowNormal/locked/offline/-j1, 선행 handle terminal 뒤 다음 heavy 배정을 유지했다.

|로그|범위|실제 결과|
|---|---|---|
|1512/1513/1514|같은입력 Native/default strict·fmt|원실제exit0 유지,이번반복실행 아님|
|1516|Native lib/jobs no-run|terminal0/18428ms|
|1517|Product145|145/145,terminal0/19954ms|
|1518|Meari Core18|18/18,terminal0/1175ms|
|1519|Meari Native26|26/26,terminal0/5317ms|
|1520|자동OS1|1/1,terminal0/1275ms|
|1521|정상Native player/player-cli|terminal0/493ms,현재산출물재사용|

각group-return의execution_result=Ok/producer_integrity=Ok와 R63 SOURCE핀을 확인했다. raw/등록/ignored/원생성자 전후식별은 runtime/verification/source-sdk-r63-exact190-r1/{product,core,native,os}에 보존했다. 독립 Mearipackages 전체·Cargo.toml과mirror diff0,Core/Native/OS producers-before는 R58과 각각diff0다. 기존622/624 생성자를 사용해 실제시험을 재실행했으며 새Meari컴파일로 주장하지 않는다. 자동OS1은 제품수동GUI/시각인수가 아니다.1522최종custody는작성시진행중이다.

fixed-r63 보존핀:

|파일|SHA256/bytes|
|---|---|
|native-bins/player.exe|6AC125A500E4CCC1AB1A3E082D4C1651B37BA018468F31615498A4E5DD46D794/163624448|
|native-bins/player-cli.exe|A7C1894E1915B35C7A8F96BF67EA3DD86EAADAD7F7B8E0B3A2A7F1784FBA691D/37023232|
|sdk-tests.exe|B09AAED1505D6601F215A47939EA3ED95F807C13A7E4BE5477495253CDC40422/192604160|
|sdk-tests.pdb|D395284F91DDBC6CB40C269B61CBBCF060D843CB90AB2B261B6A1363AB3316EC/1183494144|
|jobs-d75793112fc410a6.exe|324B7B799955FD45BF969ED7CD7AAD0931AC72476E92B76DBC63B0094E2CF0CC/28845568|
|jobs-d75793112fc410a6.pdb|52EA537DECE848B9A692128C0E4D6FA31764804BA4323660B35CFC4750D82D16/619393024|

정상 CLI는 같은 R63의1511 실제 control/close/reopen/EOF를 검증한 A7C1894E와 같은 bytes다. 과거 R61 restart/broken-ready·R59 복구 성공을 새 실행으로 계상하지 않는다. GUI는 수동 실행하지 않았다. 복사본 FileID를 원 producer 식별로 사용하지 않는다.

남은범위는구성별전체CLI 인수、실제provider제품경로、review준비취소/강제crash/전체fault、실제9GUI비교/수정/판정/이력·AI품질0/9·실행시각인수다.추론/다운로드/수동GUI/WARP/push보류유지,배포제외,goal active. 이번gate의새실패없음;R62첫seek누락실패/R60기한실패는원raw로보존.

modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B) 적용:원생성자·실제재실행·역사실패·부분인수경계를구분한다.
