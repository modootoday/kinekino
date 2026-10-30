---
id: evidence:kinekino/20261029114147
title: R82 실제 CLI 수명 후 동일입력 지정190·정상bins
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R82 실제 CLI 수명 후 동일입력 지정190·정상bins

2026-10-28 중앙 PM. [실제MKV CLI수명](review-mkv-cli-lifecycle-20261028.md)2시험통과후같은R82입력으로지정190을재실행했다. source375/privateoverlay를canonical로채택하거나원R77성공을승계하지않았다.

SOURCE511D3F5946A6155C51527E54543EC8E1209F8C95383DFE41C654D6905A824272,inventoryF74F0AF43577D1CB07578982AE4FFE4A54D6A90C6511B961E02A59CBD89EBF7E/2324. plan082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464유지.

로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1722/1727/1728|Native/default strict·fmtcheck|각exit0|
|1729|lib/jobs no-run|exit0|
|1730|첫exact호출|출력부모디렉터리누락·실행전실패보존|
|1731|Product145|145/145·exit0|
|1732|MeariCore18|18/18·exit0|
|1733|MeariNative26|26/26·exit0|
|1734|자동OS1|1/1·exit0|
|1735|정상Native player/player-cli|exit0·현재산출물재사용|
|1736|최종입력보존|exit0·2324content/identity동일|

raw와각group-return은runtime/verification/source-sdk-r82-exact190-r1/{product,core,native,os}다. Meari독립packages/Cargo.toml과mirror diff0을관측한뒤기존622/624producer manifest로현재시험을실제로재실행했다. 새Meari컴파일로표현하지않는다. 자동OS1은수동GUI시각인수가아니다.

fixed-r82/native-bins의CLI SHA EEF4EAE51B58914D65F8297C40581D1E3A64A515614E6525F64FDCDB0FA8175B,GUI SHA E10948BEB6F524B11E0F015986310A58BA6154C5212A1752CAA91DA7C0580484. GUI미실행·전체CLI명령인수/실제provider/나머지모델입출력/GUI비교·시각인수/사용자품질0/9는남는다. WIP/소유권/원실패/privateoverlay와WARP/추론/다운로드/수동GUI/push보류유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1유지.

적용스킬modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100,code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 현재실행·동일입력·지정회귀와전체제품인수를구분한다.
