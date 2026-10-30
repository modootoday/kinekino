---
id: evidence:kinekino/20261029114438
title: R58 receipt 복구 연결 후 지정190·정상bins 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R58 receipt 복구 연결 후 지정190·정상bins 검증

최종1479 terminal/exit0, 입력2311 content/identity unchanged 확인. 지정190/190·정상2bins·실제 정상CLI frame 회귀를 이번 R58 입력에서 확인했다. 아래 작성 시 진행 중 기록을 이 종료 결과로 보완한다.

중앙 PM, 2026-10-28. [원 SDK frame receipt 복구](frame-sdk-receipt-recovery-20261028.md) 변경을 포함하는 private R58 입력의 최종 영향 회귀다. 제품 전체CLI/GUI/AI 인수 또는 canonical 채택을 뜻하지 않는다.

입력2311 SOURCE SHA256 393B0981FD9596D3AD006D8EC354CD2DF6F543B3F52ECF87F96BC4FCE47E0212, inventory B460A91016198FFA385D52F7C2859B27559547E0962ED2DAA4C93D65D64544A2 유지. AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C, mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5, Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE,190 plan082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464를 확인했다.

stable MSVC cargo AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F/rustc BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72 재확인. 독립 Meari packages 전체·workspace Cargo.toml과 mirror diff0. C-only cache/heavy1/CPU2/BelowNormal/locked/offline/-j1, 원 handle terminal 뒤 다음 heavy 배정을 유지한다.

기존 R58의1468/1469 strict·1470fmt·1471최종동일은 같은 입력의 실제 결과로 유지한다. 이번 반복 실행으로 표시하지 않는다.

|로그|범위|결과|
|---|---|---|
|1472|Native lib/jobs no-run|terminal0/92250ms|
|1473|Product145|145/145,terminal0/19643ms|
|1474|Meari Core18|18/18,terminal0/1186ms|
|1475|Meari Native26|26/26,terminal0/5338ms|
|1476|자동 OS1|1/1,terminal0/1315ms|
|1477|정상Native player/player-cli|terminal0/506ms,현재 Cargo 산출물 재사용|
|1478|정상CLI actual local frame/typed client·EOF정리|exact1pass/66filtered/1.93초,runner0/14088ms|

runtime/verification/source-sdk-r58-exact190-r1/{product,core,native,os}/group-return.txt에서 execution_result=Ok/producer_integrity=Ok와 R58 SOURCE 핀을 확인했다. 전후 식별/등록/ignored 및 raw는 각 디렉터리에 보존했다. Core/Native/OS producers-before.tsv는 R52와 각각 diff0로 원 생성자 식별 유지다. 기존622/624 Cargo producer를 재사용해 실제 시험을 재실행했으며 새 Meari 컴파일로 주장하지 않는다. 자동 OS1은 제품 수동 GUI/시각 인수가 아니다.1479 최종 custody는 작성 시 진행 중이다.

fixed-r58 보존 파일:

|파일|SHA256/bytes|
|---|---|
|native-bins/player.exe|2377DDD6EB5C38AD6FF3A837936DAE627816231F83899951FF1925B2A7FAF5CE/163624448|
|native-bins/player-cli.exe|9EEC26F0E4E584060DAD1397BDD98DBEC76DBCB661AC12BD9F046E9D070B9990/37023232|
|jobs.exe|C48F6F66577EBFFF13D215B39CB69DD481BF015F704EBBDAC407E19053E9AFEC/28845568|
|jobs.pdb|A7913DA25FC7D6D7318DCF443CEEC0B41AE504905D4D062D4C4933408D653C78/619393024|
|frame-recovery-tests.exe|E786B07D8A123D2F31C89BC9AD8AEF8B1965D9ED2FA9D8969265171A1918F314/192592384|
|frame-recovery-tests.pdb|DD8E6E76450C5B35DC65FC60EEFD078972D64F852B76E0920DC65B9033ED0C08/1183068160|

원 raw 로그는 source375-datalab-b0-r1-logs/1472~1479-source-sdk-r58-*와 source-sdk-r58-normal-frame-r1/outer다. 정상 CLI의 실제 로컬 영상 결과 SHA는0B73162584F488C517833254E029CCB4D26F6C966D126D7DE7E63F1DD7B9DE2A이며 SDK/typed 결과와 일치했다. GUI는 수동 실행하지 않았다. 복사본 FileID를 원 생성자 식별로 사용하지 않는다. 이번 gate 새 실패 없음.

남은 범위는 CLI/wire 복구·연속 실패/16세대상한·전체fault/CLI·실제 provider 제품경로·GUI/AI품질0/9다. root/source375 private overlay·원 실패/미실행·WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류 유지. 배포 제외. 목표 active 유지.

modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100, code-change-review-verification9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용: 원 생성자·입력·이번 실제 재실행과 이전 checkpoint를 구분한다.
