---
id: evidence:kinekino/20261029114403
title: R92 제출 식별 연결 후 지정190·정상bins
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R92 제출 식별 연결 후 지정190·정상bins

2026-10-28 중앙 PM. [제출ID wire/실제CLI](execution-submission-wire-20261028.md)검증후같은입력으로지정190을실제재실행했다. source375 privateoverlay의canonical채택이나원R88성공승계가아니다.

SOURCEBF340669BBB6F5E5F82F470E9AD66D42ED8DDF00068F9CACCBB4FAFD2E7D2686,inventory4BF15EE245F1C1F84DCDCC7366F034877F8558856F18C759F62DC622BB6DC6B2/2335,plan082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464유지.

원로그 runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1849/1857/1858|Native/default strict·fmtcheck|각exit0|
|1859|lib/jobs no-run|exit0|
|1860|Product145|145/145·exit0|
|1861|MeariCore18|18/18·exit0|
|1862|MeariNative26|26/26·exit0|
|1863|자동OS1|1/1·exit0|
|1864|정상Native player/player-cli|exit0·현재산출물재사용|
|1865|최종custody|exit0·2335content/identity동일|

group-return의execution_result/producer_integrity=Ok·현재SOURCE를대조했다. raw는runtime/verification/source-sdk-r92-exact190-r1/{product,core,native,os}. Meari독립packages/Cargo.toml과mirror diff0관측후기존622/624producer manifest로시험을재실행했다. 새Meari컴파일이나수동시각인수로표현하지않는다.

fixed-r92/native-bins GUI SHA54FE74D9DDB2BBE64E9DB071691E8A824E42A7143DA9E28F31C6403BEFD22A0E,CLI SHA5ECE053CD866500B21F3808A76503A43C421724F09B961D44B7565ED51621BAE. GUI미실행·전체CLI제품여정/실제provider/나머지모델입출력/GUI비교·시각/사용자품질0/9는미완료다. 원실패/WIP/소유권/privateoverlay·기존보류유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 기존modootoday QA/코드검증스킬의동일입력·현재실행/과거성공·회귀/전체인수구분을적용했다.
